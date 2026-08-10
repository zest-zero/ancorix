use crate::device::Device;
use crate::gpu_timer::GpuTimer;
use crate::render_pass::RenderPass;
use crate::swapchain::Swapchain;
use ash::vk;

// Clear color for pixels no draw call covers. Loud magenta rather than
// black, so a frame that forgot to cover itself looks like the bug it is.
const UNCOVERED_COLOR: [f32; 4] = [1.0, 0.0, 1.0, 1.0];

/// One pipeline's worth of geometry to draw within a render pass instance.
// Raw handles only: this crate knows nothing of `ancorix_render`'s vertex
// formats or batching.
#[derive(Debug, Clone, Copy)]
pub struct DrawBatch {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub vertex_buffer: vk::Buffer,
    pub index_buffer: vk::Buffer,
    // lets several batches share one buffer pair, each drawing its own range -
    // how one frame's geometry is split into runs without copying anything
    // into per-run buffers (see `ancorix_render::geometry::DrawRun`).
    pub first_index: u32,
    pub index_count: u32,
    // Non-zero switches this batch to an instanced, non-indexed draw, for
    // pipelines whose shader derives its geometry from `gl_VertexIndex`.
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_instance: u32,
    // Already clamped by the caller - passed straight to `vkCmdSetScissor`.
    pub scissor: Option<vk::Rect2D>,
    // Slice of `record_frame`'s `push_constants`: batches share one buffer
    // because most push the same bytes, and a shaded one appends its own.
    pub first_push_constant: u32,
    pub push_constant_len: u32,
    // bound at set 0 before drawing, if the pipeline samples a texture.
    pub descriptor_set: Option<vk::DescriptorSet>,
}

/// A command pool, and per swapchain image: a framebuffer, a command buffer
/// and a "render finished" semaphore. Must be dropped before [`Device`].
// That semaphore lives here rather than on `FrameSync` because it has to be
// indexed by swapchain image, not by frame-in-flight slot: which image a
// frame gets is only known after `acquire_next_image`, and the driver need
// not hand them out in lockstep. Signaling one that an earlier, not yet
// presented use of the same image still waits on would be a race.
pub struct Commands {
    pool: vk::CommandPool,
    buffers: Vec<vk::CommandBuffer>,
    framebuffers: Vec<vk::Framebuffer>,
    render_finished: Vec<vk::Semaphore>,
    device: ash::Device,
}

impl Commands {
    /// # Panics
    ///
    /// Panics if creating the framebuffers, command pool, command buffers,
    /// or semaphores fails.
    pub fn new(device: &Device, render_pass: &RenderPass, swapchain: &Swapchain) -> Self {
        let framebuffers = Self::create_framebuffers(device, render_pass.raw(), swapchain);
        let pool = Self::create_pool(device);
        let buffers = Self::allocate_buffers(device, pool, framebuffers.len() as u32);
        let render_finished = Self::create_semaphores(device, framebuffers.len());

        Self {
            pool,
            buffers,
            framebuffers,
            render_finished,
            device: device.raw().clone(),
        }
    }

    /// Returns the semaphore signaled once `image_index` is ready to present.
    #[inline]
    pub fn render_finished(&self, image_index: u32) -> vk::Semaphore {
        self.render_finished[image_index as usize]
    }

