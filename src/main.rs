mod middlewares;
mod notion;
mod email;
// mod pdf;

use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Router,
};
use reqwest::Client;
use shuttle_runtime::SecretStore;
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppData {
    notion_client: Client,
    timesheet_db_id: String,
    notion_timesheet_webhook_token: String,
}

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secrets: SecretStore) -> shuttle_axum::ShuttleAxum {
    let notion_client = notion::notion_client_init(secrets.get("NOTION_API_KEY").unwrap());
    let shared_state = Arc::new(AppData {
        notion_client,
        timesheet_db_id: secrets.get("DB_ID").unwrap(),
        notion_timesheet_webhook_token: secrets.get("").unwrap(),
    });

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/notion-hook", post(notion_webhook))
        // .layer(middleware::from_fn_with_state(
        //     shared_state.clone(),
        //     middlewares::notion_verification,
        // ))
        .route("/notion-test", get(notion_webhook))
        .route("/notion-db", get(notion_db))
        .with_state(shared_state);

    Ok(router.into())
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn notion_webhook(State(state): State<Arc<AppData>>) -> String {
    let res = notion::fetch_data(&state.notion_client, &state.timesheet_db_id).await;

    format!("{}", res)
}

async fn notion_db(State(state): State<Arc<AppData>>) -> String {
    notion::retrive_db(&state.notion_client, &state.timesheet_db_id).await
}
