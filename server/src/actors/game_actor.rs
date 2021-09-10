use crate::{
    actors::ClientWsActor,
    game::{Game, TICKS_PER_SECOND},
    models::messages::{ClientStop, PlayerGameCommand, ServerCommand},
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use futures::sync::oneshot;
use spin_sleep::LoopHelper;
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::{channel, Receiver, Sender},
};
use tokyo::models::*;

#[derive(Debug)]
pub struct GameActor {
    connections: HashMap<String, Addr<ClientWsActor>>,
    spectators: HashSet<Addr<ClientWsActor>>,
    team_names: HashMap<u32, String>,
    cancel_chan: Option<oneshot::Sender<()>>,
    msg_tx: Sender<GameLoopCommand>,
    msg_rx: Option<Receiver<GameLoopCommand>>,
    player_id_counter: u32,
    api_key_to_player_id: HashMap<String, u32>,
    game_config: GameConfig,
}

#[derive(Debug)]
pub enum GameLoopCommand {
    PlayerJoined(u32),
    PlayerLeft(u32),
    GameCommand(u32, GameCommand),
    Reset,
}

impl GameActor {
    pub fn new(config: GameConfig) -> GameActor {
        let (msg_tx, msg_rx) = channel();

        GameActor {
            connections: HashMap::new(),
            spectators: HashSet::new(),
            team_names: HashMap::new(),
            cancel_chan: None,
            msg_tx,
            msg_rx: Some(msg_rx),
            player_id_counter: 0,
            api_key_to_player_id: HashMap::new(),
            game_config: config,
        }
    }
}

fn game_loop(
    game_actor: Addr<GameActor>,
    msg_chan: Receiver<GameLoopCommand>,
    mut cancel_chan: oneshot::Receiver<()>,
    config: GameConfig,
) {
    let mut loop_helper = LoopHelper::builder().build_with_target_rate(TICKS_PER_SECOND);

    let mut game = Game::new(config);

    game.init();

    loop {
        loop_helper.loop_start();

        match cancel_chan.try_recv() {
            Ok(Some(_)) | Err(_) => {
                break;
            },
            _ => {},
        }

        for cmd in msg_chan.try_iter() {
            // info!("Got a message! - {:?}", cmd);
            match cmd {
                GameLoopCommand::PlayerJoined(id) => {
                    game.add_player(id);
                },
                GameLoopCommand::PlayerLeft(id) => {
                    game.player_left(id);
                },
                GameLoopCommand::GameCommand(id, cmd) => {
                    game.handle_cmd(id, cmd);
                },
                GameLoopCommand::Reset => {
                    game.reset();
                }
            }
        }

        let dt = 1.0 / TICKS_PER_SECOND;
        game.tick(dt);

        // Send out update packets

        // TODO(bschwind) - maybe put the game state behind an Arc
        //                  instead of cloning it
        game_actor.do_send(game.state.clone());
        loop_helper.loop_sleep();
    }

    info!("game over!");
}

impl Actor for GameActor {
    type Context = Context<GameActor>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Game Actor started!");
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let addr = ctx.address();

        // "Take" the receiving end of the channel and give it
        // to the game loop thread
        let msg_rx = self.msg_rx.take().unwrap();

        let config = self.game_config;
        std::thread::spawn(move || {
            game_loop(addr, msg_rx, cancel_rx, config);
        });

        self.cancel_chan = Some(cancel_tx);
    }
}

#[derive(Debug, Message)]
pub enum SocketEvent {
    Join(String, String, Addr<ClientWsActor>),
    Leave(String, Addr<ClientWsActor>),
}

impl Handler<SocketEvent> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: SocketEvent, _ctx: &mut Self::Context) {
        match msg {
            SocketEvent::Join(api_key, team_name, addr) => {
                let key_clone = api_key.clone();
                let addr_clone = addr.clone();

                info!("person joined - {:?}", api_key);

                if api_key == "SPECTATOR" {
                    addr.do_send(ServerToClient::TeamNames(self.team_names.clone()));
                    self.spectators.insert(addr);
                } else {
                    let existing_client_opt = self.connections.insert(api_key, addr);

                    if let Some(existing_client) = existing_client_opt {
                        info!("kicking out old connection");
                        existing_client.do_send(ClientStop {});
                    }

                    let player_id =
                        if let Some(player_id) = self.api_key_to_player_id.get(&key_clone) {
                            addr_clone.do_send(ServerToClient::Id(*player_id));
                            *player_id
                        } else {
                            // This was the first time this API key connected,
                            // assign them a player ID and return it
                            let player_id = self.player_id_counter;
                            self.player_id_counter += 1;
                            info!("API key {} gets player ID {}", key_clone, player_id);

                            self.api_key_to_player_id.insert(key_clone, player_id);

                            self.msg_tx
                                .send(GameLoopCommand::PlayerJoined(player_id))
                                .expect("The game loop should always be receiving commands");

                            addr_clone.do_send(ServerToClient::Id(player_id));
                            player_id
                        };

                    // Update team name and broadcast new team names list to all sockets.
                    self.team_names.insert(player_id, team_name);
                    for addr in self.connections.values().chain(self.spectators.iter()) {
                        addr.do_send(ServerToClient::TeamNames(self.team_names.clone()));
                    }
                }
            },
            SocketEvent::Leave(api_key, addr) => {
                if api_key == "SPECTATOR" {
                    self.spectators.remove(&addr);
                } else {
                    if let Some(client_addr) = self.connections.get(&api_key) {
                        if addr == *client_addr {
                            info!("person left - {:?}", api_key);

                            if let Some(player_id) = self.api_key_to_player_id.get(&api_key) {
                                self.msg_tx
                                    .send(GameLoopCommand::PlayerLeft(*player_id))
                                    .expect("The game loop should always be receiving commands");
                            }

                            self.connections.remove(&api_key);
                        }
                    }
                }
            },
        }
    }
}

impl Handler<PlayerGameCommand> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: PlayerGameCommand, _ctx: &mut Self::Context) {
        if let Some(player_id) = self.api_key_to_player_id.get(&msg.api_key) {
            self.msg_tx
                .send(GameLoopCommand::GameCommand(*player_id, msg.cmd))
                .expect("The game loop should always be receiving commands");
        }
    }
}

impl Handler<GameState> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: GameState, _ctx: &mut Self::Context) {
        for addr in self.connections.values().chain(self.spectators.iter()) {
            addr.do_send(ServerToClient::GameState(msg.clone()));
        }
    }
}

impl Handler<ServerCommand> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: ServerCommand, _ctx: &mut Self::Context) {
        match msg {
            ServerCommand::Reset => {
                self.msg_tx
                    .send(GameLoopCommand::Reset)
                    .expect("The game loop should always be receiving commands");
            }
        }
    }
}
