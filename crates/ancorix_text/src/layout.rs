use ancorix_math::Vector2;

use crate::font::Font;

/// One glyph placed on screen: where to draw it, and which part of the
/// font's atlas texture to take it from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacedGlyph {
    /// Top-left corner of the glyph on screen.
    pub pos: Vector2,
    /// On-screen size of the glyph.
    pub size: Vector2,
    /// Top-left corner of the glyph within the atlas texture, in pixels.
    pub atlas_pos: Vector2,
    /// Size of the glyph within the atlas texture, in pixels.
    pub atlas_size: Vector2,
}

/// Walks a string, advancing a pen and yielding one [`PlacedGlyph`] per
/// character that has something to draw.
///
/// Returned by [`Font::layout()`] - iterating costs no allocation, so a
/// string can be laid out fresh every frame.
pub struct Layout<'a> {
    font: &'a Font,
    chars: std::str::Chars<'a>,
    origin: Vector2,
    pen: Vector2,
}

impl<'a> Layout<'a> {
    #[inline]
    pub(crate) fn new(font: &'a Font, text: &'a str, pos: Vector2) -> Self {
        // `pos` is the top-left of the text block, so the first baseline
        // sits one ascent below it
        let pen = Vector2::new(pos.x, pos.y + font.ascent());

        Self {
            font,
            chars: text.chars(),
            origin: pos,
            pen,
        }
    }
}

impl Iterator for Layout<'_> {
    type Item = PlacedGlyph;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ch = self.chars.next()?;

            if ch == '\n' {
                self.pen.x = self.origin.x;
                self.pen.y += self.font.line_height();
                continue;
            }

            // a character outside the font's charset advances nothing and
            // draws nothing, rather than drawing a wrong glyph
            let Some(glyph) = self.font.glyph(ch) else {
                continue;
            };

            let pos = self.pen + glyph.offset;
            self.pen.x += glyph.advance;

            // spaces carry an advance but no bitmap - the pen has already
            // moved, so there is simply no quad to emit
            if glyph.atlas_size.x == 0.0 || glyph.atlas_size.y == 0.0 {
                continue;
            }

            return Some(PlacedGlyph {
                pos,
                size: glyph.atlas_size,
                atlas_pos: glyph.atlas_pos,
                atlas_size: glyph.atlas_size,
            });
        }
    }
}
