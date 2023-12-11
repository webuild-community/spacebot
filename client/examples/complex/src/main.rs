#![allow(dead_code)]

/// WIP: The implementation is half way, and the behavior has not been verified.
///
/// A more complex example client, that uses a decision tree structure to decide
/// the next behavior at each tick.
use crate::{condition::{Always, And, AtInterval, PlayerWithin}, strategy::{PrioritizedBehavior, Strategy, StrategyNode}};
use std::time::{Duration, Instant};
use std::env;
use condition::{BulletColliding, PlayerGettingNear};
use tokyo::{self, Handler, analyzer::Analyzer, behavior::{Dodge, DodgePlayer, FireAt, Target, PickItem, GetAwayFromPlayer}, models::*};

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
                    Box::new(BulletColliding::new(300.0, 2.0)),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_high(Dodge::new(300.0, 2.0)))),
                ),
                (
                    Box::new(And::new(PlayerGettingNear::new(700.0, 300.0, 3.0), AtInterval::new(Duration::from_millis(100)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_medium(GetAwayFromPlayer::new()))),
                ),
                (
                    Box::new(And::new(PlayerWithin { radius: 400.0 }, AtInterval::new(Duration::from_millis(10)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_high(DodgePlayer::new()))),
                ),
                (
                    Box::new(And::new(PlayerWithin { radius: 1000.0 }, AtInterval::new(Duration::from_millis(2000)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_medium(FireAt::new(
                        Target::Closest,
                    )))),
                ),
                (
                    Box::new(And::new(PlayerWithin { radius: 700.0 }, AtInterval::new(Duration::from_millis(1000)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_medium(FireAt::new(
                        Target::Closest,
                    )))),
                ),
                (
                    Box::new(And::new(PlayerWithin { radius: 500.0 }, AtInterval::new(Duration::from_millis(300)))),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_medium(FireAt::new(
                        Target::Closest,
                    )))),
                ),
                (
                    Box::new(Always),
                    Box::new(StrategyNode::Leaf(PrioritizedBehavior::with_low(PickItem))),
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
    let team_name = &env::var("TEAM_NAME").unwrap_or("ThucUnbelievale".into());
    let room_token = &env::var("ROOM_TOKEN").unwrap_or("a".into());

    println!("starting up...");
    tokyo::run(api_key, team_name, room_token, Player::new()).unwrap();
}
