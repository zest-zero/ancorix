use crate::device::Device;
use crate::instance::Instance;
use crate::sync::FRAMES_IN_FLIGHT;
use ash::vk;

/// Measures GPU frame time, one start/end pair per frame-in-flight slot.
/// Must be dropped before [`Device`].
pub struct GpuTimer {
    pool: vk::QueryPool,
    period_ns: f64,
    device: ash::Device,
}

impl GpuTimer {
    /// # Panics
    ///
    /// Panics if timestamp queries aren't supported, or pool creation fails.
    pub fn new(instance: &Instance, device: &Device) -> Self {
        // SAFETY: `device.physical()` was enumerated from `instance`.
        let properties = unsafe {
            instance
                .raw()
                .get_physical_device_properties(device.physical())
        };
        let period_ns = properties.limits.timestamp_period as f64;
        assert!(period_ns > 0.0, "device doesn't support timestamp queries");

        let create_info = vk::QueryPoolCreateInfo::default()
            .query_type(vk::QueryType::TIMESTAMP)
            .query_count(Self::query_count());

        // SAFETY: `create_info` outlives this call.
        let pool = unsafe { device.raw().create_query_pool(&create_info, None) }
            .expect("failed to create query pool");

        Self {
            pool,
            period_ns,
            device: device.raw().clone(),
        }
    }

    /// Writes the start timestamp for `frame_slot`.
    pub fn cmd_begin(&self, command_buffer: vk::CommandBuffer, frame_slot: usize) {
        assert!(frame_slot < FRAMES_IN_FLIGHT, "frame_slot out of range");
        let base = Self::base_query(frame_slot);

        // SAFETY: `command_buffer` is recording; `base..base+2` in range.
        unsafe {
            self.device
                .cmd_reset_query_pool(command_buffer, self.pool, base, 2);
            self.device.cmd_write_timestamp(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                self.pool,
                base,
            );
        }
    }

    /// Writes the end timestamp for `frame_slot`.
    pub fn cmd_end(&self, command_buffer: vk::CommandBuffer, frame_slot: usize) {
        assert!(frame_slot < FRAMES_IN_FLIGHT, "frame_slot out of range");
        let base = Self::base_query(frame_slot);

        // SAFETY: same as `cmd_begin`.
        unsafe {
            self.device.cmd_write_timestamp(
                command_buffer,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                self.pool,
                base + 1,
            );
        }
    }

    /// Returns `frame_slot`'s last recorded GPU time, or `None` if it isn't
    /// ready yet.
    // Never blocks, so only call once that slot's fence is known signaled.
    ///
    /// # Panics
    ///
    /// Panics if querying the pool fails for any reason other than the
    /// result not being ready yet.
    pub fn read(&self, frame_slot: usize) -> Option<std::time::Duration> {
        assert!(frame_slot < FRAMES_IN_FLIGHT, "frame_slot out of range");
        let base = Self::base_query(frame_slot);

        let mut timestamps = [0u64; 2];
        // SAFETY: `base..base+2` in range, `timestamps` sized to match.
        let result = unsafe {
            self.device.get_query_pool_results(
                self.pool,
                base,
                &mut timestamps,
                vk::QueryResultFlags::TYPE_64,
            )
        };

        match result {
            Ok(()) => {
                let delta_ticks = timestamps[1].saturating_sub(timestamps[0]);
                let nanos = delta_ticks as f64 * self.period_ns;
                Some(std::time::Duration::from_nanos(nanos as u64))
            }
            Err(vk::Result::NOT_READY) => None,
            Err(err) => panic!("failed to read query pool results: {err}"),
        }
    }

    const fn base_query(frame_slot: usize) -> u32 {
        (frame_slot * 2) as u32
    }

    const fn query_count() -> u32 {
        (FRAMES_IN_FLIGHT * 2) as u32
    }
}

impl Drop for GpuTimer {
    fn drop(&mut self) {
        // SAFETY: called once, before the owning `Device` is dropped.
        unsafe { self.device.destroy_query_pool(self.pool, None) };
    }
}
