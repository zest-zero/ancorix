#[cfg(feature = "unstable_shaders")]
use ancorix_ash::Pipeline;
use ancorix_ash::{Device, DrawBatch, FRAMES_IN_FLIGHT, Instance, RenderPass, Texture};
use ancorix_asset::{Assets, Handle};
use ancorix_draw::DrawCmd;
use ancorix_math::Vector2;
use ash::vk;
use rustc_hash::FxHashMap;

use crate::batch::FrameBatch;
use crate::buffers::GeometryBuffers;
use crate::geometry::{self, Geometry, RunKind};
use crate::pipelines::{GLOBALS_SIZE, Pipelines};
#[cfg(feature = "unstable_shaders")]
use crate::vertex::ShadedVertex;
use crate::vertex::{RoundedRectInstance, SdfVertex, SpriteVertex, Vertex};

// Vulkan wants whole pixels inside the framebuffer, while a clip rect is
// arbitrary floats: the rect is expanded to whole pixels so nothing that
// should be visible is shaved off, then clamped, since a scissor reaching
// outside the framebuffer is invalid rather than merely ignored.
fn scissor_of(rect: ancorix_draw::Rect, viewport: Vector2) -> vk::Rect2D {
    let min_x = rect.pos.x.floor().clamp(0.0, viewport.x);
    let min_y = rect.pos.y.floor().clamp(0.0, viewport.y);
    let max_x = (rect.pos.x + rect.size.x).ceil().clamp(min_x, viewport.x);
    let max_y = (rect.pos.y + rect.size.y).ceil().clamp(min_y, viewport.y);

    vk::Rect2D {
        offset: vk::Offset2D {
            x: min_x as i32,
            y: min_y as i32,
        },
        extent: vk::Extent2D {
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        },
    }
}

/// Batches `DrawCmd`s into vertex/index data and uploads it to the GPU every
/// frame. Must be dropped before [`Device`].
pub struct Renderer {
    render_pass: RenderPass,
    pipelines: Pipelines,
    // one descriptor set per `Handle<Texture>` ever drawn - a texture's
    // view/sampler never change, so the set is valid forever once made.
    // `FxHashMap`, not the std one: the key is a `u32` looked up once per
    // sprite run per frame, and SipHash is built for hostile string keys.
    descriptor_cache: FxHashMap<Handle<Texture>, vk::DescriptorSet>,

    // one pipeline per user shader, built the first frame it is drawn with -
    // the same lazy-cache shape as `descriptor_cache`, and for the same
    // reason: nothing about a `Handle<Shader>` ever changes afterwards.
    #[cfg(feature = "unstable_shaders")]
    shader_pipelines: FxHashMap<Handle<ancorix_shader::Shader>, Pipeline>,

    // one set per frame-in-flight slot (see `ancorix_ash::FRAMES_IN_FLIGHT`)
    // - `prepare_frame`'s caller writes into `solid_buffers[frame_slot]`
    // from the CPU while the GPU may still be reading a *different* slot's
    // buffers from an earlier, not-yet-finished frame. A single shared set
    // would race the CPU write against that in-flight GPU read.
    solid_buffers: Vec<GeometryBuffers<Vertex>>,
    sdf_buffers: Vec<GeometryBuffers<SdfVertex>>,
    rounded_rect_buffers: Vec<GeometryBuffers<RoundedRectInstance>>,
    sprite_buffers: Vec<GeometryBuffers<SpriteVertex>>,
    #[cfg(feature = "unstable_shaders")]
    shaded_buffers: Vec<GeometryBuffers<ShadedVertex>>,

    geometry: Geometry,
}

impl Renderer {
    /// Returns a renderer owning the shared render pass, the four pipelines,
    /// and initial (growable) vertex/index buffers for each.
    ///
    /// # Panics
    ///
    /// Panics if shader, pipeline, or buffer creation fails.
    pub fn new(instance: &Instance, device: &Device, color_format: vk::Format) -> Self {
        let render_pass = RenderPass::new(device, color_format);
        let pipelines = Pipelines::new(device, &render_pass);

        Self {
            render_pass,
            pipelines,
            descriptor_cache: FxHashMap::default(),
            #[cfg(feature = "unstable_shaders")]
            shader_pipelines: FxHashMap::default(),
            solid_buffers: Self::per_frame_buffers(instance, device),
            sdf_buffers: Self::per_frame_buffers(instance, device),
            rounded_rect_buffers: Self::per_frame_buffers(instance, device),
            sprite_buffers: Self::per_frame_buffers(instance, device),
            #[cfg(feature = "unstable_shaders")]
            shaded_buffers: Self::per_frame_buffers(instance, device),
            geometry: Geometry::new(),
        }
    }

