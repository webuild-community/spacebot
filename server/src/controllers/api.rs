use crate::{
    actors::{ClientWsActor, CreateRoom, JoinRoom},
    models::messages::ServerCommand,
    AppState,
};
use actix_web::{http::StatusCode, HttpRequest, Query, State};
use futures::Future;

#[derive(Debug, Deserialize)]
pub struct QueryString {
    key: String,
    name: String,
}

pub fn socket_handler(
    (req, state, query): (HttpRequest<AppState>, State<AppState>, Query<QueryString>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    if crate::APP_CONFIG.dev_mode || crate::APP_CONFIG.api_keys.contains(&query.key) {
        actix_web::ws::start(
            &req,
            ClientWsActor::new(state.game_addr.clone(), query.key.clone(), query.name.clone()),
        )
    } else {
        Err(actix_web::error::ErrorBadRequest("Invalid API Key"))
    }
}

pub fn spectate_handler(
    (req, state): (HttpRequest<AppState>, State<AppState>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    // TODO(bschwind) - Make a separate spectator actor
    actix_web::ws::start(
        &req,
        ClientWsActor::new(
            state.game_addr.clone(),
            "SPECTATOR".to_string(),
            "SPECTATOR".to_string(),
        ),
    )
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
            Ok(actix_web::HttpResponse::with_body(StatusCode::OK, body))
        },
        Err(_) => Err(actix_web::error::ErrorBadRequest("Failed to create room")),
    }
}
