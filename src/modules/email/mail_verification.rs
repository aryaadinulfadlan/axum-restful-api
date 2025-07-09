use std::error::Error;
use crate::modules::email::mailer::{create_link, send_email};

pub async fn send_verification_email(to_email: &str, name: &str, token: &str) -> Result<(), Box<dyn Error>> {
    let subject = "Email Verification";
    let template_path = "src/modules/email/templates/verification-email.html";
    let base_url = "http://localhost:8000/api/auth/verify";
    let verification_link = create_link(base_url, token);
    let placeholders = vec![
        ("{{name}}".to_string(), name.to_string()),
        ("{{verification_link}}".to_string(), verification_link)
    ];
    send_email(to_email, subject, template_path, &placeholders).await
}