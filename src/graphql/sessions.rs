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

    let mut session = Session::new(user.id.clone());
    if let Some(error) = session.create().await {
        println!("Failed to create session: {:?}", error);
        return Err(FieldError::from("Failed to create session"));
    };

    Ok(session)
}

pub async fn delete_session(ctx: &Ctx) -> Result<bool, FieldError> {
    if let None = ctx.session {
        return Err(FieldError::from("Invalid session"));
    }
    let mut session = ctx.session.as_ref().unwrap().clone();

    if let Some(error) = session.delete().await {
        println!("Failed to delete session: {:?}", error);
        return Err(FieldError::from("Failed to delete session"));
    }

    Ok(true)
}
