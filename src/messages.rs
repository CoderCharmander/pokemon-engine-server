use serde::{Deserialize, Serialize};
use serde_json::{from_value, json};
use warp::ws::Message;

use crate::error::StringError;

macro_rules! server_reply {
    ($name:ident $json_name:expr => { $($field_name:ident : $field_type:ty),* $(,)? }) => {
        #[derive(Serialize, Deserialize)]
        pub struct $name {
            $(
                pub $field_name : $field_type ,
            )*
        }
        impl WsSentMessage for $name {
            fn get_type() -> &'static str {
                $json_name
            }
        }
    };
}

macro_rules! client_message {
    ($name:ident $json_name:expr => { $($field_name:ident : $field_type:ty),* $(,)? }) => {
        #[derive(Deserialize)]
        pub struct $name {
            $(
                pub $field_name: $field_type ,
            )*
        }
        impl WsRecvMessage<'_> for $name {
            fn get_type() -> &'static str {
                $json_name
            }
        }
    };
}

macro_rules! json_struct {
    (message $name:ident $json_name:expr => { $($field_name:ident : $field_type:ty),* }) => {
        client_message!{$name $json_name => { $($field_name : $field_type , )* }}
    };
    (reply $name:ident $json_name:expr => { $($field_name:ident : $field_type:ty),* }) => {
        server_reply!{$name $json_name => { $($field_name : $field_type , )* } }
    };
}

macro_rules! json_structs {
    ($(
        $type:ident $enum_name:ident $struct_name:ident $json_name:expr => {
            $($field_name:ident : $field_type:ty),* $(,)?
        }
    )+) => {
        $(
            json_struct!{ $type $struct_name $json_name => { $($field_name : $field_type),* } }
        )*
        pub enum WsMessage {
            $(
                $enum_name($struct_name)
            ),*
        }
        pub fn parse_message(msg: Message) -> Result<WsMessage, Box<dyn std::error::Error>> {
            let msg_str = msg.to_str().map_err(|_| "Not a string message")?;
            let recv_msg: ProtocolMessage = serde_json::from_str(msg_str)?;
            Ok(match recv_msg.action.as_str() {
                $(
                    $json_name => WsMessage::$enum_name(from_value(recv_msg.data)?),
                )*
                _ => return Err(Box::new(StringError("invalid action".to_string()))),
            })
        }
    };
}

json_structs! {
    reply RequestError RequestErrorMessage "request_error" => {
        reason: String,
    }

    reply UserExists UserExistsMessage "user_exists" => {}

    message Chat ChatMessage "chat" => {
        msg: String,
    }
    reply ChatNotify ChatNotifyReply "chat_notify" => {
        msg: String,
        source_name: String,
    }

    message RoomCreationRequest RoomCreationRequest "create_room" => {}
    reply RoomCreation RoomCreationReply "room_created" => {
        room_id: String,
    }

    message RoomJoinRequest RoomJoinRequest "join_room" => {
        room_id: String,
    }
    reply RoomJoinReply RoomJoinReply "room_join_status" => {
        room_id: String,
        succeeded: bool,
    }
    message RoomExitRequest RoomExitRequest "leave_room" => {}

    message BattleStartRequest BattleStartRequest "start_battle" => {
        other_user: String,
        dragon_names: Vec<String>,
    }

    reply BattleInvitation BattleInvitation "battle_invite" => {
        other_user: String
    }

    reply BattleStartNotify BattleStartNotify "battle_start" => {
        other_party: Vec<String>,
    }
}

#[derive(Serialize)]
pub struct HealthReply {
    pub code: u16,
}

pub trait WsSentMessage: Serialize {
    fn get_type() -> &'static str;
    fn into_message(&self) -> Message {
        Message::text(json!({"action": Self::get_type(), "data": self}).to_string())
    }
    fn into_jsonable(&self) -> ProtocolMessage {
        ProtocolMessage {
            action: Self::get_type().to_string(),
            data: serde_json::json!(self),
        }
    }
}

// pub enum WsMessage<'a> {
//     Welcome(WelcomeMessage<'a>),
//     Chat(ChatMessage),
// }

#[derive(Serialize, Deserialize)]
pub struct ProtocolMessage {
    action: String,
    data: serde_json::Value,
}
pub trait WsRecvMessage<'a> {
    fn get_type() -> &'static str;
    fn from_message(msg: Message) -> Result<Self, Box<dyn std::error::Error>>
    where
        for<'de> Self: Deserialize<'de>,
    {
        let msg_str = msg.to_str().map_err(|_| "Not a string message")?;
        let recv_msg: ProtocolMessage = serde_json::from_str(msg_str)?;
        Ok(serde_json::from_value(recv_msg.data)?)
    }
}

// pub fn parse_message<'a>(msg: Message) -> Result<WsMessage<'a>, Box<dyn std::error::Error>> {
//     let msg_str = msg.to_str().map_err(|_| "Not a string message")?;
//     let recv_msg: ReceivedMessage = serde_json::from_str(msg_str)?;
//     Ok(match recv_msg.action.as_str() {
//         "chat" => WsMessage::Chat(from_value(recv_msg.data)?),
//         _ => return Err(Box::new(StringError("invalid action".to_string()))),
//     })
// }

#[derive(Serialize)]
pub struct WelcomeMessage<'a> {
    pub name: &'a str,
}
impl<'a> WsSentMessage for WelcomeMessage<'a> {
    fn get_type() -> &'static str {
        "welcome"
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct ChatMessage {
//     #[serde(default)]
//     pub source_name: Option<String>,
//     pub msg: String,
// }
// impl WsSentMessage for ChatMessage {
//     fn get_type() -> &'static str {
//         "chat"
//     }
// }
// impl WsRecvMessage<'_> for ChatMessage {
//     fn get_type() -> &'static str {
//         "chat"
//     }
// }
