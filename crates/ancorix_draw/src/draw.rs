use ancorix_ash::Texture;
use ancorix_asset::Handle;
use ancorix_color::Rgba;
use ancorix_math::Vector2;
use ancorix_text::Font;

use crate::{
    cmd::DrawCmd,
    primitive::{Circle, Line, Rect, RoundedRect, Sprite, Triangle},
    transform::Transform2D,
};

const DEFAULT_CAPACITY: usize = 256;

/// Accepts immediate-mode draw commands for the current frame.
///
/// Commands are queued and flushed to the GPU once per frame by the
/// render backend. Nothing is drawn until the frame ends.
///
/// Use plain methods (`rect`, `circle`) for the common case with no
/// transformation. Use `_ex` variants (`rect_ex`, `circle_ex`) when
/// rotation, custom origin, or scaling is needed.
#[doc(alias = "canvas")]
#[doc(alias = "painter")]
#[doc(alias = "graphics")]
#[doc(alias = "renderer")]
pub struct Draw {
    pub(crate) cmds: Vec<DrawCmd>,

    // the clip in effect right now, so a nested region can intersect with it
    clip: Option<Rect>,

    #[cfg(feature = "unstable_shaders")]
    shaded: bool,

    // every shader parameter queued this frame, back to back. Reused across
    // frames like `cmds`, so a frame of shaders allocates nothing.
    #[cfg(feature = "unstable_shaders")]
    pub(crate) params: Vec<u8>,
}

impl Draw {
    /// Creates a new [`Draw`] instance with a default capacity.
    #[inline]
    pub fn new() -> Self {
        Self {
            cmds: Vec::with_capacity(DEFAULT_CAPACITY),
            clip: None,
            #[cfg(feature = "unstable_shaders")]
            shaded: false,
            #[cfg(feature = "unstable_shaders")]
            params: Vec::new(),
        }
    }

    /// Clips everything queued inside `region` to `rect`, in pixels.
    ///
    /// Nested regions intersect rather than replace, so a child can only ever
    /// shrink what its parent allowed. The clip ends with the closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use ancorix_draw::{Draw, Rect};
    /// use ancorix_color::Rgba;
    /// use ancorix_math::v2;
    ///
    /// let mut draw = Draw::new();
    /// let panel = Rect::new(v2!(20.0, 20.0), v2!(200.0, 100.0));
    ///
    /// draw.with_clip(panel, |d| {
    ///     // the parts of this row outside `panel` never reach the screen
    ///     d.rect(Rect::new(v2!(0.0, 40.0), v2!(400.0, 30.0)), Rgba::WHITE);
    /// });
    ///
    /// assert_eq!(draw.commands().len(), 3); // clip on, rect, clip off
    /// ```
    pub fn with_clip(&mut self, rect: Rect, region: impl FnOnce(&mut Draw)) {
        let outer = self.clip;
        // a child region can only shrink what its parent allowed
        let inner = match outer {
            Some(parent) => parent.intersection(rect),
            None => Some(rect),
        };

        self.clip = inner;
        self.cmds.push(DrawCmd::SetClip(inner));

        region(self);

        self.clip = outer;
        self.cmds.push(DrawCmd::SetClip(outer));
    }

    /// Draws `shape` with `shader` deciding every pixel inside it, tinted by
    /// `color` (the shader receives it as `s.color`).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ancorix_draw::{Draw, Rect};
    /// # use ancorix_color::Rgba;
    /// # use ancorix_math::v2;
    /// # fn demo(draw: &mut Draw, donut: ancorix_asset::Handle<ancorix_shader::Shader>) {
    /// draw.shader(donut, Rect::new(v2!(0.0), v2!(320.0)), Rgba::CYAN);
    /// # }
    /// ```
    ///
    /// # See also
    ///
    /// * [`Draw::shader_with()`]
    /// * [`Draw::shader_ex()`]
    /// * [`Draw::with_shader()`]
    #[cfg(feature = "unstable_shaders")]
    #[inline]
    pub fn shader(&mut self, shader: Handle<ancorix_shader::Shader>, shape: Rect, color: Rgba) {
        self.shader_ex(shader, shape, Transform2D::IDENTITY, color, &());
    }

