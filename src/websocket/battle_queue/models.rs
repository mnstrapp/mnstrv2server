use rocket::serde;
use serde::{Deserialize, Serialize};
use sqlx::{Error, Row, postgres::PgRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    database::traits::DatabaseResource,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BattleQueueChannel {
    Lobby,
    Battle,
}

impl std::fmt::Display for BattleQueueChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleQueueChannel::Lobby => write!(f, "lobby"),
            BattleQueueChannel::Battle => write!(f, "battle"),
        }
    }
}

impl From<String> for BattleQueueChannel {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "lobby" => BattleQueueChannel::Lobby,
            "battle" => BattleQueueChannel::Battle,
            _ => BattleQueueChannel::Lobby,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BattleQueueAction {
    Error,
    Joined,
    Left,
    Ready,
    Unready,
    Requested,
    Accepted,
    Rejected,
    Cancelled,
    Watching,
    List,
}

impl std::fmt::Display for BattleQueueAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleQueueAction::Error => write!(f, "error"),
            BattleQueueAction::Joined => write!(f, "joined"),
            BattleQueueAction::Left => write!(f, "left"),
            BattleQueueAction::Ready => write!(f, "ready"),
            BattleQueueAction::Unready => write!(f, "unready"),
            BattleQueueAction::Requested => write!(f, "requested"),
            BattleQueueAction::Accepted => write!(f, "accepted"),
            BattleQueueAction::Rejected => write!(f, "rejected"),
            BattleQueueAction::Cancelled => write!(f, "cancelled"),
            BattleQueueAction::Watching => write!(f, "watching"),
            BattleQueueAction::List => write!(f, "list"),
        }
    }
}

impl From<String> for BattleQueueAction {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "error" => BattleQueueAction::Error,
            "joined" => BattleQueueAction::Joined,
            "left" => BattleQueueAction::Left,
            "ready" => BattleQueueAction::Ready,
            "unready" => BattleQueueAction::Unready,
            "requested" => BattleQueueAction::Requested,
            "accepted" => BattleQueueAction::Accepted,
            "rejected" => BattleQueueAction::Rejected,
            "cancelled" => BattleQueueAction::Cancelled,
            "watching" => BattleQueueAction::Watching,
            "list" => BattleQueueAction::List,
            _ => BattleQueueAction::Joined,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleQueue {
    pub id: String,
    pub user_id: Option<String>,
    pub channel: BattleQueueChannel,
    pub action: BattleQueueAction,
    pub data: BattleQueueData,

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
}

impl BattleQueue {
    pub fn new(
        user_id: Option<String>,
        channel: BattleQueueChannel,
        action: BattleQueueAction,
        data: BattleQueueData,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            user_id,
            channel,
            action,
            data,
            created_at: Some(OffsetDateTime::now_utc()),
            updated_at: Some(OffsetDateTime::now_utc()),
            archived_at: None,
        }
    }
}

impl DatabaseResource for BattleQueue {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");
        let updated_at = row.get("updated_at");
        let archived_at = match row.get("archived_at") {
            Some(archived_at) => archived_at,
            None => None,
        };
        let channel = row.get::<String, _>("channel");
        let action = row.get::<String, _>("action");
        let data = row.get::<String, _>("data");

        Ok(BattleQueue {
            id: row.get("id"),
            user_id: match row.get("user_id") {
                Some(user_id) => user_id,
                None => None,
            },
            channel: channel.into(),
            action: action.into(),
            data: data.into(),
            created_at,
            updated_at,
            archived_at,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BattleQueueDataAction {
    Connect,
    Cancel,
    Ready,
    Unready,
    Start,
    Watch,
    Left,
    List,
    Error,
}

impl From<String> for BattleQueueDataAction {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "connect" => BattleQueueDataAction::Connect,
            "cancel" => BattleQueueDataAction::Cancel,
            "ready" => BattleQueueDataAction::Ready,
            "unready" => BattleQueueDataAction::Unready,
            "start" => BattleQueueDataAction::Start,
            "watch" => BattleQueueDataAction::Watch,
            "left" => BattleQueueDataAction::Left,
            "list" => BattleQueueDataAction::List,
            "error" => BattleQueueDataAction::Error,
            _ => BattleQueueDataAction::Connect,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleQueueData {
    pub action: BattleQueueDataAction,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub opponent_id: Option<String>,
    pub opponent_name: Option<String>,
    pub mnstr_id: Option<String>,
    pub data: Option<String>,
    pub error: Option<String>,
    pub message: Option<String>,
}

impl BattleQueueData {
    pub fn new(
        action: BattleQueueDataAction,
        user_id: Option<String>,
        user_name: Option<String>,
        opponent_id: Option<String>,
        opponent_name: Option<String>,
        mnstr_id: Option<String>,
        data: Option<String>,
        error: Option<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            action,
            user_id,
            user_name,
            opponent_id,
            opponent_name,
            mnstr_id,
            data,
            error,
            message,
        }
    }
}

impl From<String> for BattleQueueData {
    fn from(value: String) -> Self {
        let data = serde_json::from_str(&value).unwrap_or(BattleQueueData {
            action: BattleQueueDataAction::Connect,
            user_id: None,
            user_name: None,
            opponent_id: None,
            opponent_name: None,
            mnstr_id: None,
            data: None,
            error: Some("Invalid data".to_string()),
            message: None,
        });
        data
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        match value.to_lowercase().as_str() {
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
    pub status: BattleStatusState,
    pub connected: bool,

    #[serde(
        serialize_with = "serialize_offset_date_time",
        deserialize_with = "deserialize_offset_date_time"
    )]
    pub created_at: Option<OffsetDateTime>,
}

impl BattleStatus {
    pub fn new(user_id: String, status: BattleStatusState, connected: bool) -> Self {
        let created_at = OffsetDateTime::now_utc();

        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            status,
            connected,
            created_at: Some(created_at),
        }
    }
}

impl DatabaseResource for BattleStatus {
    fn from_row(row: &PgRow) -> Result<Self, Error> {
        let created_at = row.get("created_at");

        Ok(BattleStatus {
            id: row.get("id"),
            user_id: row.get("user_id"),
            status: row.get::<String, _>("status").into(),
            connected: row.get("connected"),
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
