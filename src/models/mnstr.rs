use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields,
    find_all_resources_where_fields_in, find_one_resource_where_fields,
    graphql::mnstrs::queries::{MnstrOrderByInput, MnstrOrderDirectionInput},
    insert_resource, insert_resource_batch,
    models::{generated::mnstr_xp::XP_FOR_LEVEL, user::User},
    proto::Mnstr as GrpcMnstr,
    update_resource, update_resource_batch, upsert_resource, upsert_resource_batch,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, GraphQLObject, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mnstr {
    pub id: String,
    pub user_id: String,
    pub mnstr_name: String,
    pub mnstr_description: String,
    pub mnstr_qr_code: String,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    #[graphql(skip)]
    pub created_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    #[graphql(skip)]
    pub updated_at: Option<OffsetDateTime>,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    #[graphql(skip)]
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

    pub experience_to_next_level: i32,
}

pub const DEFAULT_STAT_VALUE: i32 = 10;

impl Mnstr {
    pub fn new(
        user_id: String,
        mnstr_name: Option<String>,
        mnstr_description: Option<String>,
        mnstr_qr_code: String,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id,
            mnstr_name: mnstr_name.unwrap_or(String::new()),
            mnstr_description: mnstr_description.unwrap_or(String::new()),
            mnstr_qr_code: mnstr_qr_code,
            created_at: None,
            updated_at: None,
            archived_at: None,
            current_level: 0,
            current_experience: 0,
            current_health: DEFAULT_STAT_VALUE,
            max_health: DEFAULT_STAT_VALUE,
            current_attack: DEFAULT_STAT_VALUE,
            max_attack: DEFAULT_STAT_VALUE,
            current_defense: DEFAULT_STAT_VALUE,
            max_defense: DEFAULT_STAT_VALUE,
            current_speed: DEFAULT_STAT_VALUE,
            max_speed: DEFAULT_STAT_VALUE,
            current_intelligence: DEFAULT_STAT_VALUE,
            max_intelligence: DEFAULT_STAT_VALUE,
            current_magic: DEFAULT_STAT_VALUE,
            max_magic: DEFAULT_STAT_VALUE,
            experience_to_next_level: 0,
        }
    }

    pub fn to_grpc(&self) -> GrpcMnstr {
        GrpcMnstr {
            id: self.id.clone(),
            user_id: self.user_id.clone(),
            mnstr_name: self.mnstr_name.clone(),
            mnstr_description: self.mnstr_description.clone(),
            mnstr_qr_code: self.mnstr_qr_code.clone(),
            current_level: self.current_level,
            current_experience: self.current_experience,
            current_health: self.current_health,
            max_health: self.max_health,
            current_attack: self.current_attack,
            max_attack: self.max_attack,
            current_defense: self.current_defense,
            max_defense: self.max_defense,
            current_speed: self.current_speed,
            max_speed: self.max_speed,
            current_intelligence: self.current_intelligence,
            max_intelligence: self.max_intelligence,
            current_magic: self.current_magic,
            max_magic: self.max_magic,
            experience_to_next_level: self.experience_to_next_level,
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
        experience_to_next_level: Option<i32>,
    ) -> Self {
        let created_at = match created_at {
            Some(created_at) => Some(created_at),
            None => None,
        };
        let updated_at = match updated_at {
            Some(updated_at) => Some(updated_at),
            None => None,
        };
        let archived_at = match archived_at {
            Some(archived_at) => Some(archived_at),
            None => None,
        };

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
            experience_to_next_level: experience_to_next_level
                .unwrap_or(self.experience_to_next_level),
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            ("user_id", self.user_id.clone().into()),
            ("mnstr_name", self.mnstr_name.clone().into()),
            ("mnstr_description", self.mnstr_description.clone().into()),
            ("mnstr_qr_code", self.mnstr_qr_code.clone().into()),
            ("current_level", self.current_level.clone().into()),
            ("current_experience", self.current_experience.clone().into()),
            ("current_health", self.current_health.clone().into()),
            ("max_health", self.max_health.clone().into()),
            ("current_attack", self.current_attack.clone().into()),
            ("max_attack", self.max_attack.clone().into()),
            ("current_defense", self.current_defense.clone().into()),
            ("max_defense", self.max_defense.clone().into()),
            ("current_speed", self.current_speed.clone().into()),
            ("max_speed", self.max_speed.clone().into()),
            (
                "current_intelligence",
                self.current_intelligence.clone().into(),
            ),
            ("max_intelligence", self.max_intelligence.clone().into()),
            ("current_magic", self.current_magic.clone().into()),
            ("max_magic", self.max_magic.clone().into()),
        ];
        let mnstr = match insert_resource!(Mnstr, params).await {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("[Mnstr::create] Failed to create mnstr: {:?}", e);
                return Some(e.into());
            }
        };
        *self = mnstr;

        let mut user = match User::find_one(self.user_id.clone(), false).await {
            Ok(user) => user,
            Err(e) => {
                println!("[Mnstr::create] Failed to get user: {:?}", e);
                return Some(e.into());
            }
        };
        let xp = XP_FOR_LEVEL[user.experience_level as usize];
        println!("[Mnstr::create] XP: {:?}", xp);
        if let Some(error) = user.update_xp(xp).await {
            println!("[Mnstr::create] Failed to update user xp: {:?}", error);
            return Some(error.into());
        }
        if let Some(error) = user.add_coins(self.coins()).await {
            println!("[Mnstr::create] Failed to add coins: {:?}", error);
            return Some(error.into());
        }

        self.update_experience_to_next_level();

        None
    }

    pub async fn create_batch(
        user_id: String,
        mnstrs: Vec<Vec<(&str, Option<DatabaseValue>)>>,
    ) -> Result<Vec<Mnstr>, anyhow::Error> {
        if mnstrs.is_empty() {
            return Err(anyhow::Error::msg("No mnstrs to create"));
        }

        let mut user = match User::find_one(user_id.clone(), false).await {
            Ok(user) => user,
            Err(e) => {
                println!("[Mnstr::create_batch] Failed to get user: {:?}", e);
                return Err(e.into());
            }
        };

        let xp = XP_FOR_LEVEL[user.experience_level as usize];
        println!("[Mnstr::create_batch] XP: {:?}", xp);
        if let Some(error) = user.update_xp(xp).await {
            println!(
                "[Mnstr::create_batch] Failed to update user xp: {:?}",
                error
            );
            return Err(error.into());
        }

        let mut params: Vec<Vec<(&str, DatabaseValue)>> = Vec::new();
        for mnstr in mnstrs.iter() {
            let mut mnstr_params: Vec<(&str, DatabaseValue)> = Vec::new();
            for (field, value) in mnstr.iter() {
                if let Some(v) = value {
                    mnstr_params.push((*field, v.clone().into()));
                }
            }
            params.push(mnstr_params);
        }

        match insert_resource_batch!(Mnstr, params).await {
            Ok(mut results) => {
                for mnstr in results.iter_mut() {
                    if let Some(error) = user.add_coins(mnstr.coins()).await {
                        println!("[Mnstr::create_batch] Failed to add coins: {:?}", error);
                        return Err(error.into());
                    }
                    mnstr.update_experience_to_next_level();
                }
                Ok(results)
            }
            Err(e) => {
                println!("[Mnstr::create_batch] Failed to create mnstrs: {:?}", e);
                return Err(e.into());
            }
        }
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

        self.update_experience_to_next_level();

        None
    }

    pub async fn update_batch(
        user_id: String,
        mnstrs: Vec<Vec<(&str, Option<DatabaseValue>)>>,
    ) -> Result<Vec<Mnstr>, anyhow::Error> {
        let mut results: Vec<Mnstr> = Vec::new();
        let mut params: Vec<Vec<(&str, DatabaseValue)>> = Vec::new();
        let mut new_mnstrs: Vec<Vec<(&str, DatabaseValue)>> = Vec::new();

        let qr_codes = mnstrs
            .iter()
            .map(|mnstr| {
                mnstr
                    .iter()
                    .find(|(field, _)| field == &"mnstr_qr_code")
                    .unwrap()
                    .1
                    .clone()
            })
            .collect::<Vec<Option<DatabaseValue>>>();
        let found_mnstrs =
            match find_all_resources_where_fields_in!(Mnstr, "mnstr_qr_code", qr_codes).await {
                Ok(found_mnstrs) => Some(found_mnstrs),
                Err(_) => None,
            };

        for mnstr in mnstrs.iter() {
            let qr_code = mnstr.iter().find(|(field, _)| field == &"mnstr_qr_code");

            let mut found_mnstr = None;

            if qr_code.is_some() && found_mnstrs.is_some() {
                let qr_code: String = (qr_code.as_ref().unwrap().1.clone().unwrap()).into();
                found_mnstr = match found_mnstrs
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|mnstr| mnstr.mnstr_qr_code == qr_code && mnstr.user_id == user_id)
                {
                    Some(mnstr) => Some(mnstr.clone()),
                    None => None,
                };
            }

            let mut mnstr_params: Vec<(&str, DatabaseValue)> = Vec::new();
            for (field, value) in mnstr.iter() {
                if let Some(v) = value {
                    mnstr_params.push((*field, v.clone().into()));
                }
            }

            if let None = found_mnstr {
                let id_idx = mnstr_params.iter().position(|(field, _)| field == &"id");
                if let Some(idx) = id_idx {
                    mnstr_params.remove(idx);
                }
                new_mnstrs.push(mnstr_params);
                continue;
            }

            params.push(mnstr_params);
        }

        // TODO: use upsert_resource_batch! macro instead of insert_resource_batch! and update_resource_batch!

        if !new_mnstrs.is_empty() {
            let new_results = match insert_resource_batch!(Mnstr, new_mnstrs).await {
                Ok(results) => results,
                Err(e) => {
                    println!("[Mnstr::update_batch] Failed to create mnstrs: {:?}", e);
                    return Err(e.into());
                }
            };
            results.extend(new_results);
        }

        if !params.is_empty() {
            let updated_results = match update_resource_batch!(Mnstr, params).await {
                Ok(results) => results,
                Err(e) => {
                    println!("[Mnstr::update_batch] Failed to update mnstrs: {:?}", e);
                    return Err(e.into());
                }
            };
            results.extend(updated_results);
        }

        Ok(results)
    }

    pub async fn update_with_defaults(&mut self) -> Option<anyhow::Error> {
        println!(
            "[Mnstr::update_with_defaults] Updating mnstr with defaults: {:?}",
            self.id
        );
        self.current_health = DEFAULT_STAT_VALUE;
        self.max_health = DEFAULT_STAT_VALUE;
        self.current_attack = DEFAULT_STAT_VALUE;
        self.max_attack = DEFAULT_STAT_VALUE;
        self.current_defense = DEFAULT_STAT_VALUE;
        self.max_defense = DEFAULT_STAT_VALUE;
        self.current_speed = DEFAULT_STAT_VALUE;
        self.max_speed = DEFAULT_STAT_VALUE;
        self.current_intelligence = DEFAULT_STAT_VALUE;
        self.max_intelligence = DEFAULT_STAT_VALUE;
        self.current_magic = DEFAULT_STAT_VALUE;
        self.max_magic = DEFAULT_STAT_VALUE;
        self.update().await
    }

    pub async fn delete_permanent(&mut self) -> Option<anyhow::Error> {
        match delete_resource_where_fields!(Mnstr, vec![("id", self.id.clone().into())], true).await
        {
            Ok(_) => (),
            Err(e) => return Some(e.into()),
        };
        None
    }

    pub async fn find_one(id: String, get_relationships: bool) -> Result<Self, anyhow::Error> {
        let mut mnstr =
            match find_one_resource_where_fields!(Mnstr, vec![("id", id.clone().into())]).await {
                Ok(mnstr) => mnstr,
                Err(e) => {
                    println!("[Mnstr::find_one] Failed to get mnstr: {:?}", e);
                    return Err(e.into());
                }
            };
        if mnstr.max_health == 0 {
            if let Some(error) = mnstr.update_with_defaults().await {
                println!(
                    "[Mnstr::find_one] Failed to update with defaults: {:?}",
                    error
                );
                return Err(error.into());
            }
        }

        mnstr.update_experience_to_next_level();

        if get_relationships {
            if let Some(error) = mnstr.get_relationships().await {
                println!("[Mnstr::find_one] Failed to get relationships: {:?}", error);
                return Err(error.into());
            }
        }
        Ok(mnstr)
    }

    pub async fn find_one_by(
        params: Vec<(&str, DatabaseValue)>,
        get_relationships: bool,
    ) -> Result<Self, anyhow::Error> {
        let mut mnstr = match find_one_resource_where_fields!(Mnstr, params).await {
            Ok(mnstr) => mnstr,
            Err(e) => {
                println!("[Mnstr::find_one_by] Failed to get mnstr: {:?}", e);
                return Err(e.into());
            }
        };

        if get_relationships {
            if let Some(error) = mnstr.get_relationships().await {
                println!("[Mnstr::find_one] Failed to get relationships: {:?}", error);
                return Err(error.into());
            }
        }
        Ok(mnstr)
    }

    pub async fn find_all(
        get_relationships: bool,
        order_by: Option<MnstrOrderByInput>,
        order_direction: Option<MnstrOrderDirectionInput>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut mnstrs = match find_all_resources_where_fields!(
            Mnstr,
            vec![],
            order_by,
            order_direction
        )
        .await
        {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!("[Mnstr::find_all] Failed to get mnstrs: {:?}", e);
                return Err(e.into());
            }
        };
        for mnstr in mnstrs.iter_mut() {
            if mnstr.max_health == 0 {
                if let Some(error) = mnstr.update_with_defaults().await {
                    println!(
                        "[Mnstr::find_all] Failed to update with defaults: {:?}",
                        error
                    );
                    return Err(error.into());
                }
            }

            mnstr.update_experience_to_next_level();

            if get_relationships {
                if let Some(error) = mnstr.get_relationships().await {
                    println!("[Mnstr::find_all] Failed to get relationships: {:?}", error);
                    return Err(error.into());
                }
            }
        }
        Ok(mnstrs)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
        get_relationships: bool,
        order_by: Option<MnstrOrderByInput>,
        order_direction: Option<MnstrOrderDirectionInput>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let mut mnstrs = match find_all_resources_where_fields!(
            Mnstr,
            params,
            order_by,
            order_direction
        )
        .await
        {
            Ok(mnstrs) => mnstrs,
            Err(e) => {
                println!("[Mnstr::find_all_by] Failed to get mnstrs: {:?}", e);
                return Err(e.into());
            }
        };
        for mnstr in mnstrs.iter_mut() {
            if mnstr.max_health == 0 {
                if let Some(error) = mnstr.update_with_defaults().await {
                    println!(
                        "[Mnstr::find_all_by] Failed to update with defaults: {:?}",
                        error
                    );
                    return Err(error.into());
                }
            }

            mnstr.update_experience_to_next_level();

            if get_relationships {
                if let Some(error) = mnstr.get_relationships().await {
                    println!(
                        "[Mnstr::find_all_by] Failed to get relationships: {:?}",
                        error
                    );
                    return Err(error.into());
                }
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

    pub fn update_experience_to_next_level(&mut self) {
        let last_level_index = XP_FOR_LEVEL.len() as i32 - 1;
        let mut xp_to_next_level = XP_FOR_LEVEL[last_level_index as usize];
        if self.current_level < last_level_index {
            xp_to_next_level = XP_FOR_LEVEL[self.current_level as usize + 1];
        }
        self.experience_to_next_level = xp_to_next_level;
    }

    pub async fn update_xp(&mut self, xp: i32) -> Option<anyhow::Error> {
        self.current_experience += xp;

        let last_level_index = XP_FOR_LEVEL.len() as i32 - 1;
        let mut xp_to_next_level = XP_FOR_LEVEL[last_level_index as usize];
        if self.current_level < last_level_index {
            xp_to_next_level = XP_FOR_LEVEL[self.current_level as usize + 1];
        }
        let xp_overage = self.current_experience - xp_to_next_level;

        let mut remaining_overage = xp_overage;
        while remaining_overage >= 0 {
            self.current_experience = remaining_overage;
            self.current_level += 1;
            xp_to_next_level = XP_FOR_LEVEL[self.current_level as usize + 1];
            remaining_overage -= xp_to_next_level;

            xp_to_next_level = XP_FOR_LEVEL[self.current_level as usize + 1];
            if remaining_overage < 0 {
                self.current_experience = 0;
            }
        }

        self.experience_to_next_level = xp_to_next_level;

        if let Some(error) = self.update().await {
            println!("[Mnstr::update_xp] Failed to update mnstr xp: {:?}", error);
            return Some(error.into());
        }
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
            experience_to_next_level: 0,
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
