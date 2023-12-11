use std::collections::HashMap;

use actix::Message;
use tokyo::models::GameCommand;

#[derive(Debug, Message)]
pub struct PlayerGameCommand {
    pub api_key: String,
    pub cmd: GameCommand,
}

#[derive(Debug, Message)]
pub struct ClientStop {}

#[derive(Debug, Message)]
pub enum ServerCommand {
    Reset
}

#[derive(Message)]
#[rtype(result = "Result<String, redis::RedisError>")]
pub struct GetRoomFieldCommand {
    pub room_token: String,
    pub field: String,
}

#[derive(Message)]
#[rtype(result = "Result<String, redis::RedisError>")]
pub struct UpdateRoomFieldCommand {
    pub room_token: String,
    pub field: String,
    pub value: String,
}

#[derive(Message)]
#[rtype(result = "Result<String, redis::RedisError>")]
pub struct SetRoomCommand {
    pub room_token: String,
    pub fields: HashMap<String, String>, // Use a HashMap to represent multiple fields
}

#[derive(Message)]
#[rtype(result = "Result<u32, redis::RedisError>")]
pub struct GetRoomSizeCommand {
    pub room_token: String,
}

#[derive(Message)]
#[rtype(result = "Result<String, redis::RedisError>")]
pub struct AddRoomPlayerCommand {
    pub room_token: String,
    pub player_key: String,
}

#[derive(Message)]
#[rtype(result = "Result<String, redis::RedisError>")]
pub struct RemoveRoomPlayerCommand {
    pub room_token: String,
    pub player_key: String,
}