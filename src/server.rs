use axum::{
    extract::State,
    // middleware,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use resend_rs::types::Attachment;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    helpers::{email, job_checker::JobAlertEmailHandler},
    models::job::EmailWebhookData,
    AppData,
};


pub fn build_router(
    shared_state: Arc<AppData>,
    service_routers: Option<Vec<(&str, Router)>>,
) -> Router {
    info!("Setting up router");

    let mut router: Router = Router::new()
        .route("/", get(hello_world))
        .route(
            "/cloudflare-job-alert-reciever",
            post(cloudflare_job_alert_reciever),
        )
        // test routes ----------------
        // .route("/notion-test", get(notion_test))
        // ----------------------------
        // .route("/notion-db", get(notion_db))
        // .route("/test", get(test))
        // .route("/notion-hook", post(notion_webhook))
        // .route_layer(middleware::from_fn_with_state(
        //     shared_state.clone(),
        //     middlewares::notion_automation_check,
        // ))
        // ----------------------------
        .with_state(shared_state);

    if let Some(service_routers) = service_routers {
        for service_router in service_routers {
            router = router.nest(service_router.0, service_router.1);
        }
    }

    info!("Server initialization complete");
    router
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn cloudflare_job_alert_reciever(
    State(state): State<Arc<AppData>>,
    Json(payload): Json<EmailWebhookData>,
) -> String {
    info!("WEBHOOK RECEIVED: /cloudflare-job-alert-reciever");
    info!(
        "From: {}, To: {}, Size: {}",
        &payload.from, &payload.to, &payload.size
    );

    // Decode the email content
    match STANDARD.decode(&payload.raw_content) {
        Ok(email_bytes) => match String::from_utf8(email_bytes.clone()) {
            Ok(email_content) => {
                info!(
                    "Email content: {}",
                    email_content.chars().take(200).collect::<String>()
                );

                let handler = JobAlertEmailHandler::new(&email_bytes);
                let found_jobs = handler.results();

                // Convert the HashMap to a formatted string for email
                let job_content = if found_jobs.is_empty() {
                    "No jobs found in email".to_string()
                } else {
                    let mut content = String::from("<h2>Found Jobs</h2>");
                    for (job_title, job_url) in found_jobs.iter() {
                        content.push_str(&format!(
                            "<p><strong>{}</strong>: <a href=\"{}\">{}</a></p>",
                            job_title, job_url, job_url
                        ));
                    }
                    content
                };

                let subject = format!("Job alert processing from {}", payload.from);
                email::send_email(
                    &state.resend,
                    &job_content,
                    Some(subject.as_str()),
                    Some(
                        Attachment::from_content(email_bytes.clone())
                            .with_content_type("txt")
                            .with_filename("email_bytes"),
                    ),
                )
                .await
                .unwrap();
            }
            Err(e) => error!("Failed to parse email as UTF-8: {:?}", e),
        },
        Err(e) => error!("Failed to decode base64 content: {:?}", e),
    }

    // Send notification...
    "Thank you for your webhook!".to_string()
}
