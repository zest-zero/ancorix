use crate::device::Device;
use ash::vk;

/// A shader module, from SPIR-V bytes.
///
/// Must be dropped before [`Device`].
pub struct Shader {
    raw: vk::ShaderModule,
    device: ash::Device,
}

impl Shader {
    /// Creates a shader module from SPIR-V bytecode.
    ///
    /// # Panics
    ///
    /// Panics if `code`'s length isn't a multiple of 4 (SPIR-V is a stream
    /// of `u32` words), or if module creation fails.
    pub fn new(device: &Device, code: &[u8]) -> Self {
        assert!(
            code.len().is_multiple_of(4),
            "SPIR-V bytecode length must be a multiple of 4, got {} bytes",
            code.len()
        );

        // SPIR-V words are little-endian u32 - convert explicitly rather
        // than reinterpreting the byte slice's pointer as `*const u32`,
        // which would be UB on unaligned input and wrong on big-endian.
        let words: Vec<u32> = code
            .as_chunks::<4>()
            .0
            .iter()
            .map(|&word| u32::from_le_bytes(word))
            .collect();

        let create_info = vk::ShaderModuleCreateInfo::default().code(&words);

        // SAFETY: `words` outlives this call.
        let raw = unsafe { device.raw().create_shader_module(&create_info, None) }
            .expect("failed to create shader module");

        Self {
            raw,
            device: device.raw().clone(),
        }
    }

    /// Returns the raw [`vk::ShaderModule`].
    #[inline]
    pub fn raw(&self) -> vk::ShaderModule {
        self.raw
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped
        // (see the struct-level ordering requirement).
        unsafe { self.device.destroy_shader_module(self.raw, None) }
    }
}
