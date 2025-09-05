use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};
use sqlx::{
    Encode, Error, Postgres, Row,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgRow, PgValueRef},
};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields, find_one_resource_where_fields,
    insert_resource,
    models::wallet::Wallet,
    update_resource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, Clone)]
pub enum TransactionType {
    Credit,
    Debit,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Credit => write!(f, "credit"),
            TransactionType::Debit => write!(f, "debit"),
        }
    }
}

impl From<&str> for TransactionType {
    fn from(transaction_type: &str) -> Self {
        match transaction_type {
            "credit" => TransactionType::Credit,
            "debit" => TransactionType::Debit,
            _ => TransactionType::Credit,
        }
    }
}

impl sqlx::Decode<'_, Postgres> for TransactionType {
    fn decode(
        value: PgValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(TransactionType::from(value.as_str()?))
    }
}

impl sqlx::Type<Postgres> for TransactionType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum, Clone)]
pub enum TransactionStatus {
    Preparing,
    Pending,
    Completed,
    Failed,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Preparing => write!(f, "preparing"),
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Completed => write!(f, "completed"),
            TransactionStatus::Failed => write!(f, "failed"),
        }
    }
}

impl From<&str> for TransactionStatus {
    fn from(transaction_status: &str) -> Self {
        match transaction_status {
            "preparing" => TransactionStatus::Preparing,
            "pending" => TransactionStatus::Pending,
            "completed" => TransactionStatus::Completed,
            "failed" => TransactionStatus::Failed,
            _ => TransactionStatus::Preparing,
        }
    }
}

impl sqlx::Decode<'_, Postgres> for TransactionStatus {
    fn decode(
        value: PgValueRef,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(TransactionStatus::from(value.as_str()?))
    }
}

impl sqlx::Type<Postgres> for TransactionStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("VARCHAR")
    }
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
pub struct Transaction {
    pub id: String,
    pub wallet_id: String,
    pub transaction_type: TransactionType,
    pub transaction_amount: i32,
    pub transaction_status: TransactionStatus,
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

impl Transaction {
    pub fn new(wallet_id: String) -> Self {
        Self {
            id: "".to_string(),
            wallet_id,
            transaction_type: TransactionType::Credit,
            transaction_amount: 0,
            transaction_status: TransactionStatus::Preparing,
            transaction_data: None,
            error_message: None,
            created_at: None,
            updated_at: None,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            ("wallet_id", self.wallet_id.clone().into()),
            (
                "transaction_type",
                self.transaction_type.clone().to_string().into(),
            ),
            ("transaction_amount", self.transaction_amount.clone().into()),
            (
                "transaction_status",
                self.transaction_status.clone().to_string().into(),
            ),
            (
                "transaction_data",
                self.transaction_data
                    .clone()
                    .unwrap_or("".to_string())
                    .into(),
            ),
            (
                "error_message",
                self.error_message.clone().unwrap_or("".to_string()).into(),
            ),
        ];
        let transaction = match insert_resource!(Transaction, params).await {
            Ok(transaction) => transaction,
            Err(e) => {
                println!(
                    "[Transaction::create] Failed to create transaction: {:?}",
                    e
                );
                return Some(e.into());
            }
        };
        *self = transaction;
        None
    }

    pub async fn update(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            (
                "transaction_type",
                self.transaction_type.clone().to_string().into(),
            ),
            (
                "transaction_amount",
                self.transaction_amount.clone().to_string().into(),
            ),
            (
                "transaction_status",
                self.transaction_status.clone().to_string().into(),
            ),
            (
                "transaction_data",
                self.transaction_data
                    .clone()
                    .unwrap_or("".to_string())
                    .into(),
            ),
            (
                "error_message",
                self.error_message.clone().unwrap_or("".to_string()).into(),
            ),
        ];
        let transaction = match update_resource!(Transaction, self.id.clone(), params).await {
            Ok(transaction) => transaction,
            Err(e) => {
                println!(
                    "[Transaction::update] Failed to update transaction: {:?}",
                    e
                );
                return Some(e.into());
            }
        };
        *self = transaction;
        None
    }

    pub async fn delete(&mut self) -> Option<anyhow::Error> {
        match delete_resource_where_fields!(Transaction, vec![("id", self.id.clone().into())]).await
        {
            Ok(_) => (),
            Err(e) => {
                println!(
                    "[Transaction::delete] Failed to delete transaction: {:?}",
                    e
                );
                return Some(e.into());
            }
        };
        let transaction = match Self::find_one(self.id.clone()).await {
            Ok(transaction) => transaction,
            Err(e) => {
                println!("[Transaction::delete] Failed to find transaction: {:?}", e);
                return Some(e.into());
            }
        };
        *self = transaction;
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let mut transaction =
            match find_one_resource_where_fields!(Transaction, vec![("id", id.clone().into())])
                .await
            {
                Ok(transaction) => transaction,
                Err(e) => {
                    println!(
                        "[Transaction::find_one] Failed to find transaction: {:?}",
                        e
                    );
                    return Err(e.into());
                }
            };
        if let Some(error) = transaction.get_relationships().await {
            println!(
                "[Transaction::find_one] Failed to get relationships: {:?}",
                error
            );
            return Err(error.into());
        }
        Ok(transaction)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let mut transaction = match find_one_resource_where_fields!(Transaction, params).await {
            Ok(transaction) => transaction,
            Err(e) => {
                println!(
                    "[Transaction::find_one_by] Failed to find transaction: {:?}",
                    e
                );
                return Err(e.into());
            }
        };
        if let Some(error) = transaction.get_relationships().await {
            println!(
                "[Transaction::find_one_by] Failed to get relationships: {:?}",
                error
            );
            return Err(error.into());
        }
        Ok(transaction)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let mut transactions = match find_all_resources_where_fields!(Transaction, vec![]).await {
            Ok(transactions) => transactions,
            Err(e) => {
                println!(
                    "[Transaction::find_all] Failed to find transactions: {:?}",
                    e
                );
                return Err(e.into());
            }
        };
        for transaction in transactions.iter_mut() {
            if let Some(error) = transaction.get_relationships().await {
                println!(
                    "[Transaction::find_all] Failed to get relationships: {:?}",
                    error
                );
                return Err(error.into());
            }
        }
        Ok(transactions)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut transactions = match find_all_resources_where_fields!(Transaction, params).await {
            Ok(transactions) => transactions,
            Err(e) => {
                println!(
                    "[Transaction::find_all_by] Failed to find transactions: {:?}",
                    e
                );
                return Err(e.into());
            }
        };
        for transaction in transactions.iter_mut() {
            if let Some(error) = transaction.get_relationships().await {
                println!(
                    "[Transaction::find_all_by] Failed to get relationships: {:?}",
                    error
                );
                return Err(error.into());
            }
        }
        Ok(transactions)
    }

    pub async fn get_relationships(&mut self) -> Option<anyhow::Error> {
        None
    }
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
