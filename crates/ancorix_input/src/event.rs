use crate::Key;

/// One keyboard event, in the order it arrived within the frame.
///
/// # Examples
///
/// ```
/// use ancorix_input::{Input, Key, KeyEvent};
///
/// let mut input = Input::new();
/// input.press_key(Key::A);
/// input.push_char('a');
///
/// assert_eq!(
///     input.key_events(),
///     [KeyEvent::Pressed(Key::A), KeyEvent::Char('a')]
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
    /// A key went down. Auto-repeat arrives as more of these.
    Pressed(Key),

    /// A character the layout produced, shifted and dead-key composed.
    /// Never a control character.
    Char(char),
}
