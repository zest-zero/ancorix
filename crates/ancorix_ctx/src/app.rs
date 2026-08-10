use crate::Ctx;

/// An application: state you own, and what to draw with it each frame.
///
/// This is the only trait you have to implement. [`init()`](App::init) builds
/// your state once, [`frame()`](App::frame) is called for every frame after
/// that, and there is no loop to write - `Window::run` runs it for you.
///
/// # Examples
///
/// ```
/// use ancorix_ctx::{App, Ctx};
///
/// struct Counter {
///     frames: u32,
/// }
///
/// impl App for Counter {
///     fn init(_ctx: &mut Ctx) -> Self {
///         Self { frames: 0 }
///     }
///
///     fn frame(&mut self, ctx: &mut Ctx) {
///         self.frames += 1;
///         let _ = ctx.time.dt();
///     }
/// }
/// ```
#[doc(alias = "application")]
#[doc(alias = "game")]
#[doc(alias = "update")]
pub trait App: Sized {
    /// Builds the application state.
    ///
    /// Called once, before the first frame, with a [`Ctx`] whose window
    /// already has its final size - so measurements taken here are the ones
    /// the first frame will draw against. Load textures, fonts and shaders
    /// here rather than in [`frame()`](App::frame), where they would be
    /// loaded again every frame.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ancorix_ctx::{App, Ctx};
    /// struct Game {
    ///     center: [f32; 2],
    /// }
    ///
    /// impl App for Game {
    ///     fn init(ctx: &mut Ctx) -> Self {
    ///         let size = ctx.window.size();
    ///         Self { center: [size.x / 2.0, size.y / 2.0] }
    ///     }
    /// #   fn frame(&mut self, _ctx: &mut Ctx) {}
    /// }
    /// ```
    fn init(ctx: &mut Ctx) -> Self;

    /// Advances and draws one frame.
    ///
    /// Nothing is kept from the previous frame: whatever this method draws is
    /// exactly what appears on the screen, in the order it was drawn. Use
    /// [`Ctx::time`] for how long the last frame took, so that movement does
    /// not depend on the frame rate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ancorix_ctx::{App, Ctx};
    /// # struct Game { x: f32 }
    /// impl App for Game {
    /// #   fn init(_ctx: &mut Ctx) -> Self { Self { x: 0.0 } }
    ///     fn frame(&mut self, ctx: &mut Ctx) {
    ///         self.x += 200.0 * ctx.time.dt();
    ///     }
    /// }
    /// ```
    fn frame(&mut self, ctx: &mut Ctx);
}
