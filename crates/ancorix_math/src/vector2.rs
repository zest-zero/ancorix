//! A 2D vector, used for both positions and directions.
//!
//! [`Vector2`] is `#[repr(C)]` with two public fields, `x` and `y` - the same
//! layout as a plain `[f32; 2]`, safe to pass across FFI boundaries or upload
//! directly to the GPU.
//!
//! # Const contexts
//!
//! Operator traits (`Add`, `Sub`, `Mul`, `Div`, `Rem`) can't be `const fn` on
//! stable Rust - the trait itself would need to be a `const trait`, and the
//! standard library's isn't. For `const` contexts, use the `const_*` methods
//! (`const_add`, `const_sub`, `const_mul`, `const_div`, `const_rem`, and their
//! `_assign` variants) instead of the corresponding operators.

use crate::{EPSILON, EPSILON_SQ};
use std::fmt::Display;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
    SubAssign,
};

#[doc(alias = "vec2")]
#[doc(alias = "v2")]
#[doc(alias = "pos")]
#[doc(alias = "point")]
#[doc(alias = "position")]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
#[must_use]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vector2 {
    /// Returns a new [`Vector2`] with `x` and `y` set to `0.0`.
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl Vector2 {
    /// Returns a new [`Vector2`] with `x` and `y` set to `0.0`.
    pub const ZERO: Self = Self::splat(0.0);
    /// Returns a new [`Vector2`] with `x` and `y` set to `1.0`.
    pub const ONE: Self = Self::splat(1.0);
    /// Returns a new [`Vector2`] with `x` and `y` set to `-1.0`.
    pub const NEG_ONE: Self = Self::splat(-1.0);

    /// Returns a new [`Vector2`] with `x` set to `1.0` and `y` set to `0.0`.
    pub const X: Self = Self::new(1.0, 0.0);
    /// Returns a new [`Vector2`] with `x` set to `-1.0` and `y` set to `0.0`.
    pub const NEG_X: Self = Self::new(-1.0, 0.0);
    /// Returns a new [`Vector2`] with `x` set to `0.0` and `y` set to `1.0`.
    pub const Y: Self = Self::new(0.0, 1.0);
    /// Returns a new [`Vector2`] with `x` set to `0.0` and `y` set to `-1.0`.
    pub const NEG_Y: Self = Self::new(0.0, -1.0);

    /// Returns a new [`Vector2`] with `x` set to `0.0` and `y` set to `1.0`.
    pub const UP: Self = Self::Y;
    /// Returns a new [`Vector2`] with `x` set to `0.0` and `y` set to `-1.0`.
    pub const DOWN: Self = Self::NEG_Y;
    /// Returns a new [`Vector2`] with `x` set to `1.0` and `y` set to `0.0`.
    pub const RIGHT: Self = Self::X;
    /// Returns a new [`Vector2`] with `x` set to `-1.0` and `y` set to `0.0`.
    pub const LEFT: Self = Self::NEG_X;

    /// Returns a new [`Vector2`] with `x` set to `f32::INFINITY` and `y` set to `f32::INFINITY`.
    pub const INFINITY: Self = Self::new(f32::INFINITY, f32::INFINITY);
    /// Returns a new [`Vector2`] with `x` set to `f32::NEG_INFINITY` and `y` set to `f32::NEG_INFINITY`.
    pub const NEG_INFINITY: Self = Self::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    /// Returns a new [`Vector2`] with `x` set to `f32::NAN` and `y` set to `f32::NAN`.
    pub const NAN: Self = Self::new(f32::NAN, f32::NAN);

    /// Returns a new [`Vector2`] with given `x` and `y` values.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::Vector2;
    ///
    /// let v = Vector2::new(1.0, 2.0);
    /// assert_eq!((v.x, v.y), (1.0, 2.0));
    /// ```
    /// # See also
    ///
    /// * [`Vector2::default()`]
    /// * [`Vector2::splat()`]
    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns a new [`Vector2`] with both `x` and `y` values set to `v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::Vector2;
    ///
    /// let v = Vector2::splat(1.0);
    /// assert_eq!((v.x, v.y), (1.0, 1.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::default()`]
    /// * [`Vector2::new()`]
    #[inline(always)]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    /// Returns a reference to the component at the given index.
    ///
    /// Returns `Some(x)` for index `0` and `Some(y)` for index `1`.
    /// If the index is out of bounds, returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(1.0, 2.0);
    ///
    /// assert_eq!(v.get(0), Some(1.0));
    /// assert_eq!(v.get(1), Some(2.0));
    /// assert_eq!(v.get(2), None);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::get_mut()`]
    #[inline]
    #[must_use]
    pub const fn get(self, index: usize) -> Option<f32> {
        match index {
            0 => Some(self.x),
            1 => Some(self.y),
            _ => None,
        }
    }

    /// Returns a mutable reference to the component at the given index.
    ///
    /// Returns `Some(&mut x)` for index `0` and `Some(&mut y)` for index `1`.
    /// If the index is out of bounds, returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let mut v = v2!(1.0, 2.0);
    ///
    /// if let Some(x) = v.get_mut(0) {
    ///     *x = 5.0;
    /// }
    ///
    /// assert_eq!(v.x, 5.0);
    /// assert_eq!(v.get_mut(2), None);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::get()`]
    #[inline]
    pub const fn get_mut(&mut self, index: usize) -> Option<&mut f32> {
        match index {
            0 => Some(&mut self.x),
            1 => Some(&mut self.y),
            _ => None,
        }
    }

    /// Returns `true` if the absolute values of both `x` and `y` components
    /// are less than or equal to [`crate::EPSILON`]. Otherwise, returns `false`.
    ///
    /// Checks if the [`Vector2`] vector is approximately zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(0.0);
    /// let v1 = v2!(1.0);
    ///
    /// assert!(v0.is_zero());
    /// assert!(!v1.is_zero());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.length_squared() <= EPSILON_SQ
    }

    /// Returns `true` if all components of the [`Vector2`] vector are finite.
    ///
    /// A number is considered finite if it is neither an infinity nor `NaN` (Not a Number).
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(1.0, 2.0);
    /// let v1 = v2!(f32::INFINITY, 4.0);
    ///
    /// assert!(v0.is_finite());
    /// assert!(!v1.is_finite());
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::is_infinite()`]
    /// * [`Vector2::is_valid()`]
    #[inline]
    #[must_use]
    pub const fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Does the same thing as [`Vector2::is_finite()`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(1.0, 2.0);
    /// let v1 = v2!(f32::NAN, f32::INFINITY);
    ///
    /// assert!(v0.is_valid());
    /// assert!(!v1.is_valid());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.is_finite()
    }

    /// Returns `true` if at least one component of the [`Vector2`] vector is infinite.
    ///
    /// An infinite value can be either positive or negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(3.0, f32::INFINITY);
    /// let v1 = v2!(4.0, 2.0);
    ///
    /// assert!(v0.is_infinite());
    /// assert!(!v1.is_infinite());
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::is_finite()`]
    #[inline]
    #[must_use]
    pub const fn is_infinite(self) -> bool {
        self.x.is_infinite() || self.y.is_infinite()
    }

    /// Returns `true` if at least one component of the [`Vector2`] vector is NaN.
    ///
    /// A `NaN` (Not a Number) value represents an undefined or unrepresentable
    /// result of a floating-point operation, such as dividing zero by zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(f32::NAN, 2.0);
    /// let v1 = v2!(1.0, 5.0);
    ///
    /// assert!(v0.is_nan());
    /// assert!(!v1.is_nan());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_nan(self) -> bool {
        self.x.is_nan() || self.y.is_nan()
    }

    /// Returns `true` if the [`Vector2`] vector is approximately normalized.
    ///
    /// Checks if the vector's length is close to `1.0` within a small
    /// tolerance defined by [`crate::EPSILON`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(0.6, 0.8);
    /// let v1 = v2!(2.0);
    ///
    /// assert!(v0.is_normalized());
    /// assert!(!v1.is_normalized());
    /// ```
    ///
    /// # See also
    /// * [`Vector2::normalize()`]
    /// * [`Vector2::try_normalize()`]
    /// * [`Vector2::normalize_unchecked()`]
    #[inline]
    #[must_use]
    pub const fn is_normalized(self) -> bool {
        (self.length_squared() - 1.0).abs() <= EPSILON
    }

    /// Returns the squared length of the [`Vector2`] vector.
    ///
    /// This method is highly efficient as it avoids a costly square root operation.
    /// Useful for distance comparisons, bounding circle checks, or proximity thresholds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(3.0, 4.0);
    /// let len_sq = v.length_squared();
    ///
    /// assert_eq!(len_sq, 25.0);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::length()`]
    #[inline]
    #[must_use]
    pub const fn length_squared(self) -> f32 {
        self.dot(self)
    }

    /// Returns the length (magnitude) of the [`Vector2`] vector.
    ///
    /// This method computes the square root of the vectors squared length.
    /// Useful for calculating precise object movement, speeds, or UI layout offsets.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(3.0, 4.0);
    /// let len = v.length();
    ///
    /// assert_eq!(len, 5.0);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::length_squared()`]
    #[doc(alias = "magnitude")]
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Returns the reciprocal of the length (magnitude) of the [`Vector2`] vector.
    ///
    /// Useful for manual normalization or scaling by inverse distance.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(3.0, 4.0);
    /// let len_recip = v.length_recip();
    ///
    /// assert!((len_recip - 0.2).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::length()`]
    /// * [`Vector2::length_squared()`]
    /// * [`Vector2::normalize()`]
    #[inline]
    #[must_use]
    pub fn length_recip(self) -> f32 {
        self.length_squared().sqrt().recip()
    }

    /// Projects `self` onto `rhs`, returning the parallel component.
    ///
    /// The projection is the closest point on the line defined by `rhs`
    /// to the tip of `self`. Useful for shadow calculations, steering behaviors,
    /// or decomposing a vector into basis components.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(4.0, 0.0);
    /// let onto = v2!(1.0, 1.0);
    /// let proj = v.project(onto);
    ///
    /// assert!((proj.x - 2.0).abs() < 1e-5);
    /// assert!((proj.y - 2.0).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::reject()`]
    /// * [`Vector2::dot()`]
    #[inline]
    pub const fn project(self, rhs: Self) -> Self {
        let rhs_len_sq = rhs.length_squared();
        if rhs_len_sq <= EPSILON_SQ {
            return Self::ZERO;
        }
        rhs.const_mul(self.dot(rhs) * rhs_len_sq.recip())
    }

    /// Reflects `self` across a surface with the given `normal`.
    ///
    /// Useful for bounce physics,ray casting, or mirror-like surface interactions.
    ///
    /// # Note
    ///
    /// The `normal` vector **must be normalized** (unit length) for the reflection
    /// to be mathematically correct.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(1.0, -1.0);
    /// let normal = v2!(0.0, 1.0);
    /// let reflected = v.reflect(normal);
    ///
    /// assert!((reflected.x - 1.0).abs() < 1e-5);
    /// assert!((reflected.y - 1.0).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::project()`]
    /// * [`Vector2::reject()`]
    #[inline]
    pub const fn reflect(self, normal: Self) -> Self {
        self.const_sub(normal.const_mul(2.0 * self.dot(normal)))
    }

    /// Returns the component of `self` perpendicular to `rhs`.
    ///
    /// This is the orthogonal complement of the projection. The sum of
    /// `project(rhs)` and `reject(rhs)` equals the original vector.
    /// Useful for decomposing motion into parallel and perpendicular parts.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(4.0, 0.0);
    /// let onto = v2!(1.0, 1.0);
    /// let rej = v.reject(onto);
    ///
    /// assert!((rej.x - 2.0).abs() < 1e-5);
    /// assert!((rej.y - (-2.0)).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::project()`]
    /// * [`Vector2::dot()`]
    #[inline]
    pub const fn reject(self, rhs: Self) -> Self {
        self.const_sub(self.project(rhs))
    }

    /// Returns the signed angle from `self` to `rhs` in radians.
    ///
    /// The result is in the range `[-π, π]`. A positive value means a
    /// counter-clockwise rotation from `self` to `rhs`. Useful for
    /// steering, aiming, or determining turn direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::f32::consts::FRAC_PI_2;
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(1.0, 0.0);
    /// let v1 = v2!(0.0, 1.0);
    /// let angle = v0.angle_to(v1);
    ///
    /// assert!((angle - FRAC_PI_2).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::angle()`]
    /// * [`Vector2::rotated()`]
    /// * [`Vector2::cross()`]
    #[inline]
    #[must_use]
    pub fn angle_to(self, rhs: Self) -> f32 {
        let cross = self.cross(rhs);
        let dot = self.dot(rhs);
        cross.atan2(dot)
    }

    /// Moves `self` towards `target` by a maximum distance of `max_distance`.
    ///
    /// Useful for smooth following, homing projectiles,
    /// or UI element transitions with a speed limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let current = v2!(0.0, 0.0);
    /// let target = v2!(10.0, 0.0);
    ///
    /// let moved = current.move_towards(target, 3.0);
    /// assert_eq!(moved, v2!(3.0, 0.0));
    ///
    /// let overshoot = current.move_towards(target, 15.0);
    /// assert_eq!(overshoot, target);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::lerp()`]
    /// * [`Vector2::distance()`]
    #[inline]
    pub fn move_towards(self, target: Self, max_distance: f32) -> Self {
        let diff = target - self;
        let dist_sq = diff.length_squared();
        if dist_sq <= max_distance * max_distance || dist_sq <= EPSILON_SQ {
            return target;
        }
        let dist_inv = dist_sq.sqrt().recip();
        self + diff * (max_distance * dist_inv)
    }

    /// Returns a new [`Vector2`] vector with components multiplied component-wise.
    ///
    /// Also known as the Hadamard product. Useful for scaling axes independently,
    /// applying non-uniform transforms, or masking components.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v1 = v2!(2.0, 3.0);
    /// let v2 = v2!(4.0, 5.0);
    /// let result = v1.component_mul(v2);
    ///
    /// assert_eq!(result, v2!(8.0, 15.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::component_div()`]
    /// * [`Vector2::mul()`]
    #[inline]
    pub const fn component_mul(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }

    /// Returns a new [`Vector2`] vector with components divided component-wise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v1 = v2!(8.0, 15.0);
    /// let v2 = v2!(2.0, 3.0);
    /// let result = v1.component_div(v2);
    ///
    /// assert_eq!(result, v2!(4.0, 5.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::component_mul()`]
    /// * [`Vector2::div()`]
    #[inline]
    pub const fn component_div(self, rhs: Self) -> Self {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }

    /// Returns the midpoint between `self` and `rhs`.
    ///
    /// Equivalent to `(self + rhs) * 0.5`. Useful for finding centers of
    /// line segments, averaging positions, or splitting paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v1 = v2!(0.0, 0.0);
    /// let v2 = v2!(10.0, 6.0);
    /// let mid = v1.midpoint(v2);
    ///
    /// assert_eq!(mid, v2!(5.0, 3.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::lerp()`]
    /// * [`Vector2::distance()`]
    #[inline]
    pub const fn midpoint(self, rhs: Self) -> Self {
        Self::new((self.x + rhs.x) * 0.5, (self.y + rhs.y) * 0.5)
    }

    /// Clamps the length of the [`Vector2`] vector to be at most `max_length`.
    ///
    /// If the vector's length is already less than or equal to `max_length`,
    /// it is returned unchanged. Otherwise, it is scaled down to `max_length`.
    /// Useful for capping movement speed, limiting steering forces, or constraining
    /// joystick input.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(6.0, 8.0);
    /// let clamped = v.clamp_length(5.0);
    ///
    /// assert!((clamped.length() - 5.0).abs() < 1e-5);
    ///
    /// let short = v2!(1.0, 1.0);
    /// let clamped_short = short.clamp_length(5.0);
    /// assert_eq!(clamped_short, short);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::normalize()`]
    /// * [`Vector2::clamp()`]
    /// * [`Vector2::length()`]
    #[inline]
    pub fn clamp_length(self, max_length: f32) -> Self {
        let len_sq = self.length_squared();
        let max_len_sq = max_length * max_length;
        if len_sq <= max_len_sq || len_sq <= EPSILON_SQ {
            return self;
        }
        self * (max_length * len_sq.sqrt().recip())
    }

    /// Returns the squared distance between two [`Vector2`] vectors.
    ///
    /// This method is highly efficient and should be preferred over [`Vector2::distance()`]
    /// when you only need to compare distances between points (e.g., in proximity checks).
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v1 = v2!(1.0, 1.0);
    /// let v2 = v2!(4.0, 5.0);
    ///
    /// let dist_sq = v1.distance_squared(v2);
    ///
    /// assert_eq!(dist_sq, 25.0);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::distance()`]
    #[inline]
    #[must_use]
    pub const fn distance_squared(self, rhs: Self) -> f32 {
        self.const_sub(rhs).length_squared()
    }

    /// Returns the Euclidean distance between two [`Vector2`] vectors.
    ///
    /// Use this method when you need the actual distance in pixels or units.
    /// If you only need to compare distances, use [`Vector2::distance_squared()`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v1 = v2!(1.0, 1.0);
    /// let v2 = v2!(4.0, 5.0);
    ///
    /// let dist = v1.distance(v2);
    ///
    /// assert_eq!(dist, 5.0);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::distance_squared()`]
    #[inline]
    #[must_use]
    pub fn distance(self, rhs: Self) -> f32 {
        self.distance_squared(rhs).sqrt()
    }

    /// Returns the angle of the [`Vector2`] vector relative to the positive x-axis,
    /// in the range `-π, π`.
    ///
    /// The returned angle is in the range `[-π, π]`. Useful for UI element rotations,
    /// calculating sprite directions, or steering logic.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::f32::consts::FRAC_PI_2;
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(0.0, 1.0);
    /// let angle = v.angle();
    ///
    /// assert!((angle - FRAC_PI_2).abs() < 1e-5);
    /// ```
    #[inline]
    #[must_use]
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Returns the dot product of two [`Vector2`] vectors.
    ///
    /// The dot product is a scalar value that represents the length of one
    /// vector projected onto another. It is a powerful tool for determining the
    /// relationship between two vectors.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(7.0, 2.0);
    /// let v1 = v2!(1.0, 2.0);
    ///
    /// assert_eq!(v0.dot(v1), 11.0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Returns the 2D cross product of two [`Vector2`] vectors.
    ///
    /// The returned scalar represents the signed magnitude of the 3D cross product if
    /// the vectors were on the XY plane. Useful for determining clockwise/counter-clockwise
    /// rotation, left/right side checks, or calculating the signed area of triangles.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(1.0, 0.0);
    /// let v1 = v2!(0.0, 1.0);
    ///
    /// let cross: f32 = v0.cross(v1);
    /// assert_eq!(cross, 1.0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    /// Attempts to return a new [`Vector2`] vector with a length of 1.0.
    ///
    /// This method scales the vector to a unit length. If the vector's length
    /// is less than or equal to [`crate::EPSILON`],
    /// a zero vector [`Vector2::ZERO`] is returned. This behavior is useful for safely handling
    /// vectors that cannot be normalized.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(3.0, 4.0);
    /// // Successfully normalizes a non-zero vector
    /// let v1 = v0.normalize();
    /// assert!(v1.is_normalized());
    ///
    /// let v0 = v2!(0.0);
    /// // Returns the zero vector
    /// let v1 = v0.normalize();
    /// assert!(!v1.is_normalized());
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::length_squared()`]
    /// * [`Vector2::try_normalize()`]
    /// * [`Vector2::normalize_unchecked()`]
    /// * [`Vector2::is_normalized()`]
    #[inline]
    pub fn normalize(self) -> Self {
        let len_sq = self.length_squared();

        if len_sq <= EPSILON_SQ {
            return Self::ZERO;
        }
        self * len_sq.sqrt().recip()
    }

    /// Attempts to return a new [`Vector2`] with a length of 1.0.
    ///
    /// This method is a **safe** alternative to [`Vector2::normalize()`]. It scales the
    /// vector to a unit length, but returns `None` if the vector's length is
    /// less than or equal to [`crate::EPSILON`]. This allows for explicit error
    /// handling and prevents a potential division by zero error at runtime.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v0 = v2!(3.0, 4.0);
    /// // Successfully normalizes a non-zero vector
    /// let v1 = v0.try_normalize();
    /// assert!(v1.is_some());
    ///
    /// let v0 = v2!(0.0);
    /// // Returns None
    /// let v1 = v0.try_normalize();
    /// assert!(v1.is_none());
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::length_squared()`]
    /// * [`Vector2::normalize()`]
    /// * [`Vector2::normalize_unchecked()`]
    /// * [`Vector2::is_normalized()`]
    #[inline]
    pub fn try_normalize(self) -> Option<Self> {
        let len_sq = self.length_squared();

        if len_sq <= EPSILON_SQ {
            None
        } else {
            Some(self * len_sq.sqrt().recip())
        }
    }

    /// Returns a new [`Vector2`] vector with a length of 1.0.
    ///
    /// This method is intended for use in situations where you are confident the vector
    /// will not have a zero length, but you want to assert that assumption during
    /// development.
    ///
    /// # Panics
    ///
    /// This function **panics in debug builds** if the vector's length
    /// is less than or equal to [`crate::EPSILON`]. This is a developer assertion
    /// and will not happen in release builds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(3.0, 4.0);
    /// let normalized_v = v.normalize_unchecked();
    ///
    /// assert!((normalized_v.length() - 1.0).abs() < 1e-5);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Vector2::length_squared()`]
    /// * [`Vector2::normalize()`]
    /// * [`Vector2::try_normalize()`]
    /// * [`Vector2::is_normalized()`]
    #[inline]
    pub fn normalize_unchecked(self) -> Self {
        let len_sq = self.length_squared();

        debug_assert!(len_sq > EPSILON_SQ, "Attempt to normalize zero vector");

        self * len_sq.sqrt().recip()
    }

    /// Returns a new [`Vector2`] vector that is a certain fraction of the way from `self`
    /// to `rhs`. The value `t` should be in the range of `0.0` to `1.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let start = v2!(10.0);
    /// let end = v2!(30.0);
    ///
    /// let mid = start.lerp(end, 0.5);
    /// assert_eq!(mid, v2!(20.0));
    /// ```
    #[inline]
    pub const fn lerp(self, rhs: Self, t: f32) -> Self {
        self.const_mul(1.0 - t).const_add(rhs.const_mul(t))
    }

    /// Returns a new [`Vector2`] vector rotated 90 degrees counter-clockwise.
    ///
    /// Swaps components and inverts the sign of the new X component: `(-y, x)`.
    /// Useful for normal calculation, 2D steering behaviors, or side-scroller movement.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    ///
    /// let forward = v2!(1.0, 0.0);
    /// let left_side = forward.perp();
    ///
    /// assert_eq!(left_side, v2!(0.0, 1.0));
    /// ```
    #[inline]
    pub const fn perp(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Returns a new [`Vector2`] vector rotated by the given angle counter-clockwise.
    ///
    /// This method assumes a standard 2D coordinate system where the positive Y-axis points up.
    /// Useful for character turning, projectile trajectories, or orbital camera movement.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    /// use std::f32::consts::FRAC_PI_2;
    ///
    /// let v = v2!(1.0, 0.0);
    /// let rotated = v.rotated(FRAC_PI_2);
    ///
    /// assert!((rotated.x - 0.0).abs() < 1e-5);
    /// assert!((rotated.y - 1.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn rotated(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(self.x * cos - self.y * sin, self.x * sin + self.y * cos)
    }

    /// Returns a new [`Vector2`] vector containing the maximum values of each component
    /// from two vectors.
    ///
    /// This method compares the components of `self` and `rhs` using [`f32::max()`].
    /// Useful for bounding box expansions,
    /// collision volume generation, or UI boundary constraints.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v1 = v2!(1.0, 5.0);
    /// let v2 = v2!(4.0, 2.0);
    /// let result = v1.max(v2);
    ///
    /// assert!((result.x - 4.0).abs() < 1e-5);
    /// assert!((result.y - 5.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub const fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y))
    }

    /// Returns a new [`Vector2`] vector with minimum components of `self` and `rhs`.
    ///
    /// This method compares the components of `self` and `rhs` using [`f32::min()`]
    /// Useful for bounding box intersections, screen viewport limits,
    /// or overlapping UI frame calculations.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v1 = v2!(1.0, 5.0);
    /// let v2 = v2!(4.0, 2.0);
    /// let result = v1.min(v2);
    ///
    /// assert!((result.x - 1.0).abs() < 1e-5);
    /// assert!((result.y - 2.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub const fn min(self, rhs: Self) -> Self {
        Self::new(self.x.min(rhs.x), self.y.min(rhs.y))
    }

    /// Returns a new [`Vector2`] vector with components clamped to the specified
    /// `min` and `max` bounds.
    ///
    /// Useful for restricting player movement, camera scroll limitations,
    /// or bounding UI sliders inside their tracks.
    ///
    /// # Panics
    ///
    /// Panics if `min.x > max.x` or `min.y > max.y`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let min_bound = v2!(0.0, 0.0);
    /// let max_bound = v2!(10.0, 10.0);
    /// let position = v2!(-3.0, 15.0);
    /// let result = position.clamp(min_bound, max_bound);
    ///
    /// assert!((result.x - 0.0).abs() < 1e-5);
    /// assert!((result.y - 10.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub const fn clamp(self, min: Self, max: Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    /// Returns a new [`Vector2`] vector containing the absolute values of each component.
    ///
    /// This method applies [`f32::abs()`] to both `x` and `y`. Useful for distance scaling,
    /// bounding box size calculations, or mirroring UI elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(-1.0, 2.5);
    /// let abs = v.abs();
    ///
    /// assert_eq!(abs, v2!(1.0, 2.5));
    /// ```
    #[inline]
    pub const fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    /// Returns a new [`Vector2`] vector containing the sign of each component.
    ///
    /// This method applies [`f32::signum()`] to both `x` and `y`. Useful for
    /// determining movement directions, grid step signs, or input axes orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(-5.0, 0.5);
    /// let sign = v.signum();
    ///
    /// assert_eq!(sign, v2!(-1.0, 1.0));
    /// ```
    #[inline]
    pub const fn signum(self) -> Self {
        Self::new(self.x.signum(), self.y.signum())
    }

    /// Returns a new [`Vector2`] vector with each component rounded down to the nearest integer.
    ///
    /// This method applies `f32::floor()` to both `x` and `y`. Useful for converting
    /// pixel coordinates to grid cells, tile map lookups, or snap-to-grid UI rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(1.7, -2.3);
    /// let floor = v.floor();
    ///
    /// assert_eq!(floor, v2!(1.0, -3.0));
    /// ```
    #[inline]
    pub const fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    /// Returns a new [`Vector2`] vector with each component rounded up to the nearest integer.
    ///
    /// This method applies `f32::ceil()` to both `x` and `y`. Useful for layout sizing bounds,
    /// texture padding calculations, or multi-page layout generation.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(1.2, -2.8);
    /// let ceil = v.ceil();
    ///
    /// assert_eq!(ceil, v2!(2.0, -2.0));
    /// ```
    #[inline]
    pub const fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    /// Returns a new [`Vector2`] vector with each component rounded to the nearest integer.
    ///
    /// This method applies `f32::round()` to both `x` and `y`. Useful for rounding
    /// subpixel coordinates to whole pixels to prevent blurry rendering in UI layouts.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(1.5, 2.4);
    /// let round = v.round();
    ///
    /// assert_eq!(round, v2!(2.0, 2.0));
    /// ```
    #[inline]
    pub const fn round(self) -> Self {
        Self::new(self.x.round(), self.y.round())
    }

    /// Returns a new [`Vector2`] vector containing the fractional part of each component.
    ///
    /// This method applies `f32::fract()` to both `x` and `y`. Useful for texture
    /// coordinate wrapping (UV repeating), animation looping, or subpixel rendering offsets.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::v2;
    ///
    /// let v = v2!(1.25, -2.75);
    /// let fract = v.fract();
    ///
    /// assert!((fract.x - 0.25).abs() < 1e-5);
    /// assert!((fract.y - (-0.75)).abs() < 1e-5);
    /// ```
    #[inline]
    pub const fn fract(self) -> Self {
        Self::new(self.x.fract(), self.y.fract())
    }

    /// Returns the sum of two [`Vector2`] vectors, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_add_assign()`]
    /// * [`Vector2::const_sub()`]
    #[inline(always)]
    pub const fn const_add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }

    /// Adds `rhs` to `self` in place, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_add()`]
    /// * [`Vector2::const_sub_assign()`]
    pub const fn const_add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }

    /// Returns the difference of two [`Vector2`] vectors, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_sub_assign()`]
    /// * [`Vector2::const_add()`]
    #[inline(always)]
    pub const fn const_sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }

    /// Subtracts `rhs` from `self` in place, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_sub()`]
    /// * [`Vector2::const_add_assign()`]
    #[inline(always)]
    pub const fn const_sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }

    /// Returns `self` scaled by `rhs`, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_mul_assign()`]
    /// * [`Vector2::const_div()`]
    #[inline(always)]
    pub const fn const_mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }

    /// Scales `self` by `rhs` in place, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_mul()`]
    /// * [`Vector2::const_div_assign()`]
    #[inline(always)]
    pub const fn const_mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }

    /// Returns `self` divided by `rhs`, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_div_assign()`]
    /// * [`Vector2::const_mul()`]
    #[inline(always)]
    pub const fn const_div(self, rhs: f32) -> Self {
        Self::new(self.x / rhs, self.y / rhs)
    }

    /// Divides `self` by `rhs` in place, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_div()`]
    /// * [`Vector2::const_mul_assign()`]
    #[inline(always)]
    pub const fn const_div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }

    /// Returns the remainder of `self` divided by `rhs`, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_rem_assign()`]
    /// * [`Vector2::const_div()`]
    #[inline(always)]
    pub const fn const_rem(self, rhs: f32) -> Self {
        Self::new(self.x % rhs, self.y % rhs)
    }

    /// Takes `self` modulo `rhs` in place, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Vector2::const_rem()`]
    /// * [`Vector2::const_div_assign()`]
    #[inline(always)]
    pub const fn const_rem_assign(&mut self, rhs: f32) {
        self.x %= rhs;
        self.y %= rhs;
    }
}

