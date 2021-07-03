use std::{collections::HashMap, sync::Arc};

use tokio::sync::{mpsc, Mutex};
use warp::ws::Message;

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub tx: mpsc::UnboundedSender<Message>,
    pub current_room_id: Option<String>,
}

pub type Users = Arc<Mutex<HashMap<String, SingleUser>>>;
pub type SingleUser = User;