    /// Draws `shape` with `shader`, handing it `params` on top of `color`.
    ///
    /// `params` is any `#[repr(C)]` plain-data struct; its bytes are pushed
    /// straight after the engine's own, where the shader's own push constant
    /// block picks them up. Nothing checks that the two agree - lay the
    /// struct out exactly as the shader declares it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ancorix_draw::{Draw, Rect};
    /// # use ancorix_color::Rgba;
    /// # use ancorix_math::v2;
    /// #[repr(C)]
    /// #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    /// struct BarParams {
    ///     progress: f32,
    ///     _padding: [f32; 3],
    ///     fill: [f32; 4],
    /// }
    ///
    /// # fn demo(draw: &mut Draw, bar: ancorix_asset::Handle<ancorix_shader::Shader>) {
    /// draw.shader_with(
    ///     bar,
    ///     Rect::new(v2!(0.0), v2!(320.0, 24.0)),
    ///     Rgba::GREY,
    ///     &BarParams { progress: 0.62, _padding: [0.0; 3], fill: [0.1, 0.8, 0.3, 1.0] },
    /// );
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `params` is larger than the room left in the push constant
    /// budget (see [`Draw::MAX_SHADER_PARAMS`]).
    ///
    /// # See also
    ///
    /// * [`Draw::shader()`]
    #[cfg(feature = "unstable_shaders")]
    #[inline]
    pub fn shader_with<P: bytemuck::Pod>(
        &mut self,
        shader: Handle<ancorix_shader::Shader>,
        shape: Rect,
        color: Rgba,
        params: &P,
    ) {
        self.shader_ex(shader, shape, Transform2D::IDENTITY, color, params);
    }

    /// Draws `shape` with `shader`, a [`Transform2D`] and `params`.
    ///
    /// Pass `&()` for a shader that takes no parameters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ancorix_draw::{Draw, Rect, Transform2D};
    /// # use ancorix_color::Rgba;
    /// # use ancorix_math::v2;
    /// # fn demo(draw: &mut Draw, donut: ancorix_asset::Handle<ancorix_shader::Shader>) {
    /// let spin = Transform2D::rotated(0.5);
    /// draw.shader_ex(donut, Rect::new(v2!(0.0), v2!(320.0)), spin, Rgba::CYAN, &());
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `params` doesn't fit the push constant budget.
    ///
    /// # See also
    ///
    /// * [`Draw::shader()`]
    #[cfg(feature = "unstable_shaders")]
    pub fn shader_ex<P: bytemuck::Pod>(
        &mut self,
        shader: Handle<ancorix_shader::Shader>,
        shape: Rect,
        transform: Transform2D,
        color: Rgba,
        params: &P,
    ) {
        let slot = self.store_params(shader, params);

        self.cmds.push(DrawCmd::SetShader(Some(slot)));
        self.cmds.push(DrawCmd::Rect {
            shape,
            transform,
            color,
        });
        self.cmds.push(DrawCmd::SetShader(None));
    }

    /// Draws everything queued inside `region` with `shader` instead of the
    /// built-in solid pipeline.
    ///
    /// The region ends when the closure does, so a shader can never leak
    /// into the next frame the way a `set_shader()` switch would.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use ancorix_draw::{Draw, Rect};
    /// # use ancorix_color::Rgba;
    /// # use ancorix_math::v2;
    /// # fn demo(draw: &mut Draw, water: ancorix_asset::Handle<ancorix_shader::Shader>) {
    /// draw.with_shader(water, &(), |d| {
    ///     d.rect(Rect::new(v2!(0.0), v2!(320.0, 180.0)), Rgba::WHITE);
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the region queues a circle, rounded rect, sprite or text.
    /// Those have their own vertex formats and built-in fragment shaders, so
    /// a custom pipeline can't stand in for them yet - drawing them
    /// unshaded instead would silently ignore the region.
    ///
    /// # See also
    ///
    /// * [`Draw::shader()`]
    #[cfg(feature = "unstable_shaders")]
    pub fn with_shader<P: bytemuck::Pod>(
        &mut self,
        shader: Handle<ancorix_shader::Shader>,
        params: &P,
        region: impl FnOnce(&mut Draw),
    ) {
        let slot = self.store_params(shader, params);

        self.cmds.push(DrawCmd::SetShader(Some(slot)));
        self.shaded = true;

        region(self);

        self.shaded = false;
        self.cmds.push(DrawCmd::SetShader(None));
    }

    /// How many bytes of parameters one shader may take.
    ///
    /// What is left of the 128 bytes every Vulkan implementation guarantees,
    /// after the engine's own globals.
    #[cfg(feature = "unstable_shaders")]
    pub const MAX_SHADER_PARAMS: usize = 112;

