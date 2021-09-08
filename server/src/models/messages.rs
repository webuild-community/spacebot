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
