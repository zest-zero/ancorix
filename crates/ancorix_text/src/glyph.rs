use ancorix_math::Vector2;

/// One rasterized character: where it sits in the atlas, where it sits
/// relative to the pen, and how far the pen moves past it.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Glyph {
    /// Top-left corner of the glyph inside the atlas texture, in pixels.
    pub atlas_pos: Vector2,
    /// Size of the glyph inside the atlas texture, in pixels. Zero for a
    /// blank character such as a space.
    pub atlas_size: Vector2,
    /// Offset from the pen position (on the baseline) to the top-left
    /// corner the glyph should be drawn at.
    pub offset: Vector2,
    /// How far the pen advances horizontally after drawing this glyph.
    pub advance: f32,
}
