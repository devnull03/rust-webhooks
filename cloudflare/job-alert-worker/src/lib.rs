use worker::*;

#[event(email)]
async fn main(message: EmailMessage, _env: Env, _ctx: Context) -> Result<()> {
    console_error_panic_hook::set_once();

    console_log!("Received email, forwarding and sending webhook");

    message.forward("arnav@dvnl.work".to_string(), None).await?;

    let webhook_url = "https://hooks.dvnl.work/cloudflare-job-alert-reciever/";

    console_log!("Attempting to send webhook to: {}", webhook_url);

    let client = reqwest::Client::new();
    let _res = client
        .post("https://hooks.dvnl.work/cloudflare-job-alert-reciever")
        .body("hehe")
        .send()
        .await
        .unwrap();

    Ok(())
}
