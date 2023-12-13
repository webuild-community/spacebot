/// A simple example client that only works with the bare minimal API. If you are
/// new to Rust, or want to build your own logic from the ground up, this is a
/// good start for you.
use std::env;
use tokyo::{self, models::*, Handler};

#[derive(Default)]
struct Player {
    id: u32,
    angle: f32,
    counter: u32,
    throttle: f32,
    throttle_dir: f32,
}

const THROTTLE_VELOCITY: f32 = 0.01;

impl Handler for Player {
    fn tick(&mut self, state: &ClientState) -> Option<GameCommand> {
        if self.throttle_dir == 0.0 {
            self.throttle_dir = 1.0;
        }
        self.id = state.id;

        let angle = self.angle;
        self.angle += 0.05;

        self.counter += 1;
        self.throttle += THROTTLE_VELOCITY * self.throttle_dir;

        if self.throttle > 0.99 {
            self.throttle_dir = -self.throttle_dir;
            self.throttle += THROTTLE_VELOCITY * self.throttle_dir;
        } else if self.throttle < 0.0 {
            self.throttle_dir = -self.throttle_dir;
            self.throttle += THROTTLE_VELOCITY * self.throttle_dir;
        }

        Some(match self.counter % 4 {
            0 => GameCommand::Rotate(angle),
            1 => GameCommand::Fire,
            _ => GameCommand::Throttle(self.throttle),
        })
    }
}

fn main() {
    // TODO: Substitute with your API key and team name.
    let api_key = &env::var("API_KEY").unwrap_or("a".into());
    let team_name = &env::var("TEAM_NAME").unwrap_or("a".into());
    let room_token = &env::var("ROOM_TOKEN").unwrap_or("a".into());

    println!("starting up...");
    tokyo::run(api_key, team_name, room_token, Player::default()).unwrap();
}
