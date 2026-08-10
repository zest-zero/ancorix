use crate::device::Device;
use ash::vk;

// provisional cap, not measured - one descriptor set per unique
// `Handle<Texture>` ever drawn, not per sprite instance.
const MAX_TEXTURES: u32 = 256;

/// A descriptor set layout for one combined image sampler, and the pool to
/// allocate sets from it. Must be dropped before [`Device`].
pub struct TextureDescriptorLayout {
    raw: vk::DescriptorSetLayout,
    pool: vk::DescriptorPool,
    device: ash::Device,
}

impl TextureDescriptorLayout {
    /// # Panics
    ///
    /// Panics if layout or pool creation fails.
    pub fn new(device: &Device) -> Self {
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let bindings = [binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        // SAFETY: `layout_info` outlives this call.
        let raw = unsafe {
            device
                .raw()
                .create_descriptor_set_layout(&layout_info, None)
        }
        .expect("failed to create descriptor set layout");

        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: MAX_TEXTURES,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(MAX_TEXTURES);

        // SAFETY: `pool_info` outlives this call.
        let pool = unsafe { device.raw().create_descriptor_pool(&pool_info, None) }
            .expect("failed to create descriptor pool");

        Self {
            raw,
            pool,
            device: device.raw().clone(),
        }
    }

    /// Returns the raw `VkDescriptorSetLayout`.
    #[inline]
    pub fn raw(&self) -> vk::DescriptorSetLayout {
        self.raw
    }

    /// Allocates a new descriptor set bound to sample `view`/`sampler`.
    ///
    /// # Panics
    ///
    /// Panics if allocation fails (e.g. more than `MAX_TEXTURES` in use).
    pub fn allocate(&self, view: vk::ImageView, sampler: vk::Sampler) -> vk::DescriptorSet {
        let layouts = [self.raw];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);

        // SAFETY: `allocate_info` outlives this call; we request exactly
        // one set, so indexing the result at 0 is in bounds on success.
        let set = unsafe { self.device.allocate_descriptor_sets(&allocate_info) }
            .expect("failed to allocate descriptor set")[0];

        let image_info = vk::DescriptorImageInfo {
            sampler,
            image_view: view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };
        let write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));

        // SAFETY: `write` and everything it borrows outlive this call.
        unsafe { self.device.update_descriptor_sets(&[write], &[]) };

        set
    }
}

impl Drop for TextureDescriptorLayout {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped (see
        // the struct-level ordering requirement). Destroying the pool
        // implicitly frees any sets allocated from it.
        unsafe {
            self.device.destroy_descriptor_pool(self.pool, None);
            self.device.destroy_descriptor_set_layout(self.raw, None);
        }
    }
}
