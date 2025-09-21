use std::env;

use juniper::FieldError;
use rand::Rng;
use sendgrid::{Mail, SGClient};
use twilio::{Client, OutboundMessage};

use crate::{graphql::Ctx, models::user::User, utils::passwords::hash_password};

pub struct UserMutationType;

#[juniper::graphql_object]
impl UserMutationType {
    async fn register(
        email: Option<String>,
        phone: Option<String>,
        password: String,
        display_name: String,
    ) -> Result<User, FieldError> {
        register(email, phone, password, display_name).await
    }

    async fn verify_email(id: String, code: String) -> Result<bool, FieldError> {
        verify_email(id, code).await
    }

    async fn verify_phone(id: String, code: String) -> Result<bool, FieldError> {
        verify_phone(id, code).await
    }

    async fn unregister(ctx: &Ctx) -> Result<bool, FieldError> {
        unregister(ctx).await
    }

    async fn reset_password(id: String, password: String) -> Result<bool, FieldError> {
        reset_password(id, password).await
    }
}

pub async fn register(
    email: Option<String>,
    phone: Option<String>,
    password: String,
    display_name: String,
) -> Result<User, FieldError> {
    let mut user = User::new(email.clone(), phone.clone(), password, display_name.clone());

    let code = match generate_verification_code().await {
        Ok(code) => code,
        Err(e) => {
            println!("[register] Failed to generate verification code: {:?}", e);
            return Err(FieldError::from("Failed to generate verification code"));
        }
    };

    if email != None {
        user.email_verification_code = Some(code.to_string());
        user.email_verified = false;
    }
    if phone != None {
        user.phone_verification_code = Some(code.to_string());
        user.phone_verified = false;
    }

    if let Some(error) = user.create().await {
        println!("[register] Failed to register user: {:?}", error);
        return Err(FieldError::from("Failed to register user"));
    }

    if email != None {
        if let Err(error) =
            send_email_verification_code(display_name, email.unwrap(), code.clone()).await
        {
            println!(
                "[register] Failed to send email verification code: {:?}",
                error
            );
            return Err(FieldError::from("Failed to send email verification code"));
        }
    }

    // if phone != None {
    //     if let Err(error) = send_phone_verification_code(phone.unwrap(), code.clone()).await {
    //         println!(
    //             "[register] Failed to send phone verification code: {:?}",
    //             error
    //         );
    //         return Err(FieldError::from("Failed to send phone verification code"));
    //     }
    // }

    let user = match User::find_one(user.id.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[register] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    Ok(user)
}

pub async fn verify_email(id: String, code: String) -> Result<bool, FieldError> {
    let user_params = vec![("id", id.into()), ("email_verification_code", code.into())];
    let mut user = match User::find_one_by(user_params).await {
        Ok(user) => user,
        Err(e) => {
            println!("[verify_email] Failed to get user: {:?}", e);
            return Err(FieldError::from(
                "Failed to get user with verification code",
            ));
        }
    };

    user.email_verification_code = None;
    user.email_verified = true;
    if let Some(error) = user.update().await {
        println!("[verify_email] Failed to update user: {:?}", error);
        return Err(FieldError::from("Failed to update user email verification"));
    }
    Ok(true)
}

pub async fn verify_phone(id: String, code: String) -> Result<bool, FieldError> {
    let user_params = vec![("id", id.into()), ("phone_verification_code", code.into())];
    let mut user = match User::find_one_by(user_params).await {
        Ok(user) => user,
        Err(e) => {
            println!("[verify_phone] Failed to get user: {:?}", e);
            return Err(FieldError::from(
                "Failed to get user with verification code",
            ));
        }
    };

    user.phone_verification_code = None;
    user.phone_verified = true;
    if let Some(error) = user.update().await {
        println!("[verify_phone] Failed to update user: {:?}", error);
        return Err(FieldError::from("Failed to update user phone verification"));
    }
    Ok(true)
}

pub async fn unregister(ctx: &Ctx) -> Result<bool, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    let mut user = match User::find_one(session.user_id.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[unregister] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    if let Some(error) = user.delete_permanent().await {
        println!("[unregister] Failed to delete user: {:?}", error);
        return Err(FieldError::from("Failed to delete user"));
    }

    Ok(true)
}

pub async fn reset_password(id: String, password: String) -> Result<bool, FieldError> {
    let user = match User::find_one(id).await {
        Ok(user) => user,
        Err(e) => {
            println!("[reset_password] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    let mut user = user.clone();
    user.password_hash = hash_password(&password);
    if let Some(error) = user.update().await {
        println!("[reset_password] Failed to update user: {:?}", error);
        return Err(FieldError::from("Failed to update user"));
    }

    Ok(true)
}

async fn generate_verification_code() -> Result<String, FieldError> {
    let code = rand::rng().random_range(10000..99999);
    Ok(code.to_string())
}

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

async fn send_email_verification_code(
    display_name: String,
    email: String,
    code: String,
) -> Result<bool, FieldError> {
    let client = SGClient::new(env::var("SENDGRID_API_KEY").unwrap().as_str());
    let message = format!("Your MNSTR verification code is: {}", code);
    let message = Mail::new()
        .add_text(message.as_str())
        .add_from("MNSTR <mnstrappdev@gmail.com>")
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
