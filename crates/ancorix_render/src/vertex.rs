use ash::vk;

/// A single vertex of the batched solid-color 2D geometry.
// `color` is uploaded as `R8G8B8A8_UNORM`, normalized to a `vec4` by the GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub color: [u8; 4],
}

impl Vertex {
    /// Binding description, for `Pipeline`'s vertex input state.
    pub const BINDING: vk::VertexInputBindingDescription = vk::VertexInputBindingDescription {
        binding: 0,
        stride: size_of::<Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    };

    /// Attribute descriptions, for `Pipeline`'s vertex input state.
    pub const ATTRIBUTES: [vk::VertexInputAttributeDescription; 2] = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R8G8B8A8_UNORM,
            offset: size_of::<[f32; 2]>() as u32,
        },
    ];
}

/// A single vertex of a textured quad - `color` multiplies the sampled texel.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [u8; 4],
}

impl SpriteVertex {
    /// Binding description, for `Pipeline`'s vertex input state.
    pub const BINDING: vk::VertexInputBindingDescription = vk::VertexInputBindingDescription {
        binding: 0,
        stride: size_of::<SpriteVertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    };

    /// Attribute descriptions, for `Pipeline`'s vertex input state.
    pub const ATTRIBUTES: [vk::VertexInputAttributeDescription; 3] = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: size_of::<[f32; 2]>() as u32,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R8G8B8A8_UNORM,
            offset: (size_of::<[f32; 2]>() * 2) as u32,
        },
    ];
}

/// One rounded rect, sent to the GPU once per shape rather than once per
/// vertex.
// `roundedrect2d.vert` derives each corner from `gl_VertexIndex`, so nothing
// is repeated across the four of them - 44 bytes per shape instead of 152.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RoundedRectInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    /// `Transform2D::origin`, normalized within the shape's own box.
    pub origin: [f32; 2],
    pub scale: [f32; 2],
    pub radius: f32,
    pub rotation: f32,
    pub color: [u8; 4],
}

impl RoundedRectInstance {
    /// How many vertices the shader builds per instance.
    pub const VERTEX_COUNT: u32 = 6;

    /// Binding description, for `Pipeline`'s vertex input state.
    // Steps per instance, not per vertex - that is the whole point.
    pub const BINDING: vk::VertexInputBindingDescription = vk::VertexInputBindingDescription {
        binding: 0,
        stride: size_of::<RoundedRectInstance>() as u32,
        input_rate: vk::VertexInputRate::INSTANCE,
    };

    /// Attribute descriptions, for `Pipeline`'s vertex input state.
    pub const ATTRIBUTES: [vk::VertexInputAttributeDescription; 7] = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 8,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 16,
        },
        vk::VertexInputAttributeDescription {
            location: 3,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 24,
        },
        vk::VertexInputAttributeDescription {
            location: 4,
            binding: 0,
            format: vk::Format::R32_SFLOAT,
            offset: 32,
        },
        vk::VertexInputAttributeDescription {
            location: 5,
            binding: 0,
            format: vk::Format::R32_SFLOAT,
            offset: 36,
        },
        vk::VertexInputAttributeDescription {
            location: 6,
            binding: 0,
            format: vk::Format::R8G8B8A8_UNORM,
            offset: 40,
        },
    ];
}

/// A single vertex of geometry drawn with a user shader.
// Twice the size of `Vertex`, because a user's `pixel` function needs to know
// the shape it is inside: `uv` and `size` are what `Surface` is built from
// (`local` follows from them, so it isn't stored). Only geometry inside a
// `with_shader` region pays for this - plain rects keep the 12-byte format.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadedVertex {
    pub pos: [f32; 2],
    pub color: [u8; 4],
    pub uv: [f32; 2],
    pub size: [f32; 2],
    // Interpolated rather than read from the push constant, so that a user's
    // fragment shader never has to touch the engine's block - it needs that
    // one slot for parameters of its own.
    pub time: f32,
}

impl ShadedVertex {
    /// Binding description, for `Pipeline`'s vertex input state.
    pub const BINDING: vk::VertexInputBindingDescription = vk::VertexInputBindingDescription {
        binding: 0,
        stride: size_of::<ShadedVertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    };

    /// Attribute descriptions, for `Pipeline`'s vertex input state.
    pub const ATTRIBUTES: [vk::VertexInputAttributeDescription; 5] = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R8G8B8A8_UNORM,
            offset: 8,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 12,
        },
        vk::VertexInputAttributeDescription {
            location: 3,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 20,
        },
        vk::VertexInputAttributeDescription {
            location: 4,
            binding: 0,
            format: vk::Format::R32_SFLOAT,
            offset: 28,
        },
    ];
}
