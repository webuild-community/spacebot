use std::collections::HashMap;

use actix::prelude::*;
use redis::{Client, Commands, Connection};

use crate::models::messages::{SetScoreboardCommand, GetScoreboardCommand};

#[derive(Debug)]
pub struct StoreActor {
    client: Client,
}

impl StoreActor {
    pub fn new(redis_url: String) -> StoreActor {
        let client = Client::open(redis_url).expect("Failed to create Redis client");
        StoreActor { client }
    }
}

impl Actor for StoreActor {
    type Context = Context<Self>;
}

impl Handler<SetScoreboardCommand> for StoreActor {
    type Result = Result<(), redis::RedisError>;

    fn handle(&mut self, msg: SetScoreboardCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;
        let query_key = format!("room:{}:scoreboard", msg.room_token);
        for (player_id, points) in &msg.scoreboard {
            con.zadd(query_key.clone(), *points as f64, *player_id)?;
        }
        Ok(())
    }
}

impl Handler<GetScoreboardCommand> for StoreActor {
    type Result = Result<HashMap<u32, u32>, redis::RedisError>;

    fn handle(&mut self, msg: GetScoreboardCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;
        let query_key = format!("room:{}:scoreboard", msg.0);
        let scoreboard: Vec<(String, String)> = con.zrevrange_withscores(query_key, 0, -1)?;
        let mut result = HashMap::new();
        for (total_points_str, player_id_str) in scoreboard {
            let player_id = player_id_str.parse::<u32>().unwrap_or_default();
            let total_points = total_points_str.parse::<f64>().unwrap_or_default() as u32;
            result.insert(player_id, total_points);
        }
        
        Ok(result)
    }
}
