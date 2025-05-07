mod helpers;
mod middlewares;
// mod scheduler;
mod server;

use helpers::notion;
use reqwest::Client;
use resend_rs::Resend;
use server::setup_server;
use shuttle_runtime::SecretStore;
use std::{env, sync::Arc};
use tracing::info;

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

    setup_server(shared_state.clone())
}
