use futures_util::StreamExt as _;
use redis::AsyncTypedCommands;
use rocket_ws::{Config, Stream, WebSocket, result::Error};

use crate::{
    delete_resource_where_fields, find_all_resources_where_fields, insert_resource,
    models::user::User,
    utils::token::RawToken,
    websocket::{
        battle_queue::models::{
            BattleQueue, BattleQueueAction, BattleQueueChannel, BattleQueueData,
            BattleQueueDataAction, BattleStatus, BattleStatusState,
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
            // Check for valid session
            if let None = session {
                let battle_queue = build_error(
                    None,
                    user_name,
                    BattleQueueChannel::Lobby,
                    BattleQueueAction::Error,
                    BattleQueueDataAction::Connect,
                    "Invalid session".to_string(),
                );
                yield serde_json::to_string(&battle_queue).unwrap().into();
                return;
            }

            // Open Redis connection
            let client = match connect_to_redis().await {
                Ok(client) => client,
                Err(err) => {
                    println!("[redis] Error connecting to Redis: {:?}", err);
                    yield serde_json::to_string(&build_error(
                        Some(session.as_ref().unwrap().user_id.clone()),
                        user_name.clone(),
                        BattleQueueChannel::Lobby,
                        BattleQueueAction::Error,
                        BattleQueueDataAction::Connect,
                        "Error connecting to Redis".to_string(),
                    )).unwrap().into();
                    return;
                }
            };
            let connection = match client.get_multiplexed_async_connection().await {
                Ok(connection) => Some(connection),
                Err(err) => {
                    println!("[redis] Error getting connection: {:?}", err);
                    None
                }
            };

            // Check for valid connection
            if let None = connection {
                let battle_queue = build_error(
                    Some(session.as_ref().unwrap().user_id.clone()),
                    user_name.clone(),
                    BattleQueueChannel::Lobby,
                    BattleQueueAction::Error,
                    BattleQueueDataAction::Connect,
                    "Error getting connection".to_string(),
                );
                yield serde_json::to_string(&battle_queue).unwrap().into();
                return;
            }

            // Get Redis connection
            let mut connection = connection.unwrap();

            // Subscribe to battle queue
            let mut pubsub = client.get_async_pubsub().await.unwrap();
            pubsub.subscribe("battle_queue").await.unwrap();

            let mut pubsub_stream = pubsub.into_on_message();
            let (tx, mut rx) = rocket::tokio::sync::mpsc::unbounded_channel::<String>();

            // Insert battle status
            match insert_resource!(BattleStatus, vec![
                ("user_id", session.as_ref().unwrap().user_id.clone().into()),
                ("display_name", user_name.clone().unwrap().into()),
                ("status", BattleStatusState::InQueue.to_string().into()),
                ("connected", true.into()),
            ]).await {
                Ok(_) => {
                    // Publish battle queue
                    let battle_queue = build_success(
                        Some(session.as_ref().unwrap().user_id.clone()),
                        user_name.clone(),
                        BattleQueueChannel::Lobby,
                        BattleQueueAction::Joined,
                        BattleQueueDataAction::Connect,
                        "In the battle queue".to_string(),
                    );
                    connection.publish("battle_queue", serde_json::to_string(&battle_queue).unwrap()).await.unwrap();
                }
                Err(err) => {
                    println!("[battle_queue] Error inserting battle status: {:?}", err);
                    let battle_queue = build_error(
                        Some(session.as_ref().unwrap().user_id.clone()),
                        user_name.clone(),
                        BattleQueueChannel::Lobby,
                        BattleQueueAction::Error,
                        BattleQueueDataAction::Connect,
                        "Error updating battle status".to_string(),
                    );
                    connection.publish("battle_queue", serde_json::to_string(&battle_queue).unwrap()).await.unwrap();
                }
            }

            // Ping connection: this prevents redis timeouts
            let mut ping_connection = connection.clone();
            rocket::tokio::spawn(async move {
                loop {
                    ping_connection.ping().await.unwrap();
                    rocket::tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
            });

            // Watch for messages from the battle queue and clients
            let session_clone = session.clone();
            let user_name = user_name.clone();
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

            // React to incoming messages from the battle queue and clients
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
                                // Build battle queue
                                let queue = match build_battle_queue(message) {
                                    Ok(queue) => Some(queue),
                                    Err(err) => {
                                        println!("[battle_queue_handler] Error building battle queue: {:?}", err);
                                        // Build error
                                        let battle_queue = build_error(
                                            Some(session_clone.as_ref().unwrap().user_id.clone()),
                                            user_name.clone(),
                                            BattleQueueChannel::Lobby,
                                            BattleQueueAction::Left,
                                            BattleQueueDataAction::Left,
                                            "Player left the battle queue".to_string(),
                                        );
                                        connection.publish("battle_queue", serde_json::to_string(&battle_queue).unwrap()).await.unwrap();
                                        // Delete battle status
                                        match delete_resource_where_fields!(BattleStatus, vec![("user_id", session_clone.as_ref().unwrap().user_id.clone().into())]).await {
                                            Ok(_) => {
                                                println!("[battle_queue_handler] Battle status deleted");
                                            },
                                            Err(err) => {
                                                println!("[battle_queue_handler] Error deleting battle status: {:?}", err);
                                            }
                                        };
                                        // Publish battle queue
                                        connection.publish("battle_queue", serde_json::to_string(&battle_queue).unwrap()).await.unwrap();
                                        None
                                    }
                                };
                                if let Some(queue) = queue {
                                    match queue.data.action {
                                        BattleQueueDataAction::List => {
                                            let list = match find_all_resources_where_fields!(BattleStatus, vec![("status", BattleStatusState::InQueue.to_string().into())]).await {
                                                Ok(list) => list,
                                                Err(err) => {
                                                    println!("[battle_queue_handler] Error getting list of players in the battle queue: {:?}", err);
                                                    yield serde_json::to_string(&build_error(
                                                        Some(session_clone.as_ref().unwrap().user_id.clone()),
                                                        user_name.clone(),
                                                        BattleQueueChannel::Lobby,
                                                        BattleQueueAction::Error,
                                                        BattleQueueDataAction::List,
                                                        "Error getting list of players in the battle queue".to_string(),
                                                    )).unwrap().into();
                                                    return;
                                                }
                                            };

                                            let list: Vec<_> = list
                                                .into_iter()
                                                .filter(|item| item.user_id != session_clone.as_ref().unwrap().user_id)
                                                .collect();

                                            let mut battle_queue = build_success(
                                                Some(session_clone.as_ref().unwrap().user_id.clone()),
                                                user_name.clone(),
                                                BattleQueueChannel::Lobby,
                                                BattleQueueAction::List,
                                                BattleQueueDataAction::List,
                                                "List of players in the battle queue".to_string(),
                                            );
                                            battle_queue.data.data = Some(serde_json::to_string(&list).unwrap());
                                            yield serde_json::to_string(&battle_queue).unwrap().into();
                                        }
                                        _ => {
                                            // Publish battle queue
                                            connection.publish("battle_queue", serde_json::to_string(&queue).unwrap()).await.unwrap();
                                        }
                                    }
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

fn build_battle_queue(message: Result<rocket_ws::Message, Error>) -> Result<BattleQueue, Error> {
    let message = match message {
        Ok(message) => message.into_text()?.to_string(),
        Err(err) => return Err(err),
    };

    if message.is_empty() {
        return Ok(build_error(
            None,
            None,
            BattleQueueChannel::Lobby,
            BattleQueueAction::Error,
            BattleQueueDataAction::Error,
            "Invalid message".to_string(),
        ));
    }

    println!("[build_battle_queue] Message: {}", message);

    let queue: BattleQueue = match serde_json::from_str(&message) {
        Ok(queue) => queue,
        Err(err) => {
            println!(
                "[build_battle_queue] Error building battle queue: {:?}",
                err
            );
            return Ok(build_error(
                None,
                None,
                BattleQueueChannel::Lobby,
                BattleQueueAction::Error,
                BattleQueueDataAction::Error,
                "Invalid message".to_string(),
            ));
        }
    };

    Ok(queue)
}

fn build_error(
    user_id: Option<String>,
    user_name: Option<String>,
    channel: BattleQueueChannel,
    action: BattleQueueAction,
    data_action: BattleQueueDataAction,
    error: String,
) -> BattleQueue {
    let battle_queue_data = BattleQueueData::new(
        data_action,
        user_id.clone(),
        user_name,
        None,
        None,
        None,
        None,
        Some(error),
        None,
    );
    let battle_queue = BattleQueue::new(user_id, channel, action, battle_queue_data);
    battle_queue
}

fn build_success(
    user_id: Option<String>,
    user_name: Option<String>,
    channel: BattleQueueChannel,
    action: BattleQueueAction,
    data_action: BattleQueueDataAction,
    message: String,
) -> BattleQueue {
    let battle_queue_data = BattleQueueData::new(
        data_action,
        user_id.clone(),
        user_name,
        None,
        None,
        None,
        None,
        None,
        Some(message),
    );
    let battle_queue = BattleQueue::new(user_id, channel, action, battle_queue_data);
    battle_queue
}

async fn connect_to_redis() -> Result<redis::Client, Error> {
    let config = std::env::var("REDIS_URL").unwrap();
    let client = redis::Client::open(config).unwrap();
    Ok(client)
}
