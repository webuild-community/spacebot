use crate::{
    analyzer::{bullet::Bullet, player::Player},
    geom::*,
    models::ClientState,
};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub mod bullet;
pub mod player;

/// Collision detection etc is done at this compute interval.
pub const ANALYSIS_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug)]
pub struct Item {
    pub position: Point,
}
/// `Analyzer` provides a set of methods to analyze the current state of the
/// world, past behaviors of the `Player`s and `Bullet`s, and future projections.
///
/// # Example
///
/// ```
/// let mut analyzer = Analyzer::default();
///
/// // Call push_state at each tick.
/// analyzer.push_state(state, Instant::now());
///
/// // e.g. Find the closest player to yourself.
/// if let Some(player) = analyzer.player_closest() {
///     do_something_with(player);
/// }
/// ```
#[derive(Debug)]
pub struct Analyzer {
    own_player_id: u32,
    players: HashMap<u32, Player>,
    bullets: Vec<Bullet>,
    items: Vec<Item>,
    last_update: Instant,
}

impl PointExt for Item {
    fn point(&self) -> &Point {
        &self.position
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self {
            own_player_id: 0,
            players: HashMap::new(),
            bullets: Vec::new(),
            items: Vec::new(),
            last_update: Instant::now(),
        }
    }
}

impl Analyzer {
    /// This method needs to be called at every client tick.
    pub fn push_state(&mut self, state: &ClientState, time: Instant) {
        self.own_player_id = state.id;

        let mut players = HashMap::new();
        for player_state in state.game_state.players.iter() {
            let player = if let Some(mut prev_player) = self.players.remove(&player_state.id) {
                prev_player.push_state(&player_state, &state.game_state.scoreboard, time);
                prev_player
            } else {
                Player::with_state(&player_state, &state.game_state.scoreboard, time)
            };
            players.insert(player.id, player);
        }
        self.players = players;
        self.items = state.game_state.items.iter().map(|i| Item { position: Point::new(i.x, i.y) }).collect();

        self.bullets = state.game_state.bullets.iter().map(|state| Bullet::new(&state)).collect();

        self.last_update = time;
    }

    pub fn item_closest(&self) -> Option<&Item> {
        self.items.iter().min_by_key(|item| (self.own_player().distance(*item) * 1e3) as u64)
    }

    /// Returns the `Player` specified by an ID.
    pub fn player(&self, id: u32) -> Option<&Player> {
        self.players.get(&id)
    }

    /// Returns your own `Player`.
    ///
    /// # Panics
    ///
    /// It panics when the latest ClientState does not include your own Player.
    /// This can happen if you are dead and being respawn, but since the default
    /// game loop implementation provided by the tokyo crate checks and calls the
    /// `tick()` only when the player is alive, it should never panic in reality.
    pub fn own_player(&self) -> &Player {
        self.player(self.own_player_id).unwrap()
    }

    /// Returns an `Iterator` of `Player`s, excluding your own.
    // FWIW, conservative_impl_trait will help get rid of Box.
    // https://github.com/rust-lang/rfcs/blob/master/text/1522-conservative-impl-trait.md
    pub fn other_players<'a>(&'a self) -> impl Iterator<Item = &'a Player> {
        self.players.values().filter(move |player| player.id != self.own_player_id)
    }

    /// Returns a `Player`, who is closest to the current position of your own
    /// `Player`. None if you are the only `Player`.
    pub fn player_closest(&self) -> Option<&Player> {
        self.other_players().min_by_key(|player| (self.own_player().distance(*player) * 1e3) as u64)
    }

    /// Returns a `Player`, who has been moving the least based on the average
    /// move distance at each tick. None if you are the least moving.
    pub fn player_least_moving(&self) -> Option<&Player> {
        self.other_players()
            .min_by_key(|player| (player.trajectory.ave_abs_velocity().length() * 1e3) as u64)
    }

    /// Returns a `Player`, who has earned the highest score so far. None if you
    /// are the highest scored one.
    pub fn player_highest_score(&self) -> Option<&Player> {
        self.other_players().max_by_key(|player| player.score())
    }

    /// Returns a `Player`, who is expected to have earned the highest score
    /// after the `duration`, based on the `ScoreHistory` of each `Player`. None
    /// if you are the one.
    pub fn player_highest_score_after(&self, duration: Duration) -> Option<&Player> {
        self.other_players().max_by_key(|player| player.score_history.project(duration))
    }

    /// Returns an `Iterator` of `Player`s whose current location is within
    /// the `radius` of your own `Player`.
    pub fn players_within<'a>(&'a self, radius: f32) -> impl Iterator<Item = &'a Player> {
        self.other_players().filter(move |player| self.own_player().distance(*player) <= radius)
    }

    /// Returns an `Iterator` of `Player`s whose current location is within
    /// the `radius` of your own `Player`.
    pub fn players_within_colliding<'a>(&'a self, radius: f32, during: Duration, self_stop: bool) -> impl Iterator<Item = &'a Player> {
        self.players_within(radius)
            .filter(move |player| self.own_player().is_colliding_during(*player, during.clone(), self_stop))
    }

    /// Returns an `Iterator` of `Bullet`s that are shot by you and are still
    /// inside the arena. You can have at most 4 bullets at a time.
    pub fn own_bullets<'a>(&'a self) -> impl Iterator<Item = &'a Bullet> {
        self.bullets.iter().filter(move |bullet| bullet.player_id == self.own_player_id)
    }

    /// Returns an `Iterator` of `Bullet`s that are shot by other `Player`s and
    /// are still inside the arena.
    pub fn other_bullets<'a>(&'a self) -> impl Iterator<Item = &'a Bullet> {
        self.bullets.iter().filter(move |bullet| bullet.player_id != self.own_player_id)
    }

    /// Returns an `Iterator` of `Bullet`s that your `Player` would be colliding
    /// within the `duration`, if you stayed at the current position.
    pub fn bullets_colliding<'a>(&'a self, during: Duration) -> impl Iterator<Item = &'a Bullet> {
        self.other_bullets()
            .filter(move |bullet| self.own_player().is_colliding_during(*bullet, during.clone(), false))
    }

    /// Returns an `Iterator` of `Bullet`s that your `Player` would be colliding
    /// within the `duration`, if you stayed at the current position.
    pub fn bullets_within_colliding<'a>(&'a self, radius: f32, during: Duration) -> impl Iterator<Item = &'a Bullet> {
        self.bullets_within(radius)
            .filter(move |bullet| self.own_player().is_colliding_during(*bullet, during.clone(), false))
    }

    /// Returns an `Iterator` of `Bullet`s that are shot by other `Player`s and
    /// are within the `radius` of your current position.
    pub fn bullets_within<'a>(&'a self, radius: f32) -> impl Iterator<Item = &'a Bullet> {
        self.other_bullets().filter(move |bullet| self.own_player().distance(*bullet) <= radius)
    }
}
