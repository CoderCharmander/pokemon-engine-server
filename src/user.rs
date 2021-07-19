use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tokio::sync::{
    mpsc::{self, error::SendError},
    Mutex,
};
use warp::ws::Message;

use crate::{
    messages::WsSentMessage,
    room::Room,
};

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub tx: mpsc::UnboundedSender<Message>,
    pub current_room_id: Option<String>,
}

impl User {
    pub fn send<T: WsSentMessage>(&self, msg: T) -> Result<(), SendError<Message>> {
        self.tx.send(msg.into_message())
    }

    pub fn exit_room<T>(&mut self, rooms: &mut T) -> Option<()>
    where
        T: DerefMut + Deref<Target = HashMap<String, Room>>,
    {
        let current_room_id = self.current_room_id.as_ref()?;
        let room = rooms
            .get_mut(current_room_id)
            .unwrap();
        if room.users.len() == 1 {
            rooms.remove(current_room_id);
            return Some(());
        }
        let idx = room.users.iter().position(|s| s == &self.name).unwrap();
        room.users.remove(idx);
        Some(())
    }
}

pub type Users = Arc<Mutex<HashMap<String, SingleUser>>>;
pub type SingleUser = User;
