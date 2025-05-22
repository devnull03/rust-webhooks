use worker::*;
use reqwest;

#[event(email)]
async fn main(
    message: EmailMessage,
    _env: Env,
    _ctx: Context,
) -> Result<()> {
    console_error_panic_hook::set_once();

    message.forward("dev@dvnl.work".to_string(), None).await?;

    let client = reqwest::Client::new();
    let _res = client.post("https://hooks.dvnl.work/cloudflare-job-alert-reciever")
        .body("hehe")
        .send()
        .await.unwrap();

    Ok(())
}

