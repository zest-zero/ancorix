/// The shape the OS cursor takes while it is over the window.
///
/// Set through [`WindowInfo::set_cursor()`](crate::WindowInfo::set_cursor),
/// once per frame like everything else - an immediate-mode app says what the
/// cursor is now, not when it changed.
///
/// # Examples
///
/// ```
/// use ancorix_ctx::{Cursor, WindowInfo};
///
/// # let text_area_hovered = true;
/// let mut window = WindowInfo::new(800, 600);
///
/// window.set_cursor(if text_area_hovered {
///     Cursor::Text
/// } else {
///     Cursor::Default
/// });
///
/// assert_eq!(window.cursor(), Cursor::Text);
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[doc(alias = "mouse")]
#[doc(alias = "pointer")]
pub enum Cursor {
    /// The system's ordinary arrow.
    #[default]
    Default,

    /// An I-beam, over text that can be selected or typed into.
    Text,

    /// A pointing hand, over something that can be clicked.
    Pointer,

    /// Crosshairs, for picking an exact point.
    Crosshair,

    /// Four arrows, over something that can be moved.
    Move,

    /// An open hand, over a surface that can be dragged.
    Grab,

    /// A closed hand, while that surface is being dragged.
    Grabbing,

    /// A struck-through circle, over a target that would reject the drop.
    NotAllowed,

    /// The arrow with a spinner: busy, but still taking input.
    Progress,

    /// The spinner alone: busy, and not taking input.
    Wait,

    /// A left-right arrow, over a vertical edge that resizes.
    ResizeHorizontal,

    /// An up-down arrow, over a horizontal edge that resizes.
    ResizeVertical,

    /// A diagonal arrow, over a corner running bottom-left to top-right.
    ResizeDiagonalUp,

    /// A diagonal arrow, over a corner running top-left to bottom-right.
    ResizeDiagonalDown,

    /// The separator between two columns.
    ResizeColumn,

    /// The separator between two rows.
    ResizeRow,
}
