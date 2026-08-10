/// Per-frame timing information.
///
/// `Time` is a plain `Copy` value rather than a borrowed reference, because
/// it's just a handful of `f32`s - copying it is cheaper than threading a
/// lifetime through every place that reads the clock.
#[derive(Debug, Copy, Clone, PartialEq)]
#[doc(alias = "clock")]
#[doc(alias = "delta")]
#[doc(alias = "fps")]
#[doc(alias = "elapsed")]
pub struct Time {
    dt: f32,
    elapsed: f32,
    accumulator: f32,
}

/// `dt` values above this are clamped before accumulating for
/// [`Time::fixed_tick`], so a long stall (window drag, debugger breakpoint)
/// doesn't queue up a burst of catch-up fixed ticks next frame.
const MAX_ACCUMULATED_DT: f32 = 0.25;

impl Time {
    /// Returns a new [`Time`] with `dt` and `elapsed` both zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let time = Time::new();
    /// assert_eq!(time.dt(), 0.0);
    /// assert_eq!(time.elapsed(), 0.0);
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            dt: 0.0,
            elapsed: 0.0,
            accumulator: 0.0,
        }
    }

    /// Returns the number of seconds elapsed since the previous frame.
    ///
    /// Multiplying movement or animation speed by `dt` keeps behavior
    /// consistent regardless of frame rate - the same logic produces the
    /// same result whether the app runs at 30fps or 240fps.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let mut time = Time::new();
    /// time.advance(0.016);
    ///
    /// assert_eq!(time.dt(), 0.016);
    /// ```
    #[inline]
    pub const fn dt(self) -> f32 {
        self.dt
    }

    /// Returns the instantaneous frame rate (`1.0 / dt`), or `0.0` if `dt`
    /// is zero (e.g. on the very first frame).
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let mut time = Time::new();
    /// time.advance(1.0 / 60.0);
    ///
    /// assert!((time.fps() - 60.0).abs() < 1e-3);
    /// ```
    #[inline]
    pub fn fps(self) -> f32 {
        if self.dt > 0.0 { 1.0 / self.dt } else { 0.0 }
    }

    /// Returns the number of seconds elapsed since the application started.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let mut time = Time::new();
    /// time.advance(0.5);
    /// time.advance(0.5);
    ///
    /// assert_eq!(time.elapsed(), 1.0);
    /// ```
    #[inline]
    pub const fn elapsed(self) -> f32 {
        self.elapsed
    }

    /// Advances the clock by `dt` seconds. Called once per frame by the
    /// window backend, before the frame's `dt` is read by user code.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let mut time = Time::new();
    /// time.advance(0.016);
    /// time.advance(0.016);
    ///
    /// assert!((time.elapsed() - 0.032).abs() < 1e-6);
    /// ```
    #[inline]
    pub fn advance(&mut self, dt: f32) {
        self.dt = dt;
        self.elapsed += dt;
        self.accumulator += dt.min(MAX_ACCUMULATED_DT);
    }

    /// Drains one `1.0 / hz` seconds worth of accumulated time and returns
    /// it, or returns `None` if less than that has accumulated since the
    /// last drain.
    ///
    /// Use it in a loop to run fixed-timestep logic at a fixed rate.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let mut time = Time::new();
    /// time.advance(1.0 / 30.0);
    ///
    /// assert_eq!(time.fixed_tick(60.0), Some(1.0 / 60.0));
    /// assert_eq!(time.fixed_tick(60.0), Some(1.0 / 60.0));
    /// assert_eq!(time.fixed_tick(60.0), None);
    /// ```
    ///
    /// # See also
    ///
    /// [`Time::fixed_tick_raw`]
    #[inline]
    pub fn fixed_tick(&mut self, hz: f32) -> Option<f32> {
        let fixed_dt = 1.0 / hz;
        self.fixed_tick_raw(fixed_dt).then_some(fixed_dt)
    }

    /// Drains `fixed_dt` seconds of accumulated time and returns `true`,
    /// or returns `false` if less than that has accumulated since the
    /// last drain.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let mut time = Time::new();
    /// time.advance(1.0 / 30.0);
    ///
    /// let step = 1.0 / 60.0;
    /// assert!(time.fixed_tick_raw(step));
    /// assert!(time.fixed_tick_raw(step));
    /// assert!(!time.fixed_tick_raw(step));
    /// ```
    ///
    /// # See also
    ///
    /// [`Time::fixed_tick`]
    #[inline]
    pub fn fixed_tick_raw(&mut self, fixed_dt: f32) -> bool {
        if self.accumulator >= fixed_dt {
            self.accumulator -= fixed_dt;
            true
        } else {
            false
        }
    }

    /// Returns how far between the last completed fixed tick and the next
    /// one the accumulator is, as a value in `[0.0, 1.0]`.
    ///
    /// Call after fully draining [`Time::fixed_tick`] for this frame, and
    /// use the result to interpolate render state between the previous
    /// and current fixed-step simulation state.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::Time;
    ///
    /// let mut time = Time::new();
    /// time.advance(1.5 / 60.0);
    /// while time.fixed_tick(60.0).is_some() {}
    ///
    /// assert!((time.alpha(60.0) - 0.5).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn alpha(&self, hz: f32) -> f32 {
        (self.accumulator * hz).min(1.0)
    }
}

impl Default for Time {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_tick_drains_exact_multiples() {
        let mut time = Time::new();
        time.advance(3.0 / 60.0 + 1e-6);

        assert!(time.fixed_tick(60.0).is_some());
        assert!(time.fixed_tick(60.0).is_some());
        assert!(time.fixed_tick(60.0).is_some());
        assert!(time.fixed_tick(60.0).is_none());
    }

    #[test]
    fn fixed_tick_returns_the_step_it_drained() {
        let mut time = Time::new();
        time.advance(1.0 / 60.0);

        assert_eq!(time.fixed_tick(60.0), Some(1.0 / 60.0));
        assert_eq!(time.fixed_tick(60.0), None);
    }

    #[test]
    fn leftover_accumulator_carries_into_the_next_frame() {
        let mut time = Time::new();
        time.advance(1.5 / 60.0);

        assert!(time.fixed_tick(60.0).is_some());
        assert!(time.fixed_tick(60.0).is_none());

        time.advance(0.5 / 60.0 + 1e-6);
        assert!(time.fixed_tick(60.0).is_some());
        assert!(time.fixed_tick(60.0).is_none());
    }

    #[test]
    fn a_long_stall_does_not_queue_a_burst_of_ticks() {
        let mut time = Time::new();
        time.advance(10.0);

        let mut ticks = 0;
        while time.fixed_tick(60.0).is_some() {
            ticks += 1;
        }

        assert!(ticks <= (MAX_ACCUMULATED_DT * 60.0).ceil() as u32);
    }

    #[test]
    fn fixed_tick_raw_takes_the_step_directly() {
        let mut time = Time::new();
        time.advance(2.0 / 60.0 + 1e-6);

        let step = 1.0 / 60.0;
        assert!(time.fixed_tick_raw(step));
        assert!(time.fixed_tick_raw(step));
        assert!(!time.fixed_tick_raw(step));
    }

    #[test]
    fn alpha_reflects_progress_toward_the_next_tick() {
        let mut time = Time::new();
        time.advance(1.25 / 60.0);
        while time.fixed_tick(60.0).is_some() {}

        assert!((time.alpha(60.0) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn alpha_is_clamped_if_never_drained() {
        let mut time = Time::new();
        time.advance(10.0); // never called fixed_tick to drain it

        assert_eq!(time.alpha(60.0), 1.0);
    }
}
