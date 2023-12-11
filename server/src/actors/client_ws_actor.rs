use crate::{
    actors::{GameActor, RedisActor},
    models::messages::{ClientStop, PlayerGameCommand},
    AppState,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, StreamHandler};
use actix_web::ws::{self, CloseCode, CloseReason};
use ratelimit_meter::{DirectRateLimiter, GCRA};
use tokyo::models::ServerToClient;

const ACTIONS_PER_SECOND: u32 = 22;

#[derive(Debug)]
pub struct ClientWsActor {
    game_addr: Addr<GameActor>,
    redis_actor_addr: Addr<RedisActor>,
    room_token: String,
    api_key: String,
    team_name: String,
    rate_limiter: DirectRateLimiter<GCRA>,
}

impl ClientWsActor {
    pub fn new(game_addr: Addr<GameActor>, redis_actor_addr: Addr<RedisActor>, api_key: String, team_name: String, room_token: String) -> ClientWsActor {
        let rate_limiter = DirectRateLimiter::<GCRA>::per_second(
            std::num::NonZeroU32::new(ACTIONS_PER_SECOND).unwrap(),
        );

        ClientWsActor { game_addr, redis_actor_addr, api_key, team_name, room_token, rate_limiter }
    }
}

impl Actor for ClientWsActor {
    type Context = ws::WebsocketContext<Self, AppState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.game_addr.do_send(crate::actors::game_actor::SocketEvent::Join(
            self.api_key.clone(),
            self.team_name.clone(),
            ctx.address(),
        ));
        if self.api_key != "SPECTATOR" {
            self.redis_actor_addr.do_send(crate::models::messages::AddRoomPlayerCommand{
                room_token: self.room_token.clone(),
                player_key: self.api_key.clone(),
            });
        }
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        info!("API key {} stopped", self.api_key);
        self.game_addr.do_send(crate::actors::game_actor::SocketEvent::Leave(
            self.api_key.clone(),
            ctx.address(),
        ));
        self.redis_actor_addr.do_send(crate::models::messages::RemoveRoomPlayerCommand{
            room_token: self.room_token.clone(),
            player_key: self.api_key.clone(),
        });
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for ClientWsActor {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        match msg {
            ws::Message::Text(cmd) => {
                if self.rate_limiter.check().is_ok() {
                    let cmd_result = serde_json::from_str(&cmd);

                    if let Ok(cmd) = cmd_result {
                        self.game_addr
                            .do_send(PlayerGameCommand { api_key: self.api_key.clone(), cmd });
                    }
                } else {
                    warn!("API key {} got rate limited", self.api_key);
                }
            },
            ws::Message::Close(_) => {
                info!("API key {} close ws", self.api_key);
                self.game_addr.do_send(crate::actors::game_actor::SocketEvent::Leave(
                    self.api_key.clone(),
                    ctx.address(),
                ));
                ctx.stop();
            },
            _ => {},
        }
    }
}

impl Handler<ServerToClient> for ClientWsActor {
    type Result = ();

    fn handle(&mut self, msg: ServerToClient, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl Handler<ClientStop> for ClientWsActor {
    type Result = ();

    fn handle(&mut self, _: ClientStop, ctx: &mut Self::Context) {
        ctx.close(Some(CloseReason {
            code: CloseCode::Normal,
            description: Some("The server decided it didn't like you anymore. Or maybe you connected another client with the same API key".to_string())
        }));
    }
}
