/// A frame timer over a contiguous range of [`crate::SpriteSheet`] frames.
///
/// Holds only the playback clock, never the sheet or its texture - one
/// animation drives any sheet whose frame numbering it matches, and it is
/// advanced by the caller's own `dt` (a fixed-timestep tick's `dt` just as
/// well as the frame's).
///
/// # Examples
///
/// ```
/// use ancorix_draw::SpriteAnimation;
///
/// // frames 0..4 at 10 frames per second, looping
/// let mut anim = SpriteAnimation::new(0, 4, 10.0);
/// assert_eq!(anim.frame(), 0);
///
/// anim.advance(0.25); // 2.5 frames in
/// assert_eq!(anim.frame(), 2);
///
/// anim.advance(0.30); // 5.5 frames in, wrapped back around
/// assert_eq!(anim.frame(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpriteAnimation {
    first: u32,
    count: u32,
    frame_time: f32,
    looping: bool,
    elapsed: f32,
}

impl SpriteAnimation {
    /// Returns a new looping [`SpriteAnimation`] over `count` frames
    /// starting at frame `first`, playing at `fps` frames per second.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteAnimation;
    ///
    /// let mut anim = SpriteAnimation::new(4, 4, 12.0);
    ///
    /// anim.advance(10.0); // a loop never ends, however long it runs
    /// assert!(!anim.finished());
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `count` is zero or `fps` isn't positive.
    ///
    /// # See also
    ///
    /// * [`SpriteAnimation::once()`]
    #[inline]
    pub const fn new(first: u32, count: u32, fps: f32) -> Self {
        Self::with_looping(first, count, fps, true)
    }

    /// Returns a new [`SpriteAnimation`] that plays `count` frames from
    /// `first` once at `fps` frames per second, then holds on the last one.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteAnimation;
    ///
    /// let mut anim = SpriteAnimation::once(0, 3, 10.0);
    ///
    /// anim.advance(10.0);
    /// assert!(anim.finished());
    /// assert_eq!(anim.frame(), 2); // holds the last frame
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `count` is zero or `fps` isn't positive.
    ///
    /// # See also
    ///
    /// * [`SpriteAnimation::new()`]
    #[inline]
    pub const fn once(first: u32, count: u32, fps: f32) -> Self {
        Self::with_looping(first, count, fps, false)
    }

    /// Advances playback by `dt` seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteAnimation;
    ///
    /// let mut anim = SpriteAnimation::new(0, 4, 10.0);
    /// anim.advance(0.1);
    ///
    /// assert_eq!(anim.frame(), 1);
    /// ```
    ///
    /// # See also
    ///
    /// * [`SpriteAnimation::reset()`]
    pub fn advance(&mut self, dt: f32) {
        let duration = self.frame_time * self.count as f32;
        self.elapsed += dt;

        if self.looping {
            // `%` rather than a subtract loop - a large `dt` (a stalled
            // frame, a debugger pause) can overshoot by many periods.
            if self.elapsed >= duration {
                self.elapsed %= duration;
            }
        } else if self.elapsed > duration {
            self.elapsed = duration;
        }
    }

    /// Returns the sheet frame index to draw right now.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteAnimation;
    ///
    /// let mut anim = SpriteAnimation::new(8, 4, 10.0);
    /// anim.advance(0.2);
    ///
    /// assert_eq!(anim.frame(), 10); // third frame of the range starting at 8
    /// ```
    #[inline]
    pub fn frame(self) -> u32 {
        let step = (self.elapsed / self.frame_time) as u32;

        self.first + step.min(self.count - 1)
    }

    /// Returns `true` once a non-looping animation has played its last
    /// frame - always `false` for a looping one.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteAnimation;
    ///
    /// let mut anim = SpriteAnimation::once(0, 2, 10.0);
    /// assert!(!anim.finished());
    ///
    /// anim.advance(0.2);
    /// assert!(anim.finished());
    /// ```
    #[inline]
    pub fn finished(self) -> bool {
        !self.looping && self.elapsed >= self.frame_time * self.count as f32
    }

    /// Restarts playback from the first frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteAnimation;
    ///
    /// let mut anim = SpriteAnimation::once(0, 2, 10.0);
    /// anim.advance(0.2);
    /// anim.reset();
    ///
    /// assert!(!anim.finished());
    /// assert_eq!(anim.frame(), 0);
    /// ```
    ///
    /// # See also
    ///
    /// * [`SpriteAnimation::advance()`]
    #[inline]
    pub const fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    const fn with_looping(first: u32, count: u32, fps: f32, looping: bool) -> Self {
        assert!(count > 0, "an animation needs at least one frame");
        assert!(fps > 0.0, "an animation needs a positive frame rate");

        Self {
            first,
            count,
            frame_time: 1.0 / fps,
            looping,
            elapsed: 0.0,
        }
    }
}
