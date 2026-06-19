use anyhow::anyhow;
use sendgrid::{Mail, SGClient};
use std::env;

pub async fn send_email_verification_code(
    display_name: &str,
    email: &str,
    code: &str,
) -> Result<(), anyhow::Error> {
    let api_key = match env::var("SENDGRID_API_KEY") {
        Ok(key) => key,
        Err(e) => {
            println!(
                "[send_email_verification_code] Failed to get API key: {:?}",
                e
            );
            return Err(anyhow!("Failed to get API key"));
        }
    };

    let from_email = match env::var("SENDGRID_FROM_EMAIL") {
        Ok(email) => email,
        Err(e) => {
            println!(
                "[send_email_verification_code] Failed to get from email: {:?}",
                e
            );
            return Err(anyhow!("Failed to get from email"));
        }
    };

    let client = SGClient::new(api_key.as_str());
    let message = format!("Your MNSTR verification code is: {}", code);
    let message = Mail::new()
        .add_text(message.as_str())
        .add_from(from_email.as_str())
        .add_subject("MNSTR Verification Code")
        .add_to((email, display_name).into());
    match client.send(message).await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!(
                "[send_email_verification_code] Failed to send email: {:?}",
                e
            );
            return Err(anyhow!("Failed to send email"));
        }
    }
}
