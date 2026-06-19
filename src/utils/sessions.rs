use anyhow::Error;

use crate::models::user::User;
pub trait SessionTrait<T> {
    fn expired(&self) -> bool;
    async fn update_expired(&mut self) -> Option<anyhow::Error>;
    async fn find_one_by_token(token: String) -> Result<T, Error>;
    async fn get_user(&mut self) -> Result<User, Error>;
}

pub async fn validate_session<T: SessionTrait<T>>(session: &mut T) -> Option<anyhow::Error> {
    session.update_expired().await
}

pub async fn get_user_from_token<T: SessionTrait<T> >(token: String) -> Result<User, Error> {
    let mut session = match T::find_one_by_token(token).await {
        Ok(session) => session,
        Err(e) => return Err(e.into()),
    };
    if let Some(error) = validate_session(&mut session).await {
        return Err(error.into());
    }
    match session.get_user().await {
        Ok(user) => Ok(user),
        Err(e) => Err(e.into()),
    }
}