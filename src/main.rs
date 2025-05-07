mod helpers;
mod middlewares;

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use helpers::{
    email, notion,
    pdf::{create_sasi_timesheet, TimesheetData},
};
use reqwest::Client;
use resend_rs::Resend;
use shuttle_runtime::SecretStore;
use std::{env, sync::Arc};
use tracing::{error, info};

#[derive(Clone)]
pub struct AppData {
    notion_client: Client,
    timesheet_db_id: String,
    timesheet_automation_id: String,
    resend: Resend,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    info!("Starting Rust Webhooks server");

    let notion_api_key = secrets.get("NOTION_API_KEY").unwrap();
    info!("Initializing Notion client");
    let notion_client = notion::notion_client_init(notion_api_key).unwrap();

    info!("Configuring application state");
    let shared_state = Arc::new(AppData {
        notion_client,
        timesheet_db_id: secrets.get("DB_ID").unwrap(),
        timesheet_automation_id: secrets.get("TIMESHEET_AUTOMATION_ID").unwrap(),
        resend: Resend::new(secrets.get("RESEND_API_KEY").unwrap().as_str()),
    });

    info!("Setting up router");
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/notion-hook", post(notion_webhook))
        .layer(middleware::from_fn_with_state(
            shared_state.clone(),
            middlewares::notion_automation_check,
        ))
        // .layer(middleware::from_fn_with_state(
        //     shared_state.clone(),
        //     middlewares::notion_verification,
        // ))
        .route("/notion-test", get(notion_test))
        .route("/notion-db", get(notion_db))
        .with_state(shared_state);

    info!("Server initialization complete");
    Ok(router.into())
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn notion_webhook(
    State(state): State<Arc<AppData>>,
    Json(payload): Json<notion::structs::WebhookAutomationEvent>,
) -> String {
    info!("Received webhook from Notion");

    if payload
        .source
        .automation_id
        .ne(&state.timesheet_automation_id)
    {
        info!(
            "Automation ID mismatch. Received: {}",
            payload.source.automation_id
        );
        return "not the automation you are looking for".to_string();
    }

    info!(
        "Fetching timesheet data from Notion database: {}",
        state.timesheet_db_id
    );
    let timesheet_raw_data = notion::fetch_data(&state.notion_client, &state.timesheet_db_id)
        .await
        .unwrap();

    match TimesheetData::try_from(timesheet_raw_data.results) {
        Ok(timesheet_data) => {
            info!(
                "Successfully parsed timesheet data with {} entries",
                timesheet_data.entries.len()
            );

            match create_sasi_timesheet(timesheet_data) {
                Ok(timesheet) => {
                    info!(
                        "Successfully created timesheet PDF, size: {} bytes",
                        timesheet.len()
                    );

                    let email_res = email::send_timesheet_email(&state.resend, timesheet).await;

                    match email_res {
                        Ok(res) => {
                            info!("Email sent successfully with ID: {}", res.id);
                            res.id.to_string()
                        }
                        Err(e) => {
                            error!("Error sending email: {}", e);
                            let error_msg = format!("Error sending email (Error: {})", e);
                            let _ = email::send_error_info(&state.resend, &error_msg).await;
                            error_msg
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create timesheet PDF: {}", e);
                    let error_msg = format!("Error creating timesheet PDF: {}", e);
                    let _ = email::send_error_info(&state.resend, &error_msg).await;
                    error_msg
                }
            }
        }
        Err(err) => {
            error!("Error parsing Notion database: {}", err);
            let error_msg = format!("Error with parsing your linked database (Error: {})", err);
            let _ = email::send_error_info(&state.resend, &error_msg).await;
            error_msg
        }
    }
}

async fn notion_test(State(state): State<Arc<AppData>>) -> String {
    info!(
        "Fetching timesheet data from Notion database: {}",
        state.timesheet_db_id
    );
    let timesheet_raw_data = notion::fetch_data(&state.notion_client, &state.timesheet_db_id)
        .await
        .unwrap();

    match TimesheetData::try_from(timesheet_raw_data.results) {
        Ok(timesheet_data) => {
            info!(
                "Successfully parsed timesheet data with {} entries",
                timesheet_data.entries.len()
            );

            match create_sasi_timesheet(timesheet_data) {
                Ok(timesheet) => {
                    info!(
                        "Successfully created timesheet PDF, size: {} bytes",
                        timesheet.len()
                    );

                    let email_res = email::send_timesheet_email(&state.resend, timesheet).await;

                    match email_res {
                        Ok(res) => {
                            info!("Email sent successfully with ID: {}", res.id);
                            res.id.to_string()
                        }
                        Err(e) => {
                            error!("Error sending email: {}", e);
                            let error_msg = format!("Error sending email (Error: {})", e);
                            let _ = email::send_error_info(&state.resend, &error_msg).await;
                            error_msg
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create timesheet PDF: {}", e);
                    let error_msg = format!("Error creating timesheet PDF: {}", e);
                    let _ = email::send_error_info(&state.resend, &error_msg).await;
                    error_msg
                }
            }
        }
        Err(err) => {
            error!("Error parsing Notion database: {}", err);
            let error_msg = format!("Error with parsing your linked database (Error: {})", err);
            let _ = email::send_error_info(&state.resend, &error_msg).await;
            error_msg
        }
    }
}

async fn notion_db(State(state): State<Arc<AppData>>) -> String {
    info!(
        "Retrieving database structure for: {}",
        state.timesheet_db_id
    );
    notion::retrive_db(&state.notion_client, &state.timesheet_db_id)
        .await
        .unwrap()
}
