use crate::device::Device;
use ash::vk;

/// How many frames the CPU may have in flight before it must wait.
// Anything the CPU writes and the GPU later reads needs one instance per
// slot, indexed by `frame_index % FRAMES_IN_FLIGHT`.
pub const FRAMES_IN_FLIGHT: usize = 2;

/// One frame-in-flight slot's acquire-to-submit synchronization. Must be
/// dropped before [`Device`].
// No "render finished" semaphore here on purpose - that one is per swapchain
// image, see `Commands`.
pub struct FrameSync {
    image_available: vk::Semaphore,
    in_flight: vk::Fence,
    device: ash::Device,
}

impl FrameSync {
    /// # Panics
    ///
    /// Panics if semaphore or fence creation fails.
    pub fn new(device: &Device) -> Self {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        // starts signaled so the first `wait` doesn't block forever
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        // SAFETY: the create-infos above outlive these calls.
        let image_available = unsafe { device.raw().create_semaphore(&semaphore_info, None) }
            .expect("failed to create semaphore");
        let in_flight = unsafe { device.raw().create_fence(&fence_info, None) }
            .expect("failed to create fence");

        Self {
            image_available,
            in_flight,
            device: device.raw().clone(),
        }
    }

    /// Returns the semaphore signaled once an image is ready to render into.
    #[inline]
    pub fn image_available(&self) -> vk::Semaphore {
        self.image_available
    }

    /// Returns the fence signaled once the GPU is done with this frame.
    #[inline]
    pub fn in_flight(&self) -> vk::Fence {
        self.in_flight
    }

    /// Blocks until the previous frame in this slot has finished on the GPU.
    ///
    /// # Panics
    ///
    /// Panics if waiting on the fence fails.
    pub fn wait(&self) {
        let fences = [self.in_flight];
        // SAFETY: no precondition beyond a valid device/fence.
        unsafe {
            self.device
                .wait_for_fences(&fences, true, u64::MAX)
                .expect("failed to wait for fence");
        }
    }

    /// Resets the fence so a following `queue_submit` can signal it again.
    ///
    /// # Panics
    ///
    /// Panics if resetting the fence fails, and deadlocks a later frame if
    /// no `queue_submit` follows.
    // Separate from `wait` for exactly that reason: only a submit can signal
    // a fence, so resetting before `acquire_next_image` has succeeded leaves
    // it unsignaled forever if the frame then bails out, and the next wait on
    // this slot never returns.
    pub fn reset(&self) {
        let fences = [self.in_flight];
        // SAFETY: no precondition beyond a valid device/fence.
        unsafe {
            self.device
                .reset_fences(&fences)
                .expect("failed to reset fence");
        }
    }
}

impl Drop for FrameSync {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped
        // (see the struct-level ordering requirement), and only once the
        // GPU is done with this frame (enforced by `wait` having been
        // called at least once per submission).
        unsafe {
            self.device.destroy_semaphore(self.image_available, None);
            self.device.destroy_fence(self.in_flight, None);
        }
    }
}
