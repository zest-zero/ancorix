pub mod lerp;
pub mod matrix2;
pub mod vector2;

pub use lerp::Lerp;
pub use matrix2::Matrix2;
pub use vector2::Vector2;

/// The tolerance used when comparing floats, e.g. by
/// [`Vector2::is_zero()`](vector2::Vector2::is_zero).
pub const EPSILON: f32 = 1e-5;
pub(crate) const EPSILON_SQ: f32 = EPSILON * EPSILON;
