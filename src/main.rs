use std::{collections::HashMap, sync::Arc};

use rand::distributions::Uniform;
use rooms::Rooms;
use tokio::sync::Mutex;
use warp::{ws::Ws, Filter};

use crate::user::Users;

mod communication;
mod data;
mod error;
mod handlers;
mod messages;
mod rooms;
mod user;
mod battle;

pub struct UppercaseAlphanumericDistribution(Uniform<usize>);
impl UppercaseAlphanumericDistribution {
    pub fn new() -> Self {
        Self(Uniform::new(0, 36))
    }
}
impl rand::distributions::Distribution<char> for UppercaseAlphanumericDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> char {
        let slice: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
        slice.chars().nth(rng.sample(self.0)).unwrap()
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let users = Arc::new(Mutex::new(HashMap::new()));
    let rooms = Arc::new(Mutex::new(HashMap::new()));

    let with_users = warp::any().map(move || users.clone());
    let with_rooms = warp::any().map(move || rooms.clone());

    let health_endpoint = warp::get()
        .and(warp::path("health"))
        .and(warp::path::end())
        .and_then(handlers::health_check);
    let echo_endpoint = warp::path("echo")
        .and(warp::path::param())
        .and(warp::ws())
        .and(with_users)
        .and(with_rooms)
        .map(|name: String, ws: Ws, users: Users, rooms: Rooms| {
            ws.on_upgrade(move |ws| communication::ws_handler(ws, name, users, rooms))
        });
    let register_room_endpoint = warp::path("register-room").and_then(handlers::register_room);

    warp::serve(health_endpoint.or(echo_endpoint).or(register_room_endpoint))
        .run(([0, 0, 0, 0], 8000))
        .await;
}

// fn with_broadcast_channels<T: Send>(
//     tx: broadcast::Sender<T>,
// ) -> impl Filter<Extract = ((broadcast::Sender<T>, broadcast::Receiver<T>),), Error = Infallible> + Clone
// {
//     warp::any().map(move || (tx.clone(), tx.subscribe()))
// }
