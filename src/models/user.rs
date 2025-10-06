use anyhow::anyhow;
use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sqlx::{Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields, find_one_resource_where_fields,
    insert_resource,
    models::{generated::level_xp::XP_FOR_LEVEL, mnstr::Mnstr, session::Session, wallet::Wallet},
    update_resource,
    utils::{
        passwords::hash_password,
        time::{deserialize_offset_date_time, serialize_offset_date_time},
    },
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub email_verification_code: Option<String>,
    pub phone_verification_code: Option<String>,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub display_name: String,
    pub password_hash: String,
    pub experience_level: i32,
    pub experience_points: i32,
    pub experience_to_next_level: i32, // calculated based on the experience_level
    pub coins: i32,                    // calculated based on transaction history

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
    pub mnstrs: Vec<Mnstr>,
}

impl User {
    pub fn new(
        email: Option<String>,
        phone: Option<String>,
        password: String,
        display_name: String,
    ) -> Self {
        let password_hash = hash_password(&password);
        Self {
            id: "".to_string(),
            email,
            phone,
            email_verification_code: None,
            phone_verification_code: None,
            email_verified: false,
            phone_verified: false,
            password_hash,
            display_name,
            experience_level: 0,
            experience_points: 0,
            experience_to_next_level: 0,
            coins: 0,
            created_at: None,
            updated_at: None,
            archived_at: None,
            wallet: None,
            mnstrs: Vec::new(),
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        println!(
            "[User::create] Creating user: {:?}",
            self.display_name.clone()
        );
        let params = vec![
            ("password_hash", self.password_hash.clone().into()),
            ("phone", self.phone.clone().into()),
            ("email", self.email.clone().into()),
            ("display_name ", self.display_name.clone().into()),
            ("email_verified", self.email_verified.clone().into()),
            ("phone_verified", self.phone_verified.clone().into()),
            (
                "email_verification_code",
                self.email_verification_code.clone().into(),
            ),
            (
                "phone_verification_code",
                self.phone_verification_code.clone().into(),
            ),
        ];
        let mut user = match insert_resource!(User, params).await {
            Ok(user) => user,
            Err(e) => {
                println!("[User::create] Failed to create user: {:?}", e);
                return Some(e.into());
            }
        };

        if let Some(error) = user.create_relationships().await {
            println!("[User::create] Failed to create relationships: {:?}", error);
            return Some(error);
        }

        *self = user;
        None
    }

    pub async fn update(&mut self) -> Option<anyhow::Error> {
        println!("[User::update] Updating user: {:?}", self.id);

        let params = vec![
            ("display_name", self.display_name.clone().into()),
            ("phone", self.phone.clone().into()),
            ("email", self.email.clone().into()),
            (
                "email_verification_code",
                self.email_verification_code.clone().into(),
            ),
            (
                "phone_verification_code",
                self.phone_verification_code.clone().into(),
            ),
            ("email_verified", self.email_verified.clone().into()),
            ("phone_verified", self.phone_verified.clone().into()),
            ("experience_level", self.experience_level.clone().into()),
            ("experience_points", self.experience_points.clone().into()),
            ("password_hash", self.password_hash.clone().into()),
        ];
        let mut user = match update_resource!(User, self.id.clone(), params).await {
            Ok(user) => user,
            Err(e) => {
                println!("[User::update] Failed to update user: {:?}", e);
                return Some(e.into());
            }
        };

        if let Some(error) = user.get_relationships().await {
            println!("[User::update] Failed to get relationships: {:?}", error);
            return Some(error);
        }

        *self = user;
        None
    }

    pub async fn delete_permanent(&mut self) -> Option<anyhow::Error> {
        let user = match Self::find_one(self.id.clone(), false).await {
            Ok(user) => user,
            Err(e) => {
                println!("[User::delete_permanent] Failed to get user: {:?}", e);
                return Some(e.into());
            }
        };

        *self = user;

        for mnstr in self.mnstrs.iter_mut() {
            if let Some(error) = mnstr.delete_permanent().await {
                println!(
                    "[User::delete_permanent] Failed to delete mnstr: {:?}",
                    error
                );
                return Some(error);
            }
        }

        let mut sessions =
            match Session::find_all_by(vec![("user_id", self.id.clone().into())]).await {
                Ok(sessions) => sessions,
                Err(e) => {
                    println!("[User::delete_permanent] Failed to get sessions: {:?}", e);
                    return Some(e.into());
                }
            };

        for session in sessions.iter_mut() {
            if let Some(error) = session.delete_permanent().await {
                println!(
                    "[User::delete_permanent] Failed to delete session: {:?}",
                    error
                );
                return Some(error);
            }
        }

        if let Some(error) = self.wallet.as_mut().unwrap().delete_permanent().await {
            println!(
                "[User::delete_permanent] Failed to delete wallet: {:?}",
                error
            );
            return Some(error);
        }

        match delete_resource_where_fields!(User, vec![("id", self.id.clone().into())], true).await
        {
            Ok(_) => (),
            Err(e) => {
                println!("[User::delete_permanent] Failed to delete user: {:?}", e);
                return Some(e.into());
            }
        };
        None
    }

    pub async fn find_one(id: String, get_relationships: bool) -> Result<Self, anyhow::Error> {
        let params = vec![("id", id.clone().into())];
        let mut user = match find_one_resource_where_fields!(User, params).await {
            Ok(user) => user,
            Err(e) => {
                println!("[User::find_one] Failed to get user: {:?}", e);
                return Err(e.into());
            }
        };
        if get_relationships {
            if let Some(error) = user.get_relationships().await {
                println!("[User::find_one] Failed to get relationships: {:?}", error);
                return Err(error.into());
            }
        }
        user.update_experience_to_next_level();
        Ok(user)
    }

    pub async fn find_one_by(
        params: Vec<(&str, DatabaseValue)>,
        get_relationships: bool,
    ) -> Result<Self, anyhow::Error> {
        let mut user = match find_one_resource_where_fields!(User, params).await {
            Ok(user) => user,
            Err(e) => {
                println!("[User::find_one_by] Failed to get user: {:?}", e);
                return Err(e.into());
            }
        };
        if get_relationships {
            if let Some(error) = user.get_relationships().await {
                println!(
                    "[User::find_one_by] Failed to get relationships: {:?}",
                    error
                );
                return Err(error.into());
            }
        }
        user.update_experience_to_next_level();
        Ok(user)
    }

    pub async fn find_all(get_relationships: bool) -> Result<Vec<Self>, anyhow::Error> {
        let mut users = match find_all_resources_where_fields!(User, vec![]).await {
            Ok(users) => users,
            Err(e) => {
                println!("[User::find_all] Failed to get users: {:?}", e);
                return Err(e.into());
            }
        };
        for user in users.iter_mut() {
            user.update_experience_to_next_level();
            if get_relationships {
                if let Some(error) = user.get_relationships().await {
                    println!("[User::find_all] Failed to get relationships: {:?}", error);
                    return Err(error.into());
                }
            }
        }
        Ok(users)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
        get_relationships: bool,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut users = match find_all_resources_where_fields!(User, params).await {
            Ok(users) => users,
            Err(e) => {
                println!("[User::find_all_by] Failed to get users: {:?}", e);
                return Err(e.into());
            }
        };
        for user in users.iter_mut() {
            user.update_experience_to_next_level();
            if get_relationships {
                if let Some(error) = user.get_relationships().await {
                    println!(
                        "[User::find_all_by] Failed to get relationships: {:?}",
                        error
                    );
                    return Err(error.into());
                }
            }
        }
        Ok(users)
    }

    pub async fn get_relationships(&mut self) -> Option<anyhow::Error> {
        if let Some(error) = self.get_wallet().await {
            println!(
                "[User::get_relationships] Failed to get wallet: {:?}",
                error
            );
            return Some(error.into());
        }
        if let Some(error) = self.get_mnstrs().await {
            println!(
                "[User::get_relationships] Failed to get mnstrs: {:?}",
                error
            );
            return Some(error.into());
        }
        if let Some(error) = self.get_coins().await {
            println!("[User::get_relationships] Failed to get coins: {:?}", error);
            return Some(error.into());
        }
        None
    }

    pub async fn get_wallet(&mut self) -> Option<anyhow::Error> {
        println!("[User::get_wallet] Getting wallet: {:?}", self.id);
        let wallet = match find_one_resource_where_fields!(
            Wallet,
            vec![("user_id", self.id.clone().into())]
        )
        .await
        {
            Ok(wallet) => wallet,
            Err(e) => {
                println!("[User::get_wallet] Failed to get wallet: {:?}", e);
                return Some(e.into());
            }
        };
        self.wallet = Some(wallet);
        None
    }

    pub async fn get_mnstrs(&mut self) -> Option<anyhow::Error> {
        println!("[User::get_mnstrs] Getting mnstrs: {:?}", self.id);
        let mnstrs = match find_all_resources_where_fields!(
            Mnstr,
            vec![("user_id", self.id.clone().into())]
        )
        .await
        {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!("[User::get_mnstrs] Failed to get mnstrs: {:?}", e);
                return Some(e.into());
            }
        };
        self.mnstrs = mnstrs;
        None
    }

