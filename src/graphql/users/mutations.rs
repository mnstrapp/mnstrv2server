use juniper::FieldError;

use crate::{
    graphql::{Ctx, users::utils::send_email_verification_code},
    models::user::User,
    utils::passwords::{generate_verification_code, hash_password},
};

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

    if email != None {
        user.email_verification_code = Some(generate_verification_code());
        user.email_verified = false;
    }
    if phone != None {
        user.phone_verification_code = Some(generate_verification_code());
        user.phone_verified = false;
    }

    if let Some(error) = user.create().await {
        println!("[register] Failed to register user: {:?}", error);
        return Err(FieldError::from("Failed to register user"));
    }

    if email != None {
        if let Err(error) = send_email_verification_code(
            display_name,
            email.unwrap(),
            user.email_verification_code.unwrap(),
        )
        .await
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
    let mut user = match User::find_one(id).await {
        Ok(user) => user,
        Err(e) => {
            println!("[reset_password] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    println!("[reset_password] User: {:?}", user);

    if user.email != None && user.email_verified != true {
        return Err(FieldError::from("User email not verified"));
    }
    if user.phone != None && user.phone_verified != true {
        return Err(FieldError::from("User phone not verified"));
    }

    user.password_hash = hash_password(&password);
    if let Some(error) = user.update().await {
        println!("[reset_password] Failed to update user: {:?}", error);
        return Err(FieldError::from("Failed to update user"));
    }

    Ok(true)
}
