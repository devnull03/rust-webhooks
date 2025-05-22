use worker::*;

#[event(email)]
async fn main(
    message: EmailMessage,
    _env: Env,
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
    
    // Test request to httpbin to verify outbound connectivity
    let test_url = "https://httpbin.org/post";
    console_log!("Testing outbound connectivity with: {}", test_url);
    
    // Create RequestInit objects first
    let test_init = RequestInit {
        method: Method::Post,
        headers: headers.clone(),
        body: Some(wasm_bindgen::JsValue::from_str("Test request from Cloudflare worker").into()),
        ..Default::default()
    };
    
    // Create test request with a reference to the init object
    let test_request = Request::new_with_init(test_url, &test_init)?;
    
    // Send test request
    match Fetch::Request(test_request).send().await {
        Ok(mut test_resp) => {  // Changed to mut here
            console_log!("Test request successful! Status: {}", test_resp.status_code());
            match test_resp.text().await {
                Ok(text) => console_log!("Test response body: {}", text),
                Err(e) => console_log!("Could not read test response body: {:?}", e)
            }
        },
        Err(e) => console_log!("Test request failed: {:?}", e)
    }
    
    // Create webhook RequestInit
    let webhook_init = RequestInit {
        method: Method::Post,
        headers,
        body: Some(wasm_bindgen::JsValue::from_str("Email job alert received from Cloudflare worker").into()),
        ..Default::default()
    };
    
    // Create actual webhook request with a reference to the init object
    let webhook_request = Request::new_with_init(webhook_url, &webhook_init)?;
    
    // Send webhook request
    match Fetch::Request(webhook_request).send().await {
        Ok(mut resp) => {  // Changed to mut here
            let status = resp.status_code();
            console_log!("Webhook sent successfully. Status: {}", status);
            
            match resp.text().await {
                Ok(text) => console_log!("Response body: {}", text),
                Err(e) => console_log!("Could not read response body: {:?}", e)
            }
        },
        Err(e) => {
            console_log!("Failed to send webhook: {:?}", e);
        }
    }

    Ok(())
}

