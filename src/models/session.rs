use anyhow::anyhow;
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields, find_one_resource_where_fields,
    insert_resource,
    models::user::User,
    update_resource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
pub struct Session {
    pub id: String,
    pub session_token: String,
    pub user_id: String,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub created_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub updated_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub archived_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub expires_at: Option<OffsetDateTime>,

    // Relationships
    pub user: Option<User>,
}

impl Session {
    pub fn new(user_id: String) -> Self {
        Self {
            id: "".to_string(),
            session_token: "".to_string(),
            user_id,
            created_at: None,
            updated_at: None,
            archived_at: None,
            expires_at: None,
            user: None,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let token = Uuid::new_v4().to_string();
        let params = vec![
            ("user_id", self.user_id.clone().into()),
            ("session_token", token.into()),
        ];
        let mut session = match insert_resource!(Session, params).await {
            Ok(session) => session,
            Err(e) => return Some(e.into()),
        };
        if let Some(error) = session.get_relationships().await {
            return Some(error);
        }

        *self = session;
        None
    }

    pub async fn update(&mut self) -> Option<anyhow::Error> {
        let mut session = match update_resource!(Session, self.id.clone(), vec![]).await {
            Ok(session) => session,
            Err(e) => return Some(e.into()),
        };
        if let Some(error) = session.get_relationships().await {
            return Some(error);
        }

        *self = session;
        None
    }

    pub async fn delete(&mut self) -> Option<anyhow::Error> {
        match delete_resource_where_fields!(Session, vec![("id", self.id.clone().into())]).await {
            Ok(_) => (),
            Err(e) => return Some(e.into()),
        };
        let session = match Self::find_one(self.id.clone()).await {
            Ok(session) => session,
            Err(e) => return Some(e.into()),
        };

        *self = session;
        None
    }

    pub async fn delete_permanent(&mut self) -> Option<anyhow::Error> {
        match delete_resource_where_fields!(Session, vec![("id", self.id.clone().into())], true)
            .await
        {
            Ok(_) => (),
            Err(e) => return Some(e.into()),
        };
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let mut session =
            match find_one_resource_where_fields!(Session, vec![("id", id.clone().into())]).await {
                Ok(session) => session,
                Err(e) => return Err(e.into()),
            };
        if let Some(error) = session.get_relationships().await {
            return Err(error.into());
        }
        Ok(session)
    }

    #[allow(dead_code)]
    pub async fn find_one_by_token(token: String) -> Result<Self, anyhow::Error> {
        let params = vec![("session_token", token.clone().into())];
        let mut session = match find_one_resource_where_fields!(Session, params).await {
            Ok(session) => session,
            Err(e) => return Err(e.into()),
        };
        if let Some(error) = session.get_relationships().await {
            return Err(error.into());
        }
        Ok(session)
    }

    #[allow(dead_code)]
    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let sessions = match find_all_resources_where_fields!(Session, vec![]).await {
            Ok(sessions) => sessions,
            Err(e) => return Err(e.into()),
        };
        Ok(sessions)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let sessions = match find_all_resources_where_fields!(Session, params).await {
            Ok(sessions) => sessions,
            Err(e) => return Err(e.into()),
        };
        Ok(sessions)
    }

    pub async fn get_relationships(&mut self) -> Option<anyhow::Error> {
        self.get_user().await?;
        None
    }

    pub async fn get_user(&mut self) -> Option<anyhow::Error> {
        let user = match User::find_one(self.user_id.clone(), false).await {
            Ok(user) => user,
            Err(e) => return Some(e),
        };
        self.user = Some(user);
        None
    }
}

impl DatabaseResource for Session {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => archived_at,
            None => None,
        };
        let expires_at = match row.get("expires_at") {
            Some(expires_at) => expires_at,
            None => None,
        };

        Ok(Session {
            id: row.get("id"),
            session_token: row.get("session_token"),
            user_id: row.get("user_id"),
            created_at,
            updated_at,
            archived_at,
            expires_at,
            user: None,
        })
    }

    fn has_id() -> bool {
        true
    }

    fn is_archivable() -> bool {
        true
    }

    fn is_updatable() -> bool {
        true
    }

    fn is_creatable() -> bool {
        true
    }

    fn is_expirable() -> bool {
        true
    }

    fn is_verifiable() -> bool {
        false
    }
}

impl crate::utils::sessions::Session for Session {
    fn expired(&self) -> bool {
        self.expires_at.is_some() && self.expires_at.unwrap() < OffsetDateTime::now_utc()
    }

    async fn update_expired(&mut self) -> Option<anyhow::Error> {
        if let Some(error) = self.update().await {
            return Some(error);
        }
        None
    }
}
