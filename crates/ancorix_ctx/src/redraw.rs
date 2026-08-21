use std::time::Duration;

/// When an application needs its next frame, answered by
/// [`App::redraw()`](crate::App::redraw) after every frame.
///
/// A frame is always drawn when something happens - a key, the mouse, a
/// resize - so this only says what is needed *besides* that.
///
/// # Examples
///
/// ```
/// use ancorix_ctx::Redraw;
/// use std::time::Duration;
///
/// let scrolling = false;
///
/// let next = if scrolling {
///     Redraw::Now
/// } else {
///     Redraw::After(Duration::from_millis(500))
/// };
///
/// assert_eq!(next, Redraw::After(Duration::from_millis(500)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redraw {
    /// As soon as possible - continuous redraw, capped by the frame rate
    /// limit if one was set.
    Now,

    /// Only when something happens. The process sleeps in between.
    OnEvent,

    /// After this long, or sooner if something happens.
    After(Duration),
}