    #[cfg(feature = "unstable_shaders")]
    #[track_caller]
    fn store_params<P: bytemuck::Pod>(
        &mut self,
        shader: Handle<ancorix_shader::Shader>,
        params: &P,
    ) -> crate::cmd::ShaderSlot {
        let bytes = bytemuck::bytes_of(params);
        assert!(
            bytes.len() <= Self::MAX_SHADER_PARAMS,
            "shader parameters are {} bytes, and only {} fit in the push \
             constant budget",
            bytes.len(),
            Self::MAX_SHADER_PARAMS
        );

        let first_param = self.params.len() as u32;
        self.params.extend_from_slice(bytes);

        crate::cmd::ShaderSlot {
            shader,
            first_param,
            param_len: bytes.len() as u32,
        }
    }

    #[cfg(feature = "unstable_shaders")]
    #[track_caller]
    fn reject_in_shader_region(&self, what: &str) {
        assert!(
            !self.shaded,
            "{what} can't be drawn inside `with_shader` yet - only rect, \
             triangle, line and clear can (see `Draw::with_shader`)"
        );
    }

    #[cfg(not(feature = "unstable_shaders"))]
    #[inline(always)]
    fn reject_in_shader_region(&self, _what: &str) {}

    /// Clears the frame to `color`.
    #[inline]
    #[doc(alias = "background")]
    #[doc(alias = "cls")]
    pub fn clear(&mut self, color: Rgba) {
        self.cmds.push(DrawCmd::Clear(color));
    }

    /// Draws a filled [`Rect`] with no transformation.
    #[inline]
    pub fn rect(&mut self, shape: Rect, color: Rgba) {
        self.cmds.push(DrawCmd::Rect {
            shape,
            transform: Transform2D::IDENTITY,
            color,
        });
    }

    /// Draws a filled [`Rect`] with a [`Transform2D`].
    #[inline]
    pub fn rect_ex(&mut self, shape: Rect, transform: Transform2D, color: Rgba) {
        self.cmds.push(DrawCmd::Rect {
            shape,
            transform,
            color,
        });
    }

    /// Draws a filled [`RoundedRect`] with no transformation.
    #[inline]
    pub fn rounded_rect(&mut self, shape: RoundedRect, color: Rgba) {
        self.reject_in_shader_region("a rounded rect");
        self.cmds.push(DrawCmd::RoundedRect {
            shape,
            transform: Transform2D::IDENTITY,
            color,
        });
    }

    /// Draws a filled [`RoundedRect`] with a [`Transform2D`].
    #[inline]
    pub fn rounded_rect_ex(&mut self, shape: RoundedRect, transform: Transform2D, color: Rgba) {
        self.reject_in_shader_region("a rounded rect");
        self.cmds.push(DrawCmd::RoundedRect {
            shape,
            transform,
            color,
        });
    }

    /// Draws a filled [`Circle`] with no transformation.
    #[inline]
    pub fn circle(&mut self, shape: Circle, color: Rgba) {
        self.reject_in_shader_region("a circle");
        self.cmds.push(DrawCmd::Circle {
            shape,
            transform: Transform2D::IDENTITY,
            color,
        });
    }

    /// Draws a filled [`Circle`] with a [`Transform2D`].
    #[inline]
    pub fn circle_ex(&mut self, shape: Circle, transform: Transform2D, color: Rgba) {
        self.reject_in_shader_region("a circle");
        self.cmds.push(DrawCmd::Circle {
            shape,
            transform,
            color,
        });
    }

    /// Draws a [`Line`] between two points.
    #[inline]
    pub fn line(&mut self, shape: Line, color: Rgba) {
        self.cmds.push(DrawCmd::Line { shape, color });
    }

    /// Draws a filled [`Triangle`] with no transformation.
    #[inline]
    pub fn triangle(&mut self, shape: Triangle, color: Rgba) {
        self.cmds.push(DrawCmd::Triangle {
            shape,
            transform: Transform2D::IDENTITY,
            color,
        });
    }

    /// Draws a filled [`Triangle`] with a [`Transform2D`].
    #[inline]
    pub fn triangle_ex(&mut self, shape: Triangle, transform: Transform2D, color: Rgba) {
        self.cmds.push(DrawCmd::Triangle {
            shape,
            transform,
            color,
        });
    }

