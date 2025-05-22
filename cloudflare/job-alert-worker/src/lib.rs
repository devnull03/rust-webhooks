use worker::*;

#[event(email)]
async fn main(
    message: EmailMessage,
    env: Env,
    _ctx: Context,
) -> Result<()> {
    console_error_panic_hook::set_once();
    
    // Log that we received an email
    console_log!("Received email, forwarding and sending webhook");
    
    // Forward the email as before
    message.forward("arnav@dvnl.work".to_string(), None).await?;
    
    // Use native fetch API instead of reqwest
    let webhook_url = "https://hooks.dvnl.work/cloudflare-job-alert-reciever";
    
    // Log before attempting to send
    console_log!("Attempting to send webhook to: {}", webhook_url);
    
    // Create a simple request with headers for debugging
    let mut headers = Headers::new();
    headers.set("Content-Type", "text/plain")?;
    headers.set("User-Agent", "Cloudflare-Worker-Email-Handler")?;
    
    let request_init = RequestInit {
        method: Method::Post,
        headers,
        body: Some(Body::from("Email job alert received from Cloudflare worker")),
        ..Default::default()
    };
    
    // First try a test to httpbin to verify outbound connectivity
    let test_url = "https://httpbin.org/post";
    console_log!("Testing outbound connectivity with: {}", test_url);
    
    match Fetch::Request(Request::new_with_init(test_url, &request_init)?) {
        Ok(mut test_req) => {
            match test_req.send().await {
                Ok(test_resp) => {
                    console_log!("Test request successful! Status: {}", test_resp.status_code());
                },
                Err(e) => console_log!("Test request failed: {:?}", e),
            }
        },
        Err(e) => console_log!("Failed to create test request: {:?}", e)
    }
    
    // Now try the actual webhook
    match Fetch::Request(Request::new_with_init(webhook_url, &request_init)?) {
        Ok(mut req) => {
            match req.send().await {
                Ok(resp) => {
                    let status = resp.status_code();
                    console_log!("Webhook sent successfully. Status: {}", status);
                    
                    // Try to read response body for debugging
                    match resp.text().await {
                        Ok(text) => console_log!("Response body: {}", text),
                        Err(_) => console_log!("Could not read response body")
                    }
                },
                Err(e) => {
                    console_log!("Failed to send webhook: {:?}", e);
                }
            }
        },
        Err(e) => console_log!("Failed to create request: {:?}", e)
    }

    Ok(())
}

