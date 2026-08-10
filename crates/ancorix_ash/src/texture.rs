use crate::device::Device;
use crate::instance::Instance;
use crate::memory::Buffer;
use ash::vk;

/// How a [`Texture`]'s sampler filters texels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureFilter {
    /// Blends neighbouring texels.
    // At a sprite sheet's frame edge this blends in the neighbouring frame:
    // pack with a gutter, or use `Nearest`.
    #[default]
    Linear,
    /// Takes the closest texel. What pixel art wants.
    Nearest,
}

/// A device-local image, its view and a sampler. Must be dropped before
/// [`Device`].
// `R8G8B8A8_UNORM`, not SRGB - same reason as `Swapchain`.
pub struct Texture {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    sampler: vk::Sampler,
    width: u32,
    height: u32,
    device: ash::Device,
}

impl Texture {
    /// Uploads `pixels` (`width * height * 4` bytes, RGBA8) to a new GPU
    /// texture sampled with `filter`.
    ///
    /// # Panics
    ///
    /// Panics if `pixels` is the wrong length, or any Vulkan call fails.
    pub fn new(
        instance: &Instance,
        device: &Device,
        pixels: &[u8],
        width: u32,
        height: u32,
        filter: TextureFilter,
    ) -> Self {
        assert_eq!(
            pixels.len(),
            (width * height * 4) as usize,
            "pixel data doesn't match width * height * 4 bytes (RGBA8)"
        );

        let staging = Buffer::new(
            instance,
            device,
            pixels.len() as vk::DeviceSize,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );
        staging.write(pixels);

        let (image, memory) = Self::create_image(instance, device, width, height);
        Self::upload(device, image, &staging, width, height);
        let view = Self::create_view(device, image);
        let sampler = Self::create_sampler(device, filter);

        Self {
            image,
            memory,
            view,
            sampler,
            width,
            height,
            device: device.raw().clone(),
        }
    }

    /// Returns the raw `VkImageView`, for binding into a descriptor set.
    #[inline]
    pub fn view(&self) -> vk::ImageView {
        self.view
    }

    /// Returns the raw `VkSampler`, for binding into a descriptor set.
    #[inline]
    pub fn sampler(&self) -> vk::Sampler {
        self.sampler
    }

    /// Returns the texture's size in pixels.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn create_image(
        instance: &Instance,
        device: &Device,
        width: u32,
        height: u32,
    ) -> (vk::Image, vk::DeviceMemory) {
        let create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        // SAFETY: `create_info` outlives this call.
        let image = unsafe { device.raw().create_image(&create_info, None) }
            .expect("failed to create image");

        // SAFETY: `image` was just created on `device`.
        let requirements = unsafe { device.raw().get_image_memory_requirements(image) };
        let memory_type = Buffer::find_memory_type(
            instance,
            device,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type);

        // SAFETY: `allocate_info` outlives this call.
        let memory = unsafe { device.raw().allocate_memory(&allocate_info, None) }
            .expect("failed to allocate image memory");

        // SAFETY: `image` and `memory` were just created on the same
        // `device`, and `memory` satisfies `image`'s requirements above.
        unsafe { device.raw().bind_image_memory(image, memory, 0) }
            .expect("failed to bind image memory");

        (image, memory)
    }

    // one-time command buffer: UNDEFINED -> TRANSFER_DST -> copy -> SHADER_READ_ONLY.
    // Blocks - not a per-frame op, no need for Commands' frame-in-flight setup.
    fn upload(device: &Device, image: vk::Image, staging: &Buffer, width: u32, height: u32) {
        let pool_info =
            vk::CommandPoolCreateInfo::default().queue_family_index(device.graphics_family());
        // SAFETY: `pool_info` outlives this call.
        let pool = unsafe { device.raw().create_command_pool(&pool_info, None) }
            .expect("failed to create command pool");

        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        // SAFETY: `allocate_info` outlives this call.
        let buffer = unsafe { device.raw().allocate_command_buffers(&allocate_info) }
            .expect("failed to allocate command buffer")[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        // SAFETY: `begin_info` outlives this call.
        unsafe { device.raw().begin_command_buffer(buffer, &begin_info) }
            .expect("failed to begin command buffer");

        let subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };

        let to_transfer_dst = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource_range)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
        };

        let to_shader_read = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(subresource_range)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);

        // SAFETY: `buffer` is recording; every borrowed argument outlives
        // its call.
        unsafe {
            device.raw().cmd_pipeline_barrier(
                buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_transfer_dst],
            );
            device.raw().cmd_copy_buffer_to_image(
                buffer,
                staging.raw(),
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
            device.raw().cmd_pipeline_barrier(
                buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[to_shader_read],
            );
        }

        // SAFETY: `buffer` is in the recording state from `begin_command_buffer` above.
        unsafe { device.raw().end_command_buffer(buffer) }.expect("failed to end command buffer");

        // own fence, not `device.wait_idle()` - that would stall the whole
        // device (including any other in-flight frame), not just this upload.
        let fence_info = vk::FenceCreateInfo::default();
        // SAFETY: `fence_info` outlives this call.
        let fence = unsafe { device.raw().create_fence(&fence_info, None) }
            .expect("failed to create fence");

        let buffers = [buffer];
        let submit_info = vk::SubmitInfo::default().command_buffers(&buffers);
        // SAFETY: `submit_info` outlives this call.
        unsafe {
            device
                .raw()
                .queue_submit(device.graphics_queue(), &[submit_info], fence)
        }
        .expect("failed to submit upload command buffer");

        // SAFETY: `fence` was just submitted above.
        unsafe { device.raw().wait_for_fences(&[fence], true, u64::MAX) }
            .expect("failed to wait for upload fence");
        // SAFETY: `fence` is no longer in use, waited above.
        unsafe { device.raw().destroy_fence(fence, None) };

        // SAFETY: `pool` was created on `device` above; destroying it
        // implicitly frees `buffer`, which is done executing (waited above).
        unsafe { device.raw().destroy_command_pool(pool, None) };
    }

    fn create_view(device: &Device, image: vk::Image) -> vk::ImageView {
        let create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .components(vk::ComponentMapping::default())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        // SAFETY: `image` belongs to `device`, which outlives this view.
        unsafe { device.raw().create_image_view(&create_info, None) }
            .expect("failed to create image view")
    }

    // clamp-to-edge regardless of filter - avoids edge-repeat bleeding.
    fn create_sampler(device: &Device, filter: TextureFilter) -> vk::Sampler {
        let filter = match filter {
            TextureFilter::Linear => vk::Filter::LINEAR,
            TextureFilter::Nearest => vk::Filter::NEAREST,
        };

        let create_info = vk::SamplerCreateInfo::default()
            .mag_filter(filter)
            .min_filter(filter)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .anisotropy_enable(false)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

        // SAFETY: `create_info` outlives this call.
        unsafe { device.raw().create_sampler(&create_info, None) }
            .expect("failed to create sampler")
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped (see
        // the struct-level ordering requirement).
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
