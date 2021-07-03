use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use pokemon_engine::battle::{Battlefield, NopMessenger};

use tokio::sync::Mutex;

use crate::user::User;

pub struct Battle {
    pub usernames: (String, String),
    pub battlefield: Battlefield<NopMessenger>,
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

pub struct Room {
    pub users: Vec<String>,
    pub battle: RoomBattleStatus,
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;

// impl Room {
//     pub fn new() -> Self {
//         let (tx, rx) = mpsc::unbounded_channel();
//         Self {
//             users: HashMap::new(),
//             tx,
//             rx,
//             battle: None,
//         }
//     }
//     pub async fn run(&mut self, id: &str) {
//         while let Some(message) = self.rx.recv().await {
//             let msg = match parse_message(message) {
//                 Ok(msg) => msg,
//                 Err(e) => {
//                     error!("While parsing a message in Room {}: {}", id, e);
//                     continue;
//                 }
//             };
//             match msg {

//             }
//         }
//     }
// }

pub fn exit_room<T: DerefMut + Deref<Target = HashMap<String, Room>>>(
    rooms: &mut T,
    user: &mut User,
) {
    let room = rooms
        .get_mut(user.current_room_id.as_ref().unwrap())
        .unwrap();
    if room.users.len() == 1 {
        rooms.remove(user.current_room_id.as_ref().unwrap());
        return;
    }
    let idx = room.users.iter().position(|s| s == &user.name).unwrap();
    room.users.remove(idx);
}
