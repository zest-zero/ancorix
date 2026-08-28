use ash::{Entry, vk};
use raw_window_handle::RawDisplayHandle;

const VALIDATION_LAYER: *const std::ffi::c_char = c"VK_LAYER_KHRONOS_validation".as_ptr();

/// The Vulkan loader and instance handle.
// Field order matters for drop: `raw` goes before `entry`, so the loader
// outlives the instance it created.
pub struct Instance {
    pub(crate) raw: ash::Instance,
    pub(crate) entry: Entry,
    pub(crate) validation: bool,
}

impl Instance {
    /// Creates an instance for `display`, with validation layers in debug
    /// builds if they are installed.
    ///
    /// # Panics
    ///
    /// Panics if the Vulkan loader isn't found, a required extension is
    /// missing, or instance creation fails.
    pub fn new(display: RawDisplayHandle) -> Self {
        // SAFETY: we don't call any Vulkan function before checking that
        // loading succeeded, and `entry` (and everything loaded from it)
        // is dropped before the process exits.
        let entry =
            unsafe { Entry::load() }.expect("failed to load the Vulkan loader (libvulkan.so.1)");

        let extensions = ash_window::enumerate_required_extensions(display)
            .expect("failed to get required Vulkan extensions for this display");

        let validation = Self::check_validation(&entry);
        let layers = if validation {
            vec![VALIDATION_LAYER]
        } else {
            vec![]
        };

        let app_info = vk::ApplicationInfo::default()
            .engine_name(c"Ancorix")
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_1);

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(extensions);

        // SAFETY: `create_info` and everything it borrows (`app_info`,
        // `layers`, `extensions`) stay alive for the duration of this call.
        let raw = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("failed to create Vulkan instance")
        };

        if validation {
            eprintln!("[ancorix] Vulkan validation layers active (debug build)");
        }

        Self {
            raw,
            entry,
            validation,
        }
    }

    /// Returns the raw [`ash::Instance`].
    #[inline]
    pub fn raw(&self) -> &ash::Instance {
        &self.raw
    }

    /// Returns the [`Entry`] (Vulkan loader).
    #[inline]
    pub fn entry(&self) -> &Entry {
        &self.entry
    }

    /// Returns `true` if validation layers are active.
    #[inline]
    pub fn validation_enabled(&self) -> bool {
        self.validation
    }

    fn check_validation(entry: &Entry) -> bool {
        if !cfg!(debug_assertions) {
            return false;
        }
        // SAFETY: no precondition beyond a valid `entry`.
        let Ok(layers) = (unsafe { entry.enumerate_instance_layer_properties() }) else {
            return false;
        };
        layers.iter().any(|l| unsafe {
            // SAFETY: `layer_name` is a NUL-terminated C string filled in
            // by the driver via `enumerate_instance_layer_properties`.
            std::ffi::CStr::from_ptr(l.layer_name.as_ptr()) == c"VK_LAYER_KHRONOS_validation"
        })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // SAFETY: called once; every child object (`Device`, `Surface`)
        // documents that it must be dropped before `Instance`, and field
        // order in those structs plus this crate's own construction order
        // upholds that.
        unsafe { self.raw.destroy_instance(None) }
    }
}
