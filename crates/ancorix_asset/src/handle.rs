use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A typed, `Copy` reference to a resource stored in an [`crate::Assets<T>`]
/// registry - the same handle shape for every resource kind (`Handle<Texture>`,
/// `Handle<Shader>`, `Handle<Mesh>`, ...), so `Handle<Texture>` and
/// `Handle<Shader>` can't be mixed up even though both are just an index
/// under the hood.
///
/// # Examples
///
/// ```
/// use ancorix_asset::Assets;
///
/// let mut textures = Assets::new();
/// let handle = textures.insert("a loaded texture");
///
/// assert_eq!(textures.get(handle), Some(&"a loaded texture"));
/// ```
#[doc(alias = "id")]
#[doc(alias = "asset")]
#[doc(alias = "resource")]
pub struct Handle<T> {
    index: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    #[inline(always)]
    pub(crate) const fn new(index: u32) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub(crate) const fn index(self) -> usize {
        self.index as usize
    }
}

// manual impls, not `#[derive(...)]` - deriving would add a `T: Trait` bound
// even though `T` only appears in `PhantomData`, requiring e.g. `T: Copy`
// for `Handle<T>: Copy` when a handle should be `Copy` regardless of `T`.
impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("index", &self.index)
            .finish()
    }
}
