use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use pokemon_engine::{
    battle::{Battlefield, NopMessenger},
    party::{Party, PartyId, PartyItem},
};

use crate::{
    communication::send_request_error,
    data::{create_dragon, create_move},
    messages::*,
    room::Room,
    user::User,
};

pub type ServerMessenger = NopMessenger;

pub struct Battle {
    pub usernames: (String, String),
    pub prepared_action: Option<(PartyId, BattleAction)>,
    pub battlefield: Battlefield<ServerMessenger>,
}

impl Battle {
    pub fn user_party_id(&self, username: &str) -> Option<PartyId> {
        if self.usernames.0 == username {
            Some(PartyId::Party1)
        } else if self.usernames.1 == username {
            Some(PartyId::Party2)
        } else {
            None
        }
    }

    pub fn party_id_user(&self, party_id: PartyId) -> &str {
        match party_id {
            PartyId::Party1 => &self.usernames.0,
            PartyId::Party2 => &self.usernames.1,
        }
    }
}

pub enum RoomBattleStatus {
    None,
    Prepared {
        starter_username: String,
        starter_party: Vec<String>,
        other_username: String,
    },
    Started(Battle),
}

pub enum BattleAction {
    UseMove(String),
    Switch(u8),
}

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
        RoomBattleStatus::None => {
            other_user
                .send(BattleInvitation {
                    other_user: source_user.name.clone(),
                })
                .unwrap();
            RoomBattleStatus::Prepared {
                starter_username: String::from(source_username),
                starter_party: dragon_names,
                other_username: other_user.name.clone(),
            }
        }
        RoomBattleStatus::Prepared {
            starter_username,
            other_username,
            starter_party: starter_dragon_names,
        } => {
            if &source_user.name != other_username || &other_user.name != starter_username {
                send_request_error(&source_user.tx, "another_battle_already_prepared").unwrap();
            }

            // The user that started the battle invite
            let starter_user = users.get(starter_username).unwrap();

            // The party we're going to give to the starter user
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
                        send_request_error(&starter_user.tx, "invalid_party_item").unwrap();
                        return;
                    }
                }));
            }

            // The party we'll give to the other user
            let mut other_party = vec![];
            for d in dragon_names.iter() {
                other_party.push(PartyItem::new(match create_dragon(d) {
                    Some(d) => d,
                    None => {
                        send_request_error(&source_user.tx, "invalid_party_item").unwrap();
                        return;
                    }
                }));
            }

            starter_user
                .send(BattleStartNotify {
                    other_party: dragon_names,
                })
                .unwrap();

            source_user
                .send(BattleStartNotify {
                    other_party: starter_dragon_names.clone(),
                })
                .unwrap();

            RoomBattleStatus::Started(Battle {
                battlefield: Battlefield::new(
                    Party::new_from_vec(starter_party),
                    Party::new_from_vec(other_party),
                    NopMessenger,
                ),
                prepared_action: None,
                usernames: (starter_username.clone(), other_username.clone()),
            })
        }
        &RoomBattleStatus::Started(_) => {
            send_request_error(&source_user.tx, "ongoing_battle").unwrap();
            return;
        }
    }
}

pub async fn handle_in_battle_request<U, R>(
    req: WsMessage,
    users: U,
    mut rooms: R,
    source_username: String,
) where
    U: DerefMut + Deref<Target = HashMap<String, User>>,
    R: DerefMut + Deref<Target = HashMap<String, Room>>,
{
    let source_user = &users[&source_username];
    let room_id = match &source_user.current_room_id {
        Some(id) => id,
        None => {
            source_user
                .send_request_error("no_battle_in_main_room")
                .unwrap();
            return;
        }
    };
    let room = rooms.get_mut(room_id).unwrap();
    let battle = match &mut room.battle {
        RoomBattleStatus::None | RoomBattleStatus::Prepared { .. } => {
            source_user
                .send_request_error("no_battle_initiated")
                .unwrap();
            return;
        }
        RoomBattleStatus::Started(battle) => battle,
    };
    let source_party_id = if let Some(id) = battle.user_party_id(&source_username) {
        id
    } else {
        source_user.send_request_error("not_in_battle").unwrap();
        return;
    };
    let source_party = battle.battlefield.party_mut(source_party_id);

    let battle_action = match req {
        WsMessage::UseMoveRequest(req) => BattleAction::UseMove(req.move_name),
        WsMessage::SwitchRequest(req) => BattleAction::Switch(req.next_dragon),
        _ => unreachable!(),
    };

    if let Some((party_id, action)) = &battle.prepared_action {
      //  execute_battle_action(action, *party_id, &mut battle.battlefield, room, users);
    }
}

fn execute_battle_action<U>(
    action: &BattleAction,
    party_id: PartyId,
    battlefield: &mut Battlefield<ServerMessenger>,
    room: &Room,
    users: U,
) -> Option<()>
where
    U: Deref<Target = HashMap<String, User>>,
{
    match action {
        BattleAction::UseMove(move_name) => {
            let attack = create_move(&move_name)?;
            battlefield.attack(party_id, attack.as_ref());
        }
        BattleAction::Switch(new_dragon) => {
            if !battlefield.party_mut(party_id).switch(*new_dragon as usize) {
                room.broadcast(
                    users,
                    SwitchNotify {
                        party: party_id.into(),
                        next_idx: *new_dragon,
                        switch_allowed: false,
                    },
                );
            }
        }
    }
    Some(())
}
