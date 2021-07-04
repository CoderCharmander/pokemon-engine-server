use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use pokemon_engine::{battle::{Battlefield, NopMessenger}, party::{Party, PartyItem}};

use crate::{
    communication::send_request_error,
    data::create_dragon,
    messages::*,
    rooms::{Battle, Room, RoomBattleStatus},
    user::User,
};

pub async fn handle_battle_request<U, R>(
    BattleStartRequest {
        dragon_names,
        other_user,
    }: BattleStartRequest,
    users: U,
    mut rooms: R,
    source_username: &str,
) where
    U: DerefMut + Deref<Target = HashMap<String, User>>,
    R: DerefMut + Deref<Target = HashMap<String, Room>>,
{
    let source_user = users.get(source_username).unwrap();
    let room_id = match &source_user.current_room_id {
        Some(room_id) => room_id,
        None => {
            send_request_error(&source_user.tx, "no_battle_in_main_room").unwrap();
            return;
        }
    };
    let mut room = rooms.get_mut(room_id).unwrap();
    if !room.users.contains(&other_user) {
        send_request_error(&source_user.tx, "battle_opponent_not_found").unwrap();
        return;
    }

    let other_user = users.get(&other_user).unwrap();

    if dragon_names.len() > 6 {
        send_request_error(&source_user.tx, "too_many_party_items").unwrap();
        return;
    }

    room.battle = match &room.battle {
        RoomBattleStatus::None => RoomBattleStatus::Prepared {
            starter_username: String::from(source_username),
            starter_party: dragon_names,
            other_username: other_user.name.clone(),
        },
        RoomBattleStatus::Prepared {
            starter_username,
            other_username,
            starter_party: starter_dragon_names,
        } => {
            if &source_user.name != other_username || &other_user.name != starter_username {
                send_request_error(&source_user.tx, "another_battle_already_prepared").unwrap();
            }
            let mut starter_party = vec![];
            for d in starter_dragon_names.iter() {
                starter_party.push(PartyItem::new(match create_dragon(d) {
                    Some(d) => d,
                    None => {
                        send_request_error(
                            &source_user.tx,
                            "invalid_party_item_in_requester_party",
                        )
                        .unwrap();
                        let other_user = users.get(starter_username).unwrap();
                        send_request_error(&other_user.tx, "invalid_party_item").unwrap();
                        return;
                    }
                }));
            }
            let mut other_party = vec![];
            for d in dragon_names {
                other_party.push(PartyItem::new(match create_dragon(&d) {
                    Some(d) => d,
                    None => {
                        send_request_error(&source_user.tx, "invalid_party_item").unwrap();
                        return;
                    }
                }));
            }
            RoomBattleStatus::Started(Battle {
                battlefield: Battlefield::new(
                    Party::new_from_vec(starter_party),
                    Party::new_from_vec(other_party),
                    NopMessenger,
                ),
                usernames: (starter_username.clone(), other_username.clone()),
            })
        }
        &RoomBattleStatus::Started(_) => return,
    }
}
