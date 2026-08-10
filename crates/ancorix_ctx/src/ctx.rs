use crate::{Time, WindowInfo};
use ancorix_draw::Draw;
use ancorix_input::Input;

/// Per-frame context passed to [`App::init()`](crate::App::init) and
/// [`App::frame()`](crate::App::frame).
///
/// Provides access to input state, frame timing, window control,
/// immediate-mode drawing, and resource loading. Never store a `Ctx` in
/// your own struct - it only lives for the duration of one `init` or
/// `frame` call.
#[doc(alias = "context")]
#[doc(alias = "state")]
pub struct Ctx<'a> {
    /// Keyboard and mouse state for this frame, plus any registered
    /// virtual actions.
    pub input: &'a mut Input,

    /// Timing information for this frame.
    pub time: Time,

    /// Window size and close control.
    pub window: WindowInfo,

    /// Immediate-mode draw command queue for this frame.
    pub draw: &'a mut Draw,

    /// Loads resources (textures, ...) and hands back a [`ancorix_asset::Handle`].
    pub assets: crate::assets::Assets<'a>,
}

impl<'a> Ctx<'a> {
    /// Returns a new [`Ctx`] borrowing `input`, `draw`, and `assets`, with
    /// the given `time` and `window` state.
    ///
    /// No `# Examples` - `instance`/`device` need a real Vulkan-capable
    /// display connection, which can't be constructed outside of a real
    /// windowing backend (same reason as `ancorix_winit::feed_event`); in
    /// practice this is called from `ancorix_window::Runner`.
    #[doc(hidden)]
    #[inline]
    #[allow(clippy::too_many_arguments)] // each param is a distinct borrowed
    // resource `Ctx` composes for the frame, a wrapper struct wouldn't add
    // safety (nothing here is two of the same type that could be swapped).
    pub fn new(
        input: &'a mut Input,
        time: Time,
        window: WindowInfo,
        draw: &'a mut Draw,
        instance: &'a ancorix_ash::Instance,
        device: &'a ancorix_ash::Device,
        textures: &'a mut ancorix_asset::Assets<ancorix_ash::Texture>,
        #[cfg(feature = "unstable_shaders")] shaders: &'a mut ancorix_asset::Assets<
            ancorix_shader::Shader,
        >,
    ) -> Self {
        Self {
            input,
            time,
            window,
            draw,
            assets: crate::assets::Assets::new(
                instance,
                device,
                textures,
                #[cfg(feature = "unstable_shaders")]
                shaders,
            ),
        }
    }
}
