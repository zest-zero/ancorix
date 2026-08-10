use crate::instance::Instance;
use ash::khr::surface;
use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

/// Owns a `VkSurfaceKHR` for a window.
///
/// Must be dropped before [`Instance`].
pub struct Surface {
    pub(crate) raw: vk::SurfaceKHR,
    loader: surface::Instance,
}

impl Surface {
    /// Creates a surface for the given window.
    ///
    /// # Panics
    ///
    /// Panics if surface creation fails.
    pub fn new(instance: &Instance, display: RawDisplayHandle, window: RawWindowHandle) -> Self {
        let loader = surface::Instance::new(instance.entry(), instance.raw());

        // SAFETY: `display`/`window` are required by this function's own
        // contract to come from a live window that outlives the returned
        // `Surface`; `raw` is destroyed in `Drop` before `instance` can be.
        let raw = unsafe {
            ash_window::create_surface(instance.entry(), instance.raw(), display, window, None)
                .expect("failed to create Vulkan surface")
        };

        Self { raw, loader }
    }

    /// Returns the raw [`vk::SurfaceKHR`].
    #[inline]
    pub fn raw(&self) -> vk::SurfaceKHR {
        self.raw
    }

    /// Returns `true` if `queue_family` on `physical` can present to this
    /// surface.
    pub fn supports_present(&self, physical: vk::PhysicalDevice, queue_family: u32) -> bool {
        // SAFETY: `physical` and `queue_family` are only ever produced by
        // enumerating `physical`'s own queue families (see `device.rs`).
        unsafe {
            self.loader
                .get_physical_device_surface_support(physical, queue_family, self.raw)
        }
        .unwrap_or(false)
    }

    /// Returns `physical`'s capabilities for this surface.
    ///
    /// # Panics
    ///
    /// Panics if the query fails.
    pub fn capabilities(&self, physical: vk::PhysicalDevice) -> vk::SurfaceCapabilitiesKHR {
        // SAFETY: `physical` must belong to the instance this surface was
        // created from - upheld by every caller in this crate.
        unsafe {
            self.loader
                .get_physical_device_surface_capabilities(physical, self.raw)
        }
        .expect("failed to query surface capabilities")
    }

    /// Returns the pixel formats `physical` can use to present to this
    /// surface.
    ///
    /// # Panics
    ///
    /// Panics if the query fails.
    pub fn formats(&self, physical: vk::PhysicalDevice) -> Vec<vk::SurfaceFormatKHR> {
        // SAFETY: `physical` must belong to the instance this surface was
        // created from - upheld by every caller in this crate.
        unsafe {
            self.loader
                .get_physical_device_surface_formats(physical, self.raw)
        }
        .expect("failed to query surface formats")
    }

    /// Returns the present modes `physical` supports for this surface.
    ///
    /// # Panics
    ///
    /// Panics if the query fails.
    pub fn present_modes(&self, physical: vk::PhysicalDevice) -> Vec<vk::PresentModeKHR> {
        // SAFETY: `physical` must belong to the instance this surface was
        // created from - upheld by every caller in this crate.
        unsafe {
            self.loader
                .get_physical_device_surface_present_modes(physical, self.raw)
        }
        .expect("failed to query surface present modes")
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Instance` is dropped
        // (see the struct-level ordering requirement).
        unsafe { self.loader.destroy_surface(self.raw, None) }
    }
}
