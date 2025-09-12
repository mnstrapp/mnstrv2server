use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields, find_one_resource_where_fields,
    insert_resource,
    models::{generated::mnstr_xp::XP_FOR_LEVEL, user::User},
    update_resource,
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
    pub fn new(
        user_id: String,
        mnstr_name: String,
        mnstr_description: String,
        mnstr_qr_code: String,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id,
            mnstr_name,
            mnstr_description,
            mnstr_qr_code,
            created_at: None,
            updated_at: None,
            archived_at: None,
            current_level: 0,
            current_experience: 0,
            current_health: 0,
            max_health: 0,
            current_attack: 0,
            max_attack: 0,
            current_defense: 0,
            max_defense: 0,
            current_speed: 0,
            max_speed: 0,
            current_intelligence: 0,
            max_intelligence: 0,
            current_magic: 0,
            max_magic: 0,
        }
    }
    pub fn copy_with(
        &self,
        mnstr_name: Option<String>,
        mnstr_description: Option<String>,
        mnstr_qr_code: Option<String>,
        created_at: Option<OffsetDateTime>,
        updated_at: Option<OffsetDateTime>,
        archived_at: Option<OffsetDateTime>,
        current_level: Option<i32>,
        current_experience: Option<i32>,
        current_health: Option<i32>,
        max_health: Option<i32>,
        current_attack: Option<i32>,
        max_attack: Option<i32>,
        current_defense: Option<i32>,
        max_defense: Option<i32>,
        current_speed: Option<i32>,
        max_speed: Option<i32>,
        current_intelligence: Option<i32>,
        max_intelligence: Option<i32>,
        current_magic: Option<i32>,
        max_magic: Option<i32>,
    ) -> Self {
        let created_at = Some(created_at.unwrap_or(self.created_at.clone().unwrap()));
        let updated_at = Some(updated_at.unwrap_or(self.updated_at.clone().unwrap()));
        let archived_at = Some(archived_at.unwrap_or(self.archived_at.clone().unwrap()));

        Self {
            id: self.id.clone(),
            user_id: self.user_id.clone(),
            mnstr_name: mnstr_name.unwrap_or(self.mnstr_name.clone()),
            mnstr_description: mnstr_description.unwrap_or(self.mnstr_description.clone()),
            mnstr_qr_code: mnstr_qr_code.unwrap_or(self.mnstr_qr_code.clone()),
            created_at: created_at,
            updated_at: updated_at,
            archived_at: archived_at,
            current_level: current_level.unwrap_or(self.current_level),
            current_experience: current_experience.unwrap_or(self.current_experience),
            current_health: current_health.unwrap_or(self.current_health),
            max_health: max_health.unwrap_or(self.max_health),
            current_attack: current_attack.unwrap_or(self.current_attack),
            max_attack: max_attack.unwrap_or(self.max_attack),
            current_defense: current_defense.unwrap_or(self.current_defense),
            max_defense: max_defense.unwrap_or(self.max_defense),
            current_speed: current_speed.unwrap_or(self.current_speed),
            max_speed: max_speed.unwrap_or(self.max_speed),
            current_intelligence: current_intelligence.unwrap_or(self.current_intelligence),
            max_intelligence: max_intelligence.unwrap_or(self.max_intelligence),
            current_magic: current_magic.unwrap_or(self.current_magic),
            max_magic: max_magic.unwrap_or(self.max_magic),
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let mnstr = match insert_resource!(
            Mnstr,
            vec![
                ("user_id", self.user_id.clone().into()),
                ("mnstr_name", self.mnstr_name.clone().into()),
                ("mnstr_description", self.mnstr_description.clone().into()),
                ("mnstr_qr_code", self.mnstr_qr_code.clone().into())
            ]
        )
        .await
        {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("[Mnstr::create] Failed to create mnstr: {:?}", e);
                return Some(e.into());
            }
        };
        *self = mnstr;

        let mut user = match User::find_one(self.user_id.clone()).await {
            Ok(user) => user,
            Err(e) => {
                println!("[Mnstr::create] Failed to get user: {:?}", e);
                return Some(e.into());
            }
        };
        if let Some(error) = user
            .update_xp(XP_FOR_LEVEL[user.experience_level as usize])
            .await
        {
            println!("[Mnstr::create] Failed to update user xp: {:?}", error);
            return Some(error.into());
        }
        if let Some(error) = user.add_coins(self.coins()).await {
            println!("[Mnstr::create] Failed to add coins: {:?}", error);
            return Some(error.into());
        }
        None
    }

    pub async fn update(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            ("mnstr_name", self.mnstr_name.clone().into()),
            ("mnstr_description", self.mnstr_description.clone().into()),
            ("current_level", self.current_level.clone().into()),
            ("current_experience", self.current_experience.clone().into()),
            ("current_health", self.current_health.clone().into()),
            ("max_health", self.max_health.clone().into()),
            ("max_attack", self.max_attack.clone().into()),
            ("current_attack", self.current_attack.clone().into()),
            ("max_defense", self.max_defense.clone().into()),
            ("current_defense", self.current_defense.clone().into()),
            ("max_speed", self.max_speed.clone().into()),
            ("current_speed", self.current_speed.clone().into()),
            ("max_intelligence", self.max_intelligence.clone().into()),
            (
                "current_intelligence",
                self.current_intelligence.clone().into(),
            ),
            ("max_magic", self.max_magic.clone().into()),
            ("current_magic", self.current_magic.clone().into()),
        ];
        let mnstr = match update_resource!(Mnstr, self.id.clone(), params).await {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("[Mnstr::update] Failed to update mnstr: {:?}", e);
                return Some(e.into());
            }
        };
        *self = mnstr;
        None
    }

    pub async fn delete(&mut self) -> Option<anyhow::Error> {
        match delete_resource_where_fields!(Mnstr, vec![("id", self.id.clone().into())]).await {
            Ok(_) => (),
            Err(e) => return Some(e.into()),
        };
        let mnstr = match Self::find_one(self.id.clone()).await {
            Ok(mnstr) => mnstr,
            Err(e) => return Some(e.into()),
        };
        *self = mnstr;
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let mut mnstr =
            match find_one_resource_where_fields!(Mnstr, vec![("id", id.clone().into())]).await {
                Ok(mnstr) => mnstr,
                Err(e) => {
                    println!("[Mnstr::find_one] Failed to get mnstr: {:?}", e);
                    return Err(e.into());
                }
            };
        if let Some(error) = mnstr.get_relationships().await {
            println!("[Mnstr::find_one] Failed to get relationships: {:?}", error);
            return Err(error.into());
        }
        Ok(mnstr)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let mut mnstr = match find_one_resource_where_fields!(Mnstr, params).await {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("[Mnstr::find_one_by] Failed to get mnstr: {:?}", e);
                return Err(e.into());
            }
        };
        if let Some(error) = mnstr.get_relationships().await {
            println!("[Mnstr::find_one] Failed to get relationships: {:?}", error);
            return Err(error.into());
        }
        Ok(mnstr)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let mut mnstrs = match find_all_resources_where_fields!(Mnstr, vec![]).await {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!("[Mnstr::find_all] Failed to get mnstrs: {:?}", e);
                return Err(e.into());
            }
        };
        for mnstr in mnstrs.iter_mut() {
            if let Some(error) = mnstr.get_relationships().await {
                println!("[Mnstr::find_all] Failed to get relationships: {:?}", error);
                return Err(error.into());
            }
        }
        Ok(mnstrs)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut mnstrs = match find_all_resources_where_fields!(Mnstr, params).await {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!("[Mnstr::find_all_by] Failed to get mnstrs: {:?}", e);
                return Err(e.into());
            }
        };
        for mnstr in mnstrs.iter_mut() {
            if let Some(error) = mnstr.get_relationships().await {
                println!(
                    "[Mnstr::find_all_by] Failed to get relationships: {:?}",
                    error
                );
                return Err(error.into());
            }
        }
        Ok(mnstrs)
    }

    pub fn coins(&self) -> i32 {
        let hash = sha2::Sha256::digest(self.mnstr_qr_code.as_bytes());
        let coins_byte = hash[(hash.len() - 1) / 2];
        let multiplier_hash_byte = hash[((hash.len() - 1) / 2) + 1];

        let mut coins = coins_byte as i32;
        if coins <= 0 {
            coins = 5;
        }

        let mut multiplier = multiplier_hash_byte as i32;
        if multiplier <= 0 {
            multiplier = 10;
        }

        if multiplier >= 251 {
            coins = (coins * (multiplier / 100)) + 1000;
            if coins > 2000 {
                coins = 2000;
            }
        } else if multiplier >= 242 {
            coins = (coins * (multiplier / 100)) + 400;
            if coins > 750 {
                coins = 750;
            }
        } else if multiplier >= 216 {
            coins = (coins * (multiplier / 100)) + 150;
            if coins > 400 {
                coins = 400;
            }
        } else {
            if multiplier >= 85 {
                coins = coins * (multiplier / 100);
            }
            if coins > 25 {
                coins = coins / 10;
            }
        }

        if coins < 5 {
            coins = 5;
        }

        coins
    }

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
