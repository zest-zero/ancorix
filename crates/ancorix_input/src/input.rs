use crate::{Binding, Key, MouseButton};
use ancorix_math::Vector2;
use rustc_hash::FxHashMap;

/// Manages physical and virtual input state for one frame.
#[doc(alias = "keyboard")]
#[doc(alias = "mouse")]
#[doc(alias = "controls")]
pub struct Input {
    keys_cur: [bool; Key::COUNT],
    keys_prev: [bool; Key::COUNT],

    mouse_cur: Vec<MouseButton>,
    mouse_prev: Vec<MouseButton>,

    mouse_pos: Vector2,
    mouse_delta: Vector2,
    mouse_motion: Vector2,
    scroll_delta: f32,

    actions: FxHashMap<Box<str>, Vec<Binding>>,
}

impl Default for Input {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    /// Returns a new empty [`Input`] state.
    #[inline]
    pub fn new() -> Self {
        Self {
            keys_cur: [false; Key::COUNT],
            keys_prev: [false; Key::COUNT],
            mouse_cur: Vec::new(),
            mouse_prev: Vec::new(),
            mouse_pos: Vector2::ZERO,
            mouse_delta: Vector2::ZERO,
            mouse_motion: Vector2::ZERO,
            scroll_delta: 0.0,
            actions: FxHashMap::default(),
        }
    }

    /// Returns `true` while `key` is held down.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Input, Key};
    ///
    /// let mut input = Input::new();
    /// input.press_key(Key::W);
    ///
    /// assert!(input.is_pressed(Key::W));
    /// assert!(!input.is_pressed(Key::A));
    /// ```
    #[inline]
    #[doc(alias = "is_key_down")]
    #[doc(alias = "key_down")]
    #[doc(alias = "is_down")]
    pub const fn is_pressed(&self, key: Key) -> bool {
        self.keys_cur[key.index()]
    }

    /// Returns `true` on the first frame `key` was pressed.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Input, Key};
    ///
    /// let mut input = Input::new();
    /// input.press_key(Key::Space);
    /// assert!(input.is_just_pressed(Key::Space));
    ///
    /// input.begin_frame();
    /// assert!(!input.is_just_pressed(Key::Space));
    /// assert!(input.is_pressed(Key::Space));
    /// ```
    #[inline]
    #[doc(alias = "is_key_pressed")]
    #[doc(alias = "just_pressed")]
    pub const fn is_just_pressed(&self, key: Key) -> bool {
        let i = key.index();
        self.keys_cur[i] && !self.keys_prev[i]
    }

    /// Returns `true` on the first frame `key` was released.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Input, Key};
    ///
    /// let mut input = Input::new();
    /// input.press_key(Key::Space);
    /// input.begin_frame();
    /// input.release_key(Key::Space);
    ///
    /// assert!(input.is_just_released(Key::Space));
    /// ```
    #[inline]
    pub const fn is_just_released(&self, key: Key) -> bool {
        let i = key.index();
        !self.keys_cur[i] && self.keys_prev[i]
    }

