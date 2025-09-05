use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    find_all_resources_where_fields, find_one_resource_where_fields, insert_resource,
    models::transaction::{Transaction, TransactionStatus, TransactionType},
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

    // Relationships
    pub coins: i32,
}

impl Wallet {
    pub fn new(user_id: String) -> Self {
        Self {
            id: "".to_string(),
            user_id,
            created_at: None,
            updated_at: None,
            archived_at: None,
            coins: 0,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let wallet =
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
        if let Some(error) = wallet.get_relationships().await {
            println!(
                "[Wallet::find_one] Failed to get relationships: {:?}",
                error
            );
            return Err(error.into());
        }
        Ok(wallet)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let mut wallet = match find_one_resource_where_fields!(Wallet, params).await {
            Ok(wallet) => wallet,
            Err(e) => return Err(e.into()),
        };
        if let Some(error) = wallet.get_relationships().await {
            println!(
                "[Wallet::find_one_by] Failed to get relationships: {:?}",
                error
            );
            return Err(error.into());
        }
        Ok(wallet)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let mut wallets = match find_all_resources_where_fields!(Wallet, vec![]).await {
            Ok(wallets) => wallets,
            Err(e) => return Err(e.into()),
        };
        for wallet in wallets.iter_mut() {
            if let Some(error) = wallet.get_relationships().await {
                println!(
                    "[Wallet::find_all] Failed to get relationships: {:?}",
                    error
                );
                return Err(error.into());
            }
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
            if let Some(error) = wallet.get_relationships().await {
                println!(
                    "[Wallet::find_all_by] Failed to get relationships: {:?}",
                    error
                );
                return Err(error.into());
            }
        }
        Ok(wallets)
    }

    pub async fn get_relationships(&mut self) -> Option<anyhow::Error> {
        if let Some(error) = self.get_coins().await {
            return Some(error.into());
        }
        None
    }

    pub async fn get_coins(&mut self) -> Option<anyhow::Error> {
        let transactions = match find_all_resources_where_fields!(
            Transaction,
            vec![("wallet_id", self.id.clone().into())]
        )
        .await
        {
            Ok(transactions) => transactions,
            Err(e) => {
                println!("[Wallet::get_coins] Failed to get transactions: {:?}", e);
                return Some(e.into());
            }
        };
        self.coins = transactions.iter().map(|t| t.transaction_amount).sum();
        None
    }

    pub async fn add_coins(&mut self, coins: i32) -> Option<anyhow::Error> {
        println!("[Wallet::add_coins] Adding coins: {:?}", coins);
        let mut transaction = Transaction::new(self.id.clone());
        transaction.transaction_amount = coins;
        transaction.transaction_type = TransactionType::Credit;
        transaction.transaction_status = TransactionStatus::Completed;
        if let Some(error) = transaction.create().await {
            println!("Failed to create transaction: {:?}", error);
            return Some(error.into());
        }
        if let Some(error) = self.get_coins().await {
            println!("Failed to get coins: {:?}", error);
            return Some(error.into());
        }
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
            coins: 0,
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
