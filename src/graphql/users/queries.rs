use juniper::FieldError;

use crate::{
    graphql::{Ctx, users::utils::send_email_verification_code},
    models::user::User,
    utils::passwords::{generate_verification_code, hash_password},
};

pub struct UserQueryType;

#[juniper::graphql_object]
impl UserQueryType {
    async fn my(ctx: &Ctx) -> Result<User, FieldError> {
        get_user(ctx).await
    }

    async fn forgot_password(email: String) -> Result<String, FieldError> {
        forgot_password(email).await
    }
}

async fn get_user(ctx: &Ctx) -> Result<User, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let session = ctx.session.as_ref().unwrap().clone();

    let user = match User::find_one(session.user_id.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[get_user] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };
    Ok(user)
}

pub async fn forgot_password(email: String) -> Result<String, FieldError> {
    let user_params = vec![("email", email.into())];
    let mut user = match User::find_one_by(user_params).await {
        Ok(user) => user,
        Err(e) => {
            println!("[forgot_password] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to find user"));
        }
    };

    let code = generate_verification_code();
    user.email_verification_code = Some(code);
    if let Some(error) = user.update().await {
        println!("[forgot_password] Failed to update user: {:?}", error);
        return Err(FieldError::from("Failed to update user"));
    }

    if let Err(error) = send_email_verification_code(
        user.display_name,
        user.email.unwrap(),
        user.email_verification_code.unwrap(),
    )
    .await
    {
        println!(
            "[forgot_password] Failed to send email verification code: {:?}",
            error
        );
        return Err(FieldError::from("Failed to send email verification code"));
    }

    Ok(user.id)
}
