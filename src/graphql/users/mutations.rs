use juniper::FieldError;

use crate::{insert_resource, models::user::User, utils::passwords::hash_password};

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
}

pub async fn register(
    email: String,
    password: String,
    display_name: String,
    qr_code: String,
) -> Result<User, FieldError> {
    let password_hash = hash_password(&password);
    let params = vec![
        ("email", email.into()),
        ("password_hash", password_hash.into()),
        ("display_name", display_name.into()),
        ("qr_code", qr_code.into()),
    ];

    let mut user = match insert_resource!(User, params).await {
        Ok(user) => user,
        Err(e) => {
            println!("Failed to register user: {:?}", e);
            return Err(FieldError::from("Failed to register user"));
        }
    };

    if let Some(error) = user.create_relationships().await {
        println!("Failed to create user relationships: {:?}", error);
        return Err(FieldError::from("Failed to create user relationships"));
    }

    user.get_relationships().await;

    Ok(user)
}
