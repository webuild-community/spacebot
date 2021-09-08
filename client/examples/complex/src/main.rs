#![allow(dead_code)]

/// WIP: The implementation is half way, and the behavior has not been verified.
///
/// A more complex example client, that uses a decision tree structure to decide
/// the next behavior at each tick.
use crate::{
    condition::{Always, PlayerWithin},
    strategy::{PrioritizedBehavior, Strategy, StrategyNode},
};
use std::time::Instant;
use std::env;
use tokyo::{
    self,
    analyzer::Analyzer,
    behavior::{Dodge, FireAt, Target},
    models::*,
    Handler,
};

mod condition;
mod strategy;

struct Player {
    analyzer: Analyzer,
    strategy: Strategy,

    current_behavior: PrioritizedBehavior,
}

impl Player {
    fn new() -> Self {
        Self {
            analyzer: Analyzer::default(),
            // TODO: Replace with a deeper decision tree. The current, simple
            // logic shoots at an enemy only if it's very close; otherwise it
            // keeps dodging.
            strategy: Strategy::new(vec![
                (
                    Box::new(PlayerWithin { radius: 200.0 }),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_high(FireAt::new(
                        Target::Closest,
                    )))),
                ),
                (
                    Box::new(Always {}),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_medium(Dodge {}))),
                ),
            ]),
            current_behavior: PrioritizedBehavior::new(),
        }
    }
}

impl Handler for Player {
    fn tick(&mut self, state: &ClientState) -> Option<GameCommand> {
        self.analyzer.push_state(state, Instant::now());

        let next_command = self.current_behavior.behavior.next_command(&self.analyzer);
        if let Some(next_behavior) = self.strategy.next_behavior(&self.analyzer) {
            if next_behavior.priority > self.current_behavior.priority || next_command.is_none() {
                self.current_behavior = next_behavior;
                return self.current_behavior.behavior.next_command(&self.analyzer);
            }
        }
        next_command
    }
}

fn main() {
    // TODO: Substitute with your API key and team name.
    let api_key = &env::var("API_KEY").unwrap_or("a".into());
    let team_name = &env::var("TEAM_NAME").unwrap_or("a".into());

    println!("starting up...");
    tokyo::run(api_key, team_name, Player::new()).unwrap();
}
