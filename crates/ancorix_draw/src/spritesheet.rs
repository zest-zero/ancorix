use ancorix_math::Vector2;

use crate::primitive::{Rect, Sprite};

/// A uniform grid of animation frames packed into one texture, laid out
/// left-to-right then top-to-bottom.
///
/// Holds only the grid's geometry, never a `Handle<Texture>` - the same
/// layout is reused across every texture sharing it (character skins, for
/// instance), and the handle is passed at draw time to
/// [`crate::Draw::sprite()`].
///
/// # Examples
///
/// ```
/// use ancorix_draw::SpriteSheet;
/// use ancorix_math::v2;
///
/// // a 128x64 texture holding 4 columns and 2 rows of 32x32 frames
/// let sheet = SpriteSheet::from_grid(v2!(128.0, 64.0), 4, 2);
///
/// assert_eq!(sheet.frame_size, v2!(32.0, 32.0));
/// assert_eq!(sheet.frame_count(), 8);
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[doc(alias = "atlas")]
#[doc(alias = "tileset")]
#[doc(alias = "frames")]
pub struct SpriteSheet {
    /// The size of a single frame, in texture pixels.
    pub frame_size: Vector2,
    /// The number of frames per row.
    pub columns: u32,
    /// The number of rows of frames.
    pub rows: u32,
    /// The top-left corner of the grid within the texture, in texture
    /// pixels - a non-zero margin around the packed frames.
    pub origin: Vector2,
    /// The gap between two adjacent frames, in texture pixels.
    pub spacing: Vector2,
}

impl SpriteSheet {
    /// Returns a new [`SpriteSheet`] of `columns` x `rows` frames of
    /// `frame_size`, with no margin or spacing.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteSheet;
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::new(v2!(32.0, 32.0), 4, 2);
    ///
    /// assert_eq!(sheet.frame_count(), 8);
    /// ```
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::from_grid()`]
    #[inline(always)]
    pub const fn new(frame_size: Vector2, columns: u32, rows: u32) -> Self {
        Self {
            frame_size,
            columns,
            rows,
            origin: Vector2::ZERO,
            spacing: Vector2::ZERO,
        }
    }

    /// Returns a new [`SpriteSheet`] that divides a `texture_size` texture
    /// evenly into `columns` x `rows` frames.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteSheet;
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::from_grid(v2!(256.0, 128.0), 8, 4);
    ///
    /// assert_eq!(sheet.frame_size, v2!(32.0, 32.0));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `columns` or `rows` is zero.
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::new()`]
    #[inline]
    pub fn from_grid(texture_size: Vector2, columns: u32, rows: u32) -> Self {
        assert!(
            columns > 0 && rows > 0,
            "a sprite sheet needs at least one column and one row"
        );

        let frame_size = texture_size / Vector2::new(columns as f32, rows as f32);
        Self::new(frame_size, columns, rows)
    }

    /// Returns the same sheet with its grid starting at `origin` in the
    /// texture, for sheets packed with a margin.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Rect, SpriteSheet};
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::new(v2!(32.0, 32.0), 2, 1).with_origin(v2!(1.0, 1.0));
    ///
    /// assert_eq!(sheet.frame(0), Rect::new(v2!(1.0, 1.0), v2!(32.0, 32.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::with_spacing()`]
    #[inline]
    pub const fn with_origin(self, origin: Vector2) -> Self {
        Self { origin, ..self }
    }

    /// Returns the same sheet with a `spacing` gap between adjacent frames,
    /// for sheets packed with gutters.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Rect, SpriteSheet};
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::new(v2!(32.0, 32.0), 2, 1).with_spacing(v2!(2.0, 2.0));
    ///
    /// assert_eq!(sheet.frame(1), Rect::new(v2!(34.0, 0.0), v2!(32.0, 32.0)));
    /// ```
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::with_origin()`]
    #[inline]
    pub const fn with_spacing(self, spacing: Vector2) -> Self {
        Self { spacing, ..self }
    }

    /// Returns the total number of frames in the sheet.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteSheet;
    /// use ancorix_math::v2;
    ///
    /// assert_eq!(SpriteSheet::new(v2!(32.0, 32.0), 4, 3).frame_count(), 12);
    /// ```
    #[inline(always)]
    pub const fn frame_count(self) -> u32 {
        self.columns * self.rows
    }

    /// Returns the source rectangle of frame `index`, counted left-to-right
    /// then top-to-bottom.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Rect, SpriteSheet};
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::new(v2!(32.0, 32.0), 2, 2);
    ///
    /// // index 2 wraps onto the second row
    /// assert_eq!(sheet.frame(2), Rect::new(v2!(0.0, 32.0), v2!(32.0, 32.0)));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `index` is past the last frame.
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::frame_at()`]
    #[inline]
    pub fn frame(self, index: u32) -> Rect {
        assert!(
            index < self.frame_count(),
            "frame {index} is past the last frame of a {}x{} sheet",
            self.columns,
            self.rows
        );

        self.frame_at(index % self.columns, index / self.columns)
    }

    /// Returns the source rectangle of the frame at grid position
    /// `column`/`row` - for sheets whose rows mean something (one row per
    /// facing direction, say).
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Rect, SpriteSheet};
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::new(v2!(32.0, 32.0), 4, 2);
    ///
    /// assert_eq!(sheet.frame_at(1, 1), Rect::new(v2!(32.0, 32.0), v2!(32.0, 32.0)));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `column` or `row` is outside the grid.
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::frame()`]
    pub fn frame_at(self, column: u32, row: u32) -> Rect {
        assert!(
            column < self.columns && row < self.rows,
            "frame ({column}, {row}) is outside a {}x{} sheet",
            self.columns,
            self.rows
        );

        let step = self.frame_size + self.spacing;
        let offset = Vector2::new(step.x * column as f32, step.y * row as f32);

        Rect::new(self.origin + offset, self.frame_size)
    }

    /// Returns a [`Sprite`] drawing frame `index` at `pos`, at the frame's
    /// authored size.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteSheet;
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::new(v2!(32.0, 32.0), 4, 2);
    /// let sprite = sheet.sprite(v2!(100.0, 50.0), 3);
    ///
    /// assert_eq!(sprite.size, v2!(32.0, 32.0));
    /// assert_eq!(sprite.source, Some(sheet.frame(3)));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `index` is past the last frame.
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::sprite_sized()`]
    #[inline]
    pub fn sprite(self, pos: Vector2, index: u32) -> Sprite {
        self.sprite_sized(pos, self.frame_size, index)
    }

    /// Returns a [`Sprite`] drawing frame `index` at `pos`, stretched to
    /// `size` on screen.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::SpriteSheet;
    /// use ancorix_math::v2;
    ///
    /// let sheet = SpriteSheet::new(v2!(32.0, 32.0), 4, 2);
    /// let sprite = sheet.sprite_sized(v2!(0.0, 0.0), v2!(128.0, 128.0), 0);
    ///
    /// assert_eq!(sprite.size, v2!(128.0, 128.0));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `index` is past the last frame.
    ///
    /// # See also
    ///
    /// * [`SpriteSheet::sprite()`]
    #[inline]
    pub fn sprite_sized(self, pos: Vector2, size: Vector2, index: u32) -> Sprite {
        Sprite::new(pos, size).with_source(self.frame(index))
    }
}
