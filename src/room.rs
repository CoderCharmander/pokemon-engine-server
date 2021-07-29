use std::{collections::HashMap, ops::Deref, sync::Arc};

use pokemon_engine::battle::{Battlefield, NopMessenger};

use tokio::sync::Mutex;

use crate::{battle::RoomBattleStatus, messages::WsSentMessage, user::User};

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

    pub fn broadcast<U, M: WsSentMessage>(&self, users: U, message: M)
    where
        U: Deref<Target = HashMap<String, User>>,
    {
        for user in self.users.iter() {
            let user = &users[user];
            user.send_raw(message.into_message()).unwrap();
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
