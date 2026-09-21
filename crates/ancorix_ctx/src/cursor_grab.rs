/// How tightly the window holds on to the pointer.
///
/// Set through [`WindowInfo::set_cursor_grab()`](crate::WindowInfo::set_cursor_grab).
///
/// # Examples
///
/// ```
/// use ancorix_ctx::{CursorGrab, WindowInfo};
///
/// # let aiming = true;
/// let mut window = WindowInfo::new(800, 600);
///
/// window.set_cursor_grab(if aiming { CursorGrab::Locked } else { CursorGrab::Free });
/// window.set_cursor_visible(!aiming);
///
/// assert_eq!(window.cursor_grab(), CursorGrab::Locked);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[doc(alias = "mouse lock")]
#[doc(alias = "pointer lock")]
pub enum CursorGrab {
    /// The pointer comes and goes as it likes.
    #[default]
    Free,

    /// The pointer moves, but cannot leave the window.
    Confined,

    /// The pointer stays where it is and only its motion is reported - what
    /// a mouse-look camera wants.
    ///
    /// Not every platform can do both: X11 and Windows only confine, macOS
    /// only locks. A grab the platform cannot do falls back to the other one
    /// rather than to none.
    Locked,
}
