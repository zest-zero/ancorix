/// A mouse button.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[doc(alias = "click")]
#[doc(alias = "cursor")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}
