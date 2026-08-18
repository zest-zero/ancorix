use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use ancorix_ash::{
    Commands, Device, FRAMES_IN_FLIGHT, FrameSync, GpuTimer, Instance, Surface, Swapchain, Texture,
};
use ancorix_color::Rgba;
use ancorix_ctx::App;
use ancorix_ctx::{Ctx, Time, WindowInfo};
use ancorix_draw::Draw;
use ancorix_input::Input;
use ancorix_math::Vector2;
use ancorix_render::{FrameBatch, Renderer};
use ash::vk;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window as WinitWindow, WindowAttributes, WindowId};

use crate::Window;

// how often to re-check whether the window's real size has settled before
// running `App::init` - see `Runner::ensure_app_initialized`/`about_to_wait`.
const INIT_SETTLE_CHECK_INTERVAL: Duration = Duration::from_millis(50);

pub(crate) struct Runner<A: App> {
    config: Window,

    // drop order matters: `sync`/`commands`/`renderer`/`gpu_timer` before
    // `swapchain`, `swapchain` before `device`, `device` before `surface`,
    // `surface` before `window` (it borrows the OS window's lifetime), and
    // `instance` last (everything above depends on it).
    //
    // `sync` holds one `FrameSync` per frame-in-flight slot (empty until
    // `resumed()` creates the device) rather than `Option<FrameSync>` -
    // the empty `Vec` already serves as the "not yet initialized" sentinel
    // the `Option` fields below use for the same reason.
    sync: Vec<FrameSync>,
    frame_index: usize,
    commands: Option<Commands>,
    renderer: Option<Renderer>,

    // reused every frame so the batch list isn't reallocated per frame
    frame_batch: FrameBatch,
    // internal profiling only, off by default - see `ensure_gpu_timer`.
    // Must be declared (and therefore dropped) before `device` - see its
    // own struct-level doc comment.
    gpu_timer: Option<GpuTimer>,
    // loaded via `ctx.assets` - must be declared (and therefore dropped)
    // before `device`, same reason as `gpu_timer` (each `Texture` owns
    // GPU resources tied to `device`).
    textures: ancorix_asset::Assets<Texture>,

    #[cfg(feature = "unstable_shaders")]
    shaders: ancorix_asset::Assets<ancorix_shader::Shader>,
    swapchain: Option<Swapchain>,
    device: Option<Device>,
    surface: Option<Surface>,
    window: Option<WinitWindow>,
    instance: Option<Instance>,

    input: Input,
    time: Time,
    window_info: WindowInfo,
    draw: Draw,
    last_tick: Instant,
    app: Option<A>,

    // size seen the *previous* time `ensure_app_initialized` checked,
    // while `app` is still `None` - see that method's doc comment.
    init_size_checkpoint: Option<Vector2>,

    // last value applied to the real OS cursor, so `tick` only calls
    // `set_cursor_visible` on an actual change, not every frame.
    applied_cursor_visible: bool,
}

impl<A: App> Runner<A> {
    pub(crate) fn new(config: Window) -> Self {
        let window_info = WindowInfo::new(config.width, config.height);
        Self {
            config,
            sync: Vec::new(),
            frame_index: 0,
            commands: None,
            renderer: None,
            frame_batch: FrameBatch::new(),
            swapchain: None,
            device: None,
            surface: None,
            window: None,
            instance: None,
            input: Input::new(),
            time: Time::new(),
            window_info,
            draw: Draw::new(),
            last_tick: Instant::now(),
            app: None,
            init_size_checkpoint: None,
            gpu_timer: None,
            textures: ancorix_asset::Assets::new(),
            #[cfg(feature = "unstable_shaders")]
            shaders: ancorix_asset::Assets::new(),
            applied_cursor_visible: true,
        }
    }

