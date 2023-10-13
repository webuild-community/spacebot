use crate::{
    analyzer::ANALYSIS_INTERVAL,
    geom::*,
    models::{self, PlayerState, PLAYER_BASE_SPEED, PLAYER_MIN_THROTTLE},
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
/// `Player` struct contains the past and the current states of a single player
/// identified by an ID. You will usually be accessing `Player`s through the
/// methods provided by `Analyzer`.
pub struct Player {
    pub id: u32,
    pub angle: Radian,
    pub throttle: f32,
    pub position: Point,
    pub velocity: Vector,
    pub trajectory: Trajectory,
    pub score_history: ScoreHistory,
    pub radius: f32,
    pub bullet_speed: f32,
    pub bullet_radius: f32,
}

impl Player {
    /// Creates a new `Player` based on the given `state`.
    pub fn with_state(state: &PlayerState, scoreboard: &HashMap<u32, u32>, time: Instant) -> Self {
        let angle = Radian::new(state.angle);
        let position = Point::new(state.x, state.y);
        let velocity = Vector::with_angle(angle) * state.throttle * PLAYER_BASE_SPEED;

        let mut trajectory = Trajectory::default();
        trajectory.push(position.clone(), time);

        let mut score_history = ScoreHistory::default();
        score_history.push(*scoreboard.get(&state.id).unwrap_or(&0), time);

        Self {
            id: state.id,
            angle,
            throttle: state.throttle,
            radius: state.radius,
            bullet_speed: state.bullet_speed,
            bullet_radius: state.bullet_radius,
            position,
            velocity,
            trajectory,
            score_history,
        }
    }

    /// Updates the `Player` with a new `state`.
    pub fn push_state(
        &mut self,
        state: &PlayerState,
        scoreboard: &HashMap<u32, u32>,
        time: Instant,
    ) {
        assert_eq!(self.id, state.id);

        self.angle = Radian::new(state.angle);
        self.throttle = state.throttle;
        self.position = Point::new(state.x, state.y);
        self.velocity = Vector::with_angle(self.angle) * state.throttle * PLAYER_BASE_SPEED;
        self.trajectory.push(self.position.clone(), time);
        self.score_history.push(*scoreboard.get(&state.id).unwrap_or(&0), time);
    }

    /// Returns the current score of the `Player`.
    pub fn score(&self) -> u32 {
        self.score_history.last_score()
    }

    /// Returns whether the `Player` will be colliding the given `Bullet` at a
    /// particular time in the future, specified by `interval`.
    pub fn is_colliding_at<M: Moving>(&self, target: &M, interval: Duration, self_stop: bool) -> bool {
        let p = if self_stop {
            self.project(Duration::from_secs(0))
        } else {
            self.project(interval)
        };
        p.distance(&target.project(interval)) < target.radius() + self.radius
    }

    /// Returns whether the `Player` will be colliding the given `Bullet` during
    /// the given `interval`.
    pub fn is_colliding_during<M: Moving>(&self, target: &M, interval: Duration, self_stop: bool) -> bool {
        let num_analysis = (interval.as_millis() / ANALYSIS_INTERVAL.as_millis()) as u32;
        (1..=num_analysis)
            .map(|tick| self.is_colliding_at(target, ANALYSIS_INTERVAL * tick, self_stop))
            .any(|hit| hit)
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            angle: Radian::zero(),
            throttle: PLAYER_MIN_THROTTLE,
            radius: models::PLAYER_BASE_RADIUS,
            bullet_radius: models::BULLET_BASE_RADIUS,
            bullet_speed: models::BULLET_BASE_SPEED,
            position: Point::zero(),
            velocity: Vector::zero(),
            trajectory: Trajectory::default(),
            score_history: ScoreHistory::default(),
        }
    }
}

/// `Player` struct provides some basic geometry operations through `PointExt`
/// trait. See the `geom` mod.
impl PointExt for Player {
    fn point(&self) -> &Point {
        &self.position
    }
}

/// `Player` struct provides some basic geometry operations through `VectorExt`
/// trait. See the `geom` mod.
impl VectorExt for Player {
    fn vector(&self) -> &Vector {
        &self.velocity
    }
}

impl Moving for Player {
    fn radius(&self) -> f32 {
        self.radius
    }
}

