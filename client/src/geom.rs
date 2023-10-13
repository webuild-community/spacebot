use euclid::{Angle, Point2D, Vector2D};
use std::time::Duration;

/// A point inside the game arena, in terms of pixels. See [`euclid` documentation]
/// for its API, but you can assume all the basic linear algebra operations are
/// supported e.g. addition, subtraction, and interpolation (aka `lerp`).
///
/// [`euclid` documentation]: https://docs.rs/euclid/0.19.6/euclid/struct.TypedPoint2D.html
pub type Point = Point2D<f32>;

/// A vector inside the game arena, in terms of pixels. It's mostly used to
/// represent a velocity. See [`euclid` documentation] for its API, but you can
/// assume all the basic linear algebra operations are supported e.g. addition,
/// subtraction, magnitude (aka `length`), angle conversion (aka
/// `angle_from_x_axis`), dot and cross product.
///
/// [`euclid` documentation]: https://docs.rs/euclid/0.19.6/euclid/struct.TypedVector2D.html
pub type Vector = Vector2D<f32>;

/// The game server expects angles to be in radians. See [`euclid` documentation]
/// for its API, but you'll mostly need `new` to create and `get` to retrieve
/// radians. `Radian::degrees` method crates `Radian` from a value in degrees, which may
/// be handy. `Radian::positive` may also be useful if you want to normalize the
/// radian to be [0, 2PI).
///
/// [`euclid` documentation]: https://docs.rs/euclid/0.19.6/euclid/struct.Angle.html
pub type Radian = Angle<f32>;

trait AsSecsF32 {
    fn as_secs_f32(&self) -> f32;
}

impl AsSecsF32 for Duration {
    fn as_secs_f32(&self) -> f32 {
        self.as_nanos() as f32 / 1e9
    }
}

/// Extension methods for the `Point` type alias.
pub trait PointExt {
    fn point(&self) -> &Point;

    /// Returns the distance between this and the given points.
    fn distance(&self, other: &dyn PointExt) -> f32 {
        (*other.point() - *self.point()).length()
    }

    /// Returns the angle of the line connecting this point to the given point.
    fn angle_to(&self, other: &dyn PointExt) -> Radian {
        (*other.point() - *self.point()).angle_from_x_axis()
    }

    /// Returns the velocity at which one travels from this point to the given
    /// point for the amount of time `dt`.
    fn velocity_to(&self, other: &dyn PointExt, dt: Duration) -> Vector {
        (*other.point() - *self.point()) / dt.as_secs_f32()
    }

    /// Returns the projection of this position along the given `velocity`
    /// (pixels-per-second) for the amount of time `dt`.
    fn project_with_velocity(&self, velocity: &Vector, dt: Duration) -> Point {
        *self.point() + *velocity * dt.as_secs_f32()
    }
}

impl PointExt for Point {
    fn point(&self) -> &Point {
        self
    }
}

/// Extension methods for the `Vector` type alias.
pub trait VectorExt {
    fn vector(&self) -> &Vector;

    /// Creates a new `Vector` from an angle, from the X axis.
    fn with_angle(angle: Radian) -> Vector {
        let sin_cos = angle.sin_cos();
        Vector::new(sin_cos.1, sin_cos.0)
    }

    /// Returns an angle that is tangent to this `Vector`. Maybe useful for your
    /// dodging behavior.
    fn tangent(&self) -> Radian {
        (self.vector().angle_from_x_axis() + Radian::frac_pi_2()).positive()
    }
}

impl VectorExt for Vector {
    fn vector(&self) -> &Vector {
        self
    }
}

/// Extension methods for the `Radian` type alias.
pub trait RadianExt {
    fn radian(&self) -> &Radian;

    /// Creates a new `Radian` based on a raw value in radians.
    fn new(radians: f32) -> Radian {
        Radian::radians(radians)
    }

    /// Returns a `Radian` whose radian value is `abs()`ed. Not to be confused
    /// with `positive()` method, which is for normalization.
    fn abs(&self) -> Radian {
        Radian::radians(self.radian().radians.abs())
    }
}

impl RadianExt for Radian {
    fn radian(&self) -> &Radian {
        self
    }
}

pub trait Moving: PointExt + VectorExt {
    fn radius(&self) -> f32;
    /// Returns the projection of the current position, or `point()`, along the
    /// velocity, or `vector()`, for the amount of time `dt`.
    fn project(&self, dt: Duration) -> Point {
        self.point().project_with_velocity(self.vector(), dt)
    }
}
