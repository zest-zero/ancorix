//! Batches `ancorix_draw`'s immediate-mode `DrawCmd`s into GPU-ready
//! vertex/index data and renders them through three pipelines: solid-color
//! for rect/triangle/line/clear, instanced SDF for rounded rects and circles
//! (a circle is a rounded rect whose corner radius fills its half-size), and
//! textured quads for sprites and text.
//!
//! The Slang sources live in `shaders/`; `build.rs` compiles them with
//! `slangc` into `OUT_DIR`, falling back to the committed
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
pub use vertex::{RoundedRectInstance, ShadedVertex, SpriteVertex, Vertex};
