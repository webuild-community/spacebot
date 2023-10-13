use std::{
    fmt::Debug,
    time::{Duration, Instant},
};
use tokyo::analyzer::{Analyzer, player::Player};

pub trait Condition: Send + Debug {
    fn evaluate(&mut self, _: &Analyzer) -> bool;
}

#[derive(Debug)]
pub struct Always;
impl Condition for Always {
    fn evaluate(&mut self, _: &Analyzer) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct And<T1: Debug, T2: Debug> {
    lhs: T1,
    rhs: T2,
}


impl<T1: Condition + Debug, T2: Condition + Debug> And<T1, T2> {
    pub fn new(lhs: T1, rhs: T2) -> Self {
        Self {
            lhs,
            rhs,
        }
    }
}

impl<T1: Condition + Debug, T2: Condition + Debug> Condition for And<T1, T2> {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        self.lhs.evaluate(analyzer) && self.rhs.evaluate(analyzer)
    }
}

#[derive(Debug)]
pub struct Or<T1: Debug, T2: Debug> {
    lhs: T1,
    rhs: T2,
}

impl<T1: Condition + Debug, T2: Condition + Debug> Condition for Or<T1, T2> {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        self.lhs.evaluate(analyzer) || self.rhs.evaluate(analyzer)
    }
}

#[derive(Debug)]
pub struct Not<T: Debug> {
    inner: T,
}

impl<T: Condition + Debug> Not<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Condition + Debug> Condition for Not<T> {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        !self.inner.evaluate(analyzer)
    }
}

#[derive(Debug)]
pub struct AtInterval {
    interval: Duration,
    next: Instant,
}

impl Condition for AtInterval {
    fn evaluate(&mut self, _: &Analyzer) -> bool {
        let now = Instant::now();
        if now >= self.next {
            self.next += self.interval;
            true
        } else {
            false
        }
    }
}

impl AtInterval {
    pub fn new(interval: Duration) -> Self {
        Self { interval, next: Instant::now() }
    }
}

#[derive(Debug)]
pub struct PlayerWithin {
    pub radius: f32,
}

impl Condition for PlayerWithin {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        analyzer.players_within(self.radius).count() > 0
    }
}

#[derive(Debug)]
pub struct PlayerWithHigherScore;

impl Condition for PlayerWithHigherScore {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        analyzer.player_highest_score().is_some()
    }
}

#[derive(Debug)]
pub struct BulletColliding {
    pub radius: f32,
    pub during: Duration,
}

impl BulletColliding {
    pub fn new(radius: f32, during_secs: f32) -> Self {
        Self {
            radius,
            during: Duration::from_secs_f32(during_secs)
        }
    }
}

impl Condition for BulletColliding {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        analyzer.bullets_within_colliding(self.radius, self.during).count() > 0
    }
}


#[derive(Debug)]
pub struct PlayerGettingNear {
    pub search_radius: f32,
    pub near_radius: f32,
    pub during: Duration,
}

impl PlayerGettingNear {
    pub fn new(search_r: f32, near_r: f32, during_secs: f32) -> Self {
        Self {
            search_radius: search_r,
            near_radius: near_r,
            during: Duration::from_secs_f32(during_secs)
        }
    }
}

impl Condition for PlayerGettingNear {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        let mut own_player: Player = analyzer.own_player().clone();
        own_player.radius += self.near_radius;

        for player in analyzer.players_within(self.search_radius) {
            if own_player.is_colliding_during(player, self.during, false) {
                return true;
            }
        }
        false
    }
}
