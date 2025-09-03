use juniper::FieldError;
use uuid::Uuid;

use crate::{
    delete_resource_where_fields, find_one_unarchived_resource_where_fields,
    graphql::Ctx,
    insert_resource,
    models::{session::Session, user::User},
    utils::{passwords::hash_password, sessions::validate_session},
};

pub struct SessionMutationType;

#[juniper::graphql_object]
impl SessionMutationType {
    async fn login(email: String, password: String) -> Result<Session, FieldError> {
        create_session(email, password).await
    }

    async fn logout(ctx: &Ctx) -> Result<bool, FieldError> {
        delete_session(ctx).await
    }
}

pub async fn create_session(email: String, password: String) -> Result<Session, FieldError> {
    let password_hash = hash_password(&password);
    let params = vec![
        ("email", email.into()),
        ("password_hash", password_hash.into()),
    ];

    let user = match find_one_unarchived_resource_where_fields!(User, params).await {
        Ok(user) => user,
        Err(e) => {
            println!("Invalid email or password: {:?}", e);
            return Err(FieldError::from("Invalid email or password"));
        }
    };

    let token = Uuid::new_v4().to_string();
    let params = vec![("user_id", user.id.into()), ("session_token", token.into())];
    let session = match insert_resource!(Session, params).await {
        Ok(session) => session,
        Err(e) => {
            println!("Failed to create session: {:?}", e);
            return Err(FieldError::from("Failed to create session"));
        }
    };

    Ok(session)
}

pub async fn delete_session(ctx: &Ctx) -> Result<bool, FieldError> {
    if validate_session(ctx.session.clone()).is_err() {
        return Err(FieldError::from("Invalid session"));
    }

    let session = ctx.session.as_ref().unwrap();

    match delete_resource_where_fields!(Session, vec![("id", session.id.clone().into())]).await {
        Ok(_) => Ok(true),
        Err(e) => {
            println!("Failed to delete session: {:?}", e);
            Err(FieldError::from("Failed to delete session"))
        }
    }
}
