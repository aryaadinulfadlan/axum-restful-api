use std::error::Error;
use crate::modules::email::mailer::send_email;

pub async fn send_welcome_email(to_email: &str, name: &str) -> Result<(), Box<dyn Error>> {
    let subject = "Welcome to Application";
    let template_path = "src/modules/email/templates/welcome-email.html";
    let placeholders = vec![
        ("{{name}}".to_string(), name.to_string())
    ];
    send_email(to_email, subject, template_path, &placeholders).await
}