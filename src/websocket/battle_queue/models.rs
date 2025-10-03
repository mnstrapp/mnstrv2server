use rocket::serde;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    models::mnstr::Mnstr,
    utils::time::{deserialize_offset_date_time, serialize_offset_date_time},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
        match value.as_str() {
            "lobby" => BattleQueueChannel::Lobby,
            "battle" => BattleQueueChannel::Battle,
            _ => BattleQueueChannel::Lobby,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum BattleQueueAction {
    Error,
    Joined,
    Left,
    Ready,
    Unready,
    Requested,
    Accepted,
    Rejected,
    Cancel,
    Watching,
    List,
    Challenge,
    Accept,
    Reject,
    Ping,
    GameStarted,
    GameEnded,
    MnstrChosen,
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
            BattleQueueAction::Cancel => write!(f, "cancel"),
            BattleQueueAction::Watching => write!(f, "watching"),
            BattleQueueAction::List => write!(f, "list"),
            BattleQueueAction::Challenge => write!(f, "challenge"),
            BattleQueueAction::Accept => write!(f, "accept"),
            BattleQueueAction::Reject => write!(f, "reject"),
            BattleQueueAction::Ping => write!(f, "ping"),
            BattleQueueAction::GameStarted => write!(f, "gameStarted"),
            BattleQueueAction::GameEnded => write!(f, "gameEnded"),
            BattleQueueAction::MnstrChosen => write!(f, "mnstrChosen"),
        }
    }
}

impl From<String> for BattleQueueAction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "error" => BattleQueueAction::Error,
            "joined" => BattleQueueAction::Joined,
            "left" => BattleQueueAction::Left,
            "ready" => BattleQueueAction::Ready,
            "unready" => BattleQueueAction::Unready,
            "requested" => BattleQueueAction::Requested,
            "accepted" => BattleQueueAction::Accepted,
            "rejected" => BattleQueueAction::Rejected,
            "cancel" => BattleQueueAction::Cancel,
            "watching" => BattleQueueAction::Watching,
            "list" => BattleQueueAction::List,
            "challenge" => BattleQueueAction::Challenge,
            "accept" => BattleQueueAction::Accept,
            "reject" => BattleQueueAction::Reject,
            "ping" => BattleQueueAction::Ping,
            "gameStarted" => BattleQueueAction::GameStarted,
            "gameEnded" => BattleQueueAction::GameEnded,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum BattleQueueDataAction {
    Connect,
    Cancel,
    Ready,
    Unready,
    Ping,
    Watch,
    Left,
    List,
    Error,
    Challenge,
    Accept,
    Reject,
    GameStarted,
    GameEnded,
    MnstrChosen,
}

impl From<String> for BattleQueueDataAction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "connect" => BattleQueueDataAction::Connect,
            "cancel" => BattleQueueDataAction::Cancel,
            "ready" => BattleQueueDataAction::Ready,
            "unready" => BattleQueueDataAction::Unready,
            "watch" => BattleQueueDataAction::Watch,
            "left" => BattleQueueDataAction::Left,
            "list" => BattleQueueDataAction::List,
            "error" => BattleQueueDataAction::Error,
            "challenge" => BattleQueueDataAction::Challenge,
            "accept" => BattleQueueDataAction::Accept,
            "reject" => BattleQueueDataAction::Reject,
            "ping" => BattleQueueDataAction::Ping,
            "gameStarted" => BattleQueueDataAction::GameStarted,
            "gameEnded" => BattleQueueDataAction::GameEnded,
            "mnstrChosen" => BattleQueueDataAction::MnstrChosen,
            _ => BattleQueueDataAction::Connect,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleQueueData {
    pub action: BattleQueueDataAction,
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub user_mnstr_id: Option<String>,
    pub opponent_id: Option<String>,
    pub opponent_name: Option<String>,
    pub opponent_mnstr_id: Option<String>,
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
        user_mnstr_id: Option<String>,
        opponent_mnstr_id: Option<String>,
        data: Option<String>,
        error: Option<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            action,
            id: Some(uuid::Uuid::new_v4().to_string()),
            user_id,
            user_name,
            opponent_id,
            opponent_name,
            user_mnstr_id,
            opponent_mnstr_id,
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
            id: None,
            user_id: None,
            user_name: None,
            user_mnstr_id: None,
            opponent_id: None,
            opponent_name: None,
            opponent_mnstr_id: None,
            data: None,
            error: Some("Invalid data".to_string()),
            message: None,
        });
        data
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleQueueGameData {
    pub battle_id: Option<String>,
    pub challenger_mnstr: Option<Mnstr>,
    pub challenger_mnstrs: Option<Vec<Mnstr>>,
    pub opponent_mnstr: Option<Mnstr>,
    pub opponent_mnstrs: Option<Vec<Mnstr>>,
}
