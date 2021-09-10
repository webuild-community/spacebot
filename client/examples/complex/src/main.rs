#![allow(dead_code)]

/// WIP: The implementation is half way, and the behavior has not been verified.
///
/// A more complex example client, that uses a decision tree structure to decide
/// the next behavior at each tick.
use crate::{condition::{Always, And, AtInterval, BulletWithin, PlayerWithin}, strategy::{PrioritizedBehavior, Strategy, StrategyNode}};
use std::time::{Duration, Instant};
use std::env;
use tokyo::{self, Handler, analyzer::Analyzer, behavior::{Chase, Dodge, DodgePlayer, FireAt, Target}, models::*};

mod condition;
mod strategy;

struct Player {
    count: u64,
    elapsed: Duration,
    analyzer: Analyzer,
    strategy: Strategy,

    current_behavior: PrioritizedBehavior,
}

impl Player {
    fn new() -> Self {
        Self {
            count: 0,
            elapsed: Duration::default(),
            analyzer: Analyzer::default(),
            // TODO: Replace with a deeper decision tree. The current, simple
            // logic shoots at an enemy only if it's very close; otherwise it
            // keeps dodging.
            strategy: Strategy::new(vec![
                (
                    Box::new(And::new(BulletWithin { radius: 150.0 }, AtInterval::new(Duration::from_millis(150)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_high(Dodge {}))),
                ),
                (
                    Box::new(And::new(PlayerWithin { radius: 100.0 }, AtInterval::new(Duration::from_millis(10)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_high(DodgePlayer {}))),
                ),
                (
                    Box::new(And::new(PlayerWithin { radius: 700.0 }, AtInterval::new(Duration::from_millis(400)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_medium(FireAt::new(
                        Target::Closest,
                    )))),
                ),
                (
                    Box::new(Always),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_low(Chase::new(
                        Target::Closest,
                        500.0,
                    )))),
                ),
            ]),
            current_behavior: PrioritizedBehavior::new(),
        }
    }
}

impl Handler for Player {
    fn tick(&mut self, state: &ClientState) -> Option<GameCommand> {
        let now = Instant::now();
        self.analyzer.push_state(state, now);

        let next_command = self.current_behavior.behavior.next_command(&self.analyzer);
        if let Some(next_behavior) = self.strategy.next_behavior(&self.analyzer) {
            if next_behavior.priority > self.current_behavior.priority || next_command.is_none() {
                println!("Change behavior to: {:?}", next_behavior);
                self.current_behavior = next_behavior;
                return self.current_behavior.behavior.next_command(&self.analyzer);
            }
        }

        self.count += 1;
        self.elapsed += now.elapsed();

        if self.count % 1000 == 0 {
            println!("Elapsed time for 1000 tick: {:?}", self.elapsed);
            self.elapsed = Duration::default();
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
