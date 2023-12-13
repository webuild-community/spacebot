use crate::{
    actors::{ClientWsActor, CreateRoom, JoinRoom, ListRooms},
    models::messages::{GetScoreboardCommand, ServerCommand},
    AppState,
};
use actix_web::{http::StatusCode, HttpRequest, Path, Query, State};
use futures::Future;

#[derive(Debug, Deserialize)]
pub struct QueryString {
    room_token: String,
    key: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct ScoreboardEntry {
    player_id: u32,
    total_points: u32,
}

#[derive(Serialize, Deserialize)]
struct ScoreboardResponse {
    scoreboard: Vec<ScoreboardEntry>,
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
                ClientWsActor::new(room.game_addr, query.key.clone(), query.name.clone()),
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
            ClientWsActor::new(room.game_addr, "SPECTATOR".to_string(), "SPECTATOR".to_string()),
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

pub fn get_room_scoreboard(
    (_req, state, path): (HttpRequest<AppState>, State<AppState>, Path<String>),
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let room_token = path.into_inner();
    let result = state.redis_actor_addr.send(GetScoreboardCommand(room_token)).wait().unwrap();
    match result {
        Ok(scoreboard) => {
            let scoreboard_response: ScoreboardResponse = ScoreboardResponse {
                scoreboard: scoreboard
                    .into_iter()
                    .map(|(player_id, total_points)| ScoreboardEntry { player_id, total_points })
                    .collect(),
            };
            let body = serde_json::to_string(&scoreboard_response)?;
            Ok(actix_web::HttpResponse::with_body(StatusCode::OK, body))
        },
        Err(e) => Err(actix_web::error::ErrorBadRequest(format!(
            "Failed to get room's scoreboard: {}",
            e
        ))),
    }
}
