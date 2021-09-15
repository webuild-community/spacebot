use crate::{
    analyzer::{bullet::Bullet, player::Player, Analyzer},
    geom::*,
    models::{GameCommand, PLAYER_MAX_THROTTLE, PLAYER_MIN_THROTTLE},
};
use rand::{thread_rng, Rng};
use std::{collections::VecDeque, fmt::Debug, time::Duration};

/// `Behavior` trait abstracts an action or a series of actions that a `Player`
/// can take. It may be useful if you want to model a complex behavior, that
/// spans multiple ticks, or whose interpretation changes dynamically. You can
/// use `Sequence::with_slice()` to combine multiple behaviors.
///
/// Some `Behavior`s take `Target` as an argument to dynamically specify which
/// player to act against. See its documentation for details (later in this
/// file).
///
/// # Examples
///
/// A stateful usage of `Behavior`.
///
/// ```
/// impl Handlar for Player {
///     fn tick(...) {
///         self.analyzer.push_state(state, Instant::now());
///
///         if let Some(next_command) = self.current_behavior.next_command(&self.analyzer) {
///             return Some(next_command);
///         }
///
///         // Creates a Behavior and stores it in the Player struct, as we need to
///         // persist the state across ticks and keep track of the number of times it
///         // fired.
///         self.current_behavior = Self::next_behavior();
///
///         self.current_behavior.next_command(&analyzer)
///     }
///
///     fn next_behavior() -> Sequence {
///         // Behavior to keep chasing the target (in this case, the player with
///         // the highest score.) It yields to the next behavior when the distance
///         // to the player is less than 200.0.
///         let chase = Chase { target: Target::HighestScore, distance: 200.0 };
///
///         // Behavior to fire at the target player twice.
///         let fire = FireAt::with_times(Target::HighestScore, 2);
///
///         // A sequence of behaviors: chase and then fire twice.
///         Sequence::with_slice(&[&chase, &fire])
///     }
/// }
/// ```
///
/// A stateless usage of `Behavior`.
///
/// ```
/// impl Handlar for Player {
///     fn tick(...) {
///         self.analyzer.push_state(state, Instant::now());
///
///         // Find one of the bullets that are colliding within a second.
///         if let Some(bullet) = self.analyzer.bullets_colliding(Duration::from_secs(1)).next() {
///             let angle = bullet.velocity.tangent();
///
///             // Try to dodge from the bullet by moving to a direction roughly
///             // perpendicular to the bullet velocity.
///             let dodge = Sequence::with_slice(&[
///                 &Rotate::with_margin_degrees(angle, 30.0),
///                 &Throttle::max(),
///             ]);
///
///             // This Behavior works without persisting it across multiple tick()s as in the
///             // previous example. At the next tick(), Rotate behavior will most likely return
///             // None, proceeding immediately to the Throttle behavior. If the situation
///             // changes e.g. the bullet hit someone else, or there are other bullets
///             // colliding, then it may take the Rotate behavior again, but it's likely an
///             // optimal adjustment (assuming your logic of selecting a bullet to dodge is
///             // stable.)
///             return dodge.next_command(&self.analyzer);
///         }
///         None
///     }
/// }
/// ```
pub trait Behavior: Send + Debug {
    // Returns the next `GameCommand` to achieve this `Behavior`. None if there
    // is nothing more to do.
    fn next_command(&mut self, _: &Analyzer) -> Option<GameCommand>;

    // `Clone` does not work nicely with `Box` yet, so you'll need to implement
    // this method manually for each struct.
    fn box_clone(&self) -> Box<dyn Behavior>;
}

impl Clone for Box<dyn Behavior> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

impl Default for Box<dyn Behavior> {
    fn default() -> Self {
        Box::new(Skip {})
    }
}

/// `Sequence` represents a series of `Behavior`s. The first
/// `Behavior::next_command()` is repeatedly called until it yields `None`, and
/// then it moves to the second `Behavior`, and so forth.
#[derive(Clone, Debug)]
pub struct Sequence {
    inner: VecDeque<Box<dyn Behavior>>,
}

