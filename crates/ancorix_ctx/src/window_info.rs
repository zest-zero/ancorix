use crate::{Cursor, CursorGrab, MonitorInfo};
use ancorix_math::Vector2;

/// Per-frame window state and control.
///
/// Settings are sticky: a value set here stays until something sets it
/// otherwise, and reaches the real window after the frame. Setting the same
/// value every frame costs nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    width: u32,
    height: u32,
    resized: bool,
    monitor: MonitorInfo,
    focused: bool,
    exit_requested: bool,
    exit_code: u8,
    cursor_visible: bool,
    cursor: Cursor,
    cursor_grab: CursorGrab,
    resizable: bool,
    title: String,
    fullscreen: bool,
    maximized: bool,
    decorations: bool,
    always_on_top: bool,
    min_size: Option<Vector2>,
    max_size: Option<Vector2>,
    vsync: bool,
    target_fps: Option<u32>,
    // one-shot requests, cleared by `begin_frame`
    requested_size: Option<Vector2>,
    minimize_requested: bool,
    attention_requested: bool,
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
            focused: true,
            exit_requested: false,
            exit_code: 0,
            cursor_visible: true,
            cursor: Cursor::Default,
            cursor_grab: CursorGrab::Free,
            resizable: false,
            title: String::new(),
            fullscreen: false,
            maximized: false,
            decorations: true,
            always_on_top: false,
            min_size: None,
            max_size: None,
            vsync: false,
            target_fps: None,
            requested_size: None,
            minimize_requested: false,
            attention_requested: false,
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

    /// Returns whether the user can currently resize the window.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_resizable(true);
    ///
    /// assert!(window.resizable());
    /// ```
    ///
    /// # See also
    ///
    /// * [`WindowInfo::set_resizable`]
    #[inline]
    pub const fn resizable(&self) -> bool {
        self.resizable
    }

    /// Lets the user resize the window, or stops them.
    ///
    /// Takes effect on the next frame. A tiling window manager may resize the
    /// window regardless of what it is told.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_resizable(false);
    ///
    /// assert!(!window.resizable());
    /// ```
    ///
    /// # See also
    ///
    /// * [`WindowInfo::resizable`]
    #[inline]
    pub const fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
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

    /// Returns how tightly the window holds the pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::{CursorGrab, WindowInfo};
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.cursor_grab(), CursorGrab::Free);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_cursor_grab`]
    #[inline]
    pub const fn cursor_grab(&self) -> CursorGrab {
        self.cursor_grab
    }

    /// Confines or locks the pointer to the window, or lets it go.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::{CursorGrab, WindowInfo};
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_cursor_grab(CursorGrab::Confined);
    ///
    /// assert_eq!(window.cursor_grab(), CursorGrab::Confined);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::cursor_grab`]
    #[inline]
    pub const fn set_cursor_grab(&mut self, grab: CursorGrab) {
        self.cursor_grab = grab;
    }

    /// Returns the window's title.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_title("demo");
    ///
    /// assert_eq!(window.title(), "demo");
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_title`]
    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Renames the window.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// # let fps = 144;
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_title(&format!("demo - {fps} fps"));
    ///
    /// assert_eq!(window.title(), "demo - 144 fps");
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::title`]
    pub fn set_title(&mut self, title: &str) {
        // reuses the buffer, so a title set every frame allocates only when
        // it grows
        if self.title != title {
            self.title.clear();
            self.title.push_str(title);
        }
    }

    /// Returns whether the window covers its monitor.
    ///
    /// Follows the window manager as well: leaving fullscreen by its own
    /// shortcut shows up here.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert!(!window.fullscreen());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_fullscreen`]
    #[inline]
    pub const fn fullscreen(&self) -> bool {
        self.fullscreen
    }

    /// Puts the window over its whole monitor, without a border, or takes it
    /// back out.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// let now = window.fullscreen();
    /// window.set_fullscreen(!now);
    ///
    /// assert!(window.fullscreen());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::fullscreen`]
    #[inline]
    pub const fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    /// Returns whether the app asked for the window to be maximized.
    ///
    /// What was asked, not what the window is: a double-click on the title
    /// bar does not show up here, because no platform reports it reliably.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert!(!window.maximized());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_maximized`]
    #[inline]
    pub const fn maximized(&self) -> bool {
        self.maximized
    }

    /// Maximizes the window, or restores it.
    ///
    /// Hyprland ignores the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_maximized(true);
    ///
    /// assert!(window.maximized());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::maximized`]
    #[inline]
    pub const fn set_maximized(&mut self, maximized: bool) {
        self.maximized = maximized;
    }

    /// Returns whether the window has a title bar and border. On by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert!(window.decorations());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_decorations`]
    #[inline]
    pub const fn decorations(&self) -> bool {
        self.decorations
    }

    /// Gives the window its title bar and border, or takes them away.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_decorations(false);
    ///
    /// assert!(!window.decorations());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::decorations`]
    #[inline]
    pub const fn set_decorations(&mut self, decorations: bool) {
        self.decorations = decorations;
    }

    /// Returns whether the window asks to stay above the others.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert!(!window.always_on_top());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_always_on_top`]
    #[inline]
    pub const fn always_on_top(&self) -> bool {
        self.always_on_top
    }

    /// Asks for the window to stay above the others.
    ///
    /// Wayland has no way to ask, so there it does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_always_on_top(true);
    ///
    /// assert!(window.always_on_top());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::always_on_top`]
    #[inline]
    pub const fn set_always_on_top(&mut self, on_top: bool) {
        self.always_on_top = on_top;
    }

    /// Returns the smallest size the user can shrink the window to, in
    /// pixels, or `None` if there is no limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.min_size(), None);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_min_size`], [`WindowInfo::max_size`]
    #[inline]
    pub const fn min_size(&self) -> Option<Vector2> {
        self.min_size
    }

    /// Stops the user from shrinking the window below `size` pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    /// use ancorix_math::v2;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_min_size(Some(v2!(640, 360)));
    ///
    /// assert_eq!(window.min_size(), Some(v2!(640, 360)));
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::min_size`], [`WindowInfo::set_max_size`]
    #[inline]
    pub const fn set_min_size(&mut self, size: Option<Vector2>) {
        self.min_size = size;
    }

    /// Returns the largest size the user can grow the window to, in pixels,
    /// or `None` if there is no limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.max_size(), None);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_max_size`], [`WindowInfo::min_size`]
    #[inline]
    pub const fn max_size(&self) -> Option<Vector2> {
        self.max_size
    }

    /// Stops the user from growing the window past `size` pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    /// use ancorix_math::v2;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_max_size(Some(v2!(1920, 1080)));
    ///
    /// assert_eq!(window.max_size(), Some(v2!(1920, 1080)));
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::max_size`], [`WindowInfo::set_min_size`]
    #[inline]
    pub const fn set_max_size(&mut self, size: Option<Vector2>) {
        self.max_size = size;
    }

    /// Returns whether presenting waits for the display's refresh.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert!(!window.vsync());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_vsync`]
    #[inline]
    pub const fn vsync(&self) -> bool {
        self.vsync
    }

    /// Makes presenting wait for the display's refresh, or stop waiting.
    ///
    /// Rebuilds the swapchain, so it is for a settings menu rather than for
    /// every frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_vsync(true);
    ///
    /// assert!(window.vsync());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::vsync`]
    #[inline]
    pub const fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
    }

    /// Returns the frame rate cap, or `None` if uncapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let window = WindowInfo::new(800, 600);
    /// assert_eq!(window.target_fps(), None);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::set_target_fps`]
    #[inline]
    pub const fn target_fps(&self) -> Option<u32> {
        self.target_fps
    }

    /// Caps the frame rate. `None` and `Some(0)` both mean uncapped.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    ///
    /// window.set_target_fps(Some(60));
    /// assert_eq!(window.target_fps(), Some(60));
    ///
    /// window.set_target_fps(Some(0));
    /// assert_eq!(window.target_fps(), None);
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::target_fps`]
    #[inline]
    pub const fn set_target_fps(&mut self, fps: Option<u32>) {
        self.target_fps = match fps {
            Some(0) => None,
            other => other,
        };
    }

    /// Returns whether the window has keyboard focus.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.set_focused(false);
    ///
    /// assert!(!window.focused());
    /// ```
    #[inline]
    pub const fn focused(&self) -> bool {
        self.focused
    }

    /// Updates the tracked focus. Called by a window backend adapter.
    ///
    /// # See also
    ///
    /// [`WindowInfo::focused`]
    #[inline]
    pub const fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Asks for the window to become `size` pixels.
    ///
    /// A request, not a setting: the answer arrives later through
    /// [`WindowInfo::resized`], and the window manager may say no - Hyprland
    /// always does.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    /// use ancorix_math::v2;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.request_size(v2!(1280, 720));
    ///
    /// assert_eq!(window.requested_size(), Some(v2!(1280, 720)));
    /// assert_eq!(window.size(), v2!(800, 600)); // not yet
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::requested_size`]
    #[inline]
    pub const fn request_size(&mut self, size: Vector2) {
        self.requested_size = Some(size);
    }

    /// Returns the size asked for this frame, if any.
    ///
    /// # See also
    ///
    /// [`WindowInfo::request_size`]
    #[inline]
    pub const fn requested_size(&self) -> Option<Vector2> {
        self.requested_size
    }

    /// Asks for the window to be minimized.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.minimize();
    ///
    /// assert!(window.minimize_requested());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::minimize_requested`]
    #[inline]
    pub const fn minimize(&mut self) {
        self.minimize_requested = true;
    }

    /// Returns whether [`WindowInfo::minimize`] was called this frame.
    #[inline]
    pub const fn minimize_requested(&self) -> bool {
        self.minimize_requested
    }

    /// Asks the desktop to draw the user's attention to the window, usually
    /// by flashing it in the taskbar.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.request_attention();
    ///
    /// assert!(window.attention_requested());
    /// ```
    ///
    /// # See also
    ///
    /// [`WindowInfo::attention_requested`]
    #[inline]
    pub const fn request_attention(&mut self) {
        self.attention_requested = true;
    }

    /// Returns whether [`WindowInfo::request_attention`] was called this
    /// frame.
    #[inline]
    pub const fn attention_requested(&self) -> bool {
        self.attention_requested
    }

    /// Called at the start of each frame. Clears [`WindowInfo::resized`] and
    /// the one-shot requests.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_ctx::WindowInfo;
    ///
    /// let mut window = WindowInfo::new(800, 600);
    /// window.minimize();
    /// window.begin_frame();
    ///
    /// assert!(!window.minimize_requested());
    /// ```
    #[inline]
    pub const fn begin_frame(&mut self) {
        self.resized = false;
        self.requested_size = None;
        self.minimize_requested = false;
        self.attention_requested = false;
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
