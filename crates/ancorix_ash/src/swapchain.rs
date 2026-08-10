use crate::device::Device;
use crate::instance::Instance;
use crate::surface::Surface;
use ash::khr::swapchain;
use ash::vk;

/// Owns a `VkSwapchainKHR` and one `VkImageView` per swapchain image.
///
/// Must be dropped before [`Device`], [`Surface`], and [`Instance`].
pub struct Swapchain {
    raw: vk::SwapchainKHR,
    loader: swapchain::Device,
    device: ash::Device,
    image_views: Vec<vk::ImageView>,
    format: vk::Format,
    extent: vk::Extent2D,
}

impl Swapchain {
    /// Creates a swapchain for `surface`, sized to `width`x`height` unless
    /// the platform reports a fixed extent.
    ///
    /// # Panics
    ///
    /// Panics if querying surface support, or creating the swapchain, its
    /// images, or their views fails.
    pub fn new(
        instance: &Instance,
        device: &Device,
        surface: &Surface,
        width: u32,
        height: u32,
        vsync: bool,
    ) -> Self {
        let physical = device.physical();
        let capabilities = surface.capabilities(physical);
        let formats = surface.formats(physical);
        let present_modes = surface.present_modes(physical);

        let format = Self::choose_format(&formats);
        let present_mode = Self::choose_present_mode(&present_modes, vsync);
        let extent = Self::choose_extent(&capabilities, width, height);

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 {
            image_count = image_count.min(capabilities.max_image_count);
        }

        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface.raw())
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let loader = swapchain::Device::new(instance.raw(), device.raw());

        // SAFETY: `create_info` and everything it borrows outlive this call.
        let raw = unsafe { loader.create_swapchain(&create_info, None) }
            .expect("failed to create swapchain");

        // SAFETY: `raw` was just created by `loader`.
        let images =
            unsafe { loader.get_swapchain_images(raw) }.expect("failed to get swapchain images");

        let image_views = images
            .into_iter()
            .map(|image| Self::create_image_view(device.raw(), image, format.format))
            .collect();

        Self {
            raw,
            loader,
            device: device.raw().clone(),
            image_views,
            format: format.format,
            extent,
        }
    }

    /// Returns the raw [`vk::SwapchainKHR`].
    #[inline]
    pub fn raw(&self) -> vk::SwapchainKHR {
        self.raw
    }

    /// Returns the pixel format of the swapchain images.
    #[inline]
    pub fn format(&self) -> vk::Format {
        self.format
    }

    /// Returns the swapchain images' size in pixels.
    #[inline]
    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    /// Returns one [`vk::ImageView`] per swapchain image.
    #[inline]
    pub fn image_views(&self) -> &[vk::ImageView] {
        &self.image_views
    }

    /// Returns the next image to render into, or `None` if the swapchain
    /// needs recreating.
    ///
    /// # Panics
    ///
    /// Panics if the acquire fails for any other reason.
    pub fn acquire_next_image(&self, image_available: vk::Semaphore) -> Option<(u32, bool)> {
        // SAFETY: `image_available` is a semaphore from the same device
        // this swapchain was created on, and isn't already pending a
        // signal from another acquire.
        let result = unsafe {
            self.loader
                .acquire_next_image(self.raw, u64::MAX, image_available, vk::Fence::null())
        };

        match result {
            Ok((index, suboptimal)) => Some((index, suboptimal)),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => None,
            Err(err) => panic!("failed to acquire next swapchain image: {err}"),
        }
    }

    /// Presents `image_index` once `render_finished` is signaled, or returns
    /// `None` if the swapchain needs recreating.
    ///
    /// # Panics
    ///
    /// Panics if presenting fails for any other reason.
    pub fn present(
        &self,
        queue: vk::Queue,
        image_index: u32,
        render_finished: vk::Semaphore,
    ) -> Option<bool> {
        let wait_semaphores = [render_finished];
        let swapchains = [self.raw];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        // SAFETY: `queue` belongs to the same device this swapchain was
        // created on; `present_info` and everything it borrows outlive
        // this call.
        let result = unsafe { self.loader.queue_present(queue, &present_info) };

        match result {
            Ok(suboptimal) => Some(suboptimal),
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => None,
            Err(err) => panic!("failed to present swapchain image: {err}"),
        }
    }

    fn create_image_view(
        device: &ash::Device,
        image: vk::Image,
        format: vk::Format,
    ) -> vk::ImageView {
        let create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping::default())
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        // SAFETY: `image` belongs to the swapchain `device` was used to
        // create, which outlives this view (see struct-level ordering).
        unsafe { device.create_image_view(&create_info, None) }
            .expect("failed to create swapchain image view")
    }

    fn choose_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        // UNORM, not SRGB: an SRGB *format* makes the hardware apply an
        // automatic linear-to-sRGB encode on every write, on top of
        // `Vertex::color` values that are already final, as-authored 8-bit
        // color (from `Rgba`/`Rgba::from_hex`) - not linear light. With an
        // SRGB format that encode happens a second time, brightening every
        // color (confirmed empirically: `#1e1e1e` rendered as `#606060`,
        // matching the sRGB encode of 30/255 almost exactly). The
        // `SRGB_NONLINEAR` color space is still correct here - it's what
        // the display expects the final bytes to mean, independent of
        // whether the storage format itself auto-encodes.
        formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_UNORM
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .copied()
            .unwrap_or(formats[0])
    }

    fn choose_present_mode(modes: &[vk::PresentModeKHR], vsync: bool) -> vk::PresentModeKHR {
        // FIFO waits for the display's refresh - that *is* vsync, and it's
        // the only mode required to always be supported. MAILBOX presents
        // the newest finished frame without waiting, so it doesn't tear
        // either but lets the app run past the refresh rate.
        if !vsync && modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            vk::PresentModeKHR::FIFO
        }
    }

    fn choose_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        width: u32,
        height: u32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }

        vk::Extent2D {
            width: width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        for &view in &self.image_views {
            // SAFETY: each view was created from `self.device`, which is
            // still alive (see struct-level ordering requirement).
            unsafe { self.device.destroy_image_view(view, None) };
        }
        // SAFETY: called once, before `Device`/`Surface`/`Instance` drop.
        unsafe { self.loader.destroy_swapchain(self.raw, None) };
    }
}
