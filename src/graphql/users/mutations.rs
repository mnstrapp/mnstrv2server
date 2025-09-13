use juniper::FieldError;

use crate::models::user::User;

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
