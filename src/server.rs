use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::{
    helpers::{
        email, job_checker, notion, pdf::{create_sasi_timesheet, TimesheetData}
    },
    middlewares, AppData,
};

pub fn build_router(shared_state: Arc<AppData>) -> Router {
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
        .route("/test", get(test))
        .with_state(shared_state);

    info!("Server initialization complete");
    router
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

async fn test() -> String {
    job_checker::optum().await.unwrap()
}
