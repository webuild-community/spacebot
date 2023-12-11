use actix::prelude::*;
use redis::{Client, Commands, Connection, RedisResult};

use crate::models::messages::{GetRoomFieldCommand, UpdateRoomFieldCommand, SetRoomCommand, GetRoomSizeCommand, AddRoomPlayerCommand, RemoveRoomPlayerCommand};

#[derive(Debug)]
pub struct RedisActor {
    client: Client,
}

impl Actor for RedisActor {
    type Context = Context<Self>;
}

impl Handler<GetRoomFieldCommand> for RedisActor {
    type Result = Result<String, redis::RedisError>;

    fn handle(&mut self, msg: GetRoomFieldCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;
        let query_key = format!("room:{}", msg.room_token);
        let result: RedisResult<String> = con.hget(query_key, msg.field);
        result.map_err(|e| e.into())
    }
}

impl Handler<UpdateRoomFieldCommand> for RedisActor {
    type Result = Result<String, redis::RedisError>;

    fn handle(&mut self, msg: UpdateRoomFieldCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;

        // Clone the values before moving them into the Redis `hset` command
        let query_key = format!("room:{}", msg.room_token.clone());
        let field = msg.field.clone();
        let value = msg.value.clone();

        let result: RedisResult<()> = con.hset(query_key, field, value);
        result
            .map_err(|e| e.into())
            .map(|_| format!("Field {} set for room {}", msg.field, msg.room_token))
    }
}

impl Handler<SetRoomCommand> for RedisActor {
    type Result = Result<String, redis::RedisError>;

    fn handle(&mut self, msg: SetRoomCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;

        // Use hset_multiple to set multiple fields at the same time
        let query_key = format!("room:{}", msg.room_token.clone());
        let fields: Vec<(&str, &str)> = msg.fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let result: RedisResult<()> = con.hset_multiple(query_key, &fields);

        result.map_err(|e| e.into())
            .map(|_| format!("Fields set for room {}", msg.room_token))
    }
}

impl Handler<GetRoomSizeCommand> for RedisActor {
    type Result = Result<u32, redis::RedisError>;

    fn handle(&mut self, msg: GetRoomSizeCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;
        let query_key = format!("room:{}:players", msg.room_token);
        let result: RedisResult<u32> = con.scard(query_key);
        result.map_err(|e| e.into())
    }
}

impl Handler<AddRoomPlayerCommand> for RedisActor {
    type Result = Result<String, redis::RedisError>;

    fn handle(&mut self, msg: AddRoomPlayerCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;
        let query_key = format!("room:{}:players", msg.room_token);
        let result: RedisResult<String> = con.sadd(query_key, msg.player_key);
        result.map_err(|e| e.into())
    }
}

impl Handler<RemoveRoomPlayerCommand> for RedisActor {
    type Result = Result<String, redis::RedisError>;

    fn handle(&mut self, msg: RemoveRoomPlayerCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;
        let query_key = format!("room:{}:players", msg.room_token);
        let result: RedisResult<String> = con.srem(query_key, msg.player_key);
        result.map_err(|e| e.into())
    }
}

pub fn create_redis_actor(redis_url: String) -> Addr<RedisActor> {
    let client = Client::open(redis_url).expect("Failed to create Redis client");
    let actor = RedisActor { client };
    Actor::start(actor)
}