/// `Trajectory` contains the past positions of a `Player`. You may want to use
/// it to infer the move behavior and logic of a `Player` of your interest.
#[derive(Debug, Default, Clone)]
pub struct Trajectory {
    pub positions: Vec<(Point, Instant)>,
}

impl Trajectory {
    /// Pushes a new state to the `Trajectory`.
    pub fn push(&mut self, position: Point, time: Instant) {
        self.positions.push((position, time));
    }

    /// Returns the last known position of the `Trajectory`.
    ///
    /// # Panics
    ///
    /// It panics if the `push()` method has not been called before. It should
    /// not happen as long as you are calling `Analyzer::push_state()` at the
    /// beginning of each `tick()` method.
    pub fn last_position<'a>(&'a self) -> &'a Point {
        &self.positions.last().unwrap().0
    }

    /// Returns the last known velocity of the `Trajectory`. Zeros if there is
    /// only one data point so far, and a velocity can not be computed.
    ///
    /// # Panics
    ///
    /// It panics if the `push()` method has not been called before. It should
    /// not happen as long as you are calling `Analyzer::push_state()` at the
    /// beginning of each `tick()` method.
    pub fn last_velocity(&self) -> Vector {
        let (last_position, last_time) = self.positions.last().unwrap();
        if let Some((prev_position, prev_time)) = self.positions.get(self.positions.len() - 2) {
            prev_position.velocity_to(last_position, *last_time - *prev_time)
        } else {
            // No idea, just return zeros.
            Vector::zero()
        }
    }

    /// Returns the average of all the past moves stored in the `Trajectory`.
    /// Each move is represented by `Vector::abs()`, so it doesn't indicate in
    /// which direction it's willing to go.
    pub fn ave_abs_velocity(&self) -> Vector {
        let (items, sum) = self
            .positions
            .iter()
            .zip(self.positions.iter().skip(1))
            .map(|((prev_position, prev_time), (position, time))| {
                prev_position.velocity_to(position, *time - *prev_time).abs()
            })
            .fold((0, Vector::zero()), |acc, next| (acc.0 + 1, acc.1 + next));

        if items == 0 {
            Vector::zero()
        } else {
            sum / items as f32
        }
    }
}

/// `ScoreHistory` contains all the record of past scores of a `Player`. It may
/// be useful if you want to identify a `Player` who will likely be the highest
/// scoring in the future, instead of just looking at the current scores.
#[derive(Debug, Default, Clone)]
pub struct ScoreHistory {
    inner: Vec<(u32, Instant)>,
}

impl ScoreHistory {
    /// Pushes a new state to the `ScoreHistory`.
    pub fn push(&mut self, score: u32, time: Instant) {
        self.inner.push((score, time));
    }

    /// Returns the current score of the `Player`.
    ///
    /// # Panics
    ///
    /// It panics if the `push()` method has not been called before. It should
    /// not happen as long as you are calling `Analyzer::push_state()` at the
    /// beginning of each `tick()` method.
    pub fn last_score(&self) -> u32 {
        self.inner.last().unwrap().0
    }

    /// Returns the total score earned since the particular `past_time`.
    ///
    /// # Panics
    ///
    /// It panics if the `push()` method has not been called before. It should
    /// not happen as long as you are calling `Analyzer::push_state()` at the
    /// beginning of each `tick()` method.
    pub fn score_since(&self, past_time: Instant) -> u32 {
        let start_score = self
            .inner
            .iter()
            .rev()
            .find_map(|(score, time)| if *time <= past_time { Some(*score) } else { None })
            .unwrap_or(0u32);
        self.last_score() - start_score
    }

    /// Returns the projected score in a particular time in the future specified
    /// by `after`, based on the past scoring history.
    ///
    /// # Panics
    ///
    /// It panics if the `push()` method has not been called before. It should
    /// not happen as long as you are calling `Analyzer::push_state()` at the
    /// beginning of each `tick()` method.
    pub fn project(&self, after: Duration) -> u32 {
        let past_duration = Duration::from_secs(10); // configurable
        let past_score = self.score_since(Instant::now() - past_duration);
        self.last_score()
            + (past_score as f32 * (after.as_millis() as f32 / past_duration.as_millis() as f32))
                as u32
    }
}
