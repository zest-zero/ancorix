use ancorix_ash::Texture;
use ancorix_asset::Handle;
use ancorix_math::Vector2;
use rustc_hash::FxHashMap;

use crate::glyph::Glyph;
use crate::layout::Layout;
use crate::rasterized::Rasterized;

/// A font baked at one pixel size, together with the atlas texture holding
/// its glyphs.
///
/// Handed to the caller by value rather than behind a `Handle`, unlike a
/// texture: laying out a string needs the glyph metrics on the CPU at draw
/// time, and `Draw` has no access to the asset registry a handle would have
/// to be resolved through.
#[doc(alias = "typeface")]
#[doc(alias = "text")]
pub struct Font {
    // Hashed, not the sorted `Vec` `Rasterized` hands over: this is looked
    // up once per character of every string drawn every frame, and binary
    // search measured 3.75x slower over a realistic string - its branches
    // mispredict on every probe, the hash's don't (see `benches/`).
    glyphs: FxHashMap<char, Glyph>,
    line_height: f32,
    ascent: f32,
    size: f32,
    texture: Handle<Texture>,
}

impl Font {
    /// Pairs an already-uploaded atlas `texture` with the glyph placements
    /// [`crate::rasterize()`] produced for it.
    #[inline]
    pub fn new(rasterized: Rasterized, texture: Handle<Texture>, size: f32) -> Self {
        Self {
            glyphs: rasterized.glyphs.into_iter().collect(),
            line_height: rasterized.line_height,
            ascent: rasterized.ascent,
            size,
            texture,
        }
    }

    /// Returns the atlas texture every glyph of this font is drawn from.
    #[inline]
    pub fn texture(&self) -> Handle<Texture> {
        self.texture
    }

    /// Returns the pixel size this font was rasterized at. Drawing it at
    /// any other size resamples the atlas and softens the glyphs.
    #[inline]
    pub fn size(&self) -> f32 {
        self.size
    }

    /// Returns the baseline-to-baseline distance between two lines.
    #[inline]
    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    /// Returns the distance from the top of a line down to its baseline.
    #[inline]
    pub fn ascent(&self) -> f32 {
        self.ascent
    }

    /// Returns `ch`'s glyph, or `None` if it wasn't in the charset the font
    /// was rasterized with.
    #[inline]
    pub fn glyph(&self, ch: char) -> Option<Glyph> {
        self.glyphs.get(&ch).copied()
    }

    /// Returns the glyph quads of `text`, laid out with its top-left corner
    /// at `pos`. Characters missing from the font's charset are skipped, as
    /// are those with nothing to draw, such as spaces.
    #[inline]
    pub fn layout<'a>(&'a self, text: &'a str, pos: Vector2) -> Layout<'a> {
        Layout::new(self, text, pos)
    }

    /// Returns the size `text` occupies when laid out, counting `\n` as a
    /// line break.
    pub fn measure(&self, text: &str) -> Vector2 {
        let (mut widest, mut width, mut lines) = (0.0f32, 0.0f32, 1usize);

        for ch in text.chars() {
            if ch == '\n' {
                widest = widest.max(width);
                width = 0.0;
                lines += 1;
                continue;
            }

            if let Some(glyph) = self.glyph(ch) {
                width += glyph.advance;
            }
        }

        Vector2::new(widest.max(width), lines as f32 * self.line_height)
    }
}
