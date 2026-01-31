pub trait Session {
    fn expired(&self) -> bool;
    async fn update_expired(&mut self) -> Option<anyhow::Error>;
}

pub async fn validate_session<T: Session>(session: &mut T) -> Option<anyhow::Error> {
    session.update_expired().await
}
