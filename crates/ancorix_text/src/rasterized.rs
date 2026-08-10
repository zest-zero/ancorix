use crate::glyph::Glyph;

/// A font rasterized at one pixel size: every glyph's placement plus the
/// RGBA8 atlas holding them.
///
/// Every pixel is white with the glyph's coverage in alpha, so the ordinary
/// sprite shader's `texel * tint` already evaluates to correctly tinted,
/// antialiased text - no text-specific pipeline is involved.
pub struct Rasterized {
    /// Glyphs sorted by `char`, for binary search at layout time.
    pub glyphs: Vec<(char, Glyph)>,
    /// RGBA8 pixels, `width * height * 4` bytes.
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Baseline-to-baseline distance for one line of text.
    pub line_height: f32,
    /// Distance from the top of a line down to its baseline.
    pub ascent: f32,
}
