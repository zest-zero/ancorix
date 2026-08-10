use ancorix_ctx::App;

/// Window configuration and entry point.
///
/// Build one with [`Window::new`], optionally configure it, then call
/// [`Window::run`] to create the window and start the event loop.
#[doc(alias = "screen")]
#[doc(alias = "display")]
pub struct Window {
    pub(crate) title: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) target_fps: Option<u32>,
    pub(crate) resizable: bool,
    pub(crate) fullscreen: bool,
    pub(crate) vsync: bool,
    pub(crate) project_dir: Option<std::path::PathBuf>,
}

impl Window {
    /// Starts building a window with the given title and inner size in
    /// logical pixels.
    #[inline]
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
            target_fps: None,
            resizable: false,
            fullscreen: false,
            vsync: false,
            project_dir: None,
        }
    }

    /// Adds `dir` to the front of the search for `*.project.json(5)`, ahead
    /// of the executable's own directory and the working directory.
    ///
    /// Pass `env!("CARGO_MANIFEST_DIR")`: it expands, at compile time, to
    /// the directory of the crate calling it, so the config is found no
    /// matter which directory the binary is launched from. Without it, a
    /// `cargo run` from anywhere but the crate's own directory won't see
    /// the file at all.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_window::Window;
    ///
    /// let window = Window::new("demo", 800, 600).project_dir(env!("CARGO_MANIFEST_DIR"));
    /// ```
    #[inline]
    pub fn project_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.project_dir = Some(dir.into());
        self
    }

    /// Waits for the display's refresh before presenting, capping the frame
    /// rate to its rate and removing tearing. Off by default, which lets
    /// frames present as soon as they're ready.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_window::Window;
    ///
    /// let window = Window::new("demo", 800, 600).vsync(true);
    /// ```
    #[inline]
    pub fn vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Caps the frame rate at `fps`. Uncapped (runs as fast as the OS
    /// delivers redraw opportunities) if never called.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_window::Window;
    ///
    /// let window = Window::new("demo", 800, 600).fps(144);
    /// ```
    #[inline]
    pub fn fps(mut self, fps: u32) -> Self {
        self.target_fps = Some(fps);
        self
    }

    /// Sets whether the user can resize the window. **Not** resizable by
    /// default.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_window::Window;
    ///
    /// let window = Window::new("demo", 800, 600).resizable(true);
    /// ```
    #[inline]
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Starts the window borderless-fullscreen on the current monitor.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_window::Window;
    ///
    /// let window = Window::new("demo", 800, 600).fullscreen(true);
    /// ```
    #[inline]
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Creates the OS window and runs the event loop: calls `A::init` once,
    /// then `A::frame` every frame.
    ///
    /// If the app never calls `ctx.window.request_exit_with_code`, or calls
    /// it with `0`, this simply returns and the process exits normally. A
    /// non-zero code passed to `request_exit_with_code` is reported to the
    /// OS via `std::process::exit` - after the window and its GPU resources
    /// have already been cleanly torn down, so no cleanup is skipped.
    ///
    /// Blocks the calling thread until the window is closed - a winit 0.30
    /// constraint (`EventLoop::run_app` doesn't return until the loop
    /// exits).
    ///
    /// # Panics
    ///
    /// Panics if the platform event loop or window can't be created.
    pub fn run<A: App>(self) {
        let event_loop =
            winit::event_loop::EventLoop::new().expect("failed to create the event loop");

        let mut runner = crate::runner::Runner::<A>::new(self);
        event_loop
            .run_app(&mut runner)
            .expect("event loop exited with an error");

        let code = runner.exit_code();
        drop(runner); // tear down the window and GPU resources before exiting

        if code != 0 {
            std::process::exit(code.into());
        }
    }
}
