/// Decoded, uncompressed RGBA8 pixel data - the common format every
/// decoded image ends up in, regardless of the source file's own color
/// type or bit depth.
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    /// `width * height * 4` bytes, one `[r, g, b, a]` per pixel, row-major.
    pub pixels: Vec<u8>,
}

/// Decodes `bytes` (a whole PNG or JPEG file - format auto-detected from
/// content, not a file extension) into [`DecodedImage`].
///
/// # Examples
///
/// ```
/// use ancorix_image::decode;
/// use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
/// use std::io::Cursor;
///
/// // build a tiny PNG in memory instead of embedding real file bytes
/// let pixels: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_fn(2, 1, |x, _| {
///     if x == 0 {
///         Rgba([255, 0, 0, 255])
///     } else {
///         Rgba([0, 255, 0, 128])
///     }
/// });
/// let mut png_bytes = Vec::new();
/// DynamicImage::ImageRgba8(pixels)
///     .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
///     .unwrap();
///
/// let decoded = decode(&png_bytes).unwrap();
///
/// assert_eq!((decoded.width, decoded.height), (2, 1));
/// assert_eq!(&decoded.pixels[0..4], &[255, 0, 0, 255]);
/// assert_eq!(&decoded.pixels[4..8], &[0, 255, 0, 128]);
/// ```
///
/// # Panics
///
/// Doesn't panic - returns `Err` for malformed or unsupported input.
pub fn decode(bytes: &[u8]) -> Result<DecodedImage, Box<str>> {
    let image = image::load_from_memory(bytes).map_err(|err| err.to_string().into_boxed_str())?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(DecodedImage {
        width,
        height,
        pixels: rgba.into_raw(),
    })
}
