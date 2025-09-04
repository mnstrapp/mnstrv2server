use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    find_all_resources_where_fields, find_one_resource_where_fields, insert_resource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
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

impl Wallet {
    pub fn new(user_id: String) -> Self {
        Self {
            id: "".to_string(),
            user_id,
            created_at: None,
            updated_at: None,
            archived_at: None,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let mut wallet =
            match insert_resource!(Wallet, vec![("user_id", self.user_id.clone().into())]).await {
                Ok(wallet) => wallet,
                Err(e) => return Some(e.into()),
            };
        *self = wallet;
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let mut wallet =
            match find_one_resource_where_fields!(Wallet, vec![("id", id.clone().into())]).await {
                Ok(wallet) => wallet,
                Err(e) => return Err(e.into()),
            };
        wallet
            .get_relationships()
            .await
            .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        Ok(wallet)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let mut wallet = match find_one_resource_where_fields!(Wallet, params).await {
            Ok(wallet) => wallet,
            Err(e) => return Err(e.into()),
        };
        wallet
            .get_relationships()
            .await
            .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        Ok(wallet)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let mut wallets = match find_all_resources_where_fields!(Wallet, vec![]).await {
            Ok(wallets) => wallets,
            Err(e) => return Err(e.into()),
        };
        for wallet in wallets.iter_mut() {
            wallet
                .get_relationships()
                .await
                .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        }
        Ok(wallets)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut wallets = match find_all_resources_where_fields!(Wallet, params).await {
            Ok(wallets) => wallets,
            Err(e) => return Err(e.into()),
        };
        for wallet in wallets.iter_mut() {
            wallet
                .get_relationships()
                .await
                .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        }
        Ok(wallets)
    }

    pub async fn get_relationships(&mut self) -> Option<anyhow::Error> {
        None
    }
}

impl DatabaseResource for Wallet {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => archived_at,
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
