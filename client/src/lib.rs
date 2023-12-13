//! Unless you want to go extreme, you will not need to understand what are
//! inside this file except the `Handler` trait. Any modifications you make in
//! this file may result in some undefined behaviors or consequences. Do it as
//! your own risk :)

#[macro_use]
extern crate serde_derive;

pub mod analyzer;
pub mod behavior;
pub mod geom;
pub mod models;

use crate::models::{ClientState, GameCommand, GameState, ServerToClient, MIN_COMMAND_INTERVAL};
use failure::Error;
use futures::{Future, Sink, Stream};
use std::{
    env,
    fmt::Debug,
    sync::{Arc, Mutex},
};
use tokio_tungstenite as tokio_ws;
use tokio_ws::tungstenite as ws;
use url::{
    percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET},
    Url,
};

/// `Handler` is provided as the trait that players can implement to interact
/// with the game server.
pub trait Handler {
    /// An opportunity, provided multiple times a second, to analyze the current
    /// state of the world and do a single action based on its state. It's not
    /// called when the player is dead and waiting to be respawn.
    fn tick(&mut self, state: &ClientState) -> Option<GameCommand>;
}

fn log_err<E: Debug>(e: E) {
    eprintln!("{:?}", e)
}

fn is_player_alive(state: &ClientState) -> bool {
    state.game_state.players.iter().find(|player| player.id == state.id).is_some()
}

fn build_game_loop<H, S, D>(
    sink: S,
    client_state: Arc<Mutex<ClientState>>,
    mut handler: H,
) -> impl Future<Item = (), Error = ()>
where
    H: Handler + Send + 'static,
    S: Sink<SinkItem = ws::Message, SinkError = D>,
    D: Debug,
{
    // Create a stream that produces at our desired interval
    tokio::timer::Interval::new_interval(MIN_COMMAND_INTERVAL)
        // Give the user a chance to take a turn
        .filter_map(move |_| {
            let client_state = &*client_state.lock().unwrap();
            if is_player_alive(client_state) {
                handler.tick(client_state)
            } else {
                None
            }
        })
        // Convert their command to a websocket message
        .map(move |command: GameCommand| {
            ws::Message::Text(serde_json::to_string(&command).unwrap())
        })
        // Satisfy the type gods.
        .map_err(log_err)
        // And send the message out.
        .forward(sink.sink_map_err(log_err))
        .map(|_| ()) // throw away leftovers from forward
}

fn build_state_updater<S, D>(
    stream: S,
    client_state: Arc<Mutex<ClientState>>,
) -> impl Future<Item = (), Error = ()>
where
    S: Stream<Item = ws::Message, Error = D>,
    D: Debug,
{
    stream
        // We only care about text websocket messages.
        .filter_map(|message| message.into_text().ok())
        // We especially only care about proper JSON messages.
        .filter_map(|message| serde_json::from_str(&message).ok())
        // Update the our game state to the most recent reported by the server.
        .for_each(move |server_to_client_msg| {
            match server_to_client_msg {
                ServerToClient::Id(player_id) => {
                    (*client_state).lock().unwrap().id = player_id;
                },
                ServerToClient::GameState(state) => {
                    (*client_state).lock().unwrap().game_state = state;
                },
                _ => {},
            }

            Ok(())
        })
        .map_err(log_err)
}

/// Begin the client-side game loop, using the provided struct that implements `Handler`
/// to act on behalf of the player.
pub fn run<H>(key: &str, name: &str, room_token: &str, handler: H) -> Result<(), Error>
where
    H: Handler + Send + 'static,
{
    let host = env::var("SERVER_HOST").unwrap_or("192.168.0.199".into());
    let url = Url::parse(&format!(
        "wss://{}/socket?key={}&name={}&room_token={}",
        host,
        key,
        utf8_percent_encode(name, DEFAULT_ENCODE_SET).to_string(),
        room_token,
    ))?;

    let client_state =
        Arc::new(Mutex::new(ClientState { id: 0, game_state: GameState::default() }));

    let client = tokio_ws::connect_async(url)
        .and_then(move |(websocket, _)| {
            // Allow us to build two futures out of this connection - one for send, one for recv.
            let (sink, stream) = websocket.split();

            let game_loop = build_game_loop(sink, client_state.clone(), handler);
            let state_updater = build_state_updater(stream, client_state);

            // Return a future that will finish when either one of the two futures finish.
            state_updater.select(game_loop).then(|_| Ok(()))
        })
        .map_err(log_err);

    tokio::run(client);
    Ok(())
}

#[cfg(test)]
mod tests {}
