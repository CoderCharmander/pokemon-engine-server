use std::{
    collections::HashMap,
    sync::Arc,
};

use pokemon_engine::battle::{Battlefield, NopMessenger};

use tokio::sync::Mutex;

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

impl Room {
    pub fn new(initial_user: String) -> Self {
        Self {
            users: vec![initial_user],
            battle: RoomBattleStatus::None,
        }
    }
}

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