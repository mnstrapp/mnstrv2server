use futures_util::StreamExt as _;
use redis::AsyncTypedCommands;
use rocket_ws::{Config, Stream, WebSocket, result::Error};

use crate::{
    delete_resource_where_fields, insert_resource,
    models::{
        battle::Battle,
        battle_status::{BattleStatus, BattleStatusState},
        mnstr::Mnstr,
        user::User,
    },
    utils::token::RawToken,
    websocket::{
        battle_queue::models::{
            BattleQueue, BattleQueueAction, BattleQueueChannel, BattleQueueData,
            BattleQueueDataAction, BattleQueueGameData,
        },
        helpers::verify_session_token,
    },
};

#[get("/battle_queue/<token>")]
pub async fn battle_queue(ws: WebSocket, token: RawToken) -> Stream!['static] {
    let ws = ws.config(Config::default());
    let session = match verify_session_token(token).await {
        Ok(session) => Some(session),
        Err(err) => {
            println!("Invalid session: {:?}", err);
            None
        }
    };
    let mut user_name: Option<String> = None;
    if let Some(session_ref) = session.as_ref() {
        match User::find_one(session_ref.user_id.clone(), false).await {
            Ok(user) => {
                user_name = Some(user.display_name);
            }
            Err(err) => {
                println!("Error getting user: {:?}", err);
            }
        }
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
            let (client, mut connection) = match open_redis_with_connection().await {
                Ok((client, connection)) => (client, connection),
                Err(err) => {
                    println!("[redis] Error initializing Redis: {:?}", err);
                    yield serde_json::to_string(&build_error(
                        None,
                        user_name.clone(),
                        BattleQueueChannel::Lobby,
                        BattleQueueAction::Error,
                        BattleQueueDataAction::Connect,
                        "Error connecting to Redis".to_string(),
                    )).unwrap().into();
                    return;
                }
            };

            // Valid session is guaranteed below
            let session = session.unwrap();
            let session_user_id = session.user_id.clone();

            // Subscribe to battle queue
            let mut rx = subscribe_and_forward(&client).await;

            // Insert battle status and notify lobby
            insert_initial_status_and_notify(
                &mut connection,
                &session_user_id,
                &user_name,
            ).await;

            // Ping connection: this prevents redis timeouts
            spawn_redis_ping(connection.clone());

            let user_name = user_name.clone();

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
                                if let Ok(msg) = &message {
                                    if msg.is_empty() {
                                        on_player_left(&mut connection, &session_user_id, &user_name).await;
                                        continue;
                                    }
                                }
                            if let Some(payload) = handle_incoming_ws_message(message, &mut connection, &session_user_id, &user_name).await {
                                yield payload.into();
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

// Extracted: Open redis client and a multiplexed connection
async fn open_redis_with_connection()
-> Result<(redis::Client, redis::aio::MultiplexedConnection), Error> {
    let client = connect_to_redis().await?;
    let connection = client.get_multiplexed_async_connection().await.unwrap();
    Ok((client, connection))
}

// Extracted: Subscribe and forward pubsub messages into an internal channel
async fn subscribe_and_forward(
    client: &redis::Client,
) -> rocket::tokio::sync::mpsc::UnboundedReceiver<String> {
    let mut pubsub = client.get_async_pubsub().await.unwrap();
    pubsub.subscribe("battle_queue").await.unwrap();
    let mut pubsub_stream = pubsub.into_on_message();
    let (tx, rx) = rocket::tokio::sync::mpsc::unbounded_channel::<String>();
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
    rx
}

// Extracted: Spawn background ping to keep connection alive with reconnection attempts
fn spawn_redis_ping(mut connection: redis::aio::MultiplexedConnection) {
    rocket::tokio::spawn(async move {
        loop {
            match connection.ping().await {
                Ok(_) => {
                    rocket::tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
                Err(err) => {
                    println!("[redis] ping failed: {:?}", err);
                    if let Ok(client) = connect_to_redis().await {
                        match client.get_multiplexed_async_connection().await {
                            Ok(new_conn) => {
                                println!("[redis] ping reconnected successfully");
                                connection = new_conn;
                            }
                            Err(reconn_err) => {
                                println!("[redis] ping reconnect failed: {:?}", reconn_err);
                            }
                        }
                    }
                    rocket::tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    });
}

// Extracted: Insert initial battle status and notify lobby
async fn insert_initial_status_and_notify(
    connection: &mut redis::aio::MultiplexedConnection,
    user_id: &String,
    user_name: &Option<String>,
) {
    let mut battle_status = BattleStatus::new(
        user_id.clone(),
        user_name.clone().unwrap(),
        None,
        None,
        None,
        BattleStatusState::InQueue,
    );
    match battle_status.create().await {
        None => {
            let battle_queue = build_success(
                Some(user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Lobby,
                BattleQueueAction::Joined,
                BattleQueueDataAction::Connect,
                "In the battle queue".to_string(),
            );
            publish_queue(connection, &battle_queue).await;
        }
        Some(err) => {
            println!("[battle_queue] Error inserting battle status: {:?}", err);
            let battle_queue = build_error(
                Some(user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Lobby,
                BattleQueueAction::Error,
                BattleQueueDataAction::Connect,
                "Error updating battle status".to_string(),
            );
            publish_queue(connection, &battle_queue).await;
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

// Message handling helpers
async fn publish_queue(connection: &mut redis::aio::MultiplexedConnection, queue: &BattleQueue) {
    let payload = serde_json::to_string(&queue).unwrap();
    match queue.action {
        BattleQueueAction::Ping => {}
        _ => {
            // println!("[publish_queue] Queue: {:?}", payload);
        }
    }
    connection.publish("battle_queue", payload).await.unwrap();
}

async fn on_player_left(
    connection: &mut redis::aio::MultiplexedConnection,
    user_id: &String,
    user_name: &Option<String>,
) {
    // Best-effort cleanup of battle status
    match delete_resource_where_fields!(BattleStatus, vec![("user_id", user_id.clone().into())])
        .await
    {
        Ok(_) => {
            println!("[battle_queue_handler] Battle status deleted");
        }
        Err(err) => {
            println!(
                "[battle_queue_handler] Error deleting battle status: {:?}",
                err
            );
        }
    };

    let battle_queue = build_error(
        Some(user_id.clone()),
        user_name.clone(),
        BattleQueueChannel::Lobby,
        BattleQueueAction::Left,
        BattleQueueDataAction::Left,
        "Player left the battle queue".to_string(),
    );
    publish_queue(connection, &battle_queue).await;
}

// Extracted handler for incoming websocket messages
async fn handle_incoming_ws_message(
    message: Result<rocket_ws::Message, Error>,
    connection: &mut redis::aio::MultiplexedConnection,
    session_user_id: &String,
    user_name: &Option<String>,
) -> Option<String> {
    // Return early if message is empty
    if let Ok(msg) = &message {
        if msg.is_empty() {
            return None;
        }
    }

    match build_battle_queue(message) {
        Ok(mut queue) => match queue.data.action {
            BattleQueueDataAction::List => {
                match handle_list_request(session_user_id, user_name).await {
                    Ok(payload) => Some(payload),
                    Err(_) => Some(
                        serde_json::to_string(&build_error(
                            Some(session_user_id.clone()),
                            user_name.clone(),
                            BattleQueueChannel::Lobby,
                            BattleQueueAction::Error,
                            BattleQueueDataAction::List,
                            "Error getting list of players in the battle queue".to_string(),
                        ))
                        .unwrap(),
                    ),
                }
            }

            // BattleQueueDataAction::Challenge => {
            //     if let Err(_) =
            //         handle_challenge_request(&queue, session_user_id, user_name, connection).await
            //     {
            //         let error_queue = build_error(
            //             Some(session_user_id.clone()),
            //             user_name.clone(),
            //             BattleQueueChannel::Lobby,
            //             BattleQueueAction::Error,
            //             BattleQueueDataAction::Challenge,
            //             "Error challenging opponent".to_string(),
            //         );
            //         publish_queue(connection, &error_queue).await;
            //     }
            //     None
            // }
            BattleQueueDataAction::Accept => {
                if let Err(_) =
                    handle_accept_challenge(&queue, session_user_id, user_name, connection).await
                {
                    let error_queue = build_error(
                        Some(session_user_id.clone()),
                        user_name.clone(),
                        BattleQueueChannel::Lobby,
                        BattleQueueAction::Error,
                        BattleQueueDataAction::Accept,
                        "Error accepting challenge".to_string(),
                    );
                    publish_queue(connection, &error_queue).await;
                }
                None
            }
            BattleQueueDataAction::MnstrChosen => {
                let raw_game_data = queue.data.data.clone().unwrap();
                let mut battle_game_data: BattleQueueGameData =
                    serde_json::from_str(&raw_game_data.clone()).unwrap();
                match update_battle_mnstrs(
                    &battle_game_data.battle_id.clone().unwrap(),
                    &battle_game_data.challenger_mnstr.clone(),
                    &battle_game_data.opponent_mnstr.clone(),
                )
                .await
                {
                    Ok(battle) => {
                        battle_game_data.battle_id = Some(battle.id.clone());
                        if let Some(challenger_mnstr_id) = battle.challenger_mnstr_id {
                            let challenger_mnstr =
                                match Mnstr::find_one(challenger_mnstr_id, false).await {
                                    Ok(mnstr) => mnstr,
                                    Err(_) => {
                                        let error_queue = build_error(
                                            Some(session_user_id.clone()),
                                            user_name.clone(),
                                            BattleQueueChannel::Lobby,
                                            BattleQueueAction::Error,
                                            BattleQueueDataAction::MnstrChosen,
                                            "Error finding challenger mnstr".to_string(),
                                        );
                                        publish_queue(connection, &error_queue).await;
                                        return None;
                                    }
                                };
                            battle_game_data.challenger_mnstr = Some(challenger_mnstr);
                            queue.data.user_id = Some(battle.challenger_id.clone());
                        }
                        if let Some(opponent_mnstr_id) = battle.opponent_mnstr_id {
                            let opponent_mnstr =
                                match Mnstr::find_one(opponent_mnstr_id, false).await {
                                    Ok(mnstr) => mnstr,
                                    Err(_) => {
                                        let error_queue = build_error(
                                            Some(session_user_id.clone()),
                                            user_name.clone(),
                                            BattleQueueChannel::Lobby,
                                            BattleQueueAction::Error,
                                            BattleQueueDataAction::MnstrChosen,
                                            "Error finding opponent mnstr".to_string(),
                                        );
                                        publish_queue(connection, &error_queue).await;
                                        return None;
                                    }
                                };
                            battle_game_data.opponent_mnstr = Some(opponent_mnstr);
                            queue.data.opponent_id = Some(battle.opponent_id);
                        }
                        queue.data.data = Some(serde_json::to_string(&battle_game_data).unwrap());
                        if battle_game_data.challenger_mnstr.is_some()
                            && battle_game_data.opponent_mnstr.is_some()
                        {
                            queue.data.action = BattleQueueDataAction::GameStarted;
                            queue.action = BattleQueueAction::GameStarted;
                        }
                        publish_queue(connection, &queue).await;
                        None
                    }
                    Err(_) => {
                        let error_queue = build_error(
                            Some(session_user_id.clone()),
                            user_name.clone(),
                            BattleQueueChannel::Lobby,
                            BattleQueueAction::Error,
                            BattleQueueDataAction::MnstrChosen,
                            "Error choosing mnstr".to_string(),
                        );
                        publish_queue(connection, &error_queue).await;
                        None
                    }
                }
            }
            BattleQueueDataAction::Rejoin => {
                let raw_game_data = queue.data.data.clone().unwrap();
                let mut battle_game_data: BattleQueueGameData =
                    serde_json::from_str(&raw_game_data.clone()).unwrap();
                println!(
                    "[handle_rejoin_request] Battle game data: {:?}",
                    battle_game_data
                );
                if let None = battle_game_data.battle_id {
                    let error_queue = build_error(
                        Some(session_user_id.clone()),
                        user_name.clone(),
                        BattleQueueChannel::Battle,
                        BattleQueueAction::Error,
                        BattleQueueDataAction::Rejoin,
                        "Error rejoining battle".to_string(),
                    );
                    publish_queue(connection, &error_queue).await;
                    return None;
                }
                let battle_id = battle_game_data.battle_id.clone().unwrap();
                match handle_rejoin_request(&battle_id).await {
                    Ok(battle) => {
                        let params = vec![
                            ("user_id", session_user_id.clone().into()),
                            ("status", BattleStatusState::InQueue.to_string().into()),
                        ];
                        let error = match BattleStatus::find_one_by(params).await {
                            Ok(mut status) => {
                                status.delete().await;
                                None
                            }
                            Err(_) => {
                                println!(
                                    "[handle_rejoin_request] Error deleting old battle status"
                                );
                                Some(anyhow::Error::msg("Error deleting old battle status"))
                            }
                        };
                        if let Some(_) = error {
                            publish_queue(
                                connection,
                                &build_error(
                                    Some(session_user_id.clone()),
                                    user_name.clone(),
                                    BattleQueueChannel::Battle,
                                    BattleQueueAction::Error,
                                    BattleQueueDataAction::Rejoin,
                                    "Error deleting old battle status".to_string(),
                                ),
                            )
                            .await;
                            return None;
                        }

                        let challenger_mnstr = match Mnstr::find_one(
                            battle.challenger_mnstr_id.clone().unwrap(),
                            false,
                        )
                        .await
                        {
                            Ok(mnstr) => mnstr,
                            Err(_) => {
                                return None;
                            }
                        };
                        battle_game_data.challenger_mnstr = Some(challenger_mnstr);
                        queue.data.user_id = Some(battle.challenger_id.clone());

                        let opponent_mnstr =
                            match Mnstr::find_one(battle.opponent_mnstr_id.clone().unwrap(), false)
                                .await
                            {
                                Ok(mnstr) => mnstr,
                                Err(_) => {
                                    return None;
                                }
                            };
                        battle_game_data.opponent_mnstr = Some(opponent_mnstr);
                        queue.data.opponent_id = Some(battle.opponent_id.clone());

                        queue.data.data = Some(serde_json::to_string(&battle_game_data).unwrap());
                        queue.data.action = BattleQueueDataAction::Rejoined;
                        queue.action = BattleQueueAction::Rejoined;
                        publish_queue(connection, &queue).await;
                        None
                    }
                    Err(_) => {
                        let error_queue = build_error(
                            Some(session_user_id.clone()),
                            user_name.clone(),
                            BattleQueueChannel::Battle,
                            BattleQueueAction::Error,
                            BattleQueueDataAction::Rejoin,
                            "Error rejoining battle".to_string(),
                        );
                        publish_queue(connection, &error_queue).await;
                        return None;
                    }
                }
            }
            BattleQueueDataAction::InGameAction => None,
            _ => {
                publish_queue(connection, &queue).await;
                None
            }
        },
        Err(err) => {
            println!(
                "[battle_queue_handler] Error building battle queue: {:?}",
                err
            );
            // Notify others and cleanup
            on_player_left(connection, session_user_id, user_name).await;
            None
        }
    }
}

async fn handle_list_request(
    requester_user_id: &String,
    user_name: &Option<String>,
) -> Result<String, ()> {
    let list = BattleStatus::find_all().await.map_err(|_| ())?;
    let list = list
        .into_iter()
        .filter(|item| item.user_id != *requester_user_id)
        .collect::<Vec<_>>();

    let list = list.into_iter().fold(Vec::new(), |mut acc, item| {
        if !acc.iter().any(|x: &BattleStatus| x.user_id == item.user_id) {
            acc.push(item);
        }
        acc
    });

    let mut battle_queue = build_success(
        Some(requester_user_id.clone()),
        user_name.clone(),
        BattleQueueChannel::Lobby,
        BattleQueueAction::List,
        BattleQueueDataAction::List,
        "List of players in the battle queue".to_string(),
    );
    battle_queue.data.data = Some(serde_json::to_string(&list).unwrap());
    Ok(serde_json::to_string(&battle_queue).unwrap())
}

async fn handle_accept_challenge(
    queue: &BattleQueue,
    session_user_id: &String,
    user_name: &Option<String>,
    connection: &mut redis::aio::MultiplexedConnection,
) -> Result<(), ()> {
    let mut queue = queue.clone();
    println!("[handle_accept_challenge] Queue: {:?}", queue.clone());
    let opponent_id = queue.data.opponent_id.clone().unwrap();
    let challenger_id = queue.data.user_id.clone().unwrap();

    let battle = match create_battle(&challenger_id, &opponent_id).await {
        Ok(battle) => battle,
        Err(_) => {
            let error = build_error(
                Some(session_user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Lobby,
                BattleQueueAction::Error,
                BattleQueueDataAction::Challenge,
                "Error creating battle".to_string(),
            );
            publish_queue(connection, &error).await;
            return Err(());
        }
    };

    let error = match handle_accept_request(
        &opponent_id,
        &Some(challenger_id.clone()),
        &Some(battle.id.clone()),
    )
    .await
    {
        None => None,

        Some(_) => Some(build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Lobby,
            BattleQueueAction::Error,
            BattleQueueDataAction::Accept,
            "Error accepting challenge".to_string(),
        )),
    };
    if let Some(error) = error {
        publish_queue(connection, &error).await;
        return Err(());
    }

    let challenger_mnstrs = match load_mnstrs(&challenger_id.clone()).await {
        Ok(mnstrs) => mnstrs,
        Err(_) => {
            publish_queue(
                connection,
                &build_error(
                    Some(session_user_id.clone()),
                    user_name.clone(),
                    BattleQueueChannel::Lobby,
                    BattleQueueAction::Error,
                    BattleQueueDataAction::Challenge,
                    "Error loading mnstrs".to_string(),
                ),
            )
            .await;
            return Err(());
        }
    };
    if let Some(error) = error {
        publish_queue(connection, &error).await;
        return Err(());
    }

    let opponent_mnstrs = match load_mnstrs(&opponent_id.clone()).await {
        Ok(mnstrs) => mnstrs,
        Err(_) => {
            publish_queue(
                connection,
                &build_error(
                    Some(session_user_id.clone()),
                    user_name.clone(),
                    BattleQueueChannel::Lobby,
                    BattleQueueAction::Error,
                    BattleQueueDataAction::Challenge,
                    "Error loading mnstrs".to_string(),
                ),
            )
            .await;
            return Err(());
        }
    };

    let battle_queue_game_data_map = BattleQueueGameData {
        battle_id: Some(battle.id.clone()),
        challenger_mnstr: None,
        opponent_mnstr: None,
        challenger_mnstrs: Some(challenger_mnstrs),
        opponent_mnstrs: Some(opponent_mnstrs),
        mnstr: None,
    };

    let battle_queue_game_data = serde_json::to_string(&battle_queue_game_data_map).unwrap();

    queue.data.data = Some(battle_queue_game_data);
    queue.data.action = BattleQueueDataAction::GameStarted;
    queue.action = BattleQueueAction::GameStarted;

    publish_queue(connection, &queue).await;
    Ok(())
}

async fn handle_accept_request(
    challenger_id: &String,
    opponent_id: &Option<String>,
    battle_id: &Option<String>,
) -> Option<anyhow::Error> {
    let challenger = match User::find_one(challenger_id.clone(), false).await {
        Ok(challenger) => challenger,
        Err(_) => return Some(anyhow::Error::msg("Error finding challenger")),
    };

    let opponent = match User::find_one(opponent_id.clone().unwrap(), false).await {
        Ok(opponent) => opponent,
        Err(_) => return Some(anyhow::Error::msg("Error finding opponent")),
    };

    let params = vec![("user_id", challenger.id.clone().into())];
    let mut status = match BattleStatus::find_one_by(params).await {
        Ok(status) => status,
        Err(_) => return Some(anyhow::Error::msg("Error finding battle status")),
    };

    status.opponent_id = opponent_id.clone();
    status.opponent_name = Some(opponent.display_name.clone());

    if let Some(battle_id) = battle_id {
        status.battle_id = Some(battle_id.clone());
    }
    status.status = BattleStatusState::InBattle;

    if let Some(error) = status.update().await {
        println!(
            "[handle_accept_request] Failed to update battle status: {:?}",
            error
        );
        return Some(error.into());
    }

    let params = vec![("user_id", opponent_id.clone().into())];
    let mut status = match BattleStatus::find_one_by(params).await {
        Ok(status) => status,
        Err(_) => return Some(anyhow::Error::msg("Error finding battle status")),
    };

    status.opponent_id = Some(challenger.id.clone());
    status.opponent_name = Some(challenger.display_name.clone());

    if let Some(battle_id) = battle_id {
        status.battle_id = Some(battle_id.clone());
    }
    status.status = BattleStatusState::InBattle;

    if let Some(error) = status.update().await {
        println!(
            "[handle_accept_request] Failed to update battle status: {:?}",
            error
        );
        return Some(error.into());
    }

    None
}

async fn create_battle(challenger_id: &String, opponent_id: &String) -> Result<Battle, ()> {
    let challenger = User::find_one(challenger_id.clone(), false)
        .await
        .map_err(|_| ())?;
    let opponent = User::find_one(opponent_id.clone(), false)
        .await
        .map_err(|_| ())?;
    let mut battle = Battle::new(
        challenger.id,
        challenger.display_name,
        opponent.id,
        opponent.display_name,
    );
    if let Some(error) = battle.create().await {
        println!("[create_battle] Failed to create battle: {:?}", error);
        return Err(());
    }
    Ok(battle)
}

async fn update_battle_mnstrs(
    battle_id: &String,
    challenger_mnstr: &Option<Mnstr>,
    opponent_mnstr: &Option<Mnstr>,
) -> Result<Battle, anyhow::Error> {
    println!("[update_battle_mnstrs] Battle id: {:?}", battle_id);

    let mut battle = match Battle::find_one(battle_id.clone()).await {
        Ok(battle) => battle,
        Err(error) => {
            println!("[update_battle_mnstrs] Failed to find battle: {:?}", error);
            return Err(error.into());
        }
    };
    println!("[update_battle_mnstrs] Battle: {:?}", battle);
    if let Some(challenger_mnstr) = challenger_mnstr {
        println!(
            "[update_battle_mnstrs] Challenger mnstr: {:?}",
            challenger_mnstr.id.clone()
        );
        battle.challenger_mnstr_id = Some(challenger_mnstr.id.clone());
    }
    if let Some(opponent_mnstr) = opponent_mnstr {
        println!(
            "[update_battle_mnstrs] Opponent mnstr: {:?}",
            opponent_mnstr.id.clone()
        );
        battle.opponent_mnstr_id = Some(opponent_mnstr.id.clone());
    }
    if let Some(error) = battle.update().await {
        println!("[update_battle] Failed to update battle: {:?}", error);
        return Err(error.into());
    }
    match battle.update().await {
        None => Ok(battle),
        Some(error) => Err(error.into()),
    }
}

async fn load_mnstrs(user_id: &String) -> Result<Vec<Mnstr>, ()> {
    let mnstrs = Mnstr::find_all_by(vec![("user_id", user_id.clone().into())], false)
        .await
        .map_err(|_| ())?;
    Ok(mnstrs)
}

async fn handle_rejoin_request(battle_id: &String) -> Result<Battle, ()> {
    let battle = match Battle::find_one(battle_id.clone()).await {
        Ok(battle) => battle,
        Err(error) => {
            println!("[handle_rejoin_request] Failed to find battle: {:?}", error);
            return Err(());
        }
    };
    Ok(battle)
}
