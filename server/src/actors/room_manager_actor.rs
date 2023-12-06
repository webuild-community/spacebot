use crate::actors::GameActor;
use actix::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind, Result},
};
use tokyo::models::GameConfig;

const TOKEN_LENGTH: usize = 8;

// RoomManagerActor is responsible for creating and managing rooms
pub struct RoomManagerActor {
    config: GameConfig,
    id_counter: u32,
    rooms: HashMap<String, Room>,
}

// Room is a single game instance
struct Room {
    id: u32,
    name: String,
    max_players: u32,
    time_limit_seconds: u32,
    token: String,
    game: Addr<GameActor>,
}

impl Room {
    pub fn new(
        config: &GameConfig,
        id: u32,
        name: String,
        max_players: u32,
        time_limit_seconds: u32,
        token: String,
    ) -> Room {
        let game_cfg = GameConfig { bound_x: config.bound_x, bound_y: config.bound_y };

        let game_actor = GameActor::new(game_cfg, max_players, time_limit_seconds);
        let game_actor_addr = game_actor.start();
        Room { id, name, max_players, time_limit_seconds, token, game: game_actor_addr }
    }
}

impl RoomManagerActor {
    pub fn new(cfg: GameConfig) -> RoomManagerActor {
        RoomManagerActor { config: cfg, id_counter: 0, rooms: HashMap::new() }
    }

    pub fn create_room(
        &mut self,
        name: String,
        max_players: u32,
        time_limit_seconds: u32,
    ) -> RoomCreated {
        self.id_counter += 1;
        let new_id = self.id_counter.to_string();
        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(TOKEN_LENGTH)
            .map(char::from)
            .collect();

        self.rooms.insert(
            token.clone(),
            Room::new(
                &self.config,
                self.id_counter,
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
#[rtype(result = "Result<RoomEntry>")]
pub struct JoinRoom {
    pub room_token: String,
}

#[derive(Message)]
pub struct RoomEntry {
    pub game_addr: Addr<GameActor>,
}

impl Handler<JoinRoom> for RoomManagerActor {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        let room = self.rooms.get(&msg.room_token);
        match room {
            Some(room) => MessageResult(Result::Ok(RoomEntry { game_addr: room.game.clone() })),
            None => MessageResult(Result::Err(Error::new(ErrorKind::NotFound, "Room not found"))),
        }
    }
}

#[derive(Message)]
#[rtype(result = "RoomList")]
pub struct ListRooms;

#[derive(Message, Serialize, Deserialize)]
pub struct RoomList {
    pub rooms: Vec<RoomDetail>,
}

#[derive(Message, Serialize, Deserialize)]
pub struct RoomDetail {
    pub id: u32,
    pub name: String,
    pub max_players: u32,
    pub time_limit_seconds: u32,
    pub token: String,
}

impl Handler<ListRooms> for RoomManagerActor {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _msg: ListRooms, _ctx: &mut Self::Context) -> Self::Result {
        let mut rooms: Vec<RoomDetail> = self
            .rooms
            .iter()
            .map(|(_, room)| RoomDetail {
                id: room.id,
                name: room.name.clone(),
                max_players: room.max_players,
                time_limit_seconds: room.time_limit_seconds,
                token: room.token.clone(),
            })
            .collect();
        rooms.sort_by_key(|room| room.id);
        MessageResult(RoomList { rooms })
    }
}
