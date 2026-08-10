//! A 2x2 matrix, stored in column-major order.
//!
//! [`Matrix2`] is a linear transformation of the plane - a rotation, a scale,
//! a shear, or any combination of them. It has no translation: a 2x2 matrix
//! cannot move the origin. Translation belongs to whatever holds the matrix,
//! as a [`Vector2`] offset applied after it.
//!
//! # Layout
//!
//! A [`Matrix2`] is two columns, `x_axis` and `y_axis` - the images of the two
//! basis vectors, and the same layout Vulkan expects. [`Matrix2::new()`] still
//! takes its four scalars in the order you would write them on paper:
//!
//! ```text
//! | m00  m01 |
//! | m10  m11 |
//! ```
//!
//! # Vectors are columns
//!
//! Transforming a vector is `matrix * vector`, and composing two
//! transformations is `a * b`, which applies `b` first and then `a`.

use crate::EPSILON;
use crate::vector2::Vector2;
use std::fmt::Display;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[doc(alias = "mat2")]
#[doc(alias = "m2")]
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(C)]
#[must_use]
pub struct Matrix2 {
    /// The first column, i.e. where the basis vector `(1.0, 0.0)` lands.
    pub x_axis: Vector2,
    /// The second column, i.e. where the basis vector `(0.0, 1.0)` lands.
    pub y_axis: Vector2,
}