    /// Draws `handle` filling `shape`, at full opacity, no transformation.
    #[inline]
    #[doc(alias = "draw_texture")]
    #[doc(alias = "blit")]
    pub fn sprite(&mut self, handle: Handle<Texture>, shape: Sprite) {
        self.reject_in_shader_region("a sprite");
        self.cmds.push(DrawCmd::Sprite {
            shape,
            handle,
            transform: Transform2D::IDENTITY,
            tint: Rgba::WHITE,
        });
    }

    /// Draws `handle` filling `shape`, with a [`Transform2D`] and a `tint`
    /// color multiplied into the sampled texel.
    #[inline]
    pub fn sprite_ex(
        &mut self,
        handle: Handle<Texture>,
        shape: Sprite,
        transform: Transform2D,
        tint: Rgba,
    ) {
        self.reject_in_shader_region("a sprite");
        self.cmds.push(DrawCmd::Sprite {
            shape,
            handle,
            transform,
            tint,
        });
    }

    /// Draws `text` in `color`, with the top-left corner of its first line
    /// at `pos`. `\n` starts a new line.
    ///
    /// Every glyph is queued as a sprite from the font's atlas texture, so
    /// a whole string is one texture run and costs a single draw call.
    #[doc(alias = "draw_text")]
    #[doc(alias = "print")]
    #[doc(alias = "label")]
    pub fn text(&mut self, font: &Font, text: &str, pos: Vector2, color: Rgba) {
        self.reject_in_shader_region("text");
        for glyph in font.layout(text, pos) {
            self.cmds.push(DrawCmd::Sprite {
                shape: Sprite::new(glyph.pos, glyph.size)
                    .with_source(Rect::new(glyph.atlas_pos, glyph.atlas_size)),
                handle: font.texture(),
                transform: Transform2D::IDENTITY,
                // the atlas is white with coverage in alpha, so multiplying
                // by `color` is exactly antialiased, tinted text
                tint: color,
            });
        }
    }

    /// Draws `text` in `color` with a [`Transform2D`], pivoting the string
    /// as one piece rather than spinning each letter where it stands.
    ///
    /// [`Transform2D::origin`] is read against the whole string's box, so
    /// `origin: (0.5, 0.5)` turns the line about its own centre.
    pub fn text_ex(
        &mut self,
        font: &Font,
        text: &str,
        pos: Vector2,
        color: Rgba,
        transform: Transform2D,
    ) {
        self.reject_in_shader_region("text");
        // Every glyph is its own sprite, and a sprite's transform pivots
        // around that sprite's box - handing each glyph `transform` unchanged
        // would rotate each letter about itself. `origin` is normalized
        // within the box it's applied to but nothing restricts it to 0..1, so
        // each glyph gets the origin that resolves to the string's pivot:
        // `glyph.pos + origin * glyph.size == pivot`.
        let pivot = pos + transform.origin * font.measure(text);

        for glyph in font.layout(text, pos) {
            self.cmds.push(DrawCmd::Sprite {
                shape: Sprite::new(glyph.pos, glyph.size)
                    .with_source(Rect::new(glyph.atlas_pos, glyph.atlas_size)),
                handle: font.texture(),
                transform: Transform2D {
                    rotation: transform.rotation,
                    origin: (pivot - glyph.pos) / glyph.size,
                    scale: transform.scale,
                },
                tint: color,
            });
        }
    }

    /// Returns the queued commands for this frame.
    ///
    /// Called by the render backend before flushing to the GPU.
    #[doc(hidden)]
    #[inline]
    pub fn commands(&self) -> &[DrawCmd] {
        &self.cmds
    }

    /// Returns the shader parameter bytes queued this frame, indexed by
    /// [`crate::ShaderSlot`].
    ///
    /// Called by the render backend, alongside [`Draw::commands()`].
    #[doc(hidden)]
    #[cfg(feature = "unstable_shaders")]
    #[inline]
    pub fn shader_params(&self) -> &[u8] {
        &self.params
    }

    /// Clears the command queue after the GPU flush.
    ///
    /// Called by the render backend after submitting commands.
    #[doc(hidden)]
    #[inline]
    pub fn flush(&mut self) {
        self.cmds.clear();
        #[cfg(feature = "unstable_shaders")]
        self.params.clear();
    }
}

impl Default for Draw {
    fn default() -> Self {
        Self::new()
    }
}
