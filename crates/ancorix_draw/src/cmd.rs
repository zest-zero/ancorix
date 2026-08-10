use ancorix_ash::Texture;
use ancorix_asset::Handle;
use ancorix_color::Rgba;

use crate::{
    primitive::{Circle, Line, Rect, RoundedRect, Sprite, Triangle},
    transform::Transform2D,
};

/// A user shader, plus where its parameters sit in [`crate::Draw`]'s
/// parameter buffer.
///
/// The bytes live in one buffer rather than in the command, so that a command
/// stays `Copy` and the same size no matter how much a shader takes.
#[doc(hidden)]
#[cfg(feature = "unstable_shaders")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShaderSlot {
    pub shader: Handle<ancorix_shader::Shader>,
    pub first_param: u32,
    pub param_len: u32,
}

/// A single draw command queued for the current frame.
///
/// Flushed to the GPU at the end of each frame by the render backend.
/// The backend sorts and batches these before issuing Vulkan draw calls.
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum DrawCmd {
    Clear(Rgba),

    /// Restricts every following command to a rectangle, until the next
    /// one - `None` restores the whole framebuffer. Queued by
    /// [`crate::Draw::with_clip()`], never written by hand.
    SetClip(Option<Rect>),

    /// Switches the pipeline every following command is drawn with, until
    /// the next one - `None` restores the built-in pipelines. Queued by
    /// [`crate::Draw::with_shader()`], never written by hand.
    #[cfg(feature = "unstable_shaders")]
    SetShader(Option<ShaderSlot>),

    Rect {
        shape: Rect,
        transform: Transform2D,
        color: Rgba,
    },

    RoundedRect {
        shape: RoundedRect,
        transform: Transform2D,
        color: Rgba,
    },

    Circle {
        shape: Circle,
        transform: Transform2D,
        color: Rgba,
    },

    Line {
        shape: Line,
        color: Rgba,
    },

    Triangle {
        shape: Triangle,
        transform: Transform2D,
        color: Rgba,
    },

    Sprite {
        shape: Sprite,
        handle: Handle<Texture>,
        transform: Transform2D,
        tint: Rgba,
    },
}
