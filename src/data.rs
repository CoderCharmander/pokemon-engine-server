use std::collections::HashMap;

use lazy_static::lazy_static;
use pokemon_engine::dragon::{BattleDragon, DragonData};

lazy_static! {
    pub static ref DRAGONS: HashMap<String, DragonData> = load_dragons();
}

fn load_dragons() -> HashMap<String, DragonData> {
    serde_json::from_str(include_str!("dragons.json")).unwrap()
}

pub fn create_dragon(name: &str) -> Option<BattleDragon> {
    DRAGONS.get(name).map(|d| BattleDragon::new(d.base_stats))
}