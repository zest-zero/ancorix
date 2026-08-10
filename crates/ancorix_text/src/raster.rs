use ancorix_math::Vector2;

use crate::glyph::Glyph;
use crate::rasterized::Rasterized;

/// Transparent pixels kept between packed glyphs. One is enough: it stops a
/// `TextureFilter::Linear` sample at a glyph's edge from reaching into its
/// neighbour, the same bleed a tightly packed sprite sheet suffers.
const PADDING: u32 = 1;

/// Atlas sizes tried in order - the first one the glyphs fit in wins.
const CANDIDATE_SIZES: [u32; 6] = [128, 256, 512, 1024, 2048, 4096];

/// Rasterizes every character of `charset` at `px` pixels tall and packs
/// them into one atlas.
///
/// # Panics
///
/// Panics if `bytes` isn't a readable font, if `px` isn't positive, or if
/// the glyphs don't fit even the largest atlas size.
pub fn rasterize(bytes: &[u8], px: f32, charset: &str) -> Rasterized {
    assert!(px > 0.0, "font size must be positive");

    let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
        .unwrap_or_else(|e| panic!("failed to parse font: {e}"));

    let mut chars: Vec<char> = charset.chars().collect();
    chars.sort_unstable();
    chars.dedup();

    // rasterize first, pack second - packing needs every glyph's size, and
    // rasterizing twice would be wasted work
    let mut rasterized: Vec<(char, fontdue::Metrics, Vec<u8>)> = chars
        .into_iter()
        .map(|ch| {
            let (metrics, coverage) = font.rasterize(ch, px);
            (ch, metrics, coverage)
        })
        .collect();

    // tallest first, so shelves stay tightly packed instead of each one
    // being as tall as its single tallest straggler
    rasterized.sort_by_key(|(_, metrics, _)| std::cmp::Reverse(metrics.height));

    let (size, placements) = pack(&rasterized).expect("font atlas doesn't fit even at 4096x4096");

    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let mut glyphs = Vec::with_capacity(rasterized.len());

    for ((ch, metrics, coverage), (x, y)) in rasterized.iter().zip(placements) {
        blit(&mut pixels, size, coverage, metrics.width as u32, x, y);

        glyphs.push((
            *ch,
            Glyph {
                atlas_pos: Vector2::new(x as f32, y as f32),
                atlas_size: Vector2::new(metrics.width as f32, metrics.height as f32),
                // fontdue measures `ymin` up from the baseline to the
                // bitmap's bottom, and screen y grows downward, so the
                // quad's top sits `ymin + height` above the pen
                offset: Vector2::new(
                    metrics.xmin as f32,
                    -((metrics.ymin + metrics.height as i32) as f32),
                ),
                advance: metrics.advance_width,
            },
        ));
    }

    glyphs.sort_unstable_by_key(|(ch, _)| *ch);

    let line = font
        .horizontal_line_metrics(px)
        .expect("font has no horizontal line metrics");

    Rasterized {
        glyphs,
        pixels,
        width: size,
        height: size,
        line_height: line.new_line_size,
        ascent: line.ascent,
    }
}

// Shelf packing: walk glyphs (already tallest-first) left to right, drop to
// a new shelf when the row fills. Returns the atlas size and each glyph's
// top-left corner, in the order given.
fn pack(glyphs: &[(char, fontdue::Metrics, Vec<u8>)]) -> Option<(u32, Vec<(u32, u32)>)> {
    for &size in &CANDIDATE_SIZES {
        let mut placements = Vec::with_capacity(glyphs.len());
        let (mut pen_x, mut pen_y, mut shelf_height) = (0u32, 0u32, 0u32);
        let mut fits = true;

        for (_, metrics, _) in glyphs {
            let (w, h) = (metrics.width as u32, metrics.height as u32);

            if pen_x + w > size {
                pen_x = 0;
                pen_y += shelf_height + PADDING;
                shelf_height = 0;
            }
            if pen_y + h > size {
                fits = false;
                break;
            }

            placements.push((pen_x, pen_y));
            pen_x += w + PADDING;
            shelf_height = shelf_height.max(h);
        }

        if fits {
            return Some((size, placements));
        }
    }

    None
}

// Writes one glyph's coverage into the atlas as white with coverage alpha.
fn blit(pixels: &mut [u8], atlas_size: u32, coverage: &[u8], width: u32, x: u32, y: u32) {
    if width == 0 {
        return; // a space has metrics and an advance, but no bitmap
    }

    for (row, line) in coverage.chunks_exact(width as usize).enumerate() {
        let start = (((y + row as u32) * atlas_size + x) * 4) as usize;

        for (column, &alpha) in line.iter().enumerate() {
            let px = start + column * 4;
            pixels[px] = 255;
            pixels[px + 1] = 255;
            pixels[px + 2] = 255;
            pixels[px + 3] = alpha;
        }
    }
}
