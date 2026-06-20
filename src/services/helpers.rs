use anyhow::Error;
use crate::{models::{session::Session, user::User}, utils::sessions::validate_session};

pub async fn get_user_from_token(token: String) -> Result<User, Error> {
    let mut session = match Session::find_one_by_token(token).await {
        Ok(session) => session,
        Err(e) => return Err(e),
    };
    if let Some(error) = validate_session(&mut session).await {
        return Err(error.into());
    }
    match session.user {
        Some(user) => Ok(user),
        None => Err(anyhow::anyhow!("User not found")),
    }
}