use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::{OffsetDateTime, format_description::well_known::Iso8601};

use crate::{
    database::traits::DatabaseResource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct Mnstr {
    pub id: String,
    pub mnstr_name: String,
    pub mnstr_description: String,
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
}

impl DatabaseResource for Mnstr {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = OffsetDateTime::parse(row.get("created_at"), &Iso8601::DEFAULT).ok();
        let updated_at = OffsetDateTime::parse(row.get("updated_at"), &Iso8601::DEFAULT).ok();
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => OffsetDateTime::parse(archived_at, &Iso8601::DEFAULT).ok(),
            None => None,
        };

        Ok(Mnstr {
            id: row.get("id"),
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