    /// Returns the render pass shared by all pipelines - framebuffers must be
    /// created against it.
    #[inline]
    pub fn render_pass(&self) -> &RenderPass {
        &self.render_pass
    }

    /// Triangulates `cmds` for a `viewport`-sized framebuffer, uploads the
    /// geometry into slot `frame_slot`, and fills `out` with the frame's draw
    /// calls in the order the commands were queued.
    ///
    /// # Panics
    ///
    /// Panics if a grown buffer's allocation fails, `frame_slot` is out of
    /// range (it must be `frame_index % ancorix_ash::FRAMES_IN_FLIGHT`), or
    /// `cmds` draws a sprite whose `Handle<Texture>` isn't in `textures`.
    #[allow(clippy::too_many_arguments)] // every one is a distinct borrowed
    // resource or scalar of its own type, nothing a wrapper could stop the
    // caller from mixing up
    pub fn prepare_frame(
        &mut self,
        instance: &Instance,
        device: &Device,
        cmds: &[DrawCmd],
        viewport: Vector2,
        frame_slot: usize,
        time: f32,
        textures: &Assets<Texture>,
        #[cfg(feature = "unstable_shaders")] shaders: &Assets<ancorix_shader::Shader>,
        #[cfg(feature = "unstable_shaders")] params: &[u8],
        out: &mut FrameBatch,
    ) {
        // The caller must already have waited on `frame_slot`'s fence: growing
        // a buffer below drops the old one at once, and the GPU may still be
        // reading it otherwise.
        geometry::triangulate(cmds, viewport, time, textures, &mut self.geometry);
        let geometry = &self.geometry;

        let solid_buffers = &mut self.solid_buffers[frame_slot];
        let sdf_buffers = &mut self.sdf_buffers[frame_slot];
        let rounded_rect_buffers = &mut self.rounded_rect_buffers[frame_slot];
        let sprite_buffers = &mut self.sprite_buffers[frame_slot];

        solid_buffers.upload(
            instance,
            device,
            &geometry.solid_vertices,
            &geometry.solid_indices,
        );
        sdf_buffers.upload(
            instance,
            device,
            &geometry.sdf_vertices,
            &geometry.sdf_indices,
        );
        // instanced: one record per shape, and no index buffer at all - the
        // shader derives the quad's corners from `gl_VertexIndex`
        rounded_rect_buffers.upload(instance, device, &geometry.rounded_rect_instances, &[]);
        sprite_buffers.upload(
            instance,
            device,
            &geometry.sprite_vertices,
            &geometry.sprite_indices,
        );
        #[cfg(feature = "unstable_shaders")]
        let shaded_buffers = &mut self.shaded_buffers[frame_slot];
        #[cfg(feature = "unstable_shaders")]
        shaded_buffers.upload(
            instance,
            device,
            &geometry.shaded_vertices,
            &geometry.shaded_indices,
        );

        // One batch per run, all of them drawing out of their pipeline's
        // single buffer pair by range - splitting a frame into runs costs no
        // extra GPU buffers, only extra pipeline binds.
        out.batches.clear();
        out.push_constants.clear();
        // every batch starts from the same globals; a shaded one appends its
        // own parameters, so each batch points at its own slice
        let globals: [u8; 16] = bytemuck::cast([viewport.x, viewport.y, time, 0.0]);
        out.push_constants.extend_from_slice(&globals);

        out.batches.extend(geometry.runs.iter().map(|run| {
            let scissor = run.clip.map(|rect| scissor_of(rect, viewport));

            match run.kind {
                RunKind::Solid => DrawBatch {
                    pipeline: self.pipelines.solid.raw(),
                    layout: self.pipelines.solid.layout(),
                    vertex_buffer: solid_buffers.vertex_buffer.raw(),
                    index_buffer: solid_buffers.index_buffer.raw(),
                    first_index: run.first,
                    index_count: run.count,
                    vertex_count: 0,
                    instance_count: 0,
                    first_instance: 0,
                    first_push_constant: 0,
                    push_constant_len: GLOBALS_SIZE,
                    scissor,
                    descriptor_set: None,
                },
                #[cfg(feature = "unstable_shaders")]
                RunKind::SolidShaded(slot) => {
                    let pipeline = self.shader_pipelines.entry(slot.shader).or_insert_with(|| {
                        let shader = shaders
                            .get(slot.shader)
                            .expect("Handle<Shader> doesn't belong to this Assets registry");

                        crate::pipelines::user_shaded(device, &self.render_pass, shader.fragment())
                    });

                    let first_push_constant = out.push_constants.len() as u32;
                    out.push_constants.extend_from_slice(&globals);
                    let from = slot.first_param as usize;
                    out.push_constants
                        .extend_from_slice(&params[from..from + slot.param_len as usize]);

                    DrawBatch {
                        pipeline: pipeline.raw(),
                        layout: pipeline.layout(),
                        vertex_buffer: shaded_buffers.vertex_buffer.raw(),
                        index_buffer: shaded_buffers.index_buffer.raw(),
                        first_index: run.first,
                        index_count: run.count,
                        vertex_count: 0,
                        instance_count: 0,
                        first_instance: 0,
                        scissor,
                        first_push_constant,
                        push_constant_len: GLOBALS_SIZE + slot.param_len,
                        descriptor_set: None,
                    }
                }
                RunKind::Sdf => DrawBatch {
                    pipeline: self.pipelines.sdf.raw(),
                    layout: self.pipelines.sdf.layout(),
                    vertex_buffer: sdf_buffers.vertex_buffer.raw(),
                    index_buffer: sdf_buffers.index_buffer.raw(),
                    first_index: run.first,
                    index_count: run.count,
                    vertex_count: 0,
                    instance_count: 0,
                    first_instance: 0,
                    first_push_constant: 0,
                    push_constant_len: GLOBALS_SIZE,
                    scissor,
                    descriptor_set: None,
                },
                RunKind::RoundedRect => DrawBatch {
                    pipeline: self.pipelines.rounded_rect.raw(),
                    layout: self.pipelines.rounded_rect.layout(),
                    vertex_buffer: rounded_rect_buffers.vertex_buffer.raw(),
                    index_buffer: rounded_rect_buffers.index_buffer.raw(),
                    first_index: 0,
                    index_count: 0,
                    vertex_count: RoundedRectInstance::VERTEX_COUNT,
                    instance_count: run.count,
                    first_instance: run.first,
                    first_push_constant: 0,
                    push_constant_len: GLOBALS_SIZE,
                    scissor,
                    descriptor_set: None,
                },
                RunKind::Sprite(handle) => {
                    let descriptor_set =
                        *self.descriptor_cache.entry(handle).or_insert_with(|| {
                            let texture = textures
                                .get(handle)
                                .expect("Handle<Texture> doesn't belong to this Assets registry");
                            self.pipelines
                                .texture_descriptors
                                .allocate(texture.view(), texture.sampler())
                        });

                    DrawBatch {
                        pipeline: self.pipelines.sprite.raw(),
                        layout: self.pipelines.sprite.layout(),
                        vertex_buffer: sprite_buffers.vertex_buffer.raw(),
                        index_buffer: sprite_buffers.index_buffer.raw(),
                        first_index: run.first,
                        index_count: run.count,
                        vertex_count: 0,
                        instance_count: 0,
                        first_instance: 0,
                        first_push_constant: 0,
                        push_constant_len: GLOBALS_SIZE,
                        scissor,
                        descriptor_set: Some(descriptor_set),
                    }
                }
            }
        }));

        // `Globals` in shaders/lib/ancorix.slang: screen_size, time, padding
    }

    fn per_frame_buffers<T: bytemuck::Pod>(
        instance: &Instance,
        device: &Device,
    ) -> Vec<GeometryBuffers<T>> {
        (0..FRAMES_IN_FLIGHT)
            .map(|_| GeometryBuffers::new(instance, device))
            .collect()
    }
}