impl Default for Matrix2 {
    /// Returns the [`Matrix2::IDENTITY`] matrix.
    #[inline(always)]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Matrix2 {
    /// A [`Matrix2`] with every entry set to `0.0`.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// The identity matrix, which leaves every vector unchanged.
    pub const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 1.0);

    /// Returns a new [`Matrix2`] from four scalars, in the order they are written
    /// on paper.
    ///
    /// ```text
    /// | m00  m01 |
    /// | m10  m11 |
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let m = Matrix2::new(1.0, 2.0,
    ///                      3.0, 4.0);
    ///
    /// assert_eq!(m.x_axis, v2!(1.0, 3.0));
    /// assert_eq!(m.y_axis, v2!(2.0, 4.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::from_cols()`]
    /// * [`Matrix2::from_rows()`]
    #[inline(always)]
    pub const fn new(m00: f32, m01: f32, m10: f32, m11: f32) -> Self {
        Self {
            x_axis: Vector2::new(m00, m10),
            y_axis: Vector2::new(m01, m11),
        }
    }

    /// Returns a new [`Matrix2`] from its two columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let m = Matrix2::from_cols(v2!(1.0, 3.0), v2!(2.0, 4.0));
    ///
    /// assert_eq!(m, Matrix2::new(1.0, 2.0, 3.0, 4.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::from_rows()`]
    #[inline(always)]
    pub const fn from_cols(x_axis: Vector2, y_axis: Vector2) -> Self {
        Self { x_axis, y_axis }
    }

    /// Returns a new [`Matrix2`] from its two rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let m = Matrix2::from_rows(v2!(1.0, 2.0), v2!(3.0, 4.0));
    ///
    /// assert_eq!(m, Matrix2::new(1.0, 2.0, 3.0, 4.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::from_cols()`]
    #[inline(always)]
    pub const fn from_rows(row0: Vector2, row1: Vector2) -> Self {
        Self::new(row0.x, row0.y, row1.x, row1.y)
    }

    /// Returns a [`Matrix2`] that scales `x` and `y` independently.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let m = Matrix2::from_scale(v2!(2.0, 3.0));
    ///
    /// assert_eq!(m * v2!(1.0, 1.0), v2!(2.0, 3.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::from_angle()`]
    #[inline(always)]
    pub const fn from_scale(scale: Vector2) -> Self {
        Self::new(scale.x, 0.0, 0.0, scale.y)
    }

    /// Returns a [`Matrix2`] that rotates by `angle` radians, counter-clockwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let quarter_turn = Matrix2::from_angle(std::f32::consts::FRAC_PI_2);
    /// let turned = quarter_turn * v2!(1.0, 0.0);
    ///
    /// assert!((turned - v2!(0.0, 1.0)).length() < 1e-6);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::to_angle()`]
    /// * [`Matrix2::from_scale()`]
    #[inline]
    pub fn from_angle(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(cos, -sin, sin, cos)
    }

    /// Returns the row at `index`, or [`None`] if `index` is not `0` or `1`.
    ///
    /// Columns need no accessor - they are the [`x_axis`](Matrix2::x_axis) and
    /// [`y_axis`](Matrix2::y_axis) fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let m = Matrix2::new(1.0, 2.0, 3.0, 4.0);
    ///
    /// assert_eq!(m.row(0), Some(v2!(1.0, 2.0)));
    /// assert_eq!(m.row(1), Some(v2!(3.0, 4.0)));
    /// assert_eq!(m.row(2), None);
    /// ```
    #[inline]
    pub const fn row(self, index: usize) -> Option<Vector2> {
        match index {
            0 => Some(Vector2::new(self.x_axis.x, self.y_axis.x)),
            1 => Some(Vector2::new(self.x_axis.y, self.y_axis.y)),
            _ => None,
        }
    }

    /// Returns the determinant, i.e. the factor by which the matrix scales area.
    ///
    /// A negative determinant means the transformation flips orientation, and a
    /// determinant of `0.0` means it collapses the plane onto a line, which cannot
    /// be undone.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// assert_eq!(Matrix2::IDENTITY.determinant(), 1.0);
    /// assert_eq!(Matrix2::from_scale(v2!(2.0, 3.0)).determinant(), 6.0);
    /// assert_eq!(Matrix2::ZERO.determinant(), 0.0);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::try_inverse()`]
    #[inline]
    pub const fn determinant(self) -> f32 {
        self.x_axis.x * self.y_axis.y - self.y_axis.x * self.x_axis.y
    }

    /// Returns the matrix with its rows and columns swapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::Matrix2;
    ///
    /// let m = Matrix2::new(1.0, 2.0, 3.0, 4.0);
    ///
    /// assert_eq!(m.transpose(), Matrix2::new(1.0, 3.0, 2.0, 4.0));
    /// ```
    #[inline]
    pub const fn transpose(self) -> Self {
        Self::from_cols(
            Vector2::new(self.x_axis.x, self.y_axis.x),
            Vector2::new(self.x_axis.y, self.y_axis.y),
        )
    }

    /// Returns the matrix that undoes this one, or [`None`] if there is none.
    ///
    /// A transformation that collapses the plane onto a line has no inverse; that
    /// is the case whose determinant is `0.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let scale = Matrix2::from_scale(v2!(2.0, 4.0));
    /// let undo = scale.try_inverse().unwrap();
    ///
    /// assert_eq!(undo * (scale * v2!(3.0, 5.0)), v2!(3.0, 5.0));
    /// assert_eq!(Matrix2::ZERO.try_inverse(), None);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::inverse()`]
    #[inline]
    pub const fn try_inverse(self) -> Option<Self> {
        let det = self.determinant();

        // Not `det.abs() < EPSILON`: `f32::abs` is not const on stable.
        if det > -EPSILON && det < EPSILON {
            return None;
        }

        let inv = 1.0 / det;

        Some(Self::from_cols(
            Vector2::new(self.y_axis.y * inv, -self.x_axis.y * inv),
            Vector2::new(-self.y_axis.x * inv, self.x_axis.x * inv),
        ))
    }

    /// Returns the matrix that undoes this one.
    ///
    /// # Panics
    ///
    /// Panics if the matrix is not invertible, i.e. if its determinant is `0.0`.
    /// Use [`try_inverse()`](Matrix2::try_inverse) where that can happen.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let turn = Matrix2::from_angle(1.0);
    ///
    /// assert!((turn.inverse() * (turn * v2!(1.0, 0.0)) - v2!(1.0, 0.0)).length() < 1e-6);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::try_inverse()`]
    #[inline]
    pub const fn inverse(self) -> Self {
        match self.try_inverse() {
            Some(m) => m,
            None => panic!("Matrix2 is not invertible"),
        }
    }

    /// Returns `rhs` transformed by this matrix.
    ///
    /// This is what the `*` operator does; the method exists because operators
    /// cannot be `const`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// const M: Matrix2 = Matrix2::from_scale(v2!(2.0, 3.0));
    /// const V: ancorix_math::Vector2 = M.mul_vec2(v2!(1.0, 1.0));
    ///
    /// assert_eq!(V, v2!(2.0, 3.0));
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::mul_mat2()`]
    #[inline]
    pub const fn mul_vec2(self, rhs: Vector2) -> Vector2 {
        Vector2::new(
            self.x_axis.x * rhs.x + self.y_axis.x * rhs.y,
            self.x_axis.y * rhs.x + self.y_axis.y * rhs.y,
        )
    }

    /// Returns the composition of two matrices, applying `rhs` first.
    ///
    /// This is what the `*` operator does; the method exists because operators
    /// cannot be `const`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::{Matrix2, v2};
    ///
    /// let scale = Matrix2::from_scale(v2!(2.0, 2.0));
    /// let turn = Matrix2::from_angle(std::f32::consts::FRAC_PI_2);
    ///
    /// // Scale first, then turn.
    /// let both = turn.mul_mat2(scale);
    ///
    /// assert!((both * v2!(1.0, 0.0) - v2!(0.0, 2.0)).length() < 1e-6);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::mul_vec2()`]
    #[inline]
    pub const fn mul_mat2(self, rhs: Self) -> Self {
        Self::from_cols(self.mul_vec2(rhs.x_axis), self.mul_vec2(rhs.y_axis))
    }

    /// Returns the rotation of the matrix in radians.
    ///
    /// Only meaningful for a matrix that is a rotation, optionally with a uniform
    /// scale; a shear has no single angle to report.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::Matrix2;
    ///
    /// let m = Matrix2::from_angle(0.75);
    ///
    /// assert!((m.to_angle() - 0.75).abs() < 1e-6);
    /// ```
    ///
    /// # See also
    ///
    /// * [`Matrix2::from_angle()`]
    #[inline]
    pub fn to_angle(self) -> f32 {
        self.x_axis.y.atan2(self.x_axis.x)
    }

    /// Returns the sum of two matrices, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Matrix2::const_sub()`]
    #[inline]
    pub const fn const_add(self, rhs: Self) -> Self {
        Self::from_cols(
            self.x_axis.const_add(rhs.x_axis),
            self.y_axis.const_add(rhs.y_axis),
        )
    }

    /// Returns the difference of two matrices, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Matrix2::const_add()`]
    #[inline]
    pub const fn const_sub(self, rhs: Self) -> Self {
        Self::from_cols(
            self.x_axis.const_sub(rhs.x_axis),
            self.y_axis.const_sub(rhs.y_axis),
        )
    }

    /// Returns the matrix scaled by `rhs`, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Matrix2::const_div()`]
    #[inline]
    pub const fn const_mul(self, rhs: f32) -> Self {
        Self::from_cols(self.x_axis.const_mul(rhs), self.y_axis.const_mul(rhs))
    }

    /// Returns the matrix divided by `rhs`, evaluable in `const` contexts.
    ///
    /// # See also
    ///
    /// * [`Matrix2::const_mul()`]
    #[inline]
    pub const fn const_div(self, rhs: f32) -> Self {
        Self::from_cols(self.x_axis.const_div(rhs), self.y_axis.const_div(rhs))
    }
}

