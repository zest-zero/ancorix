use ancorix_ash::{Buffer, Device, Instance};
use ash::vk;
use std::marker::PhantomData;

// Provisional starting capacities, shared by the GPU buffers here and the
// CPU-side scratch vectors in `Renderer` - large enough that a typical frame
// (a handful to a few hundred shapes) never triggers a reallocation, but
// not measured. Revisit with a benchmark once real scenes exist.
pub(crate) const INITIAL_VERTEX_CAPACITY: usize = 1024;
pub(crate) const INITIAL_INDEX_CAPACITY: usize = 1536;

// A growable vertex/index buffer pair for one pipeline's vertex type `T`.
// Must be dropped before `Device`.
pub(crate) struct GeometryBuffers<T> {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) index_buffer: Buffer,
    vertex_capacity: usize,
    index_capacity: usize,
    _vertex: PhantomData<T>,
}

impl<T: bytemuck::Pod> GeometryBuffers<T> {
    pub(crate) fn new(instance: &Instance, device: &Device) -> Self {
        Self {
            vertex_buffer: Self::create_vertex_buffer(instance, device, INITIAL_VERTEX_CAPACITY),
            index_buffer: Self::create_index_buffer(instance, device, INITIAL_INDEX_CAPACITY),
            vertex_capacity: INITIAL_VERTEX_CAPACITY,
            index_capacity: INITIAL_INDEX_CAPACITY,
            _vertex: PhantomData,
        }
    }

    // Growing drops the old buffer immediately - only sound because
    // `Renderer::prepare_frame`'s caller already waited on this slot's fence.
    pub(crate) fn upload(
        &mut self,
        instance: &Instance,
        device: &Device,
        vertices: &[T],
        indices: &[u32],
    ) {
        if vertices.len() > self.vertex_capacity {
            self.vertex_capacity = (self.vertex_capacity * 2).max(vertices.len());
            self.vertex_buffer = Self::create_vertex_buffer(instance, device, self.vertex_capacity);
        }
        if indices.len() > self.index_capacity {
            self.index_capacity = (self.index_capacity * 2).max(indices.len());
            self.index_buffer = Self::create_index_buffer(instance, device, self.index_capacity);
        }

        self.vertex_buffer.write(bytemuck::cast_slice(vertices));
        self.index_buffer.write(bytemuck::cast_slice(indices));
    }

    fn create_vertex_buffer(instance: &Instance, device: &Device, capacity: usize) -> Buffer {
        Buffer::new(
            instance,
            device,
            (capacity * size_of::<T>()) as vk::DeviceSize,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
    }

    fn create_index_buffer(instance: &Instance, device: &Device, capacity: usize) -> Buffer {
        Buffer::new(
            instance,
            device,
            (capacity * size_of::<u32>()) as vk::DeviceSize,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
    }
}
