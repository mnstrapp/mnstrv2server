use futures_util::StreamExt as _;
use redis::AsyncTypedCommands;
use rocket_ws::{Config, Message, Stream, WebSocket, result::Error};

use crate::{
    models::user::User,
    utils::token::RawToken,
    websocket::{
        battle_queue::models::{
            BattleQueue, BattleQueueAction, BattleQueueChannel, BattleQueueData,
            BattleQueueDataAction,
        },
        helpers::verify_session_token,
    },
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
    let user = match User::find_one(session.as_ref().unwrap().user_id.clone(), false).await {
        Ok(user) => Some(user),
        Err(err) => {
            println!("Error getting user: {:?}", err);
            None
        }
    };

    let mut user_name = None;
    if let Some(user) = user {
        user_name = Some(user.display_name);
    }

    Stream! { ws => {
            if let None = session {
                let battle_queue_data = BattleQueueData::new(
                    BattleQueueDataAction::Connect,
                    None,
                    user_name,
                    None,
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
                user_name,
                None,
                None,
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

            // yield serde_json::to_string(&battle_queue).unwrap().into();

            let config = std::env::var("REDIS_URL").unwrap();
            let client = redis::Client::open(config).unwrap();
            let connection = match client.get_multiplexed_async_connection().await {
                Ok(connection) => Some(connection),
                Err(err) => {
                    println!("[redis] Error getting connection: {:?}", err);
                    None
                }
            };

            if let None = connection {
                return;
            }

            let mut connection = connection.unwrap();

            let mut pubsub = client.get_async_pubsub().await.unwrap();
            pubsub.subscribe("battle_queue").await.unwrap();

            let mut pubsub_stream = pubsub.into_on_message();
            let (tx, mut rx) = rocket::tokio::sync::mpsc::unbounded_channel::<String>();

            connection.publish("battle_queue", serde_json::to_string(&battle_queue).unwrap()).await.unwrap();

            let mut ping_connection = connection.clone();
            rocket::tokio::spawn(async move {
                loop {
                    ping_connection.ping().await.unwrap();
                    rocket::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            });

            rocket::tokio::spawn(async move {
                loop {
                    let message = match pubsub_stream.next().await {
                        Some(m) => m,
                        None => break,
                    };
                    let payload: String = match message.get_payload() {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    let _ = tx.send(payload);
                }
            });

            let mut ws = ws;
            loop {
                rocket::tokio::select! {
                    maybe_payload = rx.recv() => {
                        match maybe_payload {
                            Some(payload) => {
                                yield payload.into();
                            },
                            None => { /* channel closed */ }
                        }
                    },
                    maybe_message = ws.next() => {
                        match maybe_message {
                            Some(message) => {
                                let queue = match build_battle_queue(message) {
                                    Ok(queue) => Some(queue),
                                    Err(err) => {
                                        println!("[battle_queue] Error building battle queue: {:?}", err);
                                        None
                                    }
                                };

                                if let Some(queue) = queue {
                                    let returned_message = handle_message(&queue).unwrap();
                                    yield returned_message.into();
                                    connection.publish("battle_queue", serde_json::to_string(&queue).unwrap()).await.unwrap();
                                }
                            },
                            None => break,
                        }
                    }
                }
            }
        }
    }
}

// async fn handle_notification(notification: PgNotification) -> Result<rocket_ws::Message, Error> {
//     let message = handle_message(notification).await.unwrap();
//     Ok(message)
// }

fn build_battle_queue(message: Result<rocket_ws::Message, Error>) -> Result<BattleQueue, Error> {
    let message = match message {
        Ok(message) => message.into_text()?.to_string(),
        Err(err) => return Err(err),
    };

    let queue: BattleQueue = match serde_json::from_str(&message) {
        Ok(queue) => queue,
        Err(err) => {
            println!("[handle_message] Error parsing message: {:?}", err);
            println!(
                "[handle_message] Json: {:?}",
                formatjson::format_json(&message)
            );
            let battle_queue_data = BattleQueueData::new(
                BattleQueueDataAction::Connect,
                None,
                None,
                None,
                None,
                None,
                None,
                Some("Invalid message".to_string()),
                None,
            );
            let battle_queue = BattleQueue::new(
                None,
                BattleQueueChannel::Lobby,
                BattleQueueAction::Error,
                battle_queue_data,
            );

            return Ok(battle_queue);
        }
    };

    Ok(queue)
}

fn handle_message(queue: &BattleQueue) -> Result<rocket_ws::Message, Error> {
    let new_message = Message::from(serde_json::to_string(queue).unwrap());
    Ok(new_message)
}
