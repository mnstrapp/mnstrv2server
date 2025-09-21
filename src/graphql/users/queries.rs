use juniper::FieldError;

use crate::{graphql::Ctx, models::user::User};

pub struct UserQueryType;

#[juniper::graphql_object]
impl UserQueryType {
    async fn my(ctx: &Ctx) -> Result<User, FieldError> {
        get_user(ctx).await
    }

    async fn forgot_password(email: String, qr_code: String) -> Result<String, FieldError> {
        forgot_password(email, qr_code).await
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

pub async fn forgot_password(email: String, qr_code: String) -> Result<String, FieldError> {
    let user_params = vec![("email", email.into()), ("qr_code", qr_code.into())];
    match User::find_one_by(user_params).await {
        Ok(user) => Ok(user.id),
        Err(e) => {
            println!("[forgot_password] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to find user"));
        }
    }
}
