use crate::device::Device;
use ash::vk;

/// A render pass with one color attachment, shared by every
/// [`crate::Pipeline`] drawn within it.
///
/// Must be dropped before [`Device`].
pub struct RenderPass {
    raw: vk::RenderPass,
    device: ash::Device,
}

impl RenderPass {
    /// Creates a render pass for `color_format` with one color attachment.
    /// Uses `LOAD_OP_CLEAR` as a safety net, not `DONT_CARE` - see
    /// [`crate::Pipeline::new`].
    ///
    /// # Panics
    ///
    /// Panics if render pass creation fails.
    pub fn new(device: &Device, color_format: vk::Format) -> Self {
        let color_attachment = vk::AttachmentDescription::default()
            .format(color_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_refs = [color_attachment_ref];
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs);

        // waits for the swapchain image to be done being read (by the
        // presentation engine) before this subpass writes to it
        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let attachments = [color_attachment];
        let subpasses = [subpass];
        let dependencies = [dependency];
        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies);

        // SAFETY: `create_info` and everything it borrows outlive this call.
        let raw = unsafe { device.raw().create_render_pass(&create_info, None) }
            .expect("failed to create render pass");

        Self {
            raw,
            device: device.raw().clone(),
        }
    }

    /// Returns the raw [`vk::RenderPass`].
    #[inline]
    pub fn raw(&self) -> vk::RenderPass {
        self.raw
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped
        // (see the struct-level ordering requirement).
        unsafe { self.device.destroy_render_pass(self.raw, None) };
    }
}
