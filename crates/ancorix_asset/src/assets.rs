use crate::Handle;

/// A registry of loaded `T` resources, indexed by [`Handle<T>`].
///
/// Doesn't know how to load a `T` from anything (bytes, a file path, a GPU
/// device) - that's the job of a higher-level crate (e.g. `ancorix_image`
/// decoding bytes, or `ancorix_ash` uploading a texture to the GPU), which
/// calls [`Assets::insert()`] once it has a value in hand.
///
/// # Examples
///
/// ```
/// use ancorix_asset::Assets;
///
/// let mut assets = Assets::new();
/// let handle = assets.insert("a loaded resource");
///
/// assert_eq!(assets.get(handle), Some(&"a loaded resource"));
/// ```
pub struct Assets<T> {
    items: Vec<T>,
}

impl<T> Assets<T> {
    /// Returns a new, empty [`Assets<T>`] registry.
    #[inline]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Stores `value` and returns a [`Handle<T>`] that can later retrieve it.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_asset::Assets;
    ///
    /// let mut assets = Assets::new();
    /// let handle = assets.insert(42);
    ///
    /// assert_eq!(assets.get(handle), Some(&42));
    /// ```
    #[inline]
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let handle = Handle::new(self.items.len() as u32);
        self.items.push(value);
        handle
    }

    /// Returns a reference to the resource `handle` points to, or `None` if
    /// `handle` doesn't belong to this registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_asset::Assets;
    ///
    /// let mut assets = Assets::new();
    /// let handle = assets.insert(42);
    ///
    /// assert_eq!(assets.get(handle), Some(&42));
    /// ```
    #[inline]
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.items.get(handle.index())
    }

    /// Returns a mutable reference to the resource `handle` points to, or
    /// `None` if `handle` doesn't belong to this registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_asset::Assets;
    ///
    /// let mut assets = Assets::new();
    /// let handle = assets.insert(42);
    /// *assets.get_mut(handle).unwrap() = 7;
    ///
    /// assert_eq!(assets.get(handle), Some(&7));
    /// ```
    #[inline]
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.items.get_mut(handle.index())
    }
}

impl<T> Default for Assets<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
