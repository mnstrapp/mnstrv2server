use anyhow::{Error, anyhow};

use crate::{models::session::Session, utils::sessions::validate_session, utils::token::RawToken};

pub async fn verify_session_token(token: RawToken) -> Result<Session, Error> {
    let mut session = match Session::find_one_by_token(token.value).await {
        Ok(session) => session,
        Err(e) => return Err(e.into()),
    };
    if validate_session(&mut session).await.is_some() {
        return Err(anyhow!("Invalid session"));
    }
    Ok(session)
}