/// A macro for creating a [`Vector2`] vector with a more concise syntax.
///
/// This macro provides two forms:
/// - `v2!(x, y)`: Creates a new vector with the specified `x` and `y` components.
/// - `v2!(v)`: Creates a new vector where both `x` and `y` are set to the same value `v`.
///
/// Numeric literals are cast to `f32`, so `v2!(100)` and `v2!(3, 4)` are accepted.
/// Everything else must already be `f32`: `v2!(count)` where `count: i32` is a type
/// error, not a silent conversion.
///
/// # Examples
///
/// ```
/// use ancorix_math::Vector2;
/// use ancorix_math::v2;
///
/// let v1 = v2!(10.0, 5.0);
/// let v2 = Vector2::new(10.0, 5.0);
/// assert_eq!(v1, v2);
///
/// let v1 = v2!(10.0);
/// let v2 = Vector2::splat(10.0);
/// assert_eq!(v1, v2);
/// ```
///
/// Integer literals need no decimal point:
///
/// ```
/// use ancorix_math::v2;
///
/// assert_eq!(v2!(100), v2!(100.0, 100.0));
/// assert_eq!(v2!(3, -4), v2!(3.0, -4.0));
/// ```
///
/// # See also
///
/// * [`Vector2::new()`]
/// * [`Vector2::splat()`]
#[macro_export]
macro_rules! v2 {
    // Components are captured as `tt` rather than `literal`, and the sign is
    // spelled out in the patterns, because a `literal` matcher that sees `-`
    // commits to parsing a negative literal and hard-errors on the next token:
    // `v2!(-half, y)` would not fall through to the `expr` arm, it would fail
    // to compile. A single `tt` can never carry a leading `-`, so handing one
    // to `__v2_f32!` is safe.

    // v2!(x, y)
    ($x:tt, $y:tt $(,)?) => {
        $crate::vector2::Vector2::new($crate::__v2_f32!($x), $crate::__v2_f32!($y))
    };
    (- $x:tt, $y:tt $(,)?) => {
        $crate::vector2::Vector2::new(-$crate::__v2_f32!($x), $crate::__v2_f32!($y))
    };
    ($x:tt, - $y:tt $(,)?) => {
        $crate::vector2::Vector2::new($crate::__v2_f32!($x), -$crate::__v2_f32!($y))
    };
    (- $x:tt, - $y:tt $(,)?) => {
        $crate::vector2::Vector2::new(-$crate::__v2_f32!($x), -$crate::__v2_f32!($y))
    };
    ($x:expr, $y:expr $(,)?) => {
        $crate::vector2::Vector2::new($x, $y)
    };

    // v2!(v)
    (- $v:tt $(,)?) => {
        $crate::vector2::Vector2::splat(-$crate::__v2_f32!($v))
    };
    ($v:tt $(,)?) => {
        $crate::vector2::Vector2::splat($crate::__v2_f32!($v))
    };
    ($v:expr $(,)?) => {
        $crate::vector2::Vector2::splat($v)
    };
}

