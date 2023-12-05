use crate::actors::GameActor;
use actix::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
    sync::Mutex,
};
use tokyo::models::GameConfig;

pub struct RoomManagerActor {
    config: GameConfig,
    id_counter: Mutex<u32>,
    rooms: Mutex<HashMap<String, Room>>,
}

struct Room {
    id: String,
    name: String,
    max_players: u32,
    time_limit_seconds: u32,
    token: String,
    game: Addr<GameActor>,
}

impl Room {
    pub fn new(
        config: &GameConfig,
        id: String,
        name: String,
        max_players: u32,
        time_limit_seconds: u32,
        token: String,
    ) -> Room {
        let game_actor = GameActor::new(config.clone());
        let game_actor_addr = game_actor.start();
        Room { id, name, max_players, time_limit_seconds, token, game: game_actor_addr }
    }
}

impl RoomManagerActor {
    pub fn new(cfg: GameConfig) -> RoomManagerActor {
        RoomManagerActor {
            config: cfg,
            id_counter: Mutex::new(0),
            rooms: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_room(
        &mut self,
        name: String,
        max_players: u32,
        time_limit_seconds: u32,
    ) -> RoomCreated {
        let mut id_counter = self.id_counter.lock().unwrap();
        *id_counter += 1;

        let new_id = id_counter.to_string();
        let token: String =
            rand::thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect();

        self.rooms.lock().unwrap().insert(
            token.clone(),
            Room::new(
                &self.config,
                new_id.to_string(),
                name.to_string(),
                max_players,
                time_limit_seconds,
                token.clone(),
            ),
        );

        RoomCreated { id: new_id, name, max_players, time_limit_seconds, token }
    }
}

impl Actor for RoomManagerActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("RoomManagerActor started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("RoomManagerActor stopped");
    }
}

#[derive(Message)]
#[rtype(result = "RoomCreated")]
pub struct CreateRoom {
    pub name: String,
    pub max_players: u32,
    pub time_limit_seconds: u32,
}

#[derive(Message, Deserialize, Serialize)]
pub struct RoomCreated {
    pub id: String,
    pub name: String,
    pub max_players: u32,
    pub time_limit_seconds: u32,
    pub token: String,
}

impl Handler<CreateRoom> for RoomManagerActor {
    type Result = MessageResult<CreateRoom>;

    fn handle(&mut self, msg: CreateRoom, _ctx: &mut Self::Context) -> Self::Result {
        let room = self.create_room(msg.name, msg.max_players, msg.time_limit_seconds);
        MessageResult(room)
    }
}

#[derive(Message)]
#[rtype(result = "Result<RoomJoined>")]
pub struct JoinRoom {
    pub room_token: String,
    pub api_key: String,
    pub team_name: String,
}

#[derive(Message)]
pub struct RoomJoined {
    pub game_addr: Addr<GameActor>,
    pub room_token: String,
    pub api_key: String,
    pub team_name: String,
}

impl Handler<JoinRoom> for RoomManagerActor {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        let room_map = self.rooms.lock().unwrap();
        let room = room_map.get(&msg.room_token);
        match room {
            Some(room) => {
                let msg = RoomJoined {
                    game_addr: room.game.clone(),
                    room_token: msg.room_token,
                    api_key: msg.api_key,
                    team_name: msg.team_name,
                };
                MessageResult(Result::Ok(msg))
            },
            None => MessageResult(Result::Err(Error::new(ErrorKind::NotFound, "Room not found"))),
        }
    }
}
