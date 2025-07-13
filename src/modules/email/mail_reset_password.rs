use std::error::Error;
use crate::modules::email::mailer::{create_link, send_email};

pub async fn send_forgot_password_email(to_email: &str, name: &str, token: &str) -> Result<(), Box<dyn Error>> {
    let subject = "Reset your Password";
    let template_path = "src/modules/email/templates/reset-password-email.html";
    let base_url = "http://localhost:4000/api/auth/reset-password";
    let reset_link = create_link(base_url, token);
    let placeholders = vec![
        ("{{name}}".to_string(), name.to_string()),
        ("{{reset_link}}".to_string(), reset_link.to_string())
    ];
    send_email(to_email, subject, template_path, &placeholders).await
}