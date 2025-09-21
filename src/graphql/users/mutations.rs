use juniper::FieldError;

use crate::{graphql::Ctx, models::user::User, utils::passwords::hash_password};

pub struct UserMutationType;

#[juniper::graphql_object]
impl UserMutationType {
    async fn register(
        email: String,
        password: String,
        display_name: String,
        qr_code: String,
    ) -> Result<User, FieldError> {
        register(email, password, display_name, qr_code).await
    }

    async fn unregister(ctx: &Ctx) -> Result<bool, FieldError> {
        unregister(ctx).await
    }

    async fn reset_password(id: String, password: String) -> Result<bool, FieldError> {
        reset_password(id, password).await
    }
}

pub async fn register(
    email: String,
    password: String,
    display_name: String,
    qr_code: String,
) -> Result<User, FieldError> {
    let mut user = User::new(email, password, display_name, qr_code);

    if let Some(error) = user.create().await {
        println!("[register] Failed to register user: {:?}", error);
        return Err(FieldError::from("Failed to register user"));
    }

    let user = match User::find_one(user.id.clone()).await {
        Ok(user) => user,
        Err(e) => {
            println!("[register] Failed to get user: {:?}", e);
            return Err(FieldError::from("Failed to get user"));
        }
    };

    Ok(user)
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
