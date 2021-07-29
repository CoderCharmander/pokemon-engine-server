use std::collections::HashMap;

use lazy_static::lazy_static;
use pokemon_engine::{
    dragon::{BattleDragon, DragonData},
    moves::{MoveStats, MoveTrait, SimpleDamagingMove},
};
use serde::Deserialize;

use crate::battle::ServerMessenger;

pub mod moves;

lazy_static! {
    static ref DRAGONS: HashMap<String, DragonData> = load_dragons();
    static ref SIMPLE_DAMAGING_MOVES: HashMap<String, SimpleMoveData> = load_simple_moves();
}

fn load_dragons() -> HashMap<String, DragonData> {
    serde_json::from_str(include_str!("data/dragons.json")).unwrap()
}

pub fn create_dragon(name: &str) -> Option<BattleDragon> {
    DRAGONS.get(name).map(|d| BattleDragon::new(d.base_stats))
}

#[derive(Deserialize)]
struct SimpleMoveData {
    base_power: u32,
    name: String,
    #[serde(default)]
    crit_boost: u8,
}

fn load_simple_moves() -> HashMap<String, SimpleMoveData> {
    serde_json::from_str(include_str!("data/simple_moves.json")).unwrap()
}

fn create_simple_move(name: &str) -> Option<SimpleDamagingMove> {
    SIMPLE_DAMAGING_MOVES.get(name).map(
        |SimpleMoveData {
             base_power,
             name,
             crit_boost,
         }| SimpleDamagingMove::new_crit(name.to_owned(), *base_power, *crit_boost),
    )
}

pub fn create_move(move_name: &str) -> Option<Box<dyn MoveTrait<ServerMessenger>>> {
    if let Some(simple_move) = create_simple_move(move_name) {
        return Some(Box::new(simple_move));
    }

    None
}
