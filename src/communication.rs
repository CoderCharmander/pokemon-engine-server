use futures::{SinkExt, StreamExt};
use log::{error, info};
use rand::Rng;
use tokio::sync::mpsc::{self, error::SendError, UnboundedSender};
use warp::ws::{Message, WebSocket};

use crate::{
    battle::handle_battle_request,
    messages::*,
    rooms::{exit_room, Room, RoomBattleStatus, Rooms},
    user::{SingleUser, User, Users},
};

pub async fn ws_handler(ws: WebSocket, name: String, users: Users, rooms: Rooms) {
    let (mut sock_tx, mut sock_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let user = User {
        name,
        tx: tx.clone(),
        current_room_id: None,
    };
    {
        let mut users = users.lock().await;
        if users.contains_key(&user.name) {
            sock_tx
                .send(UserExistsMessage {}.into_message())
                .await
                .unwrap();
            sock_tx.close().await.unwrap();
            return;
        }
        users.insert(user.name.clone(), user.clone());
    }

    info!("Client connected with name: {}", user.name);

    let welcome = WelcomeMessage { name: &user.name };
    {
        let users = users.lock().await;
        broadcast(
            users.values().filter(|u| u.current_room_id.is_none()),
            welcome.into_message(),
        )
        .await
        .unwrap();
    }

    {
        let user = user.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = sock_tx.send(msg).await {
                    error!("Error while sending a message to {0}: {1}", user.name, e);
                    break;
                }
            }
        });
    }

    while let Some(msg) = sock_rx.next().await {
        let msg = match msg {
            Ok(msg) => {
                if !msg.is_text() {
                    continue;
                } else {
                    msg
                }
            }
            Err(e) => {
                error!("While receiving from {}: {}", user.name, e);
                break;
            }
        };
        let parsed = match parse_message(msg) {
            Err(e) => {
                error!("While parsing a message from {}: {}", user.name, e);
                continue;
            }
            Ok(msg) => msg,
        };
        handle_message(parsed, users.clone(), rooms.clone(), &user.name)
            .await
            .unwrap();
    }
    info!("Client disconnected");
    let mut users = users.lock().await;
    let mut user = users.remove(&user.name).unwrap();
    exit_room(&mut rooms.lock().await, &mut user);
}

