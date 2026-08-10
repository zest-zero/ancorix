use crate::{Key, MouseButton};

/// A binding - either a key or a mouse button.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[doc(alias = "keybind")]
#[doc(alias = "mapping")]
#[doc(alias = "action")]
pub enum Binding {
    Key(Key),
    Mouse(MouseButton),
}

impl From<Key> for Binding {
    #[inline]
    fn from(k: Key) -> Self {
        Binding::Key(k)
    }
}

impl From<MouseButton> for Binding {
    #[inline]
    fn from(m: MouseButton) -> Self {
        Binding::Mouse(m)
    }
}
