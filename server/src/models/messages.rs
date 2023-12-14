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
#[rtype(result = "Result<(), redis::RedisError>")]
pub struct SetScoreboardCommand {
    pub room_token: String,
    pub scoreboard: HashMap<u32, u32>,
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<u32, u32>, redis::RedisError>")]
pub struct GetScoreboardCommand(pub String);

#[derive(Message)]
#[rtype(result = "Result<String, redis::RedisError>")]
pub struct SetPlayerInfoCommand {
    pub player_id: u32,
    pub fields: HashMap<String, String>, // Use a HashMap to represent multiple fields
}

#[derive(Message)]
#[rtype(result = "Result<HashMap<u32, String>, redis::RedisError>")]
pub struct GetMultiplePlayerInfo {
    pub player_ids: Vec<u32>,
}