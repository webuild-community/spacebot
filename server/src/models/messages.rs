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
