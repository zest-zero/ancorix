use ancorix_math::Vector2;

use crate::builtin_data::{ADVANCE, BASELINE, CELL_HEIGHT, CELL_WIDTH, CHARS, GLYPHS};
use crate::glyph::Glyph;
use crate::rasterized::Rasterized;

/// Transparent pixels kept between packed cells, for the same reason
/// [`crate::raster`] keeps them - a filtered sample at a glyph's edge must
/// not reach into its neighbour.
const PADDING: u32 = 1;

/// Atlas sizes tried in order - the first the grid fits in wins.
const CANDIDATE_SIZES: [u32; 5] = [64, 128, 256, 512, 1024];

/// Expands the built-in bitmap font into an atlas, every pixel blown up
/// `scale` times.
///
/// The glyphs are hand-drawn on a 5x7 grid, so `scale` is a whole number:
/// a fractional one would land pixel edges between texels and blur exactly
/// the crispness a bitmap font exists for. Draw the result with
/// `TextureFilter::Nearest` for the same reason.
///
/// # Panics
///
/// Panics if `scale` is zero, or if the atlas would exceed 1024x1024
/// (`scale` above 13 or so).
pub fn builtin(scale: u32) -> Rasterized {
    assert!(scale > 0, "built-in font scale must be at least 1");

    let (cell_w, cell_h) = (CELL_WIDTH * scale, CELL_HEIGHT * scale);
    let count = GLYPHS.len() as u32;

    let (size, columns) = CANDIDATE_SIZES
        .iter()
        .find_map(|&size| {
            let columns = size / (cell_w + PADDING);
            let rows = count.div_ceil(columns.max(1));
            (columns > 0 && rows * (cell_h + PADDING) <= size).then_some((size, columns))
        })
        .expect("built-in font atlas doesn't fit at this scale");

    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let mut glyphs = Vec::with_capacity(GLYPHS.len());

    for (index, ch) in CHARS.chars().enumerate() {
        let index = index as u32;
        let x = (index % columns) * (cell_w + PADDING);
        let y = (index / columns) * (cell_h + PADDING);

        blit(&mut pixels, size, &GLYPHS[index as usize], scale, x, y);

        glyphs.push((
            ch,
            Glyph {
                atlas_pos: Vector2::new(x as f32, y as f32),
                atlas_size: Vector2::new(cell_w as f32, cell_h as f32),
                // the pen sits on the baseline, and the cell's top edge is
                // `BASELINE` rows above it
                offset: Vector2::new(0.0, -((BASELINE * scale) as f32)),
                advance: (ADVANCE * scale) as f32,
            },
        ));
    }

    glyphs.sort_unstable_by_key(|(ch, _)| *ch);

    Rasterized {
        glyphs,
        pixels,
        width: size,
        height: size,
        line_height: cell_h as f32,
        ascent: (BASELINE * scale) as f32,
    }
}

// Expands one glyph's packed bits into `scale` x `scale` blocks of white
// with either full or zero alpha - a bitmap font has no partial coverage.
fn blit(pixels: &mut [u8], atlas_size: u32, rows: &[u8; 10], scale: u32, x: u32, y: u32) {
    for cell_row in 0..CELL_HEIGHT {
        // row 0 is the gap above the caps and row 11 the one below the
        // descenders; only rows 1..=10 are stored
        let Some(bits) = cell_row
            .checked_sub(1)
            .and_then(|stored| rows.get(stored as usize))
        else {
            continue;
        };

        for cell_column in 0..CELL_WIDTH {
            if bits & (0x80 >> cell_column) == 0 {
                continue;
            }

            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x + cell_column * scale + dx;
                    let py = y + cell_row * scale + dy;
                    let offset = ((py * atlas_size + px) * 4) as usize;

                    pixels[offset] = 255;
                    pixels[offset + 1] = 255;
                    pixels[offset + 2] = 255;
                    pixels[offset + 3] = 255;
                }
            }
        }
    }
}
