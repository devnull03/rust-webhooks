use resend_rs::{
    types::{Attachment, CreateEmailBaseOptions, CreateEmailResponse},
    Resend,
};
use tracing::{error, info};

pub async fn send_email(
    resend: &Resend,
    email_content: &str,
    subject_: Option<&str>,
    attachment: Option<Attachment>,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["arnav@dvnl.work"];
    let subject = subject_.unwrap_or("Email sent from webhooks server");

    info!("Preparing to send email with subject: {}", subject);

    let mut email = CreateEmailBaseOptions::new(from, to, subject).with_text(email_content);

    if let Some(attachment) = attachment {
        email = email.with_attachment(attachment);
    }

    let result = resend.emails.send(email).await;
    match &result {
        Ok(response) => info!("Email sent successfully with ID: {}", response.id),
        Err(e) => error!("Failed to send email: {}", e),
    }

    result
}

pub async fn _send_error_info(
    resend: &Resend,
    error_info: &str,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["dev@dvnl.work"];
    let subject = "Error from webhooks server";

    info!("Sending error information email");
    info!("Error details: {}", error_info);

    let email = CreateEmailBaseOptions::new(from, to, subject).with_text(error_info);

    let result = resend.emails.send(email).await;
    match &result {
        Ok(response) => info!(
            "Error info email sent successfully with ID: {}",
            response.id
        ),
        Err(e) => error!("Failed to send error info email: {}", e),
    }

    result
}