    // Picks up `*.project.json(5)` and applies its `input` and `window`
    // sections. Searched in order: `Window::project_dir` if given, then the
    // running executable's directory, then the working directory (the
    // common case during `cargo run`, where the exe lives under
    // `target/debug/` instead of the project root).
    //
    // Must run before the window is created - the `window` section feeds
    // `self.config`, which is what `create_window` reads.
    fn load_project_config(&mut self) {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf));
        let cwd = std::env::current_dir().ok();

        let Some(project_file) = [self.config.project_dir.clone(), exe_dir, cwd]
            .into_iter()
            .flatten()
            .find_map(|dir| find_project_file(&dir))
        else {
            return;
        };

        let src = std::fs::read_to_string(&project_file)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", project_file.display()));
        // json5 is a superset of json - accepts both .json and .json5
        let project: serde_json::Value = json5::from_str(&src)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", project_file.display()));

        if let Some(section) = project.get("input").and_then(|v| v.as_object()) {
            for (action, value) in section {
                let Some(keys) = value.as_array() else {
                    continue;
                };
                let keys: Vec<&str> = keys.iter().filter_map(|k| k.as_str()).collect();
                self.input.bind_key_names(action.as_str(), &keys);
            }
        }

        if let Some(section) = project.get("window").and_then(|v| v.as_object()) {
            self.apply_window_section(section);
        }
    }

    // The file wins over the builder: changing the window without a rebuild
    // is the whole reason the section exists, and `Window::new` always sets
    // a title and a size, so a builder-wins rule would make it dead weight.
    // Only keys actually written in the file are applied.
    fn apply_window_section(&mut self, section: &serde_json::Map<String, serde_json::Value>) {
        if let Some(title) = section.get("title").and_then(|v| v.as_str()) {
            self.config.title = title.to_string();
        }
        if let Some(width) = section.get("width").and_then(|v| v.as_u64()) {
            self.config.width = width as u32;
        }
        if let Some(height) = section.get("height").and_then(|v| v.as_u64()) {
            self.config.height = height as u32;
        }
        if let Some(fps) = section.get("fps").and_then(|v| v.as_u64()) {
            // 0 means "uncapped" in the file, and `None` means the same to
            // the runner
            self.config.target_fps = (fps > 0).then_some(fps as u32);
        }
        if let Some(vsync) = section.get("vsync").and_then(|v| v.as_bool()) {
            self.config.vsync = vsync;
        }
    }

    // Runs one frame. Returns `true` if the app requested exit via
    // [`ancorix_ctx::WindowInfo::request_exit`] this frame.
    fn tick(&mut self) -> bool {
        let now = Instant::now();
        self.time
            .advance(now.duration_since(self.last_tick).as_secs_f32());
        self.last_tick = now;

        let mut ctx = Ctx::new(
            &mut self.input,
            self.time,
            self.window_info,
            &mut self.draw,
            self.instance.as_ref().unwrap(),
            self.device.as_ref().unwrap(),
            &mut self.textures,
            #[cfg(feature = "unstable_shaders")]
            &mut self.shaders,
        );
        if let Some(app) = &mut self.app {
            app.frame(&mut ctx);
        }
        self.time = ctx.time;
        self.window_info = ctx.window;

        let cursor_visible = self.window_info.cursor_visible();
        if cursor_visible != self.applied_cursor_visible {
            self.applied_cursor_visible = cursor_visible;
            if let Some(window) = &self.window {
                window.set_cursor_visible(cursor_visible);
            }
        }

        self.input.begin_frame();
        self.window_info.begin_frame();

        self.window_info.exit_requested()
    }

    // Draws and presents one frame. A no-op while the window is
    // minimized (zero-sized) - there's nothing to draw into.
    fn render(&mut self) {
        let window = self.window.as_ref().unwrap();
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            self.draw.flush();
            return;
        }

        let current_extent = self.swapchain.as_ref().unwrap().extent();
        if current_extent.width != size.width || current_extent.height != size.height {
            self.recreate_swapchain();
        }

        // pick this frame's slot before touching any slot-indexed state, so
        // every early return below still advances it - once `sync.wait()`
        // below returns, this slot's previous occupant is done, so there's
        // no reason to retry the same slot next frame even if this frame
        // ends up not submitting anything (see the `acquire_next_image`
        // early return below).
        let frame_slot = self.frame_index % FRAMES_IN_FLIGHT;
        self.frame_index = self.frame_index.wrapping_add(1);

        let instance = self.instance.as_ref().unwrap();
        let device = self.device.as_ref().unwrap();
        let sync = &self.sync[frame_slot];
        sync.wait();

        // safe to read here specifically: this slot's fence (waited on
        // just above) is what guarantees the GPU work that wrote these
        // timestamps last time around has finished.
        if let Some(timer) = &self.gpu_timer
            && self.frame_index.is_multiple_of(120)
            && let Some(gpu_time) = timer.read(frame_slot)
        {
            eprintln!("gpu frame time: {gpu_time:?}");
        }

        let swapchain = self.swapchain.as_ref().unwrap();
        // note: `sync`'s fence is *not* reset yet at this point - only
        // `queue_submit` can signal a fence, so it stays untouched (still
        // signaled from the wait above) until a submit for this frame is
        // guaranteed. Resetting any earlier and then bailing out here
        // without submitting would leave it permanently unsignaled - see
        // `FrameSync::reset`'s doc comment.
        let Some((image_index, _suboptimal)) = swapchain.acquire_next_image(sync.image_available())
        else {
            self.recreate_swapchain();
            self.draw.flush();
            return;
        };

        let extent = swapchain.extent();
        let viewport = Vector2::new(extent.width as f32, extent.height as f32);
        let renderer = self.renderer.as_mut().unwrap();
        renderer.prepare_frame(
            instance,
            device,
            self.draw.commands(),
            viewport,
            frame_slot,
            self.time.elapsed(),
            &self.textures,
            #[cfg(feature = "unstable_shaders")]
            &self.shaders,
            #[cfg(feature = "unstable_shaders")]
            self.draw.shader_params(),
            &mut self.frame_batch,
        );
        self.draw.flush();

        let commands = self.commands.as_ref().unwrap();
        commands.record_frame(
            image_index,
            renderer.render_pass(),
            extent,
            &self.frame_batch.batches,
            &self.frame_batch.push_constants,
            self.gpu_timer.as_ref().map(|timer| (timer, frame_slot)),
        );

        // signaled per swapchain image, not per frame-in-flight slot - see
        // `ancorix_ash::Commands::render_finished` for why.
        let render_finished = commands.render_finished(image_index);

        let wait_semaphores = [sync.image_available()];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = [commands.buffer(image_index)];
        let signal_semaphores = [render_finished];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        // reset right before the submit that's about to signal it again -
        // not any earlier (see `FrameSync::reset`'s doc comment for why).
        sync.reset();

        // SAFETY: `submit_info` and everything it borrows outlive this
        // call; `sync.in_flight()` isn't currently pending (waited on and
        // reset above), so the driver is free to signal it again.
        unsafe {
            device
                .raw()
                .queue_submit(device.graphics_queue(), &[submit_info], sync.in_flight())
        }
        .expect("failed to submit command buffer");

        if swapchain
            .present(device.graphics_queue(), image_index, render_finished)
            .is_none()
        {
            self.recreate_swapchain();
        }
    }

    fn recreate_swapchain(&mut self) {
        let device = self.device.as_ref().unwrap();
        device.wait_idle();

        self.commands = None;
        self.swapchain = None;

        let size = self.window.as_ref().unwrap().inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }

        let instance = self.instance.as_ref().unwrap();
        let surface = self.surface.as_ref().unwrap();
        let renderer = self.renderer.as_ref().unwrap();

        let swapchain = Swapchain::new(
            instance,
            device,
            surface,
            size.width,
            size.height,
            self.config.vsync,
        );
        let commands = Commands::new(device, renderer.render_pass(), &swapchain);

        self.swapchain = Some(swapchain);
        self.commands = Some(commands);
    }

    // Returns the exit code requested via
    // [`ancorix_ctx::WindowInfo::request_exit_with_code`], or `0` if none
    // was ever requested (including a plain window-close).
    pub(crate) fn exit_code(&self) -> u8 {
        self.window_info.exit_code()
    }

    // Runs [`App::init`] once the window's size has stopped changing
    // across consecutive checks - not called from `resumed()` itself, and
    // not necessarily on the first call either.
    //
    // On Wayland, a freshly created window's true geometry can still be
    // renegotiated asynchronously after creation (e.g. a fractional-scale
    // surface configure landing after the window already exists), so
    // `window.inner_size()` queried synchronously in `resumed()` can be
    // stale. `winit` gives no guarantee about how many `Resized` events
    // (or how long) this settling takes - both rust-windowing/winit#3192
    // and #2921 are open with no fix upstream - and `bevy` has the
    // identical symptom for the same reason (bevyengine/bevy#17188).
    //
    // Three narrower fixes were tried and measured live before this one,
    // each falsified by an actual repro rather than assumed correct:
    // waiting for just the first `RedrawRequested` (the real size can
    // land on the *second* one instead); comparing two checks made
    // back-to-back under `ControlFlow::Poll` (the CPU spins through both
    // before the platform's round-trip ever lands, so they agree on the
    // stale value); and comparing checks spaced out by a real wall-clock
    // pause but *without ever presenting a frame* in between - GNOME/
    // Mutter turned out to only send the corrected configure once it
    // sees an actual committed buffer from the client, so a pure wait
    // deadlocks: nothing renders because the size hasn't settled, and the
    // size never settles because nothing rendered. The caller
    // (`window_event`'s `RedrawRequested` arm) presents an empty frame on
    // every check while `app` is still `None`, which is what actually
    // unblocks the compositor; this method only compares the resulting
    // sizes across checks spaced apart by `about_to_wait`
    // ([`INIT_SETTLE_CHECK_INTERVAL`]).
    fn ensure_app_initialized(&mut self) {
        if self.app.is_some() {
            return;
        }

        let current_size = self.window_info.size();
        if self.init_size_checkpoint != Some(current_size) {
            // not settled yet - `about_to_wait` schedules the next check
            // after a real pause, not another immediate redraw.
            self.init_size_checkpoint = Some(current_size);
            return;
        }

        self.last_tick = Instant::now();
        let mut ctx = Ctx::new(
            &mut self.input,
            self.time,
            self.window_info,
            &mut self.draw,
            self.instance.as_ref().unwrap(),
            self.device.as_ref().unwrap(),
            &mut self.textures,
            #[cfg(feature = "unstable_shaders")]
            &mut self.shaders,
        );
        self.app = Some(A::init(&mut ctx));
        self.time = ctx.time;
        self.window_info = ctx.window;
    }
}

