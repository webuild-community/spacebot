#![feature(extract_if)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

mod actors;
mod controllers;
mod game;
mod models;

use crate::actors::{GameActor, RoomManagerActor};
use actix::{Actor, Addr, System};
use actix_web::{http::Method, middleware::Logger, server, App};
use actors::redis_actor::{RedisActor, create_redis_actor};
use lazy_static::lazy_static;
use listenfd::ListenFd;
use std::collections::HashSet;
use tokyo::models::GameConfig;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    server_port: Option<u16>,
    api_keys: HashSet<String>,
    dev_mode: bool,
    game_config: GameConfig,
    redis_uri: Option<String>,
}

pub struct AppState {
    game_addr: Addr<GameActor>,
    redis_actor_addr: Addr<RedisActor>,
    room_manager_addr: Addr<RoomManagerActor>,
}

const CONFIG_FILE_PATH: &str = "tokyo.toml";

lazy_static! {
    static ref APP_CONFIG: AppConfig = {
        let config = std::fs::read(CONFIG_FILE_PATH).expect("Failed to read config file");
        let config = toml::from_slice(&config).expect("failed to parse config");
        println!("Config loaded: {:?}", config);

        config
    };
}

fn main() -> Result<(), String> {
    lazy_static::initialize(&APP_CONFIG);
    env_logger::init();

    let server_port = APP_CONFIG.server_port.unwrap_or(3000);

    let actor_system = System::new("meetup-server");

    let redis_uri = APP_CONFIG.redis_uri.clone().unwrap_or("redis://127.0.0.1/".into());
    let redis_actor_addr = create_redis_actor(redis_uri);

    let game_actor = GameActor::new(APP_CONFIG.game_config, 0, 0);
    let game_actor_addr = game_actor.start();

    let room_manager_actor = actors::RoomManagerActor::new(APP_CONFIG.game_config);
    let room_manager_addr = room_manager_actor.start();

    let mut server = server::new(move || {
        let app_state = AppState {
            game_addr: game_actor_addr.clone(),
            redis_actor_addr: redis_actor_addr.clone(),
            room_manager_addr: room_manager_addr.clone(),
        };

        App::with_state(app_state)
            .middleware(Logger::default())
            .resource("/rooms", |r| {
                r.method(Method::POST).with(controllers::api::create_room_handler);
                r.method(Method::GET).with(controllers::api::list_rooms_handler);
            })
            .resource("/socket", |r| {
                r.method(Method::GET).with(controllers::api::socket_handler);
            })
            .resource("/spectate", |r| {
                r.method(Method::GET).with(controllers::api::spectate_handler);
            })
            .resource("/reset", |r| {
                r.method(Method::GET).with(controllers::api::reset_handler);
            })
            .handler(
                "/",
                actix_web::fs::StaticFiles::new("./spectator/").unwrap().index_file("index.html"),
            )
            .resource("/{tail:.*}j", |r| {
                r.method(Method::GET).with(controllers::common::index_handler)
            })
    });

    // Bind to the development file descriptor if available
    // Run with: systemfd --no-pid -s http::3000 -- cargo watch -x run
    let mut listenfd = ListenFd::from_env();
    server = if let Some(fd) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(fd)
    } else {
        server.bind(format!("0.0.0.0:{}", server_port)).unwrap()
    };

    server.start();

    let _ = actor_system.run();

    Ok(())
}