    pub async fn get_coins(&mut self) -> Option<anyhow::Error> {
        if let Some(error) = self.get_wallet().await {
            return Some(error.into());
        }
        if let Some(wallet) = &mut self.wallet {
            if let Some(error) = wallet.get_coins().await {
                println!("[User::get_coins] Failed to get coins: {:?}", error);
                return Some(error.into());
            }
            self.coins = wallet.coins;
        }
        None
    }

    pub async fn create_relationships(&mut self) -> Option<anyhow::Error> {
        if let Some(error) = Box::pin(self.create_wallet()).await {
            println!(
                "[User::create_relationships] Failed to create wallet: {:?}",
                error
            );
            return Some(error.into());
        }
        None
    }

    pub async fn create_wallet(&mut self) -> Option<anyhow::Error> {
        println!("[User::create_wallet] Creating wallet: {:?}", self.id);
        let found_wallet =
            match Wallet::find_one_by(vec![("user_id", self.id.clone().into())]).await {
                Ok(wallet) => Some(wallet),
                Err(_) => None,
            };
        if let Some(mut found_wallet) = found_wallet {
            if let Some(error) = found_wallet.get_relationships().await {
                println!(
                    "[User::create_wallet] Failed to get wallet relationships: {:?}",
                    error
                );
                return Some(error.into());
            }
            self.wallet = Some(found_wallet);
            return None;
        }

        let mut wallet = Wallet::new(self.id.clone());
        if let Some(error) = wallet.create().await {
            println!("[User::create_wallet] Failed to create wallet: {:?}", error);
            return Some(error.into());
        }
        self.wallet = Some(wallet);
        None
    }