async fn handle_message(
    msg: WsMessage,
    users: Users,
    rooms: Rooms,
    username: &str,
) -> Result<(), ()> {
    let mut users = users.lock().await;
    let mut user = users.get_mut(username).unwrap();
    match msg {
        WsMessage::Chat(ChatMessage { msg }) => {
            let chat = ChatNotifyReply {
                msg,
                source_name: user.name.clone(),
            };
            match &user.current_room_id {
                Some(id) => {
                    let rooms = rooms.lock().await;
                    let room = &rooms[id];
                    for username in room.users.iter() {
                        users[username].tx.send(chat.into_message()).unwrap();
                    }
                }
                None => {
                    if let Err(e) = broadcast(users.values(), chat.into_message()).await {
                        error!("While broadcasting a chat message: {}", e);
                        return Err(());
                    }
                }
            }
        }
        WsMessage::RoomCreationRequest(_) => {
            let room_id: String = rand::thread_rng()
                .sample_iter(crate::UppercaseAlphanumericDistribution::new())
                .take(5)
                .collect();
            {
                // let mut users = users.lock().await;
                // let user = users.get_mut(&user.name).unwrap();

                let mut rooms = rooms.lock().await;
                let mut room = Room {
                    battle: RoomBattleStatus::None,
                    users: Vec::new(),
                };
                if user.current_room_id.is_some() {
                    exit_room(&mut rooms, user);
                }
                user.current_room_id = Some(room_id.clone());
                room.users.push(user.name.clone());
                rooms.insert(room_id.clone(), room);
            }
            if let Err(e) = user.tx.send(RoomCreationReply { room_id }.into_message()) {
                error!(
                    "While sending a room creation reply to {}: {}",
                    user.name, e
                );
                return Err(());
            }
        }
        WsMessage::RoomJoinRequest(RoomJoinRequest { room_id }) => {
            let mut rooms = rooms.lock().await;
            if user.current_room_id.is_some() {
                exit_room(&mut rooms, user);
            }
            let room = match rooms.get_mut(&room_id) {
                Some(r) => r,
                None => {
                    user.tx
                        .send(
                            RoomJoinReply {
                                room_id,
                                succeeded: false,
                            }
                            .into_message(),
                        )
                        .unwrap();
                    return Ok(());
                }
            };
            room.users.push(user.name.clone());
            user.current_room_id = Some(room_id.clone());
            user.tx
                .send(
                    RoomJoinReply {
                        room_id,
                        succeeded: true,
                    }
                    .into_message(),
                )
                .unwrap();
        }
        WsMessage::RoomExitRequest(_) => {
            let mut rooms = rooms.lock().await;
            exit_room(&mut rooms, user);
            user.current_room_id = None;
        }
        WsMessage::BattleStartRequest(req) => {
            // if let Some(room_id) = &user.current_room_id {
            //     let mut rooms = rooms.lock().await;
            //     let mut room = rooms.get_mut(room_id).unwrap();
            //     if !room.users.contains(&other_user) {
            //         send_request_error(&user.tx, "battle_opponent_not_found").unwrap();
            //         return Ok(());
            //     }

            //     let username = user.name.clone();
            //     let user = users.get(&username).unwrap();
            //     let other_user = users.get(&other_user).unwrap();

            //     // let mut party_items = vec![];
            //     // for d in dragon_names {
            //     //     let party_item = match create_dragon(&d) {
            //     //         Some(d) => PartyItem::new(d),
            //     //         None => {
            //     //             send_request_error(&user.tx, "invalid_dragon_name");
            //     //             return Ok(());
            //     //         }
            //     //     };
            //     //     party_items.push(party_item);
            //     // }

            //     if dragon_names.len() > 6 {
            //         send_request_error(&user.tx, "too_many_party_items");
            //         return Ok(());
            //     }

            //     room.battle = match &room.battle {
            //         RoomBattleStatus::None => RoomBattleStatus::Prepared {
            //             starter_username: username,
            //             starter_party: dragon_names,
            //             other_username: other_user.name.clone(),
            //         },
            //         RoomBattleStatus::Prepared {
            //             starter_username,
            //             other_username,
            //             starter_party,
            //         } => {
            //             if &user.name != other_username || &other_user.name != starter_username {
            //                 send_request_error(&user.tx, "another_battle_already_prepared");
            //             }
            //             RoomBattleStatus::None
            //             // RoomBattleStatus::Started(Battle {
            //             //     battlefield: Battlefield::new(
            //             //         Party::new_from_vec(party_items),
            //             //         NopMessenger,
            //             //     ),
            //             //     usernames: (starter_username.clone(), other_username.clone())
            //             // })
            //         }
            //         &RoomBattleStatus::Started(_) => return Ok(()),
            //     }
            //     // room.battle = Some(Battle {
            //     //     usernames: (user.name.clone(), other_user.name.clone()),

            //     // })
            // } else {
            //     send_request_error(&user.tx, "no_battle_in_main_room").unwrap();
            //     return Ok(());
            // }
            handle_battle_request(req, users, rooms.lock().await, username).await;
        }
        _ => {
            user.tx
                .send(
                    RequestErrorMessage {
                        reason: "invalid_command".to_string(),
                    }
                    .into_message(),
                )
                .unwrap();
        }
    }
    Ok(())
}

async fn broadcast<'a, T: Iterator<Item = &'a SingleUser>>(
    users: T,
    message: Message,
) -> Result<(), SendError<Message>> {
    for user in users {
        user.tx.send(message.clone())?;
    }
    Ok(())
}

pub fn send_request_error(
    message_tx: &UnboundedSender<Message>,
    error_str: &str,
) -> Result<(), SendError<Message>> {
    message_tx.send(
        RequestErrorMessage {
            reason: String::from(error_str),
        }
        .into_message(),
    )
}
