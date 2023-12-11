use std::collections::HashMap;

use crate::{
    actors::{ClientWsActor, CreateRoom, JoinRoom, ListRooms, room_manager_actor::RoomCreated},
    models::messages::{ServerCommand, SetRoomCommand},
    AppState,
};
use actix_web::{http::StatusCode, HttpRequest, Query, State};
use futures::Future;

#[derive(Debug, Deserialize)]
pub struct QueryString {
    room_token: String,
    key: String,
    name: String,
}

pub fn socket_handler(
    (req, state, query): (HttpRequest<AppState>, State<AppState>, Query<QueryString>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if crate::APP_CONFIG.dev_mode || crate::APP_CONFIG.api_keys.contains(&query.key) {
        let r = state
            .room_manager_addr
            .send(JoinRoom { room_token: query.room_token.clone() })
            .wait()
            .unwrap();
        match r {
            Ok(room) => actix_web::ws::start(
                &req,
                ClientWsActor::new(room.game_addr, state.redis_actor_addr.clone(), query.key.clone(), query.name.clone(), query.room_token.clone()),
            ),
            Err(err) => Err(actix_web::error::ErrorBadRequest(err.to_string())),
        }
    } else {
        Err(actix_web::error::ErrorBadRequest("Invalid API Key"))
    }
}

#[derive(Debug, Deserialize)]
pub struct SpectatorString {
    room_token: String,
}

pub fn spectate_handler(
    (req, state, query): (HttpRequest<AppState>, State<AppState>, Query<SpectatorString>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    // TODO(bschwind) - Make a separate spectator actor
    let r = state
        .room_manager_addr
        .send(JoinRoom { room_token: query.room_token.clone() })
        .wait()
        .unwrap();
    match r {
        Ok(room) => actix_web::ws::start(
            &req,
            ClientWsActor::new(room.game_addr, state.redis_actor_addr.clone(), "SPECTATOR".to_string(), "SPECTATOR".to_string(), query.room_token.clone()),
        ),
        Err(err) => Err(actix_web::error::ErrorBadRequest(err.to_string())),
    }
}

pub fn reset_handler(
    (_req, state): (HttpRequest<AppState>, State<AppState>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    state.game_addr.do_send(ServerCommand::Reset);
    Ok(actix_web::HttpResponse::with_body(StatusCode::OK, "done"))
}

#[derive(Debug, Deserialize)]
pub struct RoomCreateRequest {
    pub name: String,
    pub max_players: u32,
    pub time_limit_seconds: u32,
}

pub fn create_room_handler(
    (_req, state, json): (
        HttpRequest<AppState>,
        State<AppState>,
        actix_web::Json<RoomCreateRequest>,
    ),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let r = state
        .room_manager_addr
        .send(CreateRoom {
            name: json.name.clone(),
            max_players: json.max_players,
            time_limit_seconds: json.time_limit_seconds,
        })
        .wait();
    match r {
        Ok(room) => {
            let body = serde_json::to_string(&room).unwrap();

            // cache created room info
            let cache_fields = create_room_fields(&room);
            let _ = state
                .redis_actor_addr
                .send(SetRoomCommand { room_token: room.token.clone(), fields: cache_fields })
                .wait()?;
            Ok(actix_web::HttpResponse::with_body(StatusCode::OK, body))
        },
        Err(_) => Err(actix_web::error::ErrorBadRequest("Failed to create room")),
    }
}

pub fn list_rooms_handler(
    (_req, state): (HttpRequest<AppState>, State<AppState>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let room_map = state.room_manager_addr.send(ListRooms);
    match room_map.wait() {
        Ok(room_list) => {
            let body = serde_json::to_string(&room_list.rooms).unwrap();
            Ok(actix_web::HttpResponse::with_body(StatusCode::OK, body))
        },
        Err(_) => Err(actix_web::error::ErrorBadRequest("Failed to list rooms")),
    }
}

fn create_room_fields(room: &RoomCreated) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), room.name.clone());
    fields.insert("time_limit_seconds".to_string(), room.time_limit_seconds.to_string());
    fields.insert("max_players".to_string(), room.max_players.to_string());
    
    // TODO(haongo) - Handle room's max_players check
    // fields.insert("status".to_string(), format!("{}", RoomStatus::Ready));

    fields
}
