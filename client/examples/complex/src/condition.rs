use std::{
    fmt::Debug,
    time::{Duration, Instant},
};
use tokyo::analyzer::Analyzer;

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
pub struct BulletWithin {
    pub radius: f32,
}

impl Condition for BulletWithin {
    fn evaluate(&mut self, analyzer: &Analyzer) -> bool {
        analyzer.bullets_within(self.radius).count() > 0
    }
}
