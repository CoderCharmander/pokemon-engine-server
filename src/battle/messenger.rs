use pokemon_engine::battle::Messenger;
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;
use crate::messages::{self, WsSentMessage};

pub struct RoomNotifierMessenger {
    room_channel: UnboundedSender<Message>
}

impl Messenger for RoomNotifierMessenger {
    fn on_attack(&self, field: &pokemon_engine::battle::Battlefield<Self>, party: pokemon_engine::party::PartyId, move_name: &str) {
        self.room_channel.send(messages::UseMoveNotify {
            party: party.into(),
            move_name: move_name.into()
        }.into_message()).unwrap();
    }

    fn on_damage(&self, field: &pokemon_engine::battle::Battlefield<Self>, party: pokemon_engine::party::PartyId, amount: u32) {
        
    }

    fn on_switch(&self, field: &pokemon_engine::battle::Battlefield<Self>, party: pokemon_engine::party::PartyId, original: u8, switched: u8) {
        
    }

    fn on_effect_applied(&self, field: &pokemon_engine::battle::Battlefield<Self>, party: pokemon_engine::party::PartyId, effect_desc: &str) {
        
    }
}