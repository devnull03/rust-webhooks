use chrono::Datelike;
use resend_rs::{
    types::{CreateEmailBaseOptions, CreateEmailResponse},
    Resend,
};
use tracing::{info, error};

use crate::notion;

pub async fn _send_email(
    resend: &Resend,
    email_content: &String,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["arnav@dvnl.work"];
    let subject = "Email sent from webhooks server";

    info!("Preparing to send email with subject: {}", subject);
    
    let email = CreateEmailBaseOptions::new(from, to, subject).with_text(email_content);

    let result = resend.emails.send(email).await;
    match &result {
        Ok(response) => info!("Email sent successfully with ID: {}", response.id),
        Err(e) => error!("Failed to send email: {}", e),
    }
    
    result
}

pub async fn _send_notion_webhook_init_email(
    resend: &Resend,
    verification_token: &String,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["arnav@dvnl.work"];
    let subject = "Notion webhook verification token";

    info!("Sending Notion webhook verification token email");
    
    let email = CreateEmailBaseOptions::new(from, to, subject).with_text(verification_token);

    let result = resend.emails.send(email).await;
    match &result {
        Ok(response) => info!("Verification email sent successfully with ID: {}", response.id),
        Err(e) => error!("Failed to send verification email: {}", e),
    }
    
    result
}

pub async fn send_timesheet_email(
    resend: &Resend,
    timesheet: Vec<u8>,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["arnav.mehta@student.ufv.ca", "arnav@dvnl.work"];

    let period = notion::utils::get_current_pay_period();
    info!("Sending timesheet for pay period: {:?} to {:?}", period.0, period.1);

    let subject = format!(
        "Timesheet {}/{} to {}/{} - Arnav Mehta",
        period.0.month(),
        period.0.day(),
        period.1.month(),
        period.1.day()
    );

    info!("Preparing email with subject: {}", subject);
    info!("Timesheet attachment size: {} bytes", timesheet.len());
    
    let email = CreateEmailBaseOptions::new(from, to, subject).with_attachment(timesheet);

    let result = resend.emails.send(email).await;
    match &result {
        Ok(response) => info!("Timesheet email sent successfully with ID: {}", response.id),
        Err(e) => error!("Failed to send timesheet email: {}", e),
    }
    
    result
}
