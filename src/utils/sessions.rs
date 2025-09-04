pub trait Session {
    fn expired(&self) -> bool;
    async fn update_expired(&mut self) -> Option<anyhow::Error>;
}

pub async fn validate_session<T: Session>(session: &mut T) -> Option<anyhow::Error> {
    if session.expired() {
        return Some(anyhow::anyhow!("Session expired"));
    }

    session.update_expired().await
}
