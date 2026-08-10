//! Batches `ancorix_draw`'s immediate-mode `DrawCmd`s into GPU-ready
//! vertex/index data and renders them through four pipelines: solid-color
//! for rect/triangle/line/clear, SDF for circles, instanced SDF for rounded
//! rects, and textured quads for sprites and text.
//!
//! The GLSL sources live in `shaders/`; `build.rs` compiles them with
//! `glslangValidator` into `OUT_DIR`, falling back to the committed
//! `shaders/prebuilt/*.spv` when the compiler isn't installed.

mod batch;
mod buffers;
pub mod geometry;
mod pipelines;
pub mod renderer;
pub mod vertex;

pub use batch::FrameBatch;
pub use geometry::{DrawRun, Geometry, RunKind, triangulate};
pub use renderer::Renderer;
pub use vertex::{RoundedRectInstance, SdfVertex, ShadedVertex, SpriteVertex, Vertex};
