use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::traits::DatabaseResource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct Transaction {
    pub id: String,
    pub wallet_id: String,
    pub transaction_type: String,
    pub transaction_amount: i32,
    pub transaction_status: String,
    pub transaction_data: Option<String>,
    pub error_message: Option<String>,

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
}

impl DatabaseResource for Transaction {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");

        Ok(Transaction {
            id: row.get("id"),
            wallet_id: row.get("wallet_id"),
            transaction_type: row.get("transaction_type"),
            transaction_amount: row.get("transaction_amount"),
            transaction_status: row.get("transaction_status"),
            transaction_data: row.get("transaction_data"),

            error_message: row.get("error_message"),
            created_at,
            updated_at,
        })
    }
    fn has_id() -> bool {
        true
    }
    fn is_archivable() -> bool {
        false
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
