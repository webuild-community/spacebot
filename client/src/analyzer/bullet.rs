use crate::{geom::*, models::{BULLET_RADIUS, BULLET_SPEED, BulletState}};

/// `Bullet` struct contains the past and the current states of a single bullet
/// identified by an ID. You will usually be accessing `Bullet`s through the
/// methods provided by `Analyzer`.
#[derive(Debug)]
pub struct Bullet {
    pub id: u32,
    pub position: Point,
    pub velocity: Vector,
    pub player_id: u32,
}

impl Bullet {
    /// Creates a new `Bullet` based on the given state.
    pub fn new(state: &BulletState) -> Self {
        Bullet {
            id: state.id,
            position: Point::new(state.x, state.y),
            velocity: Vector::with_angle(Radian::new(state.angle)) * BULLET_SPEED,
            player_id: state.player_id,
        }
    }

    /// Creates a virtual `Bullet` with `position` and `angle`, useful for
    /// collision simulation.
    pub fn with_position_angle(position: Point, angle: Radian) -> Self {
        Bullet { id: 0, position, velocity: Vector::with_angle(angle) * BULLET_SPEED, player_id: 0 }
    }
}

/// `Bullet` struct provides some basic geometry operations through `PointExt`
/// trait. See the `geom` mod.
impl PointExt for Bullet {
    fn point(&self) -> &Point {
        &self.position
    }
}

/// `Bullet` struct provides some basic geometry operations through `VectorExt`
/// trait. See the `geom` mod.
impl VectorExt for Bullet {
    fn vector(&self) -> &Vector {
        &self.velocity
    }
}

impl Moving for Bullet {
    const RADIUS: f32 = BULLET_RADIUS;
}