// Casts one component to f32 if - and only if - it is a literal. An `as` on an
// arbitrary expression would silently truncate an i32 or narrow an f64 instead
// of failing, so everything else is passed through and has to be f32 already.
//
// Exported because `v2!` is: its expansion says `$crate::__v2_f32!`, which has
// to resolve in whatever crate wrote `v2!(...)`, not just in this one.
#[doc(hidden)]
#[macro_export]
macro_rules! __v2_f32 {
    ($v:literal) => {
        $v as f32
    };
    ($v:expr) => {
        $v
    };
}

macro_rules! impl_ops {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident) => {
        // 1. v2 op v2
        impl $trait<Vector2> for Vector2 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: Vector2) -> Self::Output {
                Vector2::new(self.x.$method(rhs.x), self.y.$method(rhs.y))
            }
        }

        // &v2 op v2
        impl<'a> $trait<Vector2> for &'a Vector2 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: Vector2) -> Self::Output {
                $trait::$method(*self, rhs)
            }
        }

        // v2 op &v2
        impl<'a> $trait<&'a Vector2> for Vector2 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: &'a Vector2) -> Self::Output {
                $trait::$method(self, *rhs)
            }
        }

        // &v2 op &v2
        impl<'a, 'b> $trait<&'b Vector2> for &'a Vector2 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: &'b Vector2) -> Self::Output {
                $trait::$method(*self, *rhs)
            }
        }

        // 2. v2 op number
        impl $trait<f32> for Vector2 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: f32) -> Self::Output {
                Vector2::new(self.x.$method(rhs), self.y.$method(rhs))
            }
        }

        // &v2 op number
        impl<'a> $trait<f32> for &'a Vector2 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: f32) -> Self::Output {
                $trait::$method(*self, rhs)
            }
        }

        // 3. number op v2
        impl $trait<Vector2> for f32 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: Vector2) -> Self::Output {
                Vector2::new(self.$method(rhs.x), self.$method(rhs.y))
            }
        }

        // number op &v2
        impl<'a> $trait<&'a Vector2> for f32 {
            type Output = Vector2;
            #[inline(always)]
            fn $method(self, rhs: &'a Vector2) -> Self::Output {
                $trait::$method(self, *rhs)
            }
        }

        // 4. v2 op= v2
        impl $assign_trait<Vector2> for Vector2 {
            #[inline(always)]
            fn $assign_method(&mut self, rhs: Vector2) {
                self.x.$assign_method(rhs.x);
                self.y.$assign_method(rhs.y);
            }
        }

        // v2 op= &v2
        impl<'a> $assign_trait<&'a Vector2> for Vector2 {
            #[inline(always)]
            fn $assign_method(&mut self, rhs: &'a Vector2) {
                self.x.$assign_method(rhs.x);
                self.y.$assign_method(rhs.y);
            }
        }

        // 5. v2 op= number
        impl $assign_trait<f32> for Vector2 {
            #[inline(always)]
            fn $assign_method(&mut self, rhs: f32) {
                self.x.$assign_method(rhs);
                self.y.$assign_method(rhs);
            }
        }
    };
}

