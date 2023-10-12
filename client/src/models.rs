use actix::Message;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

pub const BULLET_BASE_LIMIT: u32 = 5;
pub const BULLET_BASE_RADIUS: f32 = 2.0;
pub const BULLET_BASE_SPEED: f32 = 600.0; // in pixels-per-second
pub const BULLET_SPEED_INCREMENTAL: f32 = 1.1;
pub const BULLET_RADIUS_INCREMENTAL: f32 = 1.05;
pub const PLAYER_RADIUS_INCREMENTAL: f32 = 1.05;

pub const ITEM_RADIUS: f32 = 20.0;

pub const PLAYER_BASE_RADIUS: f32 = 10.0;
pub const PLAYER_BASE_SPEED: f32 = 300.0;
pub const PLAYER_MIN_THROTTLE: f32 = -1.0;
pub const PLAYER_MAX_THROTTLE: f32 = 1.0;

// Send commands more frequently than this interval, and consequences.
pub const MIN_COMMAND_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct GameConfig {
    pub bound_x: f32,
    pub bound_y: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "e", content = "data")]
pub enum GameCommand {
    #[serde(rename = "rotate")]
    Rotate(f32), // In radians, no punish.

    #[serde(rename = "throttle")]
    Throttle(f32), // Between 0.0 and 1.0, otherwise consequences.

    #[serde(rename = "fire")]
    Fire, // Fire at the current angle.
}

#[derive(Debug, Serialize, Deserialize, Message)]
#[serde(tag = "e", content = "data")]
pub enum ServerToClient {
    #[serde(rename = "id")]
    Id(u32), // Tell the client their player ID

    #[serde(rename = "state")]
    GameState(GameState), // Send the game state to the client

    #[serde(rename = "teamnames")]
    TeamNames(HashMap<u32, String>), // Send the game state to the client
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: u32,
    pub angle: f32,
    pub throttle: f32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub bullet_radius: f32,
    pub bullet_speed: f32,
    pub bullet_limit: u32,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BulletState {
    pub id: u32,
    pub player_id: u32,
    pub angle: f32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub speed: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadPlayer {
    pub respawn: SystemTime,
    pub player: PlayerState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ItemType {
    FasterBullet,
    MoreBullet,
    BiggerBullet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub item_type: ItemType,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, Message)]
pub struct GameState {
    pub bounds: (f32, f32),
    pub players: Vec<PlayerState>,
    pub items: Vec<Item>,
    pub dead: Vec<DeadPlayer>,
    pub bullets: Vec<BulletState>,
    pub scoreboard: HashMap<u32, u32>,
}

impl PlayerState {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            angle: 0f32,
            throttle: 0f32,
            x: 0f32,
            y: 0f32,
            radius: PLAYER_BASE_RADIUS,
            bullet_radius: BULLET_BASE_RADIUS,
            bullet_speed: BULLET_BASE_SPEED,
            bullet_limit: BULLET_BASE_LIMIT,
        }
    }

    pub fn randomize(&mut self, rng: &mut impl rand::Rng, (bound_right, bound_bottom): (f32, f32)) {
        self.angle = rng.gen_range(0.0, std::f32::consts::PI * 2.0);
        self.throttle = 0.0;
        self.x = rng.gen_range(0.0, bound_right);
        self.y = rng.gen_range(0.0, bound_bottom);
        // reset stats
        self.radius = PLAYER_BASE_RADIUS;
        self.bullet_radius = BULLET_BASE_RADIUS;
        self.bullet_speed = BULLET_BASE_SPEED;
        self.bullet_limit = BULLET_BASE_LIMIT;
    }
}

impl Item {
    pub fn new_randomized(id: u32, rng: &mut impl rand::Rng, (bound_right, bound_bottom): (f32, f32)) -> Self {
        let x = rng.gen_range(0.0, bound_right);
        let y = rng.gen_range(0.0, bound_bottom);
        let item_type = match rng.gen_range(0, 3) {
            0 => ItemType::FasterBullet,
            1 => ItemType::MoreBullet,
            _ => ItemType::BiggerBullet,
        };

        Self {
            id, x, y, item_type,
            radius: ITEM_RADIUS,
        }
    }

    pub fn apply_to(&self, player: &mut PlayerState) {
        match self.item_type {
            ItemType::FasterBullet => {
                player.bullet_speed *= BULLET_SPEED_INCREMENTAL;
                player.radius *= PLAYER_RADIUS_INCREMENTAL;
            },
            ItemType::MoreBullet => {
                player.bullet_limit += 1;
                player.radius *= PLAYER_RADIUS_INCREMENTAL;
            },
            ItemType::BiggerBullet => {
                player.bullet_radius *= BULLET_RADIUS_INCREMENTAL;
                player.radius *= PLAYER_RADIUS_INCREMENTAL;
            },
        }
    }
}

impl GameState {
    pub fn new(bounds: (f32, f32)) -> Self {
        Self { bounds, ..Default::default() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Message)]
pub struct ClientState {
    pub id: u32,
    pub game_state: GameState,
}
