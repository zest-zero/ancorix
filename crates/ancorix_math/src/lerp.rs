/// Types that can be linearly interpolated between two values.
#[doc(alias = "mix")]
#[doc(alias = "interpolate")]
#[doc(alias = "interpolation")]
pub trait Lerp {
    /// Returns a value that is a fraction `t` of the way from `self` to
    /// `rhs`. `t` should be in the range `0.0` to `1.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_math::Lerp;
    ///
    /// assert_eq!(10.0_f32.lerp(30.0, 0.5), 20.0);
    /// ```
    fn lerp(self, rhs: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    #[inline]
    fn lerp(self, rhs: Self, t: f32) -> Self {
        self + (rhs - self) * t
    }
}

impl Lerp for crate::Vector2 {
    #[inline]
    fn lerp(self, rhs: Self, t: f32) -> Self {
        Self::lerp(self, rhs, t)
    }
}
