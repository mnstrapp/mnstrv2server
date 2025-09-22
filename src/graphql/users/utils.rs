use std::env;

use juniper::FieldError;
use sendgrid::{Mail, SGClient};
use twilio::{Client, OutboundMessage};

async fn send_phone_verification_code(phone: String, code: String) -> Result<bool, FieldError> {
    let client = Client::new(
        env::var("TWILIO_ACCOUNT_SSID").unwrap().as_str(),
        env::var("TWILIO_AUTH_TOKEN").unwrap().as_str(),
    );
    let message = format!("Your MNSTR verification code is: {}", code);
    match client
        .send_message(OutboundMessage::new(
            env::var("TWILIO_PHONE_NUMBER").unwrap().as_str(),
            phone.as_str(),
            message.as_str(),
        ))
        .await
    {
        Ok(_) => Ok(true),
        Err(e) => {
            println!(
                "[send_phone_verification_code] Failed to send message: {:?}",
                e
            );
            return Err(FieldError::from("Failed to send message"));
        }
    }
}

pub async fn send_email_verification_code(
    display_name: String,
    email: String,
    code: String,
) -> Result<bool, FieldError> {
    let api_key = match env::var("SENDGRID_API_KEY") {
        Ok(key) => key,
        Err(e) => {
            println!(
                "[send_email_verification_code] Failed to get API key: {:?}",
                e
            );
            return Err(FieldError::from("Failed to get API key"));
        }
    };

    let from_email = match env::var("SENDGRID_FROM_EMAIL") {
        Ok(email) => email,
        Err(e) => {
            println!(
                "[send_email_verification_code] Failed to get from email: {:?}",
                e
            );
            return Err(FieldError::from("Failed to get from email"));
        }
    };

    let client = SGClient::new(api_key.as_str());
    let message = format!("Your MNSTR verification code is: {}", code);
    let message = Mail::new()
        .add_text(message.as_str())
        .add_from(from_email.as_str())
        .add_subject("MNSTR Verification Code")
        .add_to((email.as_str(), display_name.as_str()).into());
    match client.send(message).await {
        Ok(_) => Ok(true),
        Err(e) => {
            println!(
                "[send_email_verification_code] Failed to send email: {:?}",
                e
            );
            return Err(FieldError::from("Failed to send email"));
        }
    }
}
