// One list, three tables. The enum, `name`, `from_name` and `ALL` all come
// from the macro below, because a key added to one of four hand-written
// lists and forgotten in the other three is a bug nothing catches: the key
// simply never arrives, and the binding silently does nothing.
macro_rules! keys {
    ($($key:ident),* $(,)?) => {
        /// A physical keyboard key.
        ///
        /// Physical: the key in that position on the keyboard, whatever the
        /// layout prints on it. `Key::W` is the key above `Key::S` on QWERTY
        /// and on AZERTY alike - which is what a movement binding wants, and
        /// the opposite of what typing wants. Typed text arrives as
        /// [`KeyEvent::Char`](crate::KeyEvent::Char) instead.
        #[repr(u16)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        #[doc(alias = "keycode")]
        #[doc(alias = "scancode")]
        #[doc(alias = "keyboard")]
        pub enum Key {
            $($key,)*

            #[doc(hidden)]
            NotAKey,
        }

        impl Key {
            /// Every key, in the order their indices run.
            ///
            /// # Examples
            ///
            /// ```
            /// use ancorix_input::Key;
            ///
            /// assert_eq!(Key::ALL.len(), Key::COUNT);
            /// assert!(Key::ALL.contains(&Key::Space));
            /// ```
            pub const ALL: [Key; Key::COUNT] = [$(Key::$key),*];

            /// Returns the key's name, the one [`Key::from_name()`] parses.
            ///
            /// # Examples
            ///
            /// ```
            /// use ancorix_input::Key;
            ///
            /// assert_eq!(Key::PageDown.name(), "PageDown");
            /// ```
            ///
            /// # See also
            ///
            /// * [`Key::from_name()`]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Key::$key => stringify!($key),)*
                    Key::NotAKey => "NotAKey",
                }
            }

            /// Parses a key from its variant name (`"Space"`, `"F5"`,
            /// `"Num0"`, ...). Returns `None` for unrecognized names.
            ///
            /// For bindings that arrive as strings - from a config file, or
            /// from a settings screen.
            ///
            /// # Examples
            ///
            /// ```
            /// use ancorix_input::Key;
            ///
            /// assert_eq!(Key::from_name("NumpadEnter"), Some(Key::NumpadEnter));
            /// assert_eq!(Key::from_name("Any"), None);
            /// ```
            ///
            /// # See also
            ///
            /// * [`Key::name()`]
            pub fn from_name(name: &str) -> Option<Self> {
                Some(match name {
                    $(stringify!($key) => Key::$key,)*
                    _ => return None,
                })
            }
        }
    };
}

keys! {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    Up, Down, Left, Right,

    Space, Enter, Escape, Backspace, Tab,

    ShiftLeft, ShiftRight,
    CtrlLeft, CtrlRight,
    AltLeft, AltRight,
    SuperLeft, SuperRight,

    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,

    Insert, Delete, Home, End, PageUp, PageDown,

    Minus, Equal,
    BracketLeft, BracketRight,
    Backslash, Semicolon, Quote,
    Comma, Period, Slash, Backquote,

    // the keys a US keyboard does not have: the 102nd key next to the left
    // Shift, and the two on Japanese layouts
    IntlBackslash, IntlRo, IntlYen,

    CapsLock, NumLock, ScrollLock, PrintScreen, Pause, ContextMenu,

    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide,
    NumpadDecimal, NumpadComma, NumpadEnter, NumpadEqual,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_round_trips_through_its_name() {
        for key in Key::ALL {
            assert_eq!(Key::from_name(key.name()), Some(key), "{key:?}");
        }
    }

    #[test]
    fn all_is_in_index_order_and_covers_every_key() {
        assert_eq!(Key::ALL.len(), Key::COUNT);

        for (index, key) in Key::ALL.iter().enumerate() {
            assert_eq!(key.index(), index, "{key:?}");
        }
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<&str> = Key::ALL.iter().map(|key| key.name()).collect();
        names.sort_unstable();

        let count = names.len();
        names.dedup();

        assert_eq!(names.len(), count, "two keys share a name");
    }
}
