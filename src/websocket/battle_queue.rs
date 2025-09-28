use rocket_ws::{Config, Stream, WebSocket};
use serde::{Deserialize, Serialize};
use sqlx::{Error, Postgres, Row, postgres::PgRow};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    database::traits::DatabaseResource,
    utils::{
        time::{deserialize_offset_date_time, serialize_offset_date_time},
        token::RawToken,
    },
    websocket::helpers::verify_session_token,
};

#[get("/battle_queue")]
pub async fn battle_queue(ws: WebSocket, token: RawToken) -> Stream!['static] {
    let ws = ws.config(Config::default());
    let session = match verify_session_token(token).await {
        Ok(session) => Some(session),
        Err(err) => {
            println!("Invalid session: {:?}", err);
            None
        }
    };

    Stream! { ws =>
        if let None = session {
            let battle_queue_data = BattleQueueData::new(
                BattleQueueDataAction::Connect,
                None,
                None,
                None,
                None,
                Some("Invalid session".to_string()),
            );
            let battle_queue = BattleQueue::new(
                None,
                BattleQueueChannel::Lobby,
                BattleQueueAction::Error,
                battle_queue_data,
            );
            yield serde_json::to_string(&battle_queue).unwrap().into();
            return;
        }

        let battle_queue_data = BattleQueueData::new(
            BattleQueueDataAction::Connect,
            Some(session.as_ref().unwrap().user_id.clone()),
            None,
            None,
            None,
            Some("In the battle queue".to_string()),
        );

        let battle_queue = BattleQueue::new(
            Some(session.as_ref().unwrap().user_id.clone()),
            BattleQueueChannel::Lobby,
            BattleQueueAction::Joined,
            battle_queue_data,
        );
        println!("Battle queue: {:?}", battle_queue);
        yield serde_json::to_string(&battle_queue).unwrap().into();

        for await message in ws {
            yield message?;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum BattleQueueChannel {
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
enum BattleQueueAction {
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
            _ => BattleQueueAction::Joined,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BattleQueue {
    id: String,
    user_id: Option<String>,
    channel: BattleQueueChannel,
    action: BattleQueueAction,
    data: BattleQueueData,

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
    fn new(
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
enum BattleQueueDataAction {
    Connect,
    Cancel,
    Ready,
    Unready,
    Start,
    Watch,
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
            _ => BattleQueueDataAction::Connect,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BattleQueueData {
    action: BattleQueueDataAction,
    user_id: Option<String>,
    opponent_id: Option<String>,
    mnstr_id: Option<String>,
    error: Option<String>,
    message: Option<String>,
}

impl BattleQueueData {
    fn new(
        action: BattleQueueDataAction,
        user_id: Option<String>,
        opponent_id: Option<String>,
        mnstr_id: Option<String>,
        error: Option<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            action,
            user_id,
            opponent_id,
            mnstr_id,
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
            opponent_id: None,
            mnstr_id: None,
            error: Some("Invalid data".to_string()),
            message: None,
        });
        data
    }
}
