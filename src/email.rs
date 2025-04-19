use resend_rs::{
    types::{CreateEmailBaseOptions, CreateEmailResponse},
    Resend,
};

pub async fn send_email(
    resend: &Resend,
    email_content: &String,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["arnav@dvnl.work"];
    let subject = "Email sent from webhooks server";

    let email = CreateEmailBaseOptions::new(from, to, subject).with_text(email_content);

    resend.emails.send(email).await
}

pub async fn _send_notion_webhook_init_email(
    resend: &Resend,
    verification_token: &String,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["arnav@dvnl.work"];
    let subject = "Notion webhook verification token";

    let email = CreateEmailBaseOptions::new(from, to, subject).with_text(verification_token);

    resend.emails.send(email).await
}

pub async fn send_timesheet_email(
    resend: &Resend,
) -> Result<CreateEmailResponse, resend_rs::Error> {
    let from = "devnull03 <dev@dvnl.work>";
    let to = ["arnav.mehta@student.ufv.ca", "arnav@dvnl.work"];
    let subject = "Notion webhook verification token";

    let email = CreateEmailBaseOptions::new(from, to, subject).with_text("to be implimented");

    resend.emails.send(email).await
}
