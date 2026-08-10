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

        Self {
            raw,
            memory,
            size,
            device: device.raw().clone(),
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
    /// Panics if `data` is larger than the buffer, or if mapping fails.
    pub fn write(&self, data: &[u8]) {
        assert!(
            data.len() as vk::DeviceSize <= self.size,
            "write of {} bytes overflows a {}-byte buffer",
            data.len(),
            self.size
        );

        // SAFETY: `memory` belongs to `device` and isn't mapped elsewhere -
        // this type never keeps a persistent mapping.
        let ptr = unsafe {
            self.device
                .map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty())
        }
        .expect("failed to map buffer memory");

        // SAFETY: `ptr` is valid for `self.size` bytes (just mapped above),
        // and `data.len() <= self.size` was checked above.
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast(), data.len()) };

        // SAFETY: `memory` was mapped by the call above.
        unsafe { self.device.unmap_memory(self.memory) };
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
