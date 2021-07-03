use rand::Rng;

use warp::{reply::json, Rejection, Reply};

use crate::messages::{self, *};

pub async fn health_check() -> Result<impl Reply, Rejection> {
    Ok(json(&messages::HealthReply { code: 200 }))
}

pub async fn register_room() -> Result<impl Reply, Rejection> {
    let room_id = rand::thread_rng()
        .sample_iter(&crate::UppercaseAlphanumericDistribution::new())
        .take(5)
        .collect();
    Ok(json(
        &messages::RoomCreationReply { room_id }.into_jsonable(),
    ))
}
