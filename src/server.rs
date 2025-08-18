use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use resend_rs::types::Attachment;
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    helpers::{
        email,
        job_checker::{self, JobAlertEmailHandler},
        notion,
        pdf::{create_sasi_timesheet, TimesheetData},
    },
    middlewares,
    models::{
        self,
        job::{optum::Job, EmailWebhookData},
    },
    AppData,
};

pub fn build_router(shared_state: Arc<AppData>) -> Router {
    info!("Setting up router");
    let router = Router::new()
        .route("/notion-hook", post(notion_webhook))
        .route_layer(middleware::from_fn_with_state(
            shared_state.clone(),
            middlewares::notion_automation_check,
        ))
        .route("/", get(hello_world))
        .route(
            "/cloudflare-job-alert-reciever",
            post(cloudflare_job_alert_reciever),
        )
        // test routes ----------------
        .route("/notion-test", get(notion_test))
        .route("/notion-db", get(notion_db))
        .route("/test", get(test))
        // ----------------------------
        .with_state(shared_state);

    info!("Server initialization complete");
    router
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn notion_webhook(
    State(state): State<Arc<AppData>>,
    Json(payload): Json<models::notion::WebhookAutomationEvent>,
) -> String {
    info!("Received webhook from Notion");

    if payload
        .source
        .automation_id
        .ne(&state.timesheet.automation_id)
    {
        info!(
            "Automation ID mismatch. Received: {}",
            payload.source.automation_id
        );
        return "not the automation you are looking for".to_string();
    }

    info!(
        "Fetching timesheet data from Notion database: {}",
        state.timesheet.db_id
    );
    let timesheet_raw_data =
        notion::fetch_data(&state.timesheet.notion_client, &state.timesheet.db_id)
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
        state.timesheet.db_id
    );
    let timesheet_raw_data =
        notion::fetch_data(&state.timesheet.notion_client, &state.timesheet.db_id)
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

                    // "Timesheet generated successfully".to_string()
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
        state.timesheet.db_id
    );
    notion::retrive_db(&state.timesheet.notion_client, &state.timesheet.db_id)
        .await
        .unwrap()
}

async fn test() -> axum::Json<Vec<Job>> {
    let jobs = job_checker::scheduler::optum().await.unwrap();
    // let mut res: Vec<String> = vec![];

    // for ele in jobs {
    //     res.push(ele.to_string());
    // }

    axum::Json(jobs)
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
                        content.push_str(&format!("<p><strong>{}</strong>: <a href=\"{}\">{}</a></p>", 
                            job_title, job_url, job_url));
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
