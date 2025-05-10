
pub async fn optum() -> Result<String, Box<dyn std::error::Error>> {
    let body = reqwest::get("").await?.text().await?;
    Ok(body)
}