    pub fn update_experience_to_next_level(&mut self) {
        let last_level_index = XP_FOR_LEVEL.len() as i32 - 1;
        let mut xp_to_next_level = XP_FOR_LEVEL[last_level_index as usize];
        if self.experience_level < last_level_index {
            xp_to_next_level = XP_FOR_LEVEL[self.experience_level as usize + 1];
        }
        self.experience_to_next_level = xp_to_next_level;
    }

    pub async fn update_xp(&mut self, xp: i32) -> Option<anyhow::Error> {
        self.experience_points += xp;

        let last_level_index = XP_FOR_LEVEL.len() as i32 - 1;
        let mut xp_to_next_level = XP_FOR_LEVEL[last_level_index as usize];
        if self.experience_level < last_level_index {
            xp_to_next_level = XP_FOR_LEVEL[self.experience_level as usize + 1];
        }
        let xp_overage = self.experience_points - xp_to_next_level;

        let mut remaining_overage = xp_overage;
        while remaining_overage >= 0 {
            self.experience_points = remaining_overage;
            self.experience_level += 1;
            xp_to_next_level = XP_FOR_LEVEL[self.experience_level as usize + 1];
            remaining_overage -= xp_to_next_level;

            xp_to_next_level = XP_FOR_LEVEL[self.experience_level as usize + 1];
            if remaining_overage < 0 {
                self.experience_points = 0;
            }
        }

        self.experience_to_next_level = xp_to_next_level;

        if let Some(error) = self.update().await {
            println!("[User::update_xp] Failed to update user xp: {:?}", error);
            return Some(error.into());
        }
        None
    }

    pub async fn add_coins(&mut self, coins: i32) -> Option<anyhow::Error> {
        println!("[User::add_coins] Adding coins: {:?}", coins);
        if let Some(error) = self.get_wallet().await {
            println!("[User::add_coins] Failed to get wallet: {:?}", error);
            return Some(error.into());
        }
        if let Some(wallet) = &mut self.wallet {
            if let Some(error) = wallet.add_coins(coins).await {
                println!("[User::add_coins] Failed to add coins: {:?}", error);
                return Some(error.into());
            }
            self.coins = wallet.coins;
        }
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

        let experience_level = row.get("experience_level");
        let experience_points = row.get("experience_points");

        let email_verification_code = match row.get::<Option<String>, _>("email_verification_code")
        {
            Some(code) => Some(code),
            None => None,
        };

        let phone_verification_code = match row.get::<Option<String>, _>("phone_verification_code")
        {
            Some(code) => Some(code),
            None => None,
        };

        let email_verified = row.get::<bool, _>("email_verified");
        let phone_verified = row.get::<bool, _>("phone_verified");

        Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            phone: row.get("phone"),
            display_name: row.get("display_name"),
            password_hash: row.get("password_hash"),
            email_verification_code,
            phone_verification_code,
            email_verified,
            phone_verified,
            experience_level,
            experience_points,
            experience_to_next_level: 0,
            coins: 0,
            created_at,
            updated_at,
            archived_at,
            wallet: None,
            mnstrs: Vec::new(),
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
