use crate::{
    actors::ClientWsActor, AppState,
    models::messages::ServerCommand,
};
use actix_web::{HttpRequest, Query, State, http::StatusCode};

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
