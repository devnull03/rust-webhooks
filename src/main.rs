mod helpers;
mod middlewares;
mod models;
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
pub struct TimesheetAppData {
    notion_client: Client,
    db_id: String,
    automation_id: String,
}

#[derive(Clone)]
pub struct JobAlertAutomationAppData {
    notion_client: Client,
    db_id: String,
}

#[derive(Clone)]
pub struct AppData {
    resend: Resend,
    timesheet: TimesheetAppData,
    job_alert: JobAlertAutomationAppData,
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

    let timesheet_app_data: TimesheetAppData = {
        let notion_api_key = secrets.get("TIMESHEET_NOTION_API_KEY").unwrap();
        info!("Initializing Timesheet Notion client");
        let notion_client = notion::notion_client_init(notion_api_key).unwrap();
        TimesheetAppData {
            notion_client,
            db_id: secrets.get("TIMESHEET_DB_ID").unwrap(),
            automation_id: secrets.get("TIMESHEET_AUTOMATION_ID").unwrap(),
        }
    };

    let job_alert_app_data: JobAlertAutomationAppData = {
        let notion_api_key = secrets.get("JOB_ALERT_NOTION_API_KEY").unwrap();
        info!("Initializing Timesheet Notion client");
        let notion_client = notion::notion_client_init(notion_api_key).unwrap();
        JobAlertAutomationAppData {
            notion_client,
            db_id: secrets.get("JOB_ALERT_DB_ID").unwrap(),
        }
    };

    info!("Configuring application state");
    let shared_state: Arc<AppData> = Arc::new(AppData {
        resend: Resend::new(secrets.get("RESEND_API_KEY").unwrap().as_str()),
        timesheet: timesheet_app_data,
        job_alert: job_alert_app_data,
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
            self.monitor.run().await.unwrap();
        };

        let _res = tokio::join!(http, monitor);

        Ok(())
    }
}
