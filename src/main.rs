mod notion;

use axum::{routing::get, Router};
use dotenv::dotenv;
use std::env;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    dotenv().ok();

    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}

async fn hello_world() -> &'static str {
    "Hello, world!"
}

