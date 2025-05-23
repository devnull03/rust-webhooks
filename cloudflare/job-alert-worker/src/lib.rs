use worker::*;
use serde_json::json;
use base64::{Engine as _, engine::general_purpose};

#[event(email)]
async fn main(message: EmailMessage, _env: Env, _ctx: Context) -> Result<()> {
    console_error_panic_hook::set_once();
    console_log!("Received email, forwarding and sending webhook");
    // message.forward("arnav@dvnl.work".to_string(), None).await?;

    // let webhook_url = "https://hooks.dvnl.work/cloudflare-job-alert-reciever";
    let webhook_url = "https://rust-webhooks-wxht.shuttle.app/cloudflare-job-alert-reciever";
    console_log!("Attempting to send webhook to: {}", webhook_url);

    let email_data = json!({
        "from": message.from_email(),
        "to": message.to_email(),
        "raw_content": general_purpose::STANDARD.encode(message.raw_bytes().await.unwrap()),
        "size": message.raw_size()
    });

    let client = reqwest::Client::new();
    let _res = client
        .post(webhook_url)
        .header("content-type", "application/json")
        .body(email_data.to_string())
        .send()
        .await
        .unwrap();

    Ok(())
}
