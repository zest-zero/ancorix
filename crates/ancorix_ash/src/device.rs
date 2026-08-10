use crate::instance::Instance;
use crate::surface::Surface;
use ash::vk;

const REQUIRED_EXTENSIONS: &[*const std::ffi::c_char] = &[ash::khr::swapchain::NAME.as_ptr()];

/// Owns the logical Vulkan device and exposes the physical device and queues.
///
/// Must be dropped before [`Instance`].
pub struct Device {
    pub(crate) physical: vk::PhysicalDevice,
    pub(crate) raw: ash::Device,
    pub(crate) graphics_family: u32,
    pub(crate) graphics_queue: vk::Queue,
}

impl Device {
    /// Selects a physical device, preferring a discrete GPU, and creates a
    /// logical device from it.
    // Falls back to any
    /// device with a queue family that supports both graphics and
    /// presenting to `surface`, and the required extensions.
    ///
    /// # Panics
    ///
    /// Panics if no suitable GPU is found or logical device creation fails.
    pub fn new(instance: &Instance, surface: &Surface) -> Self {
        let physical = Self::pick_physical(instance, surface);
        let graphics_family = Self::find_graphics_family(instance, surface, physical);

        let priority = [1.0_f32];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_family)
            .queue_priorities(&priority);

        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(REQUIRED_EXTENSIONS);

        // SAFETY: `physical` was just enumerated from this same `instance`.
        let raw = unsafe {
            instance
                .raw()
                .create_device(physical, &create_info, None)
                .expect("failed to create logical device")
        };

        // SAFETY: `graphics_family` was returned by enumerating `physical`'s
        // own queue families, and index 0 always exists - `queue_info`
        // above requested exactly one queue from it.
        let graphics_queue = unsafe { raw.get_device_queue(graphics_family, 0) };

        Self {
            physical,
            raw,
            graphics_family,
            graphics_queue,
        }
    }

    /// Returns the raw [`ash::Device`].
    #[inline]
    pub fn raw(&self) -> &ash::Device {
        &self.raw
    }

    /// Returns the selected [`vk::PhysicalDevice`].
    #[inline]
    pub fn physical(&self) -> vk::PhysicalDevice {
        self.physical
    }

    /// Returns the graphics queue family index.
    #[inline]
    pub fn graphics_family(&self) -> u32 {
        self.graphics_family
    }

    /// Returns the graphics [`vk::Queue`].
    #[inline]
    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    /// Waits for the device to finish all pending work.
    ///
    /// # Panics
    ///
    /// Panics if the wait fails.
    #[inline]
    pub fn wait_idle(&self) {
        // SAFETY: no precondition beyond a valid `self.raw`, which the
        // type always holds.
        unsafe {
            self.raw
                .device_wait_idle()
                .expect("device_wait_idle failed")
        }
    }

    fn pick_physical(instance: &Instance, surface: &Surface) -> vk::PhysicalDevice {
        // SAFETY: `instance` outlives this call.
        let devices = unsafe { instance.raw().enumerate_physical_devices() }
            .expect("failed to enumerate physical devices");

        assert!(!devices.is_empty(), "no Vulkan-capable GPU found");

        devices
            .into_iter()
            .filter(|&d| Self::is_suitable(instance, surface, d))
            .max_by_key(|&d| {
                // SAFETY: `d` was just enumerated from `instance`.
                let props = unsafe { instance.raw().get_physical_device_properties(d) };
                match props.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 2u32,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                    _ => 0,
                }
            })
            .expect("no GPU supports required Vulkan extensions, a graphics queue, and presenting to the window surface")
    }

    fn is_suitable(instance: &Instance, surface: &Surface, device: vk::PhysicalDevice) -> bool {
        Self::find_graphics_family_opt(instance, surface, device).is_some()
            && Self::check_extensions(instance, device)
    }

    fn find_graphics_family(
        instance: &Instance,
        surface: &Surface,
        device: vk::PhysicalDevice,
    ) -> u32 {
        Self::find_graphics_family_opt(instance, surface, device)
            .expect("no graphics queue family found")
    }

    // finds a queue family that supports both GRAPHICS and presenting to `surface`
    fn find_graphics_family_opt(
        instance: &Instance,
        surface: &Surface,
        device: vk::PhysicalDevice,
    ) -> Option<u32> {
        // SAFETY: `device` was enumerated from `instance`.
        unsafe {
            instance
                .raw()
                .get_physical_device_queue_family_properties(device)
        }
        .into_iter()
        .enumerate()
        .find(|(i, f)| {
            f.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && surface.supports_present(device, *i as u32)
        })
        .map(|(i, _)| i as u32)
    }

    fn check_extensions(instance: &Instance, device: vk::PhysicalDevice) -> bool {
        // SAFETY: `device` was enumerated from `instance`.
        let available = unsafe { instance.raw().enumerate_device_extension_properties(device) }
            .unwrap_or_default();

        REQUIRED_EXTENSIONS.iter().all(|&ptr| {
            // SAFETY: entries in `REQUIRED_EXTENSIONS` are `'static` C
            // strings from `ash`'s own extension-name constants.
            let name = unsafe { std::ffi::CStr::from_ptr(ptr) };
            available.iter().any(|ext| {
                // SAFETY: `extension_name` is a NUL-terminated C string
                // filled in by the driver via `enumerate_device_extension_properties`.
                unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) == name }
            })
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Instance` is dropped
        // (see the struct-level ordering requirement). All queues
        // retrieved from this device stop being valid after this call,
        // which is upheld because nothing else in this crate stores one
        // past the `Device`'s lifetime.
        unsafe { self.raw.destroy_device(None) }
    }
}
