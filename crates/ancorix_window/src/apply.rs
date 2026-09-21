use ancorix_ctx::{Cursor, CursorGrab, WindowInfo};
use ancorix_math::Vector2;
use winit::dpi::PhysicalSize;
use winit::window::{CursorGrabMode, CursorIcon, Fullscreen, Window, WindowLevel};

// What the real window was last told, so a frame only calls into winit for
// what actually changed. Setting a value every frame is how an
// immediate-mode app says it, and most of these calls are round trips to the
// compositor.
pub(crate) struct Applied {
    title: String,
    cursor_visible: bool,
    cursor: Cursor,
    cursor_grab: CursorGrab,
    resizable: bool,
    fullscreen: bool,
    maximized: bool,
    decorations: bool,
    always_on_top: bool,
    min_size: Option<Vector2>,
    max_size: Option<Vector2>,
    vsync: bool,
}

impl Applied {
    // What a window created from `info` already is.
    pub(crate) fn of(info: &WindowInfo) -> Self {
        Self {
            title: info.title().to_owned(),
            cursor_visible: info.cursor_visible(),
            cursor: info.cursor(),
            cursor_grab: info.cursor_grab(),
            resizable: info.resizable(),
            fullscreen: info.fullscreen(),
            maximized: info.maximized(),
            decorations: info.decorations(),
            always_on_top: info.always_on_top(),
            min_size: info.min_size(),
            max_size: info.max_size(),
            vsync: info.vsync(),
        }
    }

    // Records a change the window manager made on its own, so it is not
    // taken for a request and pushed back.
    //
    // Fullscreen only. `is_maximized()` is not trustworthy enough to follow:
    // on Hyprland 0.56 it is true for every window, floating ones included,
    // and following it would turn `set_maximized(true)` into a no-op there.
    pub(crate) fn follow(&mut self, info: &mut WindowInfo, window: &Window) {
        let fullscreen = window.fullscreen().is_some();

        self.fullscreen = fullscreen;
        info.set_fullscreen(fullscreen);
    }

    // Pushes every setting that differs from what the window was last told,
    // plus this frame's one-shot requests. Returns `true` if vsync changed,
    // which the caller answers by rebuilding the swapchain.
    pub(crate) fn apply(&mut self, info: &WindowInfo, window: &Window) -> bool {
        if self.title != info.title() {
            self.title.clear();
            self.title.push_str(info.title());
            window.set_title(info.title());
        }

        if self.cursor_visible != info.cursor_visible() {
            self.cursor_visible = info.cursor_visible();
            window.set_cursor_visible(self.cursor_visible);
        }

        if self.cursor != info.cursor() {
            self.cursor = info.cursor();
            window.set_cursor(cursor_icon(self.cursor));
        }

        if self.cursor_grab != info.cursor_grab() {
            self.cursor_grab = info.cursor_grab();
            grab(window, self.cursor_grab);
        }

        if self.resizable != info.resizable() {
            self.resizable = info.resizable();
            window.set_resizable(self.resizable);
        }

        if self.fullscreen != info.fullscreen() {
            self.fullscreen = info.fullscreen();
            // `None` monitor means "the one the window is on"
            window.set_fullscreen(self.fullscreen.then_some(Fullscreen::Borderless(None)));
        }

        if self.maximized != info.maximized() {
            self.maximized = info.maximized();
            window.set_maximized(self.maximized);
        }

        if self.decorations != info.decorations() {
            self.decorations = info.decorations();
            window.set_decorations(self.decorations);
        }

        if self.always_on_top != info.always_on_top() {
            self.always_on_top = info.always_on_top();
            window.set_window_level(if self.always_on_top {
                WindowLevel::AlwaysOnTop
            } else {
                WindowLevel::Normal
            });
        }

        if self.min_size != info.min_size() {
            self.min_size = info.min_size();
            window.set_min_inner_size(self.min_size.map(physical));
        }

        if self.max_size != info.max_size() {
            self.max_size = info.max_size();
            window.set_max_inner_size(self.max_size.map(physical));
        }

        if let Some(size) = info.requested_size() {
            // the answer, if any, arrives as `WindowEvent::Resized`
            let _ = window.request_inner_size(physical(size));
        }

        if info.minimize_requested() {
            window.set_minimized(true);
        }

        if info.attention_requested() {
            window.request_user_attention(Some(winit::window::UserAttentionType::Informational));
        }

        let vsync_changed = self.vsync != info.vsync();
        self.vsync = info.vsync();
        vsync_changed
    }
}

// Each platform supports one mode or the other (X11 and Windows confine,
// macOS locks, Wayland does both), so the other mode is the fallback - a
// camera that wanted locking still keeps the pointer inside the window.
fn grab(window: &Window, grab: CursorGrab) {
    let (wanted, fallback) = match grab {
        CursorGrab::Free => (CursorGrabMode::None, None),
        CursorGrab::Confined => (CursorGrabMode::Confined, Some(CursorGrabMode::Locked)),
        CursorGrab::Locked => (CursorGrabMode::Locked, Some(CursorGrabMode::Confined)),
    };

    if window.set_cursor_grab(wanted).is_err()
        && let Some(fallback) = fallback
    {
        let _ = window.set_cursor_grab(fallback);
    }
}

fn physical(size: Vector2) -> PhysicalSize<u32> {
    PhysicalSize::new(size.x.max(1.0) as u32, size.y.max(1.0) as u32)
}

fn cursor_icon(cursor: Cursor) -> CursorIcon {
    match cursor {
        Cursor::Default => CursorIcon::Default,
        Cursor::Text => CursorIcon::Text,
        Cursor::Pointer => CursorIcon::Pointer,
        Cursor::Crosshair => CursorIcon::Crosshair,
        Cursor::Move => CursorIcon::Move,
        Cursor::Grab => CursorIcon::Grab,
        Cursor::Grabbing => CursorIcon::Grabbing,
        Cursor::NotAllowed => CursorIcon::NotAllowed,
        Cursor::Progress => CursorIcon::Progress,
        Cursor::Wait => CursorIcon::Wait,
        Cursor::ResizeHorizontal => CursorIcon::EwResize,
        Cursor::ResizeVertical => CursorIcon::NsResize,
        Cursor::ResizeDiagonalUp => CursorIcon::NeswResize,
        Cursor::ResizeDiagonalDown => CursorIcon::NwseResize,
        Cursor::ResizeColumn => CursorIcon::ColResize,
        Cursor::ResizeRow => CursorIcon::RowResize,
    }
}
