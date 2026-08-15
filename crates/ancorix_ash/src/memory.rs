use crate::device::Device;
use crate::instance::Instance;
use ash::vk;

/// Owns a `VkBuffer` and its bound `VkDeviceMemory`.
///
/// Must be dropped before [`Device`].
pub struct Buffer {
    raw: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    device: ash::Device,
    mapped: *mut u8,
}

impl Buffer {
    /// Creates a buffer of `size` bytes for `usage`, backed by memory
    /// satisfying `properties`.
    ///
    /// # Panics
    ///
    /// Panics if buffer creation, allocation, or binding fails, or if no
    /// memory type satisfies `properties`.
    pub fn new(
        instance: &Instance,
        device: &Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Self {
        let create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        // SAFETY: `create_info` outlives this call.
        let raw = unsafe { device.raw().create_buffer(&create_info, None) }
            .expect("failed to create buffer");

        // SAFETY: `raw` was just created on `device`.
        let requirements = unsafe { device.raw().get_buffer_memory_requirements(raw) };
        let memory_type =
            Self::find_memory_type(instance, device, requirements.memory_type_bits, properties);

        let allocate_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type);

        // SAFETY: `allocate_info` outlives this call.
        let memory = unsafe { device.raw().allocate_memory(&allocate_info, None) }
            .expect("failed to allocate buffer memory");

        // SAFETY: `raw` and `memory` were just created on the same
        // `device`, and `memory` satisfies `raw`'s requirements above.
        unsafe { device.raw().bind_buffer_memory(raw, memory, 0) }
            .expect("failed to bind buffer memory");

        // SAFETY: `memory` was just allocated and bound to `raw` above, and
        // is never mapped anywhere else - `size` bytes, offset 0.
        let mapped = unsafe {
            device
                .raw()
                .map_memory(memory, 0, size, vk::MemoryMapFlags::empty())
        }
        .expect("failed to map buffer memory - does it include HOST_VISIBLE?")
            as *mut u8;

        Self {
            raw,
            memory,
            size,
            device: device.raw().clone(),
            mapped,
        }
    }

    /// Returns the raw [`vk::Buffer`].
    #[inline]
    pub fn raw(&self) -> vk::Buffer {
        self.raw
    }

    /// Returns the buffer's size in bytes.
    #[inline]
    pub fn size(&self) -> vk::DeviceSize {
        self.size
    }

    /// Copies `data` into the buffer's memory at offset 0. Only valid for
    /// memory created with `HOST_VISIBLE`.
    ///
    /// # Panics
    ///
    /// Panics if `data` is larger than the buffer.
    pub fn write(&self, data: &[u8]) {
        assert!(
            data.len() as vk::DeviceSize <= self.size,
            "write of {} bytes overflows a {}-byte buffer",
            data.len(),
            self.size
        );

        // SAFETY: `mapped` is valid for `self.size` bytes for the whole
        // lifetime of `self` (mapped once in `new`), and `data.len() <=
        // self.size` was just checked above. `HOST_COHERENT` means no
        // explicit flush is needed for the write to become visible to the
        // device.
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), self.mapped, data.len()) };
    }

    // `pub(crate)`, not private - `Texture::create_image` (texture.rs)
    // reuses this instead of duplicating the same memory-type search.
    pub(crate) fn find_memory_type(
        instance: &Instance,
        device: &Device,
        type_bits: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> u32 {
        // SAFETY: `device.physical()` was enumerated from `instance`.
        let memory_properties = unsafe {
            instance
                .raw()
                .get_physical_device_memory_properties(device.physical())
        };

        (0..memory_properties.memory_type_count)
            .find(|&i| {
                type_bits & (1 << i) != 0
                    && memory_properties.memory_types[i as usize]
                        .property_flags
                        .contains(properties)
            })
            .expect("no memory type satisfies the requested buffer properties")
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped
        // (see the struct-level ordering requirement).
        unsafe {
            self.device.destroy_buffer(self.raw, None);
            self.device.free_memory(self.memory, None);
        }
    }
}
