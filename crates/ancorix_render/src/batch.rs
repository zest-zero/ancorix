use ancorix_ash::DrawBatch;

/// One frame's draw calls, in the order the commands were queued, and the
/// push constant bytes they index into.
// Filled by `Renderer::prepare_frame` and owned by the caller across frames,
// so the per-frame `Vec` is reused rather than reallocated.
#[derive(Default)]
pub struct FrameBatch {
    pub batches: Vec<DrawBatch>,
    pub push_constants: Vec<u8>,
}

impl FrameBatch {
    /// Returns an empty [`FrameBatch`], to be filled by
    /// [`crate::Renderer::prepare_frame`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }
}
