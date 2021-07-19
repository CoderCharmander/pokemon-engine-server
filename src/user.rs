use std::{collections::HashMap, sync::Arc};

use tokio::sync::{
    mpsc::{self, error::SendError},
    Mutex,
};
use warp::ws::Message;

use crate::messages::WsSentMessage;

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
}

pub type Users = Arc<Mutex<HashMap<String, SingleUser>>>;
pub type SingleUser = User;
