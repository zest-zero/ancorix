//! Everything a game touches, in one import.
//!
//! What a game reaches through `ctx` - the draw queue, the window, the clock -
//! isn't here: it arrives already borrowed.

pub use crate::{
    App, Binding, Circle, Ctx, Font, Handle, Interpolated, Key, Line, MouseButton, Rect, Rgba,
    RoundedRect, Sprite, SpriteSheet, Texture, TextureFilter, Transform2D, Triangle, Vector2,
    Window, rgba, v2,
};

#[cfg(feature = "sprite_animation")]
pub use crate::SpriteAnimation;

#[cfg(feature = "unstable_shaders")]
pub use crate::Shader;
