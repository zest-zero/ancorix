use ancorix_ash::{Device, Pipeline, RenderPass, Shader, TextureDescriptorLayout};
use ash::vk;

#[cfg(feature = "unstable_shaders")]
use crate::vertex::ShadedVertex;
use crate::vertex::{RoundedRectInstance, SpriteVertex, Vertex};

// Compiled by `build.rs` from `shaders/` into OUT_DIR - either with
// glslangValidator, or copied from `shaders/prebuilt` when it isn't
// installed. Never read from the source tree directly.
macro_rules! spirv {
    ($name:literal) => {
        include_bytes!(concat!(env!("OUT_DIR"), "/", $name, ".spv"))
    };
}

// `Globals`: `screen_size` plus `time`, padded to 16 by Slang's constant
// buffer layout. Every pipeline starts its push constants with this.
pub(crate) const GLOBALS_SIZE: u32 = size_of::<[f32; 4]>() as u32;

// The push constant budget a user pipeline declares: the minimum every
// Vulkan implementation must offer, so a shader that runs here runs
// anywhere. A pipeline may declare more room than its shader reads.
#[cfg(feature = "unstable_shaders")]
const USER_BUDGET: u32 = 128;

// Visible to both stages: the built-in shaders read the globals while
// positioning vertices, a user's `pixel` reads its parameters per fragment.
const fn push_constant_range(size: u32) -> vk::PushConstantRange {
    vk::PushConstantRange {
        stage_flags: vk::ShaderStageFlags::from_raw(
            vk::ShaderStageFlags::VERTEX.as_raw() | vk::ShaderStageFlags::FRAGMENT.as_raw(),
        ),
        offset: 0,
        size,
    }
}

const PUSH_CONSTANT_RANGES: [vk::PushConstantRange; 1] = [push_constant_range(GLOBALS_SIZE)];

// Must be dropped before `Device`; `texture_descriptors` is declared last so
// it outlives the pipelines built against it.
pub(crate) struct Pipelines {
    pub(crate) solid: Pipeline,
    pub(crate) rounded_rect: Pipeline,
    pub(crate) sprite: Pipeline,
    pub(crate) texture_descriptors: TextureDescriptorLayout,
}

impl Pipelines {
    pub(crate) fn new(device: &Device, render_pass: &RenderPass) -> Self {
        let solid = build(
            device,
            render_pass,
            spirv!("solid2d.vert"),
            spirv!("solid2d.frag"),
            std::slice::from_ref(&Vertex::BINDING),
            &Vertex::ATTRIBUTES,
            &[],
        );

        let rounded_rect = build(
            device,
            render_pass,
            spirv!("roundedrect2d.vert"),
            spirv!("roundedrect2d.frag"),
            std::slice::from_ref(&RoundedRectInstance::BINDING),
            &RoundedRectInstance::ATTRIBUTES,
            &[],
        );

        let texture_descriptors = TextureDescriptorLayout::new(device);
        let sprite = build(
            device,
            render_pass,
            spirv!("sprite2d.vert"),
            spirv!("sprite2d.frag"),
            std::slice::from_ref(&SpriteVertex::BINDING),
            &SpriteVertex::ATTRIBUTES,
            std::slice::from_ref(&texture_descriptors.raw()),
        );

        Self {
            solid,
            rounded_rect,
            sprite,
            texture_descriptors,
        }
    }
}

// A user pipeline: the engine's vertex stage with their fragment shader, so
// writing one means writing `float4 pixel(Surface s)` and nothing else.
//
// Its push constant range is the whole 128-byte budget rather than just the
// globals: the shader's own parameters go after them, and how many bytes it
// declares is only known to the shader.
#[cfg(feature = "unstable_shaders")]
pub(crate) fn user_shaded(
    device: &Device,
    render_pass: &RenderPass,
    fragment_spirv: &[u8],
) -> Pipeline {
    let vertex = Shader::new(device, spirv!("shaded2d.vert"));
    let fragment = Shader::new(device, fragment_spirv);

    Pipeline::new(
        device,
        render_pass,
        &vertex,
        &fragment,
        std::slice::from_ref(&ShadedVertex::BINDING),
        &ShadedVertex::ATTRIBUTES,
        &[push_constant_range(USER_BUDGET)],
        &[],
    )
}

// The shader modules are only needed while the pipeline is being created, so
// they're built and dropped here rather than kept in `Pipelines`.
fn build(
    device: &Device,
    render_pass: &RenderPass,
    vertex_spirv: &[u8],
    fragment_spirv: &[u8],
    vertex_bindings: &[vk::VertexInputBindingDescription],
    vertex_attributes: &[vk::VertexInputAttributeDescription],
    descriptor_set_layouts: &[vk::DescriptorSetLayout],
) -> Pipeline {
    let vertex = Shader::new(device, vertex_spirv);
    let fragment = Shader::new(device, fragment_spirv);

    Pipeline::new(
        device,
        render_pass,
        &vertex,
        &fragment,
        vertex_bindings,
        vertex_attributes,
        &PUSH_CONSTANT_RANGES,
        descriptor_set_layouts,
    )
}
