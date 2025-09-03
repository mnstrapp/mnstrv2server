use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::traits::DatabaseResource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
pub struct Mnstr {
    pub id: String,
    pub user_id: String,

    #[graphql(name = "name")]
    pub mnstr_name: String,

    #[graphql(name = "description")]
    pub mnstr_description: String,

    #[graphql(name = "qrCode")]
    pub mnstr_qr_code: String,

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

    pub current_level: i32,
    pub current_experience: i32,
    pub current_health: i32,
    pub max_health: i32,
    pub current_attack: i32,
    pub max_attack: i32,
    pub current_defense: i32,
    pub max_defense: i32,
    pub current_speed: i32,
    pub max_speed: i32,
    pub current_intelligence: i32,
    pub max_intelligence: i32,
    pub current_magic: i32,
    pub max_magic: i32,
    // Relationships
}

impl Mnstr {
    pub async fn get_relationships(&mut self) -> Option<Error> {
        None
    }
}

impl DatabaseResource for Mnstr {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => archived_at,
            None => None,
        };

        Ok(Mnstr {
            id: row.get("id"),
            user_id: row.get("user_id"),
            mnstr_name: row.get("mnstr_name"),
            mnstr_description: row.get("mnstr_description"),
            mnstr_qr_code: row.get("mnstr_qr_code"),
            created_at,
            updated_at,
            archived_at,
            current_level: row.get("current_level"),
            current_experience: row.get("current_experience"),
            current_health: row.get("current_health"),
            max_health: row.get("max_health"),
            current_attack: row.get("current_attack"),
            max_attack: row.get("max_attack"),
            current_defense: row.get("current_defense"),
            max_defense: row.get("max_defense"),
            current_speed: row.get("current_speed"),
            max_speed: row.get("max_speed"),
            current_intelligence: row.get("current_intelligence"),
            max_intelligence: row.get("max_intelligence"),
            current_magic: row.get("current_magic"),
            max_magic: row.get("max_magic"),
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
        false
    }
    fn is_verifiable() -> bool {
        false
    }
}
