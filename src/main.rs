mod helpers;
mod middlewares;
mod scheduler;
mod server;

use apalis::prelude::Monitor;
use axum::Router;
use helpers::notion;
use reqwest::Client;
use resend_rs::Resend;
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
pub struct CustomService {
    router: Router,
    monitor: Monitor,
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> Result<CustomService, shuttle_runtime::Error> {
    info!("Starting Rust Webhooks server");

    let notion_api_key = secrets.get("NOTION_API_KEY").unwrap();
    info!("Initializing Notion client");
    let notion_client = notion::notion_client_init(notion_api_key).unwrap();

    info!("Configuring application state");
    let shared_state: Arc<AppData> = Arc::new(AppData {
        notion_client,
        timesheet_db_id: secrets.get("DB_ID").unwrap(),
        timesheet_automation_id: secrets.get("TIMESHEET_AUTOMATION_ID").unwrap(),
        resend: Resend::new(secrets.get("RESEND_API_KEY").unwrap().as_str()),
    });

    let router = server::build_router(shared_state.clone());

    let monitor = scheduler::build_cron_worker_monitor(shared_state.clone());

    Ok(CustomService { router, monitor })
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CustomService {
    async fn bind(mut self, addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        let http = async {
            axum::serve(listener, self.router).await.unwrap();
        };

        let monitor = async {
            // self.monitor.run().await.unwrap();
        };

        let _res = tokio::join!(http, monitor);

        Ok(())
    }
}
