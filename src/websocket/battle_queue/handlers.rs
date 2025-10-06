use futures_util::StreamExt as _;
use rand::Rng;
use redis::AsyncTypedCommands;
use rocket_ws::{Config, Stream, WebSocket, result::Error};

use crate::{
    delete_resource_where_fields, insert_resource,
    models::{
        battle::Battle,
        battle_log::{BattleLog, BattleLogAction},
        battle_status::{BattleStatus, BattleStatusState},
        generated::mnstr_xp::XP_FOR_LEVEL,
        mnstr::Mnstr,
        user::User,
    },
    utils::token::RawToken,
    websocket::{
        battle_queue::models::{
            BattleLogData, BattleQueue, BattleQueueAction, BattleQueueChannel, BattleQueueData,
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
            println!("[build_battle_queue] Message: {:?}", message);
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
            BattleQueueDataAction::Connect => {
                insert_initial_status_and_notify(connection, session_user_id, user_name).await;
                None
            }
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
                        if let Some(challenger_mnstr_id) = battle.challenger_mnstr_id.clone() {
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
                        if let Some(opponent_mnstr_id) = battle.opponent_mnstr_id.clone() {
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
                            queue.data.opponent_id = Some(battle.opponent_id.clone());
                        }

                        let coin_flip = rand::rng().random_range(0..2);
                        let turn_user_id;
                        if coin_flip == 0 {
                            turn_user_id = battle.challenger_id.clone();
                        } else {
                            turn_user_id = battle.opponent_id.clone();
                        }
                        battle_game_data.turn_user_id = Some(turn_user_id);

                        queue.data.data = Some(serde_json::to_string(&battle_game_data).unwrap());
                        if battle.challenger_mnstr_id.is_some()
                            && battle.opponent_mnstr_id.is_some()
                        {
                            queue.data.action = BattleQueueDataAction::GameStarted;
                            queue.action = BattleQueueAction::GameStarted;
                        }
                        println!("[handle_incoming_ws_message] Queue: {:?}", queue);
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
            BattleQueueDataAction::Escape => {
                let game_data = queue.data.data.clone().unwrap();
                let mut game_data: BattleQueueGameData =
                    serde_json::from_str(&game_data.clone()).unwrap();

                if let None = game_data.winner_id.clone() {
                    let winner_id: String;
                    let challenger_mnstr = game_data.challenger_mnstr.clone().unwrap();
                    let opponent_mnstr = game_data.opponent_mnstr.clone().unwrap();

                    if challenger_mnstr.user_id.clone() == session_user_id.clone() {
                        winner_id = opponent_mnstr.user_id.clone();
                    } else {
                        winner_id = challenger_mnstr.user_id.clone();
                    }
                    game_data.winner_id = Some(winner_id);
                    queue.data.data = Some(serde_json::to_string(&game_data).unwrap());
                }

                if let Some(error) = handle_game_ended(&mut queue, session_user_id, user_name).await
                {
                    publish_queue(connection, &error).await;
                    return None;
                }
                publish_queue(connection, &queue).await;
                None
            }
            BattleQueueDataAction::Attack => {
                if let Some(error) = handle_attack(&mut queue, session_user_id, user_name).await {
                    publish_queue(connection, &error).await;
                    return None;
                }
                println!("[handle_attack] Publishing queue: {:?}", queue);
                publish_queue(connection, &queue).await;
                None
            }
            BattleQueueDataAction::Defend => None,
            BattleQueueDataAction::Magic => None,
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

    let coin_flip = rand::rng().random_range(0..2);
    let turn_user_id;
    if coin_flip == 0 {
        turn_user_id = challenger_id.clone();
    } else {
        turn_user_id = opponent_id.clone();
    }

    let battle_queue_game_data_map = BattleQueueGameData {
        battle_id: Some(battle.id.clone()),
        challenger_mnstr: None,
        opponent_mnstr: None,
        challenger_mnstrs: Some(challenger_mnstrs),
        opponent_mnstrs: Some(opponent_mnstrs),
        mnstr: None,
        winner_id: None,
        winner_xp_awarded: None,
        winner_coins_awarded: None,
        loser_xp_awarded: None,
        loser_coins_awarded: None,
        turn_user_id: Some(turn_user_id),
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
    Ok(battle)
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

async fn handle_left(session_user_id: &String) -> Option<String> {
    let mut status =
        match BattleStatus::find_one_by(vec![("user_id", session_user_id.clone().into())]).await {
            Ok(status) => status,
            Err(error) => {
                println!("[handle_left] Failed to find battle status: {:?}", error);
                return Some("Error finding battle status".to_string());
            }
        };
    if let Some(error) = status.delete().await {
        println!("[handle_left] Failed to delete battle status: {:?}", error);
        return Some("Error deleting battle status".to_string());
    }
    None
}

async fn handle_attack(
    queue: &mut BattleQueue,
    session_user_id: &String,
    user_name: &Option<String>,
) -> Option<BattleQueue> {
    let game_data = queue.data.data.clone().unwrap();
    let mut battle_game_data: BattleQueueGameData =
        serde_json::from_str(&game_data.clone()).unwrap();

    let battle_id = battle_game_data.battle_id.clone().unwrap();
    let challenger = battle_game_data.challenger_mnstr.clone().unwrap();
    let opponent = battle_game_data.opponent_mnstr.clone().unwrap();
    let turn_user_id = battle_game_data.turn_user_id.clone().unwrap();

    let mut attacker;
    let mut defender;
    if turn_user_id == challenger.user_id {
        attacker = opponent.clone();
        defender = challenger.clone();
    } else {
        attacker = challenger.clone();
        defender = opponent.clone();
    }

    let attacker_roll = roll_dice(20) + (attacker.current_speed / 20) as i32;
    let defender_roll = roll_dice(20) + (defender.current_intelligence / 20) as i32;

    let mut battle_log_data = BattleLogData {
        missed: None,
        hit: None,
        damage: None,
    };

    let battle_log_action;

    if attacker_roll > defender_roll {
        let attack = attacker_roll;
        if attack > defender.current_defense {
            defender.current_health = 0;
        } else {
            defender.current_health -= attack;
        }

        battle_log_data.hit = Some(true);
        battle_log_data.damage = Some(attack);
        battle_log_action = BattleLogAction::Hit;
        println!("[handle_attack] Hit! {:?}", attack);
    } else {
        battle_log_data.missed = Some(true);
        battle_log_action = BattleLogAction::Missed;
        println!("[handle_attack] Missed");
    }

    let battle_log_data = serde_json::to_string(&battle_log_data).unwrap();
    let mut battle_log = BattleLog::new(
        battle_id.clone(),
        attacker.user_id.clone(),
        attacker.id.clone(),
        battle_log_action,
        battle_log_data,
    );

    println!("[handle_attack] Creating battle log");
    if let Some(error) = battle_log.create().await {
        println!("[handle_attack] Failed to create battle log: {:?}", error);
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Attack,
            "Error creating battle log".to_string(),
        );
        return Some(error_queue);
    }

    attacker.current_attack -= 1;
    attacker.current_speed -= 1;
    defender.current_defense -= 1;
    defender.current_intelligence -= 1;

    println!("[handle_attack] Updating attacker");
    if let Some(error) = attacker.update().await {
        println!("[handle_attack] Failed to update attacker: {:?}", error);
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating attacker".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_attack] Updating defender");
    if let Some(error) = defender.update().await {
        println!("[handle_attack] Failed to update defender: {:?}", error);
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating defender".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_attack] Updating battle game data");
    if attacker.user_id == challenger.user_id {
        battle_game_data.opponent_mnstr = Some(defender.clone());
        battle_game_data.challenger_mnstr = Some(attacker.clone());
    } else {
        battle_game_data.opponent_mnstr = Some(attacker.clone());
        battle_game_data.challenger_mnstr = Some(defender.clone());
    }
    battle_game_data.turn_user_id = Some(defender.user_id.clone());

    if defender.current_health <= 0 {
        println!("[handle_attack] Defender is dead!");
        battle_game_data.winner_id = Some(attacker.user_id.clone());
        queue.data.data = Some(serde_json::to_string(&battle_game_data).unwrap());
        if let Some(error) = handle_game_ended(queue, session_user_id, user_name).await {
            return Some(error);
        }
    } else {
        queue.data.data = Some(serde_json::to_string(&battle_game_data).unwrap());
    }
    None
}

fn roll_dice(number: i32) -> i32 {
    rand::rng().random_range(1..(number + 1))
}

async fn handle_game_ended(
    queue: &mut BattleQueue,
    session_user_id: &String,
    user_name: &Option<String>,
) -> Option<BattleQueue> {
    println!("[handle_game_ended] Ending game");

    println!("[handle_game_ended] Leaving battle");
    if let Some(error) = handle_left(session_user_id).await {
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            queue.data.action.clone(),
            error.clone(),
        );
        return Some(error_queue);
    }

    let raw_game_data = queue.data.data.clone().unwrap();
    let battle_game_data: BattleQueueGameData =
        serde_json::from_str(&raw_game_data.clone()).unwrap();

    println!("[handle_game_ended] Finding battle");
    let mut battle = match Battle::find_one(battle_game_data.battle_id.clone().unwrap()).await {
        Ok(battle) => battle,
        Err(_) => {
            let error_queue = build_error(
                Some(session_user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Battle,
                BattleQueueAction::Error,
                queue.data.action.clone(),
                "Error finding battle".to_string(),
            );
            return Some(error_queue);
        }
    };

    println!("[handle_game_ended] Finding challenger mnstr");
    let challenger_mnstr =
        match Mnstr::find_one(battle.challenger_mnstr_id.clone().unwrap(), false).await {
            Ok(mnstr) => mnstr,
            Err(_) => {
                let error_queue = build_error(
                    Some(session_user_id.clone()),
                    user_name.clone(),
                    BattleQueueChannel::Battle,
                    BattleQueueAction::Error,
                    queue.data.action.clone(),
                    "Error finding challenger mnstr".to_string(),
                );
                return Some(error_queue);
            }
        };

    println!("[handle_game_ended] Finding opponent mnstr");
    let opponent_mnstr =
        match Mnstr::find_one(battle.opponent_mnstr_id.clone().unwrap(), false).await {
            Ok(mnstr) => mnstr,
            Err(_) => {
                let error_queue = build_error(
                    Some(session_user_id.clone()),
                    user_name.clone(),
                    BattleQueueChannel::Battle,
                    BattleQueueAction::Error,
                    queue.data.action.clone(),
                    "Error finding opponent mnstr".to_string(),
                );
                return Some(error_queue);
            }
        };

    println!("[handle_game_ended] Finding winner");
    let winner_user_id: String;
    let winner_mnstr_id: String;
    let loser_user_id: String;
    let loser_mnstr_id: String;

    if let Some(winner_id) = battle_game_data.winner_id.clone() {
        if winner_id == session_user_id.clone() {
            if battle.challenger_id == session_user_id.clone() {
                winner_user_id = challenger_mnstr.user_id.clone();
                winner_mnstr_id = challenger_mnstr.id.clone();
                loser_user_id = opponent_mnstr.user_id.clone();
                loser_mnstr_id = opponent_mnstr.id.clone();
            } else {
                winner_user_id = opponent_mnstr.user_id.clone();
                winner_mnstr_id = opponent_mnstr.id.clone();
                loser_user_id = challenger_mnstr.user_id.clone();
                loser_mnstr_id = challenger_mnstr.id.clone();
            }
        } else {
            if battle.challenger_id != session_user_id.clone() {
                winner_user_id = challenger_mnstr.user_id.clone();
                winner_mnstr_id = challenger_mnstr.id.clone();
                loser_user_id = opponent_mnstr.user_id.clone();
                loser_mnstr_id = opponent_mnstr.id.clone();
            } else {
                winner_user_id = opponent_mnstr.user_id.clone();
                winner_mnstr_id = opponent_mnstr.id.clone();
                loser_user_id = challenger_mnstr.user_id.clone();
                loser_mnstr_id = challenger_mnstr.id.clone();
            }
        }
    } else {
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            queue.data.action.clone(),
            "Error finding winner".to_string(),
        );
        return Some(error_queue);
    }

    battle.winner_id = Some(winner_user_id.clone());
    battle.winner_mnstr_id = Some(winner_mnstr_id.clone());

    println!("[handle_game_ended] Updating battle");
    if let Some(error) = battle.update().await {
        println!(
            "[handle_escape_request] Failed to update battle: {:?}",
            error
        );
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating battle".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Deleting battle");
    if let Some(error) = battle.delete().await {
        println!("[handle_game_ended] Failed to delete battle: {:?}", error);
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error deleting battle".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Finding loser");
    let mut loser = match User::find_one(loser_user_id.clone(), false).await {
        Ok(user) => user,
        Err(_) => {
            let error_queue = build_error(
                Some(session_user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Battle,
                BattleQueueAction::Error,
                BattleQueueDataAction::Escape,
                "Error finding loser".to_string(),
            );
            return Some(error_queue);
        }
    };

    println!("[handle_game_ended] Finding loser mnstr");
    let mut loser_mnstr = match Mnstr::find_one(loser_mnstr_id.clone(), false).await {
        Ok(mnstr) => mnstr,
        Err(_) => {
            let error_queue = build_error(
                Some(session_user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Battle,
                BattleQueueAction::Error,
                BattleQueueDataAction::Escape,
                "Error finding loser mnstr".to_string(),
            );
            return Some(error_queue);
        }
    };

    println!("[handle_game_ended] Finding winner");
    let mut winner = match User::find_one(winner_user_id.clone(), false).await {
        Ok(user) => user,
        Err(_) => {
            let error_queue = build_error(
                Some(session_user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Battle,
                BattleQueueAction::Error,
                BattleQueueDataAction::Escape,
                "Error finding winner".to_string(),
            );
            return Some(error_queue);
        }
    };

    println!("[handle_game_ended] Finding winner mnstr");
    let mut winner_mnstr = match Mnstr::find_one(winner_mnstr_id.clone(), false).await {
        Ok(mnstr) => mnstr,
        Err(_) => {
            let error_queue = build_error(
                Some(session_user_id.clone()),
                user_name.clone(),
                BattleQueueChannel::Battle,
                BattleQueueAction::Error,
                BattleQueueDataAction::Escape,
                "Error finding winner mnstr".to_string(),
            );
            return Some(error_queue);
        }
    };

    println!("[handle_game_ended] Updating winner");
    let xp_to_next_level = XP_FOR_LEVEL[loser_mnstr.current_level as usize + 1];
    let winner_xp_awarded = (xp_to_next_level as f64 / 4.0).floor() as i32;
    let loser_xp_awarded = (xp_to_next_level as f64 / 8.0).floor() as i32;
    let winner_coins_awarded = loser_mnstr.coins();
    let loser_coins_awarded = 5;

    println!("[handle_game_ended] Updating winner xp");
    if let Some(error) = winner.update_xp(winner_xp_awarded).await {
        println!(
            "[handle_escape_request] Failed to update winner xp: {:?}",
            error
        );
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating winner xp".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Updating winner coins");
    if let Some(error) = winner.add_coins(winner_coins_awarded).await {
        println!(
            "[handle_escape_request] Failed to update winner coins: {:?}",
            error
        );
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating winner coins".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Updating winner mnstr xp");
    if let Some(error) = winner_mnstr.update_xp(winner_xp_awarded).await {
        println!(
            "[handle_escape_request] Failed to update winner xp: {:?}",
            error
        );
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating winner xp".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Updating loser");
    if let Some(error) = loser.update_xp(loser_xp_awarded).await {
        println!(
            "[handle_escape_request] Failed to update loser xp: {:?}",
            error
        );
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating loser xp".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Updating loser coins");
    if let Some(error) = loser.add_coins(loser_coins_awarded).await {
        println!(
            "[handle_escape_request] Failed to update loser coins: {:?}",
            error
        );
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating loser coins".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Updating loser xp");
    if let Some(error) = loser_mnstr.update_xp(loser_xp_awarded).await {
        println!(
            "[handle_escape_request] Failed to update loser xp: {:?}",
            error
        );
        let error_queue = build_error(
            Some(session_user_id.clone()),
            user_name.clone(),
            BattleQueueChannel::Battle,
            BattleQueueAction::Error,
            BattleQueueDataAction::Escape,
            "Error updating loser xp".to_string(),
        );
        return Some(error_queue);
    }

    println!("[handle_game_ended] Updating battle game data");
    let battle_game_data = BattleQueueGameData {
        winner_id: Some(winner_user_id),
        opponent_mnstr: Some(opponent_mnstr),
        challenger_mnstr: Some(challenger_mnstr),
        battle_id: Some(battle.id.clone()),
        challenger_mnstrs: None,
        opponent_mnstrs: None,
        mnstr: None,
        winner_xp_awarded: Some(winner_xp_awarded),
        winner_coins_awarded: Some(winner_coins_awarded),
        loser_coins_awarded: Some(loser_coins_awarded),
        loser_xp_awarded: Some(loser_xp_awarded),
        turn_user_id: None,
    };

    println!("[handle_game_ended] Updating battle queue");
    queue.data.data = Some(serde_json::to_string(&battle_game_data).unwrap());
    queue.data.user_id = Some(battle.challenger_id.clone());
    queue.data.opponent_id = Some(battle.opponent_id.clone());
    queue.data.action = BattleQueueDataAction::GameEnded;
    queue.action = BattleQueueAction::GameEnded;
    None
}
