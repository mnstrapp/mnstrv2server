pub trait Session {
    fn expired(&self) -> bool;
}

pub fn validate_session<T: Session>(session: Option<T>) -> Result<(), String> {
    if session.is_none() {
        return Err("Session not found".to_string());
    }

    let session = session.unwrap();

    if session.expired() {
        return Err("Session expired".to_string());
    }

    Ok(())
}
