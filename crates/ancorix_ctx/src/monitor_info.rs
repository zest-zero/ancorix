use ancorix_math::Vector2;

/// The monitor a window is currently displayed on.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MonitorInfo {
    /// DPI scale factor reported by the OS/compositor (`1.0` = no scaling).
    pub scale_factor: f32,
    /// The monitor's own resolution, in physical pixels - not the
    /// window's size.
    pub physical_size: Vector2,
}

impl MonitorInfo {
    /// Returns a new [`MonitorInfo`] with the given `scale_factor` and
    /// `physical_size`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::MonitorInfo;
    /// use ancorix_math::v2;
    ///
    /// let monitor = MonitorInfo::new(1.25, v2!(1920.0, 1080.0));
    /// assert_eq!(monitor.scale_factor, 1.25);
    /// ```
    #[inline]
    pub const fn new(scale_factor: f32, physical_size: Vector2) -> Self {
        Self {
            scale_factor,
            physical_size,
        }
    }
}
