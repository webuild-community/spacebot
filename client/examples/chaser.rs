/// An example client that uses the `Analyzer` and `Behavior` structs provided by
/// the client library (aka `tokyo crate`.) `Analyzer` should provide a good
/// basis for modeling the past, current and predicted state of the world, and
/// hopefully is easy enough to get started with. `Behavior` traits and some of
/// the predefined behavior structs involve some more levels of abstraction,
/// which you may or may not like. Please see the documentation in the `behavior`
/// mod for more details.
use std::env;
use std::time::Instant;
use tokyo::{
    self,
    analyzer::Analyzer,
    behavior::{Behavior, Chase, FireAt, Sequence, Target},
    models::*,
    Handler,
};

#[derive(Default)]
struct Player {
    analyzer: Analyzer,
    current_behavior: Box<dyn Behavior>,
}

fn chase() -> Box<dyn Behavior> {
    // Behavior to fire at the target player twice.
    let fire_highest = FireAt::with_times(Target::HighestScore, 1);
    // Behavior to keep chasing the target (in this case, the player with
    // the highest score.) It yields to the next behavior when the distance
    // to the player is less than 200.0.
    let chase = Chase { target: Target::HighestScore, distance: 400.0 };

    // Behavior to fire at the target player twice.
    let fire = FireAt::with_times(Target::HighestScore, 1);

    // A sequence of behaviors: chase and then fire.
    Box::new(Sequence::with_slice(&[&fire_highest, &chase, &fire]))
}

impl Handler for Player {
    fn tick(&mut self, state: &ClientState) -> Option<GameCommand> {
        self.analyzer.push_state(state, Instant::now());

        if let Some(command) = self.current_behavior.next_command(&self.analyzer) {
            Some(command)
        } else {
            // chase() returns a stateful Behavior, which we want to persist
            // across ticks.
            self.current_behavior = chase();
            self.current_behavior.next_command(&self.analyzer)
        }
    }
}

fn main() {
    let api_key = &env::var("API_KEY").unwrap_or("a".into());
    let team_name = &env::var("TEAM_NAME").unwrap_or("a".into());
    let room_token = &env::var("ROOM_TOKEN").unwrap_or("a".into());

    println!("starting up...");
    tokyo::run(api_key, team_name, room_token, Player::default()).unwrap();
}
