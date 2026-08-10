/// A physical keyboard key.
#[repr(u16)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[doc(alias = "keycode")]
#[doc(alias = "scancode")]
#[doc(alias = "keyboard")]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    Up,
    Down,
    Left,
    Right,

    Space,
    Enter,
    Escape,
    Backspace,
    Tab,

    ShiftLeft,
    ShiftRight,
    CtrlLeft,
    CtrlRight,
    AltLeft,
    AltRight,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

    #[doc(hidden)]
    NotAKey,
}

impl Key {
    /// Total number of distinct keys. Equal to [`Key::NotAKey`] as an index,
    /// so adding new key variants never requires updating a manual count.
    pub const COUNT: usize = Key::NotAKey as usize;

    /// Returns the array index for this key.
    #[inline(always)]
    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    /// Parses a key from its variant name (`"Space"`, `"F5"`, `"Num0"`, ...).
    /// Returns `None` for unrecognized names.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "A" => Self::A,
            "B" => Self::B,
            "C" => Self::C,
            "D" => Self::D,
            "E" => Self::E,
            "F" => Self::F,
            "G" => Self::G,
            "H" => Self::H,
            "I" => Self::I,
            "J" => Self::J,
            "K" => Self::K,
            "L" => Self::L,
            "M" => Self::M,
            "N" => Self::N,
            "O" => Self::O,
            "P" => Self::P,
            "Q" => Self::Q,
            "R" => Self::R,
            "S" => Self::S,
            "T" => Self::T,
            "U" => Self::U,
            "V" => Self::V,
            "W" => Self::W,
            "X" => Self::X,
            "Y" => Self::Y,
            "Z" => Self::Z,

            "Num0" => Self::Num0,
            "Num1" => Self::Num1,
            "Num2" => Self::Num2,
            "Num3" => Self::Num3,
            "Num4" => Self::Num4,
            "Num5" => Self::Num5,
            "Num6" => Self::Num6,
            "Num7" => Self::Num7,
            "Num8" => Self::Num8,
            "Num9" => Self::Num9,

            "Up" => Self::Up,
            "Down" => Self::Down,
            "Left" => Self::Left,
            "Right" => Self::Right,

            "Space" => Self::Space,
            "Enter" => Self::Enter,
            "Escape" => Self::Escape,
            "Backspace" => Self::Backspace,
            "Tab" => Self::Tab,

            "ShiftLeft" => Self::ShiftLeft,
            "ShiftRight" => Self::ShiftRight,
            "CtrlLeft" => Self::CtrlLeft,
            "CtrlRight" => Self::CtrlRight,
            "AltLeft" => Self::AltLeft,
            "AltRight" => Self::AltRight,

            "F1" => Self::F1,
            "F2" => Self::F2,
            "F3" => Self::F3,
            "F4" => Self::F4,
            "F5" => Self::F5,
            "F6" => Self::F6,
            "F7" => Self::F7,
            "F8" => Self::F8,
            "F9" => Self::F9,
            "F10" => Self::F10,
            "F11" => Self::F11,
            "F12" => Self::F12,

            "Insert" => Self::Insert,
            "Delete" => Self::Delete,
            "Home" => Self::Home,
            "End" => Self::End,
            "PageUp" => Self::PageUp,
            "PageDown" => Self::PageDown,

            _ => return None,
        })
    }
}
