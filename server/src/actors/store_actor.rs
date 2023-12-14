use std::collections::HashMap;

use actix::prelude::*;
use redis::{Client, Commands, Connection};

use crate::models::messages::{GetScoreboardCommand, SetPlayerInfoCommand, SetScoreboardCommand, GetMultiplePlayerInfo};

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

impl Handler<SetPlayerInfoCommand> for StoreActor {
    type Result = Result<String, redis::RedisError>;

    fn handle(&mut self, msg: SetPlayerInfoCommand, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;

        // Use hset_multiple to set multiple fields at the same time
        let query_key = format!("player:{}:info", msg.player_id);
        let fields: Vec<(&str, &str)> =
            msg.fields.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let result: redis::RedisResult<()> = con.hset_multiple(query_key, &fields);

        result
            .map_err(|e| e.into())
            .map(|_| format!("Fields are set for player_id {}", msg.player_id))
    }
}

impl Handler<GetMultiplePlayerInfo> for StoreActor {
    type Result = Result<HashMap<u32, String>, redis::RedisError>;

    fn handle(&mut self, msg: GetMultiplePlayerInfo, _: &mut Self::Context) -> Self::Result {
        let mut con: Connection = self.client.get_connection()?;
        let mut results = HashMap::new();
        for key in msg.player_ids {
            let hash_key: String = format!("player:{}:info", key);
            let player_info: HashMap<String, String> = con.hgetall(&hash_key)?;
            results.insert(key.clone(), serde_json::to_string(&player_info).unwrap());
        }

        Ok(results)
    }
}