// Looks for `*.project.json5` / `*.project.json` directly inside `dir`.
// Prefers `.json5` for comments, falling back to `.json`.
fn find_project_file(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                return false;
            };
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                return false;
            };
            stem.ends_with(".project") && (ext == "json5" || ext == "json")
        })
}

fn monitor_info(monitor: &winit::monitor::MonitorHandle) -> ancorix_ctx::MonitorInfo {
    let size = monitor.size();
    ancorix_ctx::MonitorInfo::new(
        monitor.scale_factor() as f32,
        Vector2::new(size.width as f32, size.height as f32),
    )
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        // before the window exists: the `window` section feeds `self.config`
        self.load_project_config();
        // `WindowInfo` was seeded in `Runner::new` from the pre-config size
        self.window_info
            .resize(self.config.width, self.config.height);

        // resolve the target monitor synchronously (available without a
        // window) so a fullscreen request can be seeded with its true size
        // up front, instead of waiting on the compositor's asynchronous
        // resize negotiation - see `WindowInfo::resized` doc comment for
        // why that negotiation can't be waited out reliably.
        // `primary_monitor()` can return `None` on Wayland - there's no
        // universal protocol for compositors to advertise one - so fall
        // back to just taking the first available monitor.
        let monitor = event_loop
            .primary_monitor()
            .or_else(|| event_loop.available_monitors().next());

        // `PhysicalSize`, not `LogicalSize`: `Window::new(w, h)` means
        // exactly `w`x`h` pixels, not device-independent units that the
        // OS silently multiplies by the monitor's DPI scale factor.
        // Confirmed live on GNOME at 100% vs 125% scale: with
        // `LogicalSize`, a requested 1280 came out as literally 1280px at
        // 100% but 1600px at 125% (1280 * 1.25 = 1600, exact). DPI-aware
        // proportional content is a real, separate need - handled
        // explicitly via `Window::virtual_resolution` (open question 20),
        // not by silently resizing the window itself.
        let attrs = WindowAttributes::default()
            .with_title(self.config.title.as_str())
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.config.width,
                self.config.height,
            ))
            .with_resizable(self.config.resizable)
            .with_fullscreen(
                self.config
                    .fullscreen
                    .then(|| winit::window::Fullscreen::Borderless(monitor.clone())),
            );

        self.window = Some(
            event_loop
                .create_window(attrs)
                .expect("failed to create window"),
        );
        let window = self.window.as_ref().unwrap();

        let display_handle = event_loop
            .display_handle()
            .expect("no display handle")
            .as_raw();
        let window_handle = window.window_handle().expect("no window handle").as_raw();

        let instance = Instance::new(display_handle);
        let surface = Surface::new(&instance, display_handle, window_handle);
        let device = Device::new(&instance, &surface);

        // for fullscreen, trust the monitor's own reported size over
        // `window.inner_size()` - immediately after creation the latter can
        // still reflect a pre-fullscreen intermediate size on Wayland.
        let size = match (self.config.fullscreen, &monitor) {
            (true, Some(monitor)) => monitor.size(),
            _ => window.inner_size(),
        };
        self.window_info = WindowInfo::new(size.width, size.height);
        if let Some(handle) = window.current_monitor().or_else(|| monitor.clone()) {
            self.window_info.set_monitor(monitor_info(&handle));
        }
        let swapchain = Swapchain::new(
            &instance,
            &device,
            &surface,
            size.width,
            size.height,
            self.config.vsync,
        );

        let renderer = Renderer::new(&instance, &device, swapchain.format());
        let commands = Commands::new(&device, renderer.render_pass(), &swapchain);
        let sync = (0..FRAMES_IN_FLIGHT)
            .map(|_| FrameSync::new(&device))
            .collect();

        // opt-in internal profiling, off by default - zero cost unless set.
        if std::env::var_os("ANCORIX_GPU_TIMING").is_some() {
            self.gpu_timer = Some(GpuTimer::new(&instance, &device));
        }

        self.instance = Some(instance);
        self.surface = Some(surface);
        self.device = Some(device);
        self.swapchain = Some(swapchain);
        self.renderer = Some(renderer);
        self.commands = Some(commands);
        self.sync = sync;

        self.window.as_ref().unwrap().request_redraw();
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(
            cause,
            StartCause::ResumeTimeReached { .. } | StartCause::Poll
        ) && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        ancorix_winit::feed_device_event(&mut self.input, &event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        ancorix_winit::feed_event(&mut self.input, &event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.window_info.resize(size.width, size.height),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // the window may have moved to a monitor with a different
                // DPI - `current_monitor()` is queried fresh rather than
                // reusing whatever `resumed()` resolved at startup
                if let Some(handle) = self.window.as_ref().and_then(WinitWindow::current_monitor) {
                    let mut info = monitor_info(&handle);
                    info.scale_factor = scale_factor as f32;
                    self.window_info.set_monitor(info);
                }
            }
            WindowEvent::RedrawRequested => {
                if self.app.is_none() {
                    // present a frame *before* checking stability: measured
                    // live, some compositors (GNOME/Mutter included) only
                    // finalize an async fractional-scale reconfigure once
                    // they've seen the client actually commit a real buffer -
                    // purely waiting without ever presenting anything never
                    // observes the correction at all. See
                    // `ensure_app_initialized`'s doc comment for the full
                    // story (this was the second failed attempt at this fix).
                    //
                    // Clears to black, not left uncovered: the render pass's
                    // `LOAD_OP_CLEAR` always clears to the loud debug-magenta
                    // `UNCOVERED_COLOR` first (see `ancorix_ash::commands`) -
                    // a deliberate "you forgot to call `ctx.draw.clear()`"
                    // signal for a real frame. These priming frames aren't
                    // that: `App::init()` hasn't run yet, so nothing *could*
                    // have queued a clear. Queuing one here ourselves keeps
                    // that signal meaningful (still fires if the app's own
                    // `frame()` really does forget to clear) instead of
                    // flashing magenta on every single startup.
                    self.draw.clear(Rgba::BLACK);
                    self.render();
                    self.ensure_app_initialized();
                } else if self.tick() {
                    event_loop.exit();
                } else {
                    self.render();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            return;
        }

        if self.app.is_none() {
            // still waiting for the window's real size to settle (see
            // `ensure_app_initialized`) - throttle to a real wall-clock
            // interval rather than `ControlFlow::Poll`, which spins as
            // fast as the CPU allows. Measured live: under `Poll`, two
            // consecutive checks can both land *before* the platform's
            // async configure round-trip (e.g. Wayland) ever completes,
            // making them agree on the stale size and falsely declaring
            // it settled. A real pause between checks gives that
            // round-trip actual wall-clock time to land first.
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + INIT_SETTLE_CHECK_INTERVAL,
            ));
            return;
        }

        match self.config.target_fps {
            Some(fps) => {
                let frame_time = Duration::from_secs_f64(1.0 / fps as f64);
                event_loop.set_control_flow(ControlFlow::WaitUntil(self.last_tick + frame_time));
            }
            None => event_loop.set_control_flow(ControlFlow::Poll),
        }
    }
}
