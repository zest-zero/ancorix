//! Vulkan backend for Ancorix.
//!
//! Wraps `ash` with a minimal, explicit API. Nothing is hidden behind
//! abstraction layers - if you need to call a raw Vulkan function, every
//! handle is accessible through the `raw()` methods.

//#![allow(unused)]

pub mod commands;
pub mod descriptor;
pub mod device;
pub mod gpu_timer;
pub mod instance;
pub mod memory;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod surface;
pub mod swapchain;
pub mod sync;
pub mod texture;

pub use commands::{Commands, DrawBatch};
pub use descriptor::TextureDescriptorLayout;
pub use device::Device;
pub use gpu_timer::GpuTimer;
pub use instance::Instance;
pub use memory::Buffer;
pub use pipeline::Pipeline;
pub use render_pass::RenderPass;
pub use shader::Shader;
pub use surface::Surface;
pub use swapchain::Swapchain;
pub use sync::{FRAMES_IN_FLIGHT, FrameSync};
pub use texture::{Texture, TextureFilter};