impl Add for Matrix2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.const_add(rhs)
    }
}

impl AddAssign for Matrix2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = self.const_add(rhs);
    }
}

impl Sub for Matrix2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.const_sub(rhs)
    }
}

impl SubAssign for Matrix2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.const_sub(rhs);
    }
}

impl Mul<f32> for Matrix2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        self.const_mul(rhs)
    }
}

impl Mul<Matrix2> for f32 {
    type Output = Matrix2;

    #[inline]
    fn mul(self, rhs: Matrix2) -> Self::Output {
        rhs.const_mul(self)
    }
}

impl MulAssign<f32> for Matrix2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = self.const_mul(rhs);
    }
}

impl Mul<Vector2> for Matrix2 {
    type Output = Vector2;

    #[inline]
    fn mul(self, rhs: Vector2) -> Self::Output {
        self.mul_vec2(rhs)
    }
}

impl Mul for Matrix2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_mat2(rhs)
    }
}

impl MulAssign for Matrix2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul_mat2(rhs);
    }
}

impl Div<f32> for Matrix2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        self.const_div(rhs)
    }
}

impl DivAssign<f32> for Matrix2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        *self = self.const_div(rhs);
    }
}

impl Neg for Matrix2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.const_mul(-1.0)
    }
}

impl From<[[f32; 2]; 2]> for Matrix2 {
    /// Builds a [`Matrix2`] from two columns.
    #[inline]
    fn from(cols: [[f32; 2]; 2]) -> Self {
        Self::from_cols(Vector2::from(cols[0]), Vector2::from(cols[1]))
    }
}

impl From<Matrix2> for [[f32; 2]; 2] {
    /// Splits a [`Matrix2`] into its two columns.
    #[inline]
    fn from(m: Matrix2) -> Self {
        [m.x_axis.into(), m.y_axis.into()]
    }
}

impl Display for Matrix2 {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Matrix2({:.5}, {:.5}, {:.5}, {:.5})",
            self.x_axis.x, self.y_axis.x, self.x_axis.y, self.y_axis.y
        )
    }
}
