use crate::{Key, KeyEvent};

/// A single-line editable string with a cursor.
///
/// Selection, clipboard and undo are deliberately absent - those belong to
/// a widget, and this is a buffer.
///
/// # Examples
///
/// ```
/// use ancorix_input::{Input, Key, TextField};
///
/// let mut input = Input::new();
/// input.push_char('h');
/// input.push_char('i');
/// input.press_key(Key::Backspace);
///
/// let mut field = TextField::default();
/// field.apply(input.key_events());
///
/// assert_eq!(field.as_str(), "h");
/// ```
#[derive(Debug, Default, Clone)]
pub struct TextField {
    text: String,
    // a byte index, always on a char boundary
    cursor: usize,
}

impl TextField {
    /// Returns a [`TextField`] holding `text`, cursor at its end.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let field = TextField::new("name");
    /// assert_eq!(field.before_cursor(), "name");
    /// ```
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            cursor: text.len(),
            text,
        }
    }

    /// Applies every event in `events` in order: characters are inserted,
    /// Backspace and Delete remove, the arrows, Home and End move.
    ///
    /// Other keys are ignored, so what Enter or Escape mean stays the
    /// caller's decision.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Key, KeyEvent, TextField};
    ///
    /// let mut field = TextField::new("ab");
    /// field.apply(&[KeyEvent::Pressed(Key::Left), KeyEvent::Char('c')]);
    ///
    /// assert_eq!(field.as_str(), "acb");
    /// ```
    pub fn apply(&mut self, events: &[KeyEvent]) {
        for event in events {
            match *event {
                KeyEvent::Char(ch) => self.insert(ch),
                KeyEvent::Pressed(Key::Backspace) => self.backspace(),
                KeyEvent::Pressed(Key::Delete) => self.delete(),
                KeyEvent::Pressed(Key::Left) => self.move_left(),
                KeyEvent::Pressed(Key::Right) => self.move_right(),
                KeyEvent::Pressed(Key::Home) => self.move_start(),
                KeyEvent::Pressed(Key::End) => self.move_end(),
                _ => {}
            }
        }
    }

    /// Returns the whole text.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// assert_eq!(TextField::new("hi").as_str(), "hi");
    /// ```
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Returns the text left of the cursor - its measured width is where a
    /// caret is drawn.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Key, KeyEvent, TextField};
    ///
    /// let mut field = TextField::new("hi");
    /// field.apply(&[KeyEvent::Pressed(Key::Left)]);
    ///
    /// assert_eq!(field.before_cursor(), "h");
    /// ```
    #[inline]
    pub fn before_cursor(&self) -> &str {
        &self.text[..self.cursor]
    }

    /// Returns `true` while the text is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// assert!(TextField::default().is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the text, leaving the field empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::new("sent");
    ///
    /// assert_eq!(field.take(), "sent");
    /// assert!(field.is_empty());
    /// ```
    pub fn take(&mut self) -> String {
        self.cursor = 0;
        std::mem::take(&mut self.text)
    }

    /// Inserts `ch` at the cursor and steps past it.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::default();
    /// field.insert('a');
    ///
    /// assert_eq!(field.as_str(), "a");
    /// ```
    pub fn insert(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    /// Removes the character before the cursor, if there is one.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::new("ab");
    /// field.backspace();
    ///
    /// assert_eq!(field.as_str(), "a");
    /// ```
    pub fn backspace(&mut self) {
        if let Some(prev) = self.prev_boundary() {
            self.text.remove(prev);
            self.cursor = prev;
        }
    }

    /// Removes the character at the cursor, if there is one.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::new("ab");
    /// field.move_start();
    /// field.delete();
    ///
    /// assert_eq!(field.as_str(), "b");
    /// ```
    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    /// Moves the cursor one character left.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::new("ab");
    /// field.move_left();
    ///
    /// assert_eq!(field.before_cursor(), "a");
    /// ```
    ///
    /// # See also
    ///
    /// * [`TextField::move_right()`]
    pub fn move_left(&mut self) {
        if let Some(prev) = self.prev_boundary() {
            self.cursor = prev;
        }
    }

    /// Moves the cursor one character right.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::new("ab");
    /// field.move_start();
    /// field.move_right();
    ///
    /// assert_eq!(field.before_cursor(), "a");
    /// ```
    ///
    /// # See also
    ///
    /// * [`TextField::move_left()`]
    pub fn move_right(&mut self) {
        if let Some(ch) = self.text[self.cursor..].chars().next() {
            self.cursor += ch.len_utf8();
        }
    }

    /// Moves the cursor before the first character.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::new("ab");
    /// field.move_start();
    ///
    /// assert_eq!(field.before_cursor(), "");
    /// ```
    ///
    /// # See also
    ///
    /// * [`TextField::move_end()`]
    #[inline]
    pub fn move_start(&mut self) {
        self.cursor = 0;
    }

    /// Moves the cursor past the last character.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::TextField;
    ///
    /// let mut field = TextField::new("ab");
    /// field.move_start();
    /// field.move_end();
    ///
    /// assert_eq!(field.before_cursor(), "ab");
    /// ```
    ///
    /// # See also
    ///
    /// * [`TextField::move_start()`]
    #[inline]
    pub fn move_end(&mut self) {
        self.cursor = self.text.len();
    }

    // `cursor - 1` would land inside a multi-byte character and panic
    fn prev_boundary(&self) -> Option<usize> {
        self.text[..self.cursor]
            .chars()
            .next_back()
            .map(|ch| self.cursor - ch.len_utf8())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editing_stays_on_char_boundaries() {
        let mut field = TextField::new("привет");

        field.backspace();
        assert_eq!(field.as_str(), "приве");

        field.move_left();
        field.move_left();
        assert_eq!(field.before_cursor(), "при");

        field.insert('в');
        assert_eq!(field.as_str(), "привве");

        field.delete();
        assert_eq!(field.as_str(), "приве");
    }

    #[test]
    fn cursor_stops_at_both_ends() {
        let mut field = TextField::new("й");

        field.move_right();
        field.delete();
        assert_eq!(field.as_str(), "й");

        field.move_left();
        field.move_left();
        field.backspace();
        assert_eq!(field.as_str(), "й");
    }

    #[test]
    fn take_resets_the_cursor() {
        let mut field = TextField::new("ok");

        assert_eq!(field.take(), "ok");
        field.insert('a');
        assert_eq!(field.as_str(), "a");
    }
}