    /// Returns `true` while mouse `button` is held down.
    #[inline]
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_cur.contains(&button)
    }

    /// Returns `true` on the first frame mouse `button` was pressed.
    #[inline]
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_cur.contains(&button) && !self.mouse_prev.contains(&button)
    }

    /// Returns `true` on the first frame mouse `button` was released.
    #[inline]
    pub fn is_mouse_just_released(&self, button: MouseButton) -> bool {
        !self.mouse_cur.contains(&button) && self.mouse_prev.contains(&button)
    }

    /// Returns the current mouse position in screen coordinates.
    #[inline]
    pub const fn mouse_pos(&self) -> Vector2 {
        self.mouse_pos
    }

    /// Returns the accumulated mouse movement delta since last frame,
    /// derived from cursor position - clamped at the screen edges.
    ///
    /// # See also
    ///
    /// [`Input::mouse_motion`]
    #[inline]
    pub const fn mouse_delta(&self) -> Vector2 {
        self.mouse_delta
    }

    /// Returns the raw, unclamped mouse movement accumulated this frame.
    ///
    /// Unlike [`Input::mouse_delta`], this comes from the OS's raw motion
    /// events, not cursor position - it keeps reporting movement past the
    /// screen edges or while the cursor is locked/hidden, which is what a
    /// camera-look control needs.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::Input;
    ///
    /// let mut input = Input::new();
    /// input.add_mouse_motion(3.0, -1.0);
    ///
    /// assert_eq!(input.mouse_motion().x, 3.0);
    /// assert_eq!(input.mouse_motion().y, -1.0);
    ///
    /// input.begin_frame();
    /// assert_eq!(input.mouse_motion().x, 0.0);
    /// ```
    ///
    /// # See also
    ///
    /// [`Input::mouse_delta`], [`Input::add_mouse_motion`]
    #[inline]
    pub const fn mouse_motion(&self) -> Vector2 {
        self.mouse_motion
    }

    /// Returns the scroll wheel delta accumulated this frame.
    #[inline]
    pub const fn scroll(&self) -> f32 {
        self.scroll_delta
    }

    /// Registers a named action with a list of bindings.
    ///
    /// Calling `bind` again with the same name replaces the previous bindings.
    /// Accepts any string-like type, so action names can come from string
    /// literals or be loaded from a config file.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Input, Key, MouseButton, Binding};
    ///
    /// let mut input = Input::new();
    /// input.bind("fire", &[Binding::Mouse(MouseButton::Left), Binding::Key(Key::F)]);
    ///
    /// input.press_mouse(MouseButton::Left);
    /// assert!(input.action_pressed("fire"));
    /// ```
    #[inline]
    pub fn bind(&mut self, action: impl Into<Box<str>>, bindings: &[Binding]) {
        self.actions.insert(action.into(), bindings.to_vec());
    }

    /// Registers a named action bound only to keyboard keys.
    ///
    /// Convenience over [`Input::bind`] for the common case of binding
    /// one or more keys without mouse buttons.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Input, Key};
    ///
    /// let mut input = Input::new();
    /// input.bind_keys("jump", &[Key::Space, Key::Enter]);
    ///
    /// input.press_key(Key::Enter);
    /// assert!(input.action_pressed("jump"));
    /// ```
    pub fn bind_keys(&mut self, action: impl Into<Box<str>>, keys: &[Key]) {
        let bindings = keys.iter().map(|&k| Binding::Key(k)).collect();
        self.actions.insert(action.into(), bindings);
    }

    /// Loads named actions from a compiled `.axb` input section
    /// replacing any existing bindings with the same name.
    /// Unrecognized key names are skipped.
    ///
    /// # Errors
    ///
    /// Returns an error if `bytes` contains no input section.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` is not a well-formed `.axb` file (see
    /// `ancorix_axb::read`).
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Input, Key};
    /// use ancorix_axb::{SectionType, Writer};
    ///
    /// let mut w = Writer::new(SectionType::Input, 1);
    /// w.u16(1); // one action
    /// w.str("jump");
    /// w.u8(1); // one binding
    /// w.str("Space");
    /// let bytes = w.finish();
    ///
    /// let mut input = Input::new();
    /// input.load_bindings(&bytes).unwrap();
    ///
    /// input.press_key(Key::Space);
    /// assert!(input.action_pressed("jump"));
    /// ```
    pub fn load_bindings(&mut self, bytes: &[u8]) -> Result<(), Box<str>> {
        let input_section = ancorix_axb::read(bytes)
            .into_iter()
            .find_map(|section| match section {
                ancorix_axb::Section::Input(input) => Some(input),
                _ => None,
            })
            .ok_or("no input section in .axb file")?;

        for (action, keys) in input_section.actions {
            let bindings = keys
                .iter()
                .filter_map(|k| Key::from_name(k))
                .map(Binding::Key)
                .collect();
            self.actions.insert(action.into_boxed_str(), bindings);
        }

        Ok(())
    }

    /// Returns `true` while any binding for `action` is held.
    ///
    /// Returns `false` if the action was never registered.
    #[inline]
    pub fn action_pressed(&self, action: &str) -> bool {
        self.actions
            .get(action)
            .map(|b| b.iter().any(|b| self.binding_pressed(*b)))
            .unwrap_or(false)
    }

    /// Returns `true` on the first frame any binding for `action` was pressed.
    ///
    /// Returns `false` if the action was never registered.
    #[inline]
    pub fn action_just_pressed(&self, action: &str) -> bool {
        self.actions
            .get(action)
            .map(|b| b.iter().any(|b| self.binding_just_pressed(*b)))
            .unwrap_or(false)
    }

    /// Returns `true` on the first frame any binding for `action` was released.
    ///
    /// Returns `false` if the action was never registered.
    #[inline]
    pub fn action_just_released(&self, action: &str) -> bool {
        self.actions
            .get(action)
            .map(|b| b.iter().any(|b| self.binding_just_released(*b)))
            .unwrap_or(false)
    }

    /// Combines four [`Input::action_pressed()`] checks into one 2D
    /// direction vector - `x` from `negative_x`/`positive_x`, `y` from
    /// `negative_y`/`positive_y`. Clamped to length `1.0`, so diagonal
    /// input (e.g. both `W`/`D` held)
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_input::{Input, Key};
    /// use ancorix_math::v2;
    ///
    /// let mut input = Input::new();
    /// input.bind_keys("left", &[Key::A]);
    /// input.bind_keys("right", &[Key::D]);
    /// input.bind_keys("up", &[Key::W]);
    /// input.bind_keys("down", &[Key::S]);
    ///
    /// assert_eq!(input.action_vector("left", "right", "up", "down"), v2!(0.0, 0.0));
    ///
    /// input.press_key(Key::D);
    /// assert_eq!(input.action_vector("left", "right", "up", "down"), v2!(1.0, 0.0));
    ///
    /// input.press_key(Key::W);
    /// let diagonal = input.action_vector("left", "right", "up", "down");
    /// assert!((diagonal.length() - 1.0).abs() < 1e-5);
    /// ```
    pub fn action_vector(
        &self,
        negative_x: &str,
        positive_x: &str,
        negative_y: &str,
        positive_y: &str,
    ) -> Vector2 {
        let x = self.action_pressed(positive_x) as i8 - self.action_pressed(negative_x) as i8;
        let y = self.action_pressed(positive_y) as i8 - self.action_pressed(negative_y) as i8;
        Vector2::new(x as f32, y as f32).clamp_length(1.0)
    }

    #[inline]
    fn binding_pressed(&self, b: Binding) -> bool {
        match b {
            Binding::Key(k) => self.is_pressed(k),
            Binding::Mouse(m) => self.is_mouse_pressed(m),
        }
    }

    #[inline]
    fn binding_just_pressed(&self, b: Binding) -> bool {
        match b {
            Binding::Key(k) => self.is_just_pressed(k),
            Binding::Mouse(m) => self.is_mouse_just_pressed(m),
        }
    }

    #[inline]
    fn binding_just_released(&self, b: Binding) -> bool {
        match b {
            Binding::Key(k) => self.is_just_released(k),
            Binding::Mouse(m) => self.is_mouse_just_released(m),
        }
    }

    /// Called at the start of each frame. Rotates current state into previous.
    // fed by the platform adapter (`ancorix_winit`), never by a game
    #[doc(hidden)]
    pub fn begin_frame(&mut self) {
        self.keys_prev = self.keys_cur;

        self.mouse_prev.clear();
        self.mouse_prev.extend_from_slice(&self.mouse_cur);

        self.mouse_delta = Vector2::ZERO;
        self.mouse_motion = Vector2::ZERO;
        self.scroll_delta = 0.0;
    }

    /// Marks `key` as pressed. Called by a window backend adapter.
    // fed by the platform adapter (`ancorix_winit`), never by a game
    #[doc(hidden)]
    #[inline]
    pub const fn press_key(&mut self, key: Key) {
        self.keys_cur[key.index()] = true;
    }

    /// Marks `key` as released. Called by a window backend adapter.
    // fed by the platform adapter (`ancorix_winit`), never by a game
    #[doc(hidden)]
    #[inline]
    pub const fn release_key(&mut self, key: Key) {
        self.keys_cur[key.index()] = false;
    }

    /// Marks `button` as pressed. Called by a window backend adapter.
    // fed by the platform adapter (`ancorix_winit`), never by a game
    #[doc(hidden)]
    #[inline]
    pub fn press_mouse(&mut self, button: MouseButton) {
        if !self.mouse_cur.contains(&button) {
            self.mouse_cur.push(button);
        }
    }

    /// Marks `button` as released. Called by a window backend adapter.
    // fed by the platform adapter (`ancorix_winit`), never by a game
    #[doc(hidden)]
    #[inline]
    pub fn release_mouse(&mut self, button: MouseButton) {
        self.mouse_cur.retain(|&b| b != button);
    }

    /// Updates the cursor position, accumulating the delta from the
    /// previous position. Called by a window backend adapter.
    // fed by the platform adapter (`ancorix_winit`), never by a game
    #[doc(hidden)]
    #[inline]
    pub const fn move_cursor(&mut self, x: f32, y: f32) {
        let new_pos = Vector2::new(x, y);
        self.mouse_delta
            .const_add_assign(new_pos.const_sub(self.mouse_pos));
        self.mouse_pos = new_pos;
    }

    /// Accumulates a scroll wheel delta for this frame. Called by a window
    /// backend adapter.
    // fed by the platform adapter (`ancorix_winit`), never by a game
    #[doc(hidden)]
    #[inline]
    pub const fn add_scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    /// Accumulates raw mouse motion for this frame. Called by a window
    /// backend adapter.
    ///
    /// # See also
    ///
    /// [`Input::mouse_motion`]
    #[inline]
    pub const fn add_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.mouse_motion.x += dx;
        self.mouse_motion.y += dy;
    }
}