impl Behavior for Sequence {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        while let Some(next) = self.inner.front_mut() {
            if let Some(command) = next.next_command(analyzer) {
                return Some(command);
            }
            self.inner.pop_front();
        }
        None
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Sequence {
    pub fn new() -> Self {
        Sequence::with_slice(&[])
    }

    pub fn with_slice(behaviors: &[&dyn Behavior]) -> Self {
        Self { inner: behaviors.into_iter().map(|b| b.box_clone()).collect::<VecDeque<_>>() }
    }
}

/// A `Behavior` that always evaluates to `None`.
#[derive(Clone, Debug)]
pub struct Skip;

impl Behavior for Skip {
    fn next_command(&mut self, _: &Analyzer) -> Option<GameCommand> {
        None
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// A `Behavior` that set throttle to 0.
#[derive(Clone, Debug)]
pub struct Stop;

impl Behavior for Stop {
    fn next_command(&mut self, _: &Analyzer) -> Option<GameCommand> {
        Some(GameCommand::Throttle(0.0))
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// A `Behavior` that have no effect but still consumes an action.
#[derive(Clone, Debug)]
pub struct Noop;

impl Behavior for Noop {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        // Rotate to the current angle; thus no effect.
        Some(GameCommand::Rotate(analyzer.own_player().angle.positive().get()))
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// A `Behavior` to set the current throttle value, unless it's within an error
/// margin (hard-coded as 0.05 now).
#[derive(Clone, Debug)]
pub struct Throttle {
    pub value: f32,
}

impl Behavior for Throttle {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        if (analyzer.own_player().throttle - self.value).abs() > 0.05 {
            Some(GameCommand::Throttle(self.value))
        } else {
            None
        }
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Throttle {
    pub fn stop() -> Self {
        Self { value: 0.0 }
    }

    pub fn max() -> Self {
        Self { value: PLAYER_MAX_THROTTLE }
    }
}

/// A `Behavior` to move to the `destination`.
#[derive(Clone, Debug)]
pub struct MoveTo {
    pub destination: Point,
    /// Whether to stop at the end of the behavior.
    pub end_with_brake: bool,
}

impl Behavior for MoveTo {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        let own_player = analyzer.own_player();
        if own_player.distance(&self.destination) < 10.0 {
            if self.end_with_brake {
                self.end_with_brake = false;
                return Some(GameCommand::Throttle(0.0));
            } else {
                return None;
            }
        }

        // TODO: Don't block with Noop.
        let angle = own_player.angle_to(&self.destination);
        Sequence::with_slice(&[
            &Rotate::with_margin_degrees(angle, 5.0),
            &Throttle::max(),
            &Noop {},
        ])
        .next_command(analyzer)
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// A `Behavior` to rotate to the specified `angle`. It yield `None` if the
/// current angle is within the error `margin`.
#[derive(Clone, Debug)]
pub struct Rotate {
    angle: Radian,
    margin: Radian,
}

impl Behavior for Rotate {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        if (analyzer.own_player().angle.positive() - self.angle.positive()).abs() > self.margin {
            Some(GameCommand::Rotate(self.angle.positive().get()))
        } else {
            None
        }
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Rotate {
    pub fn new(angle: Radian) -> Self {
        Self::with_margin_degrees(angle, 0.1)
    }

    pub fn with_margin_degrees(angle: Radian, margin_degrees: f32) -> Self {
        Self { angle, margin: Radian::degrees(margin_degrees) }
    }
}

/// A `Behavior` to fire the specified number of `times`.j
#[derive(Clone, Debug)]
pub struct Fire {
    times: u32,
}

impl Behavior for Fire {
    fn next_command(&mut self, _: &Analyzer) -> Option<GameCommand> {
        if self.times > 0 {
            self.times -= 1;
            Some(GameCommand::Fire)
        } else {
            None
        }
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl Fire {
    pub fn new() -> Self {
        Self::with_times(1)
    }

    pub fn with_times(times: u32) -> Self {
        Self { times }
    }
}

/// A `Behavior` to rotate to the `target` and fire the specified number of
/// `times`.
#[derive(Clone, Debug)]
pub struct FireAt {
    target: Target,
    times: u32,
    next: Sequence,
}

impl Behavior for FireAt {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        if let Some(next_command) = self.next.next_command(analyzer) {
            return Some(next_command);
        }

        if self.times > 0 {
            if let Some(target) = self.target.get(analyzer) {
                self.times -= 1;

                let own_player = analyzer.own_player();
                let angle = own_player.angle_to(target);
                // Don't bother solving the math. Monte Carlo would do in this small world.
                let corrected_angle = (-30..30)
                    .map(|da| angle / 10.0 + Radian::degrees(da as f32))
                    .filter(|angle| {
                        target.is_colliding_during(
                            &Bullet::with_position_angle(own_player.position, *angle),
                            Duration::from_secs(4),
                        )
                    })
                    .next()
                    .unwrap_or(angle);
                self.next = Sequence::with_slice(&[
                    &Rotate::with_margin_degrees(corrected_angle, 0.0),
                    &Fire::new(),
                ]);
                return self.next.next_command(analyzer);
            }
        }
        None
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

impl FireAt {
    pub fn new(target: Target) -> Self {
        Self::with_times(target, 1)
    }

    pub fn with_times(target: Target, times: u32) -> Self {
        Self { target, times, next: Sequence::new() }
    }
}

/// A `Behavior` to send a random command.
#[derive(Clone, Debug)]
struct Random;

impl Behavior for Random {
    fn next_command(&mut self, _: &Analyzer) -> Option<GameCommand> {
        let mut rng = thread_rng();
        match rng.gen_range(0, 4) {
            0 => None,
            1 => Some(GameCommand::Rotate(rng.gen_range(0.0, 2.0 * std::f32::consts::PI))),
            2 => {
                Some(GameCommand::Throttle(rng.gen_range(PLAYER_MIN_THROTTLE, PLAYER_MAX_THROTTLE)))
            },
            3 => Some(GameCommand::Fire),
            _ => unreachable!(),
        }
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// A `Behavior` to keep moving towards the specified `target` until it reaches
/// within the `distance`.
#[derive(Clone, Debug)]
pub struct Chase {
    pub target: Target,
    pub distance: f32,
}

impl Chase {
    pub fn new(target: Target, distance: f32) -> Self {
        Self {
            target, distance
        }
    }
}

impl Behavior for Chase {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        if let Some(target) = self.target.get(analyzer) {
            let distance_to_target = analyzer.own_player().distance(target);
            if distance_to_target > self.distance {
                let angle = analyzer.own_player().angle_to(target);
                // TODO: Don't block with Noop.
                return Sequence::with_slice(&[
                    &Rotate::with_margin_degrees(angle, 10.0),
                    &Throttle::max(),
                    &Noop {},
                ])
                .next_command(analyzer);
            }
        }
        None
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// A `Behavior` to keep dodging nearby bullets as much as possible at the
/// maximum throttle.
#[derive(Clone, Debug)]
pub struct Dodge;

impl Behavior for Dodge {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        if let Some(bullet) = analyzer.bullets_within_colliding(300.0, Duration::from_secs(2)).next() {
            let angle = bullet.velocity.tangent();
            if analyzer.own_player().velocity.eq(&Vector::zero()) {
                Sequence::with_slice(&[&Throttle::max(), &Rotate::with_margin_degrees(angle, 15.0)])
                    .next_command(analyzer)
            } else {
                Sequence::with_slice(&[&Rotate::with_margin_degrees(angle, 15.0)])
                    .next_command(analyzer)
            }
        } else {
            None
        }
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// A `Behavior` to keep dodging nearby bullets as much as possible at the
/// maximum throttle.
#[derive(Clone, Debug)]
pub struct DodgePlayer;

impl Behavior for DodgePlayer {
    fn next_command(&mut self, analyzer: &Analyzer) -> Option<GameCommand> {
        if let Some(player) = analyzer.players_within_colliding(300.0, Duration::from_secs(2)).next() {
            println!("Player will collide: {}, velocity: {}", player.id, player.velocity);
            let angle = player.velocity.tangent();
            if analyzer.own_player().velocity.eq(&Vector::zero()) {
                Sequence::with_slice(&[&Throttle::max(), &Rotate::with_margin_degrees(angle, 15.0)])
                    .next_command(analyzer)
            } else {
                Sequence::with_slice(&[&Rotate::with_margin_degrees(angle, 15.0)])
                    .next_command(analyzer)
            }
        } else {
            None
        }
    }

    fn box_clone(&self) -> Box<dyn Behavior> {
        Box::new(self.clone())
    }
}

/// `Target enum` is used to specify a `Player` based on some predefined
/// conditions. Some `Behavior`s like `FireAt` works with `Target` to dynamically
/// compute the target `Player`.
#[derive(Clone, Debug)]
pub enum Target {
    /// Player specified by an ID.
    Id(u32),

    /// Player currently closest to you.
    Closest,

    /// Player that is least moving in the past.b
    LeastMoving,

    /// Player with the highest score so far.
    HighestScore,

    /// Player with the highest predicted score at a certain time in the future.
    HighestScoreAfter(Duration),
}

impl Target {
    /// Returns a reference to a `Player` based on the condition. `None` if no
    /// players match the condition.
    pub fn get<'a>(&self, analyzer: &'a Analyzer) -> Option<&'a Player> {
        match self {
            Target::Id(id) => analyzer.player(*id),
            Target::Closest => analyzer.player_closest(),
            Target::LeastMoving => analyzer.player_least_moving(),
            Target::HighestScore => analyzer.player_highest_score(),
            Target::HighestScoreAfter(after) => analyzer.player_highest_score_after(*after),
        }
    }
}
