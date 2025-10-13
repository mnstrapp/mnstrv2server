use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    delete_resource_where_fields, find_all_resources_where_fields, find_one_resource_where_fields,
    insert_resource, update_resource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum BattleStatusState {
    InQueue,
    InBattle,
    Watching,
}

impl std::fmt::Display for BattleStatusState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleStatusState::InQueue => write!(f, "inQueue"),
            BattleStatusState::InBattle => write!(f, "inBattle"),
            BattleStatusState::Watching => write!(f, "watching"),
        }
    }
}

impl From<String> for BattleStatusState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "inQueue" => BattleStatusState::InQueue,
            "inBattle" => BattleStatusState::InBattle,
            "watching" => BattleStatusState::Watching,
            _ => BattleStatusState::InQueue,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleStatus {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    pub opponent_id: Option<String>,
    pub opponent_name: Option<String>,
    pub battle_id: Option<String>,
    pub status: BattleStatusState,

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

impl BattleStatus {
    pub fn new(
        user_id: String,
        display_name: String,
        opponent_id: Option<String>,
        opponent_name: Option<String>,
        battle_id: Option<String>,
        status: BattleStatusState,
    ) -> Self {
        Self {
            id: "".to_string(),
            user_id,
            display_name,
            opponent_id,
            opponent_name,
            battle_id,
            status,
            created_at: None,
            updated_at: None,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            ("user_id", self.user_id.clone().into()),
            ("display_name", self.display_name.clone().into()),
            ("opponent_id", self.opponent_id.clone().into()),
            ("opponent_name", self.opponent_name.clone().into()),
            ("battle_id", self.battle_id.clone().into()),
            ("status", self.status.clone().to_string().into()),
            ("updated_at", self.updated_at.clone().into()),
        ];
        let battle_status = match insert_resource!(BattleStatus, params).await {
            Ok(battle_status) => battle_status,
            Err(e) => return Some(e.into()),
        };
        *self = battle_status;
        None
    }

    pub async fn update(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            ("opponent_id", self.opponent_id.clone().into()),
            ("opponent_name", self.opponent_name.clone().into()),
            ("battle_id", self.battle_id.clone().into()),
            ("status", self.status.clone().to_string().into()),
        ];
        let battle_status = match update_resource!(BattleStatus, self.id.clone(), params).await {
            Ok(battle_status) => battle_status,
            Err(e) => return Some(e.into()),
        };
        *self = battle_status;
        None
    }

    pub async fn delete(&mut self) -> Option<anyhow::Error> {
        let params = vec![("id", self.id.clone().into())];
        match delete_resource_where_fields!(BattleStatus, params).await {
            Ok(_) => (),
            Err(e) => return Some(e.into()),
        };
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let battle_status =
            match find_one_resource_where_fields!(BattleStatus, vec![("id", id.clone().into())])
                .await
            {
                Ok(battle_status) => battle_status,
                Err(e) => return Err(e.into()),
            };
        Ok(battle_status)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let battle_status = match find_one_resource_where_fields!(BattleStatus, params).await {
            Ok(battle_status) => battle_status,
            Err(e) => return Err(e.into()),
        };
        Ok(battle_status)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let battle_statuses = match find_all_resources_where_fields!(BattleStatus, vec![]).await {
            Ok(battle_statuses) => battle_statuses,
            Err(e) => return Err(e.into()),
        };
        Ok(battle_statuses)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let battle_statuses = match find_all_resources_where_fields!(BattleStatus, params).await {
            Ok(battle_statuses) => battle_statuses,
            Err(e) => return Err(e.into()),
        };
        Ok(battle_statuses)
    }
}

impl DatabaseResource for BattleStatus {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        
        Ok(BattleStatus {
            id: row.get("id"),
            user_id: row.get("user_id"),
            display_name: row.get("display_name"),
            opponent_id: row.get("opponent_id"),
            opponent_name: row.get("opponent_name"),
            battle_id: row.get("battle_id"),
            status: row.get::<String, _>("status").into(),
            created_at: Some(created_at),
            updated_at: Some(updated_at),
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
