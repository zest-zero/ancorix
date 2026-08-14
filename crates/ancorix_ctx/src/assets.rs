use ancorix_ash::{Device, Instance, Texture, TextureFilter};
use ancorix_asset::Handle;
use ancorix_math::Vector2;
use ancorix_text::Font;

/// Loads resources and hands back a [`Handle<T>`] to query them by later.
///
/// Borrows the live GPU device for the duration of one [`crate::Ctx`], the
/// same way [`crate::Ctx::draw`] borrows the frame's draw queue - never
/// stored in a struct across frames, see [`crate::Ctx`]'s own doc comment.
pub struct Assets<'a> {
    instance: &'a Instance,
    device: &'a Device,
    textures: &'a mut ancorix_asset::Assets<Texture>,

    #[cfg(feature = "unstable_shaders")]
    shaders: &'a mut ancorix_asset::Assets<ancorix_shader::Shader>,
}

impl<'a> Assets<'a> {
    #[inline]
    pub(crate) fn new(
        instance: &'a Instance,
        device: &'a Device,
        textures: &'a mut ancorix_asset::Assets<Texture>,
        #[cfg(feature = "unstable_shaders")] shaders: &'a mut ancorix_asset::Assets<
            ancorix_shader::Shader,
        >,
    ) -> Self {
        Self {
            instance,
            device,
            textures,
            #[cfg(feature = "unstable_shaders")]
            shaders,
        }
    }

    /// Registers a compiled fragment shader, to be drawn with through
    /// [`ancorix_draw::Draw::with_shader()`].
    ///
    /// Only the fragment stage - the engine pairs it with its own vertex
    /// stage, so a shader is one `pixel` function and nothing else.
    ///
    /// Ancorix doesn't compile shader source - the bytes come from
    /// `glslangValidator`, `slangc`, or any other SPIR-V producer, usually
    /// through `include_bytes!`.
    ///
    /// # Panics
    ///
    /// Panics if the slice isn't SPIR-V.
    #[cfg(feature = "unstable_shaders")]
    pub fn shader(&mut self, fragment: &[u8]) -> Handle<ancorix_shader::Shader> {
        self.shaders.insert(ancorix_shader::Shader::new(fragment))
    }

    /// Decodes `bytes` (a whole PNG or JPEG file) and uploads it to a new
    /// GPU texture, smoothly filtered.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` can't be decoded as a supported image format, or
    /// if the GPU upload fails.
    ///
    /// # See also
    ///
    /// * [`Assets::texture_ex()`]
    pub fn texture(&mut self, bytes: &[u8]) -> Handle<Texture> {
        self.texture_ex(bytes, TextureFilter::Linear)
    }

    /// Decodes `bytes` (a whole PNG or JPEG file) and uploads it to a new
    /// GPU texture sampled with `filter` - [`TextureFilter::Nearest`] for
    /// pixel art, and for tightly packed sprite sheets, whose frames
    /// otherwise bleed into each other at their edges.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` can't be decoded as a supported image format, or
    /// if the GPU upload fails.
    ///
    /// # See also
    ///
    /// * [`Assets::texture()`]
    pub fn texture_ex(&mut self, bytes: &[u8], filter: TextureFilter) -> Handle<Texture> {
        let decoded = crate::decode::decode(bytes).expect("failed to decode image");
        let texture = Texture::new(
            self.instance,
            self.device,
            &decoded.pixels,
            decoded.width,
            decoded.height,
            filter,
        );
        self.textures.insert(texture)
    }

    /// Rasterizes `bytes` (a whole TTF or OTF file) at `size` pixels tall
    /// and uploads its glyph atlas, using the default charset (printable
    /// ASCII plus Cyrillic).
    ///
    /// # Panics
    ///
    /// Panics if `bytes` isn't a readable font, if `size` isn't positive,
    /// or if the glyphs don't fit the largest atlas.
    ///
    /// # See also
    ///
    /// * [`Assets::font_ex()`]
    #[cfg(feature = "ttf")]
    pub fn font(&mut self, bytes: &[u8], size: f32) -> Font {
        self.font_ex(bytes, size, ancorix_text::charset::DEFAULT)
    }

    /// Rasterizes `bytes` at `size` pixels tall, baking only `charset` into
    /// the atlas - a character left out of it is skipped when drawn.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` isn't a readable font, if `size` isn't positive,
    /// or if the glyphs don't fit the largest atlas.
    ///
    /// # See also
    ///
    /// * [`Assets::font()`]
    #[cfg(feature = "ttf")]
    pub fn font_ex(&mut self, bytes: &[u8], size: f32, charset: &str) -> Font {
        let rasterized = ancorix_text::rasterize(bytes, size, charset);

        // Linear, not Nearest: glyphs land on fractional pixel positions,
        // and the 1px padding between them already prevents atlas bleed.
        let texture = Texture::new(
            self.instance,
            self.device,
            &rasterized.pixels,
            rasterized.width,
            rasterized.height,
            TextureFilter::Linear,
        );

        Font::new(rasterized, self.textures.insert(texture), size)
    }

    /// Builds the engine's own bitmap font, every pixel blown up `scale`
    /// times - a debug font that needs no font file at all.
    ///
    /// Its glyphs are hand-drawn on a 5x7 grid and cover printable ASCII
    /// only. For real typography load a TTF with [`Assets::font()`].
    ///
    /// # Panics
    ///
    /// Panics if `scale` is zero.
    ///
    /// # See also
    ///
    /// * [`Assets::font()`]
    pub fn builtin_font(&mut self, scale: u32) -> Font {
        let rasterized = ancorix_text::builtin(scale);

        // Nearest, and only whole scales: anything else resamples a bitmap
        // font into mush.
        let texture = Texture::new(
            self.instance,
            self.device,
            &rasterized.pixels,
            rasterized.width,
            rasterized.height,
            TextureFilter::Nearest,
        );
        let size = rasterized.line_height;

        Font::new(rasterized, self.textures.insert(texture), size)
    }

    /// Returns the pixel size of the texture `handle` points to - the size
    /// a [`ancorix_draw::SpriteSheet`] divides into frames.
    ///
    /// # Panics
    ///
    /// Panics if `handle` came from a different registry, the same way
    /// drawing a sprite with it would.
    pub fn texture_size(&self, handle: Handle<Texture>) -> Vector2 {
        let (width, height) = self
            .textures
            .get(handle)
            .expect("Handle<Texture> doesn't belong to this Assets registry")
            .size();

        Vector2::new(width as f32, height as f32)
    }
}
