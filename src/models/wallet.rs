use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::{OffsetDateTime, format_description::well_known::Iso8601};

use crate::{
    database::traits::DatabaseResource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct Wallet {
    pub id: String,
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
}

impl DatabaseResource for Wallet {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = OffsetDateTime::parse(row.get("created_at"), &Iso8601::DEFAULT).ok();
        let updated_at = OffsetDateTime::parse(row.get("updated_at"), &Iso8601::DEFAULT).ok();
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => OffsetDateTime::parse(archived_at, &Iso8601::DEFAULT).ok(),
            None => None,
        };

        Ok(Wallet {
            id: row.get("id"),
            user_id: row.get("user_id"),
            created_at,
            updated_at,
            archived_at,
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
