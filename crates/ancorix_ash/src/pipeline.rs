use crate::device::Device;
use crate::render_pass::RenderPass;
use crate::shader::Shader;
use ash::vk;

/// One graphics pipeline drawing into a shared [`RenderPass`]. Must be
/// dropped before [`Device`].
pub struct Pipeline {
    raw: vk::Pipeline,
    layout: vk::PipelineLayout,
    device: ash::Device,
}

impl Pipeline {
    /// Creates a graphics pipeline for `vertex`/`fragment` drawing into
    /// `render_pass`, which it does not own.
    // Viewport and scissor are dynamic state, so a resize needs no rebuild.
    ///
    /// # Panics
    ///
    /// Panics if layout or pipeline creation fails.
    #[allow(clippy::too_many_arguments)] // no two args are the same type
    pub fn new(
        device: &Device,
        render_pass: &RenderPass,
        vertex: &Shader,
        fragment: &Shader,
        vertex_bindings: &[vk::VertexInputBindingDescription],
        vertex_attributes: &[vk::VertexInputAttributeDescription],
        push_constant_ranges: &[vk::PushConstantRange],
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> Self {
        let layout = Self::create_layout(device, push_constant_ranges, descriptor_set_layouts);
        let raw = Self::create_pipeline(
            device,
            render_pass.raw(),
            layout,
            vertex,
            fragment,
            vertex_bindings,
            vertex_attributes,
        );

        Self {
            raw,
            layout,
            device: device.raw().clone(),
        }
    }

    /// Returns the raw [`vk::Pipeline`].
    #[inline]
    pub fn raw(&self) -> vk::Pipeline {
        self.raw
    }

    /// Returns the raw [`vk::PipelineLayout`].
    #[inline]
    pub fn layout(&self) -> vk::PipelineLayout {
        self.layout
    }

    fn create_layout(
        device: &Device,
        push_constant_ranges: &[vk::PushConstantRange],
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .push_constant_ranges(push_constant_ranges)
            .set_layouts(descriptor_set_layouts);

        // SAFETY: `create_info` outlives this call.
        unsafe { device.raw().create_pipeline_layout(&create_info, None) }
            .expect("failed to create pipeline layout")
    }

    fn create_pipeline(
        device: &Device,
        render_pass: vk::RenderPass,
        layout: vk::PipelineLayout,
        vertex: &Shader,
        fragment: &Shader,
        vertex_bindings: &[vk::VertexInputBindingDescription],
        vertex_attributes: &[vk::VertexInputAttributeDescription],
    ) -> vk::Pipeline {
        let entry_point = c"main";

        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex.raw())
                .name(entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment.raw())
                .name(entry_point),
        ];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(vertex_bindings)
            .vertex_attribute_descriptions(vertex_attributes);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        // counts only - actual viewport/scissor rects are dynamic state
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        // no culling: flat 2D geometry has no back face, and triangulated
        // shapes (see `ancorix_render::geometry`) don't guarantee a fixed
        // winding order
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .line_width(1.0);

        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // standard "over" compositing for straight (non-premultiplied) alpha,
        // matching how `Vertex::color` is unpacked (R8G8B8A8_UNORM, no
        // premultiplication step in `ancorix_render::geometry`)
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blend_attachments = [color_blend_attachment];
        let color_blend =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blend_attachments);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .color_blend_state(&color_blend)
            .dynamic_state(&dynamic_state)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0);

        // SAFETY: `create_info` and everything it borrows outlive this
        // call. We request exactly one pipeline, so indexing the result
        // at 0 is in bounds on success.
        let pipelines = unsafe {
            device.raw().create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&create_info),
                None,
            )
        }
        .map_err(|(_, err)| err)
        .expect("failed to create graphics pipeline");

        pipelines[0]
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped
        // (see the struct-level ordering requirement).
        unsafe {
            self.device.destroy_pipeline(self.raw, None);
            self.device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
