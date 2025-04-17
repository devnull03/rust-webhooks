mod notion;
// mod pdf;

use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use reqwest::Client;
use std::{env, sync::Arc};

struct AppData {
    notion_client: Client,
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    dotenv().ok();

    let notion_client = notion::notion_client_init();
    let shared_state = Arc::new(AppData { notion_client });

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/notion-hook", post(notion_webhook))
        .route("/notion-test", get(notion_webhook))
        .route("/notion-db", get(notion_db))
        .with_state(shared_state);

    Ok(router.into())
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

async fn notion_webhook(State(state): State<Arc<AppData>>) -> String {
    let res = notion::fetch_data(&state.notion_client).await;

    format!("{}", res)
}

async fn notion_db(State(state): State<Arc<AppData>>) -> String {
    notion::util_retrive_db(&state.notion_client).await
}
