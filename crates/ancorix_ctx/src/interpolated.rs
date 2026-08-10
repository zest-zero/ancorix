use ancorix_math::Lerp;

/// A value with its two most recent fixed-tick states, for smooth
/// rendering between physics steps that run slower than the display's
/// refresh rate.
#[derive(Debug, Clone, Copy, PartialEq)]
#[doc(alias = "smooth")]
#[doc(alias = "smoothing")]
pub struct Interpolated<T> {
    prev: T,
    curr: T,
}

impl<T: Copy> Interpolated<T> {
    /// Returns a new [`Interpolated`] with both states set to `value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Interpolated;
    ///
    /// let value = Interpolated::new(5.0);
    /// assert_eq!(value.get(), 5.0);
    /// ```
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            prev: value,
            curr: value,
        }
    }

    /// Snapshots the current state into the previous one. Call once at the
    /// start of each fixed tick, before mutating the value through
    /// [`Interpolated::get_mut`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Interpolated;
    ///
    /// let mut value = Interpolated::new(5.0);
    /// value.begin_tick();
    /// *value.get_mut() = 8.0;
    ///
    /// assert_eq!(value.sample(0.0), 5.0);
    /// assert_eq!(value.sample(1.0), 8.0);
    /// ```
    #[inline]
    pub fn begin_tick(&mut self) {
        self.prev = self.curr;
    }

    /// Returns the current state.
    ///
    /// # See also
    ///
    /// [`Interpolated::get_mut`]
    #[inline]
    pub const fn get(&self) -> T {
        self.curr
    }

    /// Returns a mutable reference to the current state, to update in
    /// place during a fixed tick.
    ///
    /// # See also
    ///
    /// [`Interpolated::get`], [`Interpolated::begin_tick`]
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.curr
    }
}

impl<T: Lerp + Copy> Interpolated<T> {
    /// Returns the value interpolated `alpha` of the way from the previous
    /// tick's state to the current one - pass [`crate::Time::alpha`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Interpolated;
    ///
    /// let mut value = Interpolated::new(10.0);
    /// value.begin_tick();
    /// *value.get_mut() = 30.0;
    ///
    /// assert_eq!(value.sample(0.5), 20.0);
    /// ```
    #[inline]
    pub fn sample(&self, alpha: f32) -> T {
        self.prev.lerp(self.curr, alpha)
    }
}
