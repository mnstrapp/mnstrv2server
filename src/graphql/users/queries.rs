use juniper::FieldError;

use crate::{graphql::Ctx, models::user::User};

pub struct UserQueryType;

#[juniper::graphql_object]
impl UserQueryType {
    async fn my(ctx: &Ctx) -> Result<User, FieldError> {
        get_user(ctx).await
    }
}

async fn get_user(ctx: &Ctx) -> Result<User, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let mut session = ctx.session.as_ref().unwrap().clone();

    let user = match User::find_one(session.user_id.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[get_user] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };
    Ok(user)
}
