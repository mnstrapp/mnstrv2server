use anyhow::anyhow;
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields, find_one_resource_where_fields,
    insert_resource,
    models::{mnstr::Mnstr, wallet::Wallet},
    update_resource,
    utils::{
        passwords::hash_password,
        time::{deserialize_offset_date_time, serialize_offset_date_time},
    },
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
    pub fn new(email: String, password: String, display_name: String, qr_code: String) -> Self {
        let password_hash = hash_password(&password);
        Self {
            id: "".to_string(),
            email,
            password_hash,
            display_name,
            qr_code,
            created_at: None,
            updated_at: None,
            archived_at: None,
            wallet: None,
            mnstrs: None,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let mut user = match insert_resource!(
            User,
            vec![
                ("email", self.email.clone().into()),
                ("password_hash", self.password_hash.clone().into()),
                ("display_name", self.display_name.clone().into()),
                ("qr_code", self.qr_code.clone().into()),
            ]
        )
        .await
        {
            Ok(user) => user,
            Err(e) => return Some(e.into()),
        };

        if let Some(error) = user.create_relationships().await {
            return Some(error);
        }

        *self = user;
        None
    }

    pub async fn update(&mut self) -> Option<anyhow::Error> {
        let mut user = match update_resource!(
            User,
            self.id.clone(),
            vec![("display_name", self.display_name.clone().into()),]
        )
        .await
        {
            Ok(user) => user,
            Err(e) => return Some(e.into()),
        };

        if let Some(error) = user.create_relationships().await {
            return Some(error);
        }

        *self = user;
        None
    }

    pub async fn delete(&mut self) -> Option<anyhow::Error> {
        match delete_resource_where_fields!(User, vec![("id", self.id.clone().into())]).await {
            Ok(_) => (),
            Err(e) => return Some(e.into()),
        };
        let user = match Self::find_one(self.id.clone()).await {
            Ok(user) => user,
            Err(e) => return Some(e.into()),
        };
        *self = user;
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let mut user =
            match find_one_resource_where_fields!(User, vec![("id", id.clone().into())]).await {
                Ok(user) => user,
                Err(e) => return Err(e.into()),
            };
        user.get_relationships()
            .await
            .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        Ok(user)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let mut user = match find_one_resource_where_fields!(User, params).await {
            Ok(user) => user,
            Err(e) => return Err(e.into()),
        };
        user.get_relationships()
            .await
            .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        Ok(user)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let mut users = match find_all_resources_where_fields!(User, vec![]).await {
            Ok(users) => users,
            Err(e) => return Err(e.into()),
        };
        for user in users.iter_mut() {
            user.get_relationships()
                .await
                .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        }
        Ok(users)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut users = match find_all_resources_where_fields!(User, params).await {
            Ok(users) => users,
            Err(e) => return Err(e.into()),
        };
        for user in users.iter_mut() {
            user.get_relationships()
                .await
                .ok_or(anyhow::anyhow!("Failed to get relationships"))?;
        }
        Ok(users)
    }

    pub async fn get_relationships(&mut self) -> Option<anyhow::Error> {
        if let Some(error) = self.get_wallet().await {
            return Some(error.into());
        }
        if let Some(error) = self.get_mnstrs().await {
            return Some(error.into());
        }
        None
    }

    pub async fn get_wallet(&mut self) -> Option<anyhow::Error> {
        let wallet = match find_one_resource_where_fields!(
            Wallet,
            vec![("user_id", self.id.clone().into())]
        )
        .await
        {
            Ok(wallet) => wallet,
            Err(e) => return Some(e.into()),
        };
        self.wallet = Some(wallet);
        None
    }

    pub async fn get_mnstrs(&mut self) -> Option<anyhow::Error> {
        let mnstrs = match find_all_resources_where_fields!(
            Mnstr,
            vec![("user_id", self.id.clone().into())]
        )
        .await
        {
            Ok(mnstrs) => mnstrs,
            Err(e) => return Some(e.into()),
        };
        self.mnstrs = Some(mnstrs);
        None
    }

    pub async fn create_relationships(&mut self) -> Option<anyhow::Error> {
        println!("Creating wallet for user: {:?}", self.id);
        if let Some(error) = self.create_wallet().await {
            return Some(error.into());
        }
        println!("Creating mnstr for user: {:?}", self.id);
        if let Some(error) = self.create_mnstr().await {
            return Some(error.into());
        }
        None
    }

    pub async fn create_wallet(&mut self) -> Option<anyhow::Error> {
        let found_wallet =
            match Wallet::find_one_by(vec![("user_id", self.id.clone().into())]).await {
                Ok(wallet) => Some(wallet),
                Err(_) => None,
            };
        if let Some(mut found_wallet) = found_wallet {
            if let Some(error) = found_wallet.get_relationships().await {
                return Some(error.into());
            }
            self.wallet = Some(found_wallet);
            return None;
        }

        let mut wallet = Wallet::new(self.id.clone());
        if let Some(error) = wallet.create().await {
            return Some(error.into());
        }
        self.wallet = Some(wallet);
        None
    }

    pub async fn create_mnstr(&mut self) -> Option<anyhow::Error> {
        let found_mnstr = match Mnstr::find_one_by(vec![("user_id", self.id.clone().into())]).await
        {
            Ok(mnstr) => Some(mnstr),
            Err(_) => None,
        };
        if let Some(mut found_mnstr) = found_mnstr {
            if let Some(error) = found_mnstr.get_relationships().await {
                return Some(error.into());
            }
            self.mnstrs = Some(vec![found_mnstr]);
            return None;
        }

        let mut mnstr = Mnstr::new(
            self.id.clone(),
            self.display_name.clone(),
            format!("{}'s Mnstr", self.display_name.clone()),
            self.qr_code.clone(),
        );
        if let Some(error) = mnstr.create().await {
            return Some(error.into());
        }
        if let Some(error) = mnstr.get_relationships().await {
            return Some(error.into());
        }

        self.mnstrs = Some(vec![mnstr]);
        None
    }
}

impl DatabaseResource for User {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
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