// -v2
impl Neg for Vector2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl Index<usize> for Vector2 {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        const _: () = assert!(std::mem::size_of::<Vector2>() == std::mem::size_of::<[f32; 2]>());

        let arr: &[f32; 2] = unsafe { std::mem::transmute(self) };

        &arr[index]
    }
}

impl IndexMut<usize> for Vector2 {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => {
                debug_assert!(index < 2, "Index out of bounds: {}", index);
                unsafe { std::hint::unreachable_unchecked() }
            }
        }
    }
}

// v2.as_ref().iter()
impl AsRef<[f32; 2]> for Vector2 {
    #[inline]
    fn as_ref(&self) -> &[f32; 2] {
        unsafe { &*(self as *const Self as *const [f32; 2]) }
    }
}

// v2.as_mut().iter_mut()
impl AsMut<[f32; 2]> for Vector2 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 2] {
        unsafe { &mut *(self as *mut Self as *mut [f32; 2]) }
    }
}

impl From<[f32; 2]> for Vector2 {
    #[inline]
    fn from(array: [f32; 2]) -> Self {
        Self::new(array[0], array[1])
    }
}

impl From<Vector2> for [f32; 2] {
    #[inline]
    fn from(vector: Vector2) -> Self {
        [vector.x, vector.y]
    }
}

impl From<(f32, f32)> for Vector2 {
    #[inline]
    fn from(tuple: (f32, f32)) -> Self {
        Self::new(tuple.0, tuple.1)
    }
}

impl From<Vector2> for (f32, f32) {
    #[inline]
    fn from(vector: Vector2) -> Self {
        (vector.x, vector.y)
    }
}

impl Display for Vector2 {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector2({:.5}, {:.5})", self.x, self.y)
    }
}

// v2 + v2, v2 + num, num + v2, v2 += v2, v2 += num, num += v2
impl_ops!(Add, add, AddAssign, add_assign);
// v2 - v2, v2 - num, num - v2, v2 -= v2, v2 -= num, num -= v2
impl_ops!(Sub, sub, SubAssign, sub_assign);
// v2 * v2, v2 * num, num * v2, v2 *= v2, v2 *= num, num *= v2
impl_ops!(Mul, mul, MulAssign, mul_assign);
// v2 / v2, v2 / num, num / v2, v2 /= v2, v2 /= num, num /= v2
impl_ops!(Div, div, DivAssign, div_assign);
// v2 % v2, v2 % num, num % v2, v2 %= v2, v2 %= num, num %= v2
impl_ops!(Rem, rem, RemAssign, rem_assign);
