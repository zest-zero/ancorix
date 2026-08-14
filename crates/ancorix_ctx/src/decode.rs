pub(crate) struct DecodedImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<u8>,
}

pub(crate) fn decode(bytes: &[u8]) -> Result<DecodedImage, Box<str>> {
    let image = image::load_from_memory(bytes).map_err(|err| err.to_string().into_boxed_str())?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();

    Ok(DecodedImage {
        width,
        height,
        pixels: rgba.into_raw(),
    })
}
