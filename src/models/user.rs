use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::traits::DatabaseResource,
    find_all_resources_where_fields, find_one_resource_where_fields, insert_resource,
    models::{mnstr::Mnstr, wallet::Wallet},
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub qr_code: String,

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
    pub wallet: Option<Wallet>,
    pub mnstrs: Option<Vec<Mnstr>>,
}

impl User {
    pub async fn get_relationships(&mut self) -> Option<Error> {
        if let Some(error) = self.get_wallet().await {
            return Some(error);
        }
        if let Some(error) = self.get_mnstrs().await {
            return Some(error);
        }
        None
    }

    pub async fn get_wallet(&mut self) -> Option<Error> {
        let wallet = match find_one_resource_where_fields!(
            Wallet,
            vec![("user_id", self.id.clone().into())]
        )
        .await
        {
            Ok(wallet) => wallet,
            Err(e) => return Some(e),
        };
        self.wallet = Some(wallet);
        None
    }

    pub async fn get_mnstrs(&mut self) -> Option<Error> {
        let mnstrs = match find_all_resources_where_fields!(
            Mnstr,
            vec![("user_id", self.id.clone().into())]
        )
        .await
        {
            Ok(mnstrs) => mnstrs,
            Err(e) => return Some(e),
        };
        self.mnstrs = Some(mnstrs);
        None
    }

    pub async fn create_relationships(&mut self) -> Option<Error> {
        println!("Creating wallet for user: {:?}", self.id);
        if let Some(error) = self.create_wallet().await {
            return Some(error);
        }
        println!("Creating mnstr for user: {:?}", self.id);
        if let Some(error) = self.create_mnstr().await {
            return Some(error);
        }
        None
    }

    pub async fn create_wallet(&mut self) -> Option<Error> {
        let params = vec![("user_id", self.id.clone().into())];
        let wallet = match find_one_resource_where_fields!(Wallet, params).await {
            Ok(wallet) => Some(wallet),
            Err(_) => None,
        };
        if wallet.is_some() {
            self.wallet = wallet;
            return None;
        }

        let params = vec![("user_id", self.id.clone().into())];
        let wallet = match insert_resource!(Wallet, params).await {
            Ok(wallet) => wallet,
            Err(e) => {
                println!("Failed to create wallet: {:?}", e);
                return Some(e);
            }
        };
        self.wallet = Some(wallet);
        None
    }

    pub async fn create_mnstr(&mut self) -> Option<Error> {
        let params = vec![
            ("user_id", self.id.clone().into()),
            ("mnstr_qr_code", self.qr_code.clone().into()),
        ];
        let mnstr = match find_one_resource_where_fields!(Mnstr, params).await {
            Ok(mnstr) => Some(mnstr),
            Err(_) => None,
        };
        if mnstr.is_some() {
            let mut mnstr = mnstr.clone().unwrap();
            mnstr.get_relationships().await?;
            self.mnstrs = Some(vec![mnstr]);
            return None;
        }

        let params = vec![
            ("user_id", self.id.clone().into()),
            ("mnstr_name", self.display_name.clone().into()),
            (
                "mnstr_description",
                format!("{}'s Mnstr", self.display_name.clone()).into(),
            ),
            ("mnstr_qr_code", self.qr_code.clone().into()),
        ];
        let mut mnstr = match insert_resource!(Mnstr, params).await {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("Failed to create mnstr: {:?}", e);
                return Some(e);
            }
        };

        mnstr.get_relationships().await?;

        self.mnstrs = Some(vec![mnstr]);
        None
    }
}

impl DatabaseResource for User {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => archived_at,
            None => None,
        };

        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            password_hash: row.get("password_hash"),
            qr_code: row.get("qr_code"),
            created_at,
            updated_at,
            archived_at,
            wallet: None,
            mnstrs: None,
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
