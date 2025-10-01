use rocket::serde;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::{traits::DatabaseResource, values::DatabaseValue},
    find_all_resources_where_fields, find_one_resource_where_fields, insert_resource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum BattleLogAction {
    Joined,
    Attacked,
    Defended,
    Missed,
    Hit,
    Killed,
    Won,
    Lost,
    Error,
}

impl std::fmt::Display for BattleLogAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleLogAction::Joined => write!(f, "joined"),
            BattleLogAction::Attacked => write!(f, "attacked"),
            BattleLogAction::Defended => write!(f, "defended"),
            BattleLogAction::Missed => write!(f, "missed"),
            BattleLogAction::Hit => write!(f, "hit"),
            BattleLogAction::Killed => write!(f, "killed"),
            BattleLogAction::Won => write!(f, "won"),
            BattleLogAction::Lost => write!(f, "lost"),
            BattleLogAction::Error => write!(f, "error"),
        }
    }
}

impl From<String> for BattleLogAction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "joined" => BattleLogAction::Joined,
            "attacked" => BattleLogAction::Attacked,
            "defended" => BattleLogAction::Defended,
            "missed" => BattleLogAction::Missed,
            "hit" => BattleLogAction::Hit,
            "killed" => BattleLogAction::Killed,
            "won" => BattleLogAction::Won,
            "lost" => BattleLogAction::Lost,
            "error" => BattleLogAction::Error,
            _ => BattleLogAction::Joined,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleLog {
    pub id: String,
    pub battle_id: String,
    pub user_id: String,
    pub mnstr_id: String,
    pub action: BattleLogAction,
    pub data: String,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub created_at: Option<OffsetDateTime>,
}

impl BattleLog {
    pub fn new(
        battle_id: String,
        user_id: String,
        mnstr_id: String,
        action: BattleLogAction,
        data: String,
    ) -> Self {
        Self {
            id: "".to_string(),
            battle_id,
            user_id,
            mnstr_id,
            action,
            data,
            created_at: None,
        }
    }

    pub async fn create(&mut self) -> Option<anyhow::Error> {
        let params = vec![
            ("id", uuid::Uuid::new_v4().to_string().into()),
            ("battle_id", self.battle_id.clone().into()),
            ("user_id", self.user_id.clone().into()),
            ("mnstr_id", self.mnstr_id.clone().into()),
            ("action", self.action.clone().to_string().into()),
            ("data", self.data.clone().into()),
        ];
        let battle_log = match insert_resource!(BattleLog, params).await {
            Ok(battle_log) => battle_log,
            Err(e) => return Some(e.into()),
        };
        *self = battle_log;
        None
    }

    pub async fn find_one(id: String) -> Result<Self, anyhow::Error> {
        let battle_log =
            match find_one_resource_where_fields!(BattleLog, vec![("id", id.clone().into())]).await
            {
                Ok(battle_log) => battle_log,
                Err(e) => return Err(e.into()),
            };
        Ok(battle_log)
    }

    pub async fn find_one_by(params: Vec<(&str, DatabaseValue)>) -> Result<Self, anyhow::Error> {
        let battle_log = match find_one_resource_where_fields!(BattleLog, params).await {
            Ok(battle_log) => battle_log,
            Err(e) => return Err(e.into()),
        };
        Ok(battle_log)
    }

    pub async fn find_all() -> Result<Vec<Self>, anyhow::Error> {
        let battle_logs = match find_all_resources_where_fields!(BattleLog, vec![]).await {
            Ok(battle_logs) => battle_logs,
            Err(e) => return Err(e.into()),
        };
        Ok(battle_logs)
    }

    pub async fn find_all_by(
        params: Vec<(&str, DatabaseValue)>,
    ) -> Result<Vec<Self>, anyhow::Error> {
        let battle_logs = match find_all_resources_where_fields!(BattleLog, params).await {
            Ok(battle_logs) => battle_logs,
            Err(e) => return Err(e.into()),
        };
        Ok(battle_logs)
    }
}

impl DatabaseResource for BattleLog {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        Ok(Self {
            id: row.get("id"),
            battle_id: row.get("battle_id"),
            user_id: row.get("user_id"),
            mnstr_id: row.get("mnstr_id"),
            action: row.get::<String, _>("action").into(),
            data: row.get("data"),
            created_at: Some(created_at),
        })
    }

    fn has_id() -> bool {
        true
    }

    fn is_archivable() -> bool {
        false
    }

    fn is_updatable() -> bool {
        false
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