    /// Records `batches` into `image_index`'s framebuffer, in one render pass
    /// instance and in the order given.
    ///
    /// # Panics
    ///
    /// Panics if resetting or recording the command buffer fails.
    pub fn record_frame(
        &self,
        image_index: u32,
        render_pass: &RenderPass,
        extent: vk::Extent2D,
        batches: &[DrawBatch],
        push_constants: &[u8],
        gpu_timer: Option<(&GpuTimer, usize)>,
    ) {
        let buffer = self.buffers[image_index as usize];
        let framebuffer = self.framebuffers[image_index as usize];

        // SAFETY: the caller waited on this frame's fence (via
        // `FrameSync::wait_and_reset`) before calling this, so the GPU is
        // done with `buffer`.
        unsafe {
            self.device
                .reset_command_buffer(buffer, vk::CommandBufferResetFlags::empty())
        }
        .expect("failed to reset command buffer");

        let begin_info = vk::CommandBufferBeginInfo::default();
        // SAFETY: `begin_info` outlives this call.
        unsafe { self.device.begin_command_buffer(buffer, &begin_info) }
            .expect("failed to begin command buffer");

        if let Some((timer, frame_slot)) = gpu_timer {
            timer.cmd_begin(buffer, frame_slot);
        }

        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: UNCOVERED_COLOR,
            },
        }];
        let render_pass_begin = vk::RenderPassBeginInfo::default()
            .render_pass(render_pass.raw())
            .framebuffer(framebuffer)
            .render_area(render_area)
            .clear_values(&clear_values);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        // SAFETY: `buffer` was just begun above and is bound to this
        // thread; every borrowed argument outlives its call.
        unsafe {
            self.device.cmd_begin_render_pass(
                buffer,
                &render_pass_begin,
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_set_viewport(buffer, 0, &[viewport]);
            self.device.cmd_set_scissor(buffer, 0, &[scissor]);

            // re-issued only when a batch actually asks for a different one
            let mut current_scissor = scissor;

            for batch in batches {
                let instanced = batch.instance_count > 0;
                if !instanced && batch.index_count == 0 {
                    continue;
                }

                let wanted_scissor = batch.scissor.unwrap_or(scissor);
                if wanted_scissor != current_scissor {
                    self.device.cmd_set_scissor(buffer, 0, &[wanted_scissor]);
                    current_scissor = wanted_scissor;
                }

                self.device.cmd_bind_pipeline(
                    buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    batch.pipeline,
                );
                self.device
                    .cmd_bind_vertex_buffers(buffer, 0, &[batch.vertex_buffer], &[0]);
                if !instanced {
                    self.device.cmd_bind_index_buffer(
                        buffer,
                        batch.index_buffer,
                        0,
                        vk::IndexType::UINT32,
                    );
                }
                let first = batch.first_push_constant as usize;
                self.device.cmd_push_constants(
                    buffer,
                    batch.layout,
                    // a user shader reads its parameters from the fragment
                    // stage, the built-in ones read the globals from the
                    // vertex stage - one push covers both
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    &push_constants[first..first + batch.push_constant_len as usize],
                );
                if let Some(set) = batch.descriptor_set {
                    self.device.cmd_bind_descriptor_sets(
                        buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        batch.layout,
                        0,
                        &[set],
                        &[],
                    );
                }
                if instanced {
                    self.device.cmd_draw(
                        buffer,
                        batch.vertex_count,
                        batch.instance_count,
                        0,
                        batch.first_instance,
                    );
                } else {
                    self.device.cmd_draw_indexed(
                        buffer,
                        batch.index_count,
                        1,
                        batch.first_index,
                        0,
                        0,
                    );
                }
            }

            self.device.cmd_end_render_pass(buffer);
        }

        if let Some((timer, frame_slot)) = gpu_timer {
            timer.cmd_end(buffer, frame_slot);
        }

        // SAFETY: `buffer` is in the recording state from `begin_command_buffer` above.
        unsafe { self.device.end_command_buffer(buffer) }.expect("failed to end command buffer");
    }

    /// Returns the recorded command buffer for `image_index`.
    #[inline]
    pub fn buffer(&self, image_index: u32) -> vk::CommandBuffer {
        self.buffers[image_index as usize]
    }

    fn create_framebuffers(
        device: &Device,
        render_pass: vk::RenderPass,
        swapchain: &Swapchain,
    ) -> Vec<vk::Framebuffer> {
        let extent = swapchain.extent();

        swapchain
            .image_views()
            .iter()
            .map(|&view| {
                let attachments = [view];
                let create_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                // SAFETY: `create_info` and `attachments` outlive this call.
                unsafe { device.raw().create_framebuffer(&create_info, None) }
                    .expect("failed to create framebuffer")
            })
            .collect()
    }

    fn create_semaphores(device: &Device, count: usize) -> Vec<vk::Semaphore> {
        let create_info = vk::SemaphoreCreateInfo::default();
        (0..count)
            .map(|_| {
                // SAFETY: `create_info` outlives this call.
                unsafe { device.raw().create_semaphore(&create_info, None) }
                    .expect("failed to create semaphore")
            })
            .collect()
    }

    fn create_pool(device: &Device) -> vk::CommandPool {
        let create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(device.graphics_family());

        // SAFETY: `create_info` outlives this call.
        unsafe { device.raw().create_command_pool(&create_info, None) }
            .expect("failed to create command pool")
    }

    fn allocate_buffers(
        device: &Device,
        pool: vk::CommandPool,
        count: u32,
    ) -> Vec<vk::CommandBuffer> {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count);

        // SAFETY: `allocate_info` outlives this call.
        unsafe { device.raw().allocate_command_buffers(&allocate_info) }
            .expect("failed to allocate command buffers")
    }
}

impl Drop for Commands {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped
        // (see the struct-level ordering requirement). Destroying the
        // pool implicitly frees the command buffers allocated from it.
        unsafe {
            for &framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            for &semaphore in &self.render_finished {
                self.device.destroy_semaphore(semaphore, None);
            }
            self.device.destroy_command_pool(self.pool, None);
        }
    }
}
