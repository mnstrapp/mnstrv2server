use rocket::serde;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;

use crate::{
    database::traits::DatabaseResource,
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
