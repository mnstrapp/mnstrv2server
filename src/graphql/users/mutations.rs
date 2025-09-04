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
    let mut user = User::new(email, password, display_name, qr_code);

    if let Some(error) = user.create().await {
        println!("Failed to register user: {:?}", error);
        return Err(FieldError::from("Failed to register user"));
    }

    user.get_relationships().await;

    Ok(user)
}
