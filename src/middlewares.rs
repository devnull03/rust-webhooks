use std::{sync::Arc, usize};

use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tracing::{info, error};

use crate::{helpers::notion::structs::WebhookAutomationEvent, AppData};

pub async fn notion_automation_check(
    State(state): State<Arc<AppData>>,
    request: Request,
    next: Next,
) -> Response {
    info!("Checking Notion automation webhook request");

    if request
        .headers()
        .get("user-agent")
        .unwrap()
        .ne("NotionAutomation")
    {
        let error_message = "Invalid user agent";
        error!("Automation webhook validation failed: {}", error_message);
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(error_message))
            .unwrap();
    }

    let (parts, body) = request.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let payload: WebhookAutomationEvent = serde_json::from_slice(&bytes).unwrap();

    if payload
        .source
        .automation_id
        .ne(&state.timesheet_automation_id)
    {
        let error_message = "Invalid automation id";
        error!("Automation webhook validation failed: {}", error_message);
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from(error_message))
            .unwrap();
    }

    info!("Automation webhook validation successful");

    let request = Request::from_parts(parts, Body::from(bytes));
    next.run(request).await
}

pub async fn _notion_verification(
    State(_state): State<Arc<AppData>>,
    request: Request,
    next: Next,
) -> Response {
    //     let verification_token = &state.notion_timesheet_webhook_token;
    let verification_token = "";
    let request_signature_string = match request.headers().get("X-Notion-Signature") {
        Some(signature) => signature.to_str().unwrap_or("").to_string(),
        None => {
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::from("Missing signature header"))
                .unwrap();
        }
    };

    let verification_token_string = verification_token.to_string();
    let request = match _buffer_request_body(
        request,
        &request_signature_string,
        &verification_token_string,
    )
    .await
    {
        Ok(req) => req,
        Err(response) => return response,
    };

    next.run(request).await
}

fn _verify_payload(
    body_bytes: Bytes,
    request_signature: &String,
    verification_token: &String,
) -> bool {
    // Create the HMAC signature
    let mut mac = Hmac::<Sha256>::new_from_slice(verification_token.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(&body_bytes);
    let calculated_signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    // Constant-time comparison of signatures
    let is_trusted_payload: bool = calculated_signature
        .as_bytes()
        .ct_eq(request_signature.as_bytes())
        .into();

    is_trusted_payload
}

async fn _buffer_request_body(
    request: Request<Body>,
    request_signature: &String,
    verification_token: &String,
) -> Result<Request<Body>, Response> {
    let (parts, body) = request.into_parts();

    // this wont work if the body is an long running stream
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response())?;

    let is_trusted_payload = _verify_payload(bytes.clone(), request_signature, verification_token);
    if !is_trusted_payload {
        return Err(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::from("Invalid signature"))
            .unwrap());
    }

    Ok(Request::from_parts(parts, Body::from(bytes)))
}
