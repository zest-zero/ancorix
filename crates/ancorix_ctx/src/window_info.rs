use crate::{Cursor, MonitorInfo};
use ancorix_math::Vector2;

/// Per-frame window state and control.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WindowInfo {
    width: u32,
    height: u32,
    resized: bool,
    monitor: MonitorInfo,
    exit_requested: bool,
    exit_code: u8,
    cursor_visible: bool,
    cursor: Cursor,
}

impl WindowInfo {
    /// Returns a new [`WindowInfo`] for a window of the given size.
    ///
    /// [`WindowInfo::monitor`] starts as a `1.0` scale factor the size of
    /// the window itself - a placeholder until a window backend adapter
    /// calls [`WindowInfo::set_monitor`] with the real monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.size(), (800.0, 600.0).into());
    /// ```
    #[inline]
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            // true, so an app that rebuilds size-dependent state on
            // `resized()` gets its first build without a special case for
            // the first frame - and without `init` having to guess a size
            // the compositor has not finished deciding
            resized: true,
            monitor: MonitorInfo::new(1.0, Vector2::new(width as f32, height as f32)),
            exit_requested: false,
            exit_code: 0,
            cursor_visible: true,
            cursor: Cursor::Default,
        }
    }

    /// Returns the window's inner size in pixels, as `(width, height)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.size(), (800.0, 600.0).into());
    /// ```
    #[inline]
    pub const fn size(&self) -> Vector2 {
        Vector2::new(self.width as f32, self.height as f32)
    }

    /// Updates the tracked window size. Called by a window backend adapter.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.resize(1024, 768);
    /// assert_eq!(window.size(), (1024.0, 768.0).into());
    /// ```
    #[inline]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.resized |= self.width != width || self.height != height;
        self.width = width;
        self.height = height;
    }

    /// Returns `true` if the window's size changed since the last frame.
    ///
    /// True on the first frame as well, so this is the one place
    /// size-dependent state is built: once at the start, and again whenever
    /// the size changes. A window manager can resize the window after
    /// creation for its own reasons (a tiling WM, a fractional-scale
    /// surface settling), independent of anything the app asked for.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// assert!(window.resized()); // true on the first frame
    ///
    /// window.begin_frame();
    /// window.resize(1024, 768);
    /// assert!(window.resized());
    ///
    /// window.begin_frame();
    /// assert!(!window.resized());
    /// ```
    #[inline]
    pub const fn resized(&self) -> bool {
        self.resized
    }

    /// Returns the monitor the window is currently displayed on.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::{MonitorInfo, WindowInfo};
    /// use ancorix_math::v2;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_monitor(MonitorInfo::new(1.25, v2!(1920.0, 1080.0)));
    ///
    /// assert_eq!(window.monitor().scale_factor, 1.25);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_monitor`]
    #[inline]
    pub const fn monitor(&self) -> MonitorInfo {
        self.monitor
    }

    /// Updates the tracked monitor. Called by a window backend adapter.
    ///
    /// # See also
    ///
    /// [`WindowInfo::monitor`]
    #[inline]
    pub const fn set_monitor(&mut self, monitor: MonitorInfo) {
        self.monitor = monitor;
    }

    /// Returns whether the OS cursor should currently be visible over the
    /// window. Visible by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert!(window.cursor_visible());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_cursor_visible`]
    #[inline]
    pub const fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Shows or hides the OS cursor while it's over the window.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_cursor_visible(false);
    ///
    /// assert!(!window.cursor_visible());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::cursor_visible`]
    #[inline]
    pub const fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Returns the shape the cursor takes over the window.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::{Cursor, WindowInfo};
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.cursor(), Cursor::Default);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_cursor`]
    #[inline]
    pub const fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Sets the shape the cursor takes over the window.
    ///
    /// Sticky, like every other window setting: set it each frame from what
    /// is under the pointer, and it stays until something sets it otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::{Cursor, WindowInfo};
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_cursor(Cursor::Text);
    ///
    /// assert_eq!(window.cursor(), Cursor::Text);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::cursor`]
    #[inline]
    pub const fn set_cursor(&mut self, cursor: Cursor) {
        self.cursor = cursor;
    }

    /// Called at the start of each frame. Clears [`WindowInfo::resized`].
    #[inline]
    pub const fn begin_frame(&mut self) {
        self.resized = false;
    }

    /// Requests that the application close after the current frame, with
    /// exit code `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.request_exit();
    /// assert!(window.exit_requested());
    /// assert_eq!(window.exit_code(), 0);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::request_exit_with_code`], [`WindowInfo::exit_requested`]
    #[inline]
    pub fn request_exit(&mut self) {
        self.request_exit_with_code(0);
    }

    /// Requests that the application close after the current frame, with
    /// the given process exit code.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.request_exit_with_code(1);
    /// assert!(window.exit_requested());
    /// assert_eq!(window.exit_code(), 1);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::request_exit`], [`WindowInfo::exit_code`]
    #[inline]
    pub fn request_exit_with_code(&mut self, code: u8) {
        self.exit_requested = true;
        self.exit_code = code;
    }

    /// Returns `true` if [`WindowInfo::request_exit`] (or the `_with_code`
    /// variant) was called.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert!(!window.exit_requested());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::request_exit`]
    #[inline]
    pub const fn exit_requested(&self) -> bool {
        self.exit_requested
    }

    /// Returns the exit code set by [`WindowInfo::request_exit_with_code`],
    /// or `0` if exit was never requested or requested via plain
    /// [`WindowInfo::request_exit`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.exit_code(), 0);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::request_exit_with_code`]
    #[inline]
    pub const fn exit_code(&self) -> u8 {
        self.exit_code
    }
}
