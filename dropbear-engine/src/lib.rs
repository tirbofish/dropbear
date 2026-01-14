pub mod asset;
pub mod attenuation;
pub mod buffer;
pub mod camera;
pub mod colour;
pub mod egui_renderer;
pub mod entity;
pub mod graphics;
pub mod input;
pub mod lighting;
pub mod mipmap;
pub mod model;
pub mod panic;
pub mod procedural;
pub mod resources;
pub mod scene;
pub mod shader;
pub mod utils;

pub static WGPU_BACKEND: OnceLock<String> = OnceLock::new();
pub const PHYSICS_STEP_RATE: u32 = 120;
const MAX_PHYSICS_STEPS_PER_FRAME: usize = 4;
/// Note: image size is 256x256
pub const LOGO_AS_BYTES: &[u8] = include_bytes!("../../resources/eucalyptus-editor.png");

use app_dirs2::{AppDataType, AppInfo};
use bytemuck::Contiguous;
use chrono::Local;
use colored::Colorize;
use dropbear_future_queue::FutureQueue;
use egui::TextureId;
use egui_wgpu::ScreenDescriptor;
use env_logger::Builder;
use futures::executor::block_on;
use gilrs::{Gilrs, GilrsBuilder};
use log::LevelFilter;
use parking_lot::{Mutex, RwLock};
use spin_sleep::SpinSleeper;
use std::fs::OpenOptions;
use std::sync::OnceLock;
use std::{fs, sync::Arc, time::{Duration, Instant}};
use std::collections::HashMap;
use std::rc::Rc;
use wgpu::{BindGroupLayout, Device, ExperimentalFeatures, Instance, Queue, Surface, SurfaceConfiguration, SurfaceError, TextureFormat};
use winit::event::{DeviceEvent, DeviceId};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::Window,
};

use crate::{egui_renderer::EguiRenderer, graphics::Texture};

pub use dropbear_future_queue as future;
pub use gilrs;
pub use wgpu;
pub use winit;
use winit::window::{WindowAttributes, WindowId};
use crate::scene::Scene;

/// The backend information, such as the device, queue, config, surface, renderer, window and more.
pub struct State {
    // keep top for drop order
    pub window: Arc<Window>,
    pub instance: Arc<Instance>,

    pub surface: Arc<Surface<'static>>,
    pub surface_format: TextureFormat,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub config: SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub depth_texture: Texture,
    pub texture_bind_layout: BindGroupLayout,
    pub material_tint_bind_layout: BindGroupLayout,
    pub egui_renderer: Arc<Mutex<EguiRenderer>>,
    pub viewport_texture: Texture,
    pub texture_id: Arc<TextureId>,
    pub future_queue: Arc<FutureQueue>,

    physics_accumulator: Duration,

    pub scene_manager: scene::Manager,
}

/// Generates the dropbear engine logo in a form that [winit::window::Icon] can accept. 
/// 
/// Returns (the bytes, width, height) in resp order. 
pub fn gen_logo() -> anyhow::Result<(Vec<u8>, u32, u32)> {
    let image = image::load_from_memory(LOGO_AS_BYTES)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Ok((rgba, width, height))

}

impl State {
    /// Asynchronously initialised the state and sets up the backend and surface for wgpu to render to.
    pub async fn new(window: Arc<Window>, instance: Arc<Instance>, future_queue: Arc<FutureQueue>) -> anyhow::Result<Self> {
        let title = window.title();

        let size = window.inner_size();

        let initial_width = size.width.max(1);
        let initial_height = size.height.max(1);
        let is_surface_configured = size.width > 0 && size.height > 0;

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some(format!("{} graphics device", title).as_str()),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: unsafe { ExperimentalFeatures::enabled() },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        if WGPU_BACKEND.get().is_none() {
            let info = adapter.get_info();
            let os_info = os_info::get();
            log::info!(
            "\n==================== BACKEND INFO ====================
Backend: {}

Software:
    Architecture: {:?}
    Bitness: {:?}
    Codename: {:?}
    Edition: {:?}
    Os Type: {:?}
    Version: {:?}
    TLDR: {}


Hardware:
    Adapter Name: {}
    Vendor: {}
    Device: {}
    Type: {:?}
    Driver: {}
    Driver Info: {}
=======================================================
",
                info.backend,
                os_info.architecture(),
                os_info.bitness(),
                os_info.codename(),
                os_info.edition(),
                os_info.os_type(),
                os_info.version(),
                os_info,
                info.name,
                info.vendor,
                info.device,
                info.device_type,
                info.driver,
                info.driver_info,
            );
            let _ = WGPU_BACKEND.set(format!("{}", info.backend));
        }

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(TextureFormat::Rgba16Float);
        
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: initial_width,
            height: initial_height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        if is_surface_configured {
            surface.configure(&device, &config);
        }

        let depth_texture = Texture::create_depth_texture(&config, &device, Some("depth texture"));
        let viewport_texture =
            Texture::create_viewport_texture(&config, &device, Some("viewport texture"));

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // normal map
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let material_tint_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("material_tint_bind_group_layout"),
            });

        let mut egui_renderer = Arc::new(Mutex::new(EguiRenderer::new(
            &device,
            config.format,
            None,
            1,
            &window,
        )));

        let texture_id = Arc::get_mut(&mut egui_renderer)
            .unwrap()
            .lock()
            .renderer()
            .register_native_texture(&device, &viewport_texture.view, wgpu::FilterMode::Linear);

        let result = Self {
            surface: Arc::new(surface),
            surface_format,
            device: Arc::new(device),
            queue: Arc::new(queue),
            config,
            is_surface_configured,
            depth_texture,
            texture_bind_layout: texture_bind_group_layout,
            material_tint_bind_layout: material_tint_bind_group_layout,
            window,
            egui_renderer,
            viewport_texture,
            texture_id: Arc::new(texture_id),
            future_queue,
            instance,
            physics_accumulator: Duration::ZERO,
            scene_manager: scene::Manager::new(),
        };

        Ok(result)
    }

    /// A helper function that changes the surface config when resized (+ depth texture).
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }

        self.depth_texture =
            Texture::create_depth_texture(&self.config, &self.device, Some("depth texture"));
        self.viewport_texture =
            Texture::create_viewport_texture(&self.config, &self.device, Some("viewport texture"));
        self.egui_renderer
            .lock()
            .renderer()
            .update_egui_texture_from_wgpu_texture(
                &self.device,
                &self.viewport_texture.view,
                wgpu::FilterMode::Linear,
                *self.texture_id,
            );
    }

    /// Renders the scene and the egui renderer. I don't know what else to say.
    /// Returns any window-level commands that need to be handled by the App.
    fn render(
        &mut self,
        previous_dt: f32,
        event_loop: &ActiveEventLoop,
    ) -> anyhow::Result<Vec<scene::SceneCommand>> {
        if !self.is_surface_configured {
            return Ok(Vec::new());
        }

        let output = match self.surface.get_current_texture() {
            Ok(val) => val,
            Err(e) => {
                return match e {
                    SurfaceError::Lost => {
                        log_once::warn_once!("Surface lost, reconfiguring...");
                        self.surface.configure(&self.device, &self.config);
                        Ok(Vec::new())
                    }
                    SurfaceError::Outdated => {
                        log_once::warn_once!("Surface outdated, reconfiguring...");
                        self.surface.configure(&self.device, &self.config);
                        Ok(Vec::new())
                    }
                    SurfaceError::Timeout => {
                        log_once::warn_once!("Surface timeout, skipping frame");
                        Ok(Vec::new())
                    }
                    SurfaceError::OutOfMemory => {
                        Err(anyhow::anyhow!("Surface out of memory: {:?}", e))
                    }
                    SurfaceError::Other => {
                        log_once::warn_once!("Surface error (Other): {:?}, skipping frame", e);
                        Ok(Vec::new())
                    }
                };
            }
        };

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let viewport_view = { &self.viewport_texture.view.clone() };

        self.egui_renderer.lock().begin_frame(&self.window);

        let mut scene_manager = std::mem::replace(&mut self.scene_manager, scene::Manager::new());

        let physics_dt = Duration::from_secs_f32(1.0 / PHYSICS_STEP_RATE as f32);
        let frame_dt = Duration::from_secs_f32(previous_dt).min(Duration::from_millis(250));
        let mut physics_accumulator = self.physics_accumulator + frame_dt;

        let window_commands = {
            let mut graphics = graphics::RenderContext::from_state(self, viewport_view, &mut encoder);

            let mut steps = 0usize;
            while physics_accumulator >= physics_dt && steps < MAX_PHYSICS_STEPS_PER_FRAME {
                scene_manager.physics_update(physics_dt.as_secs_f32(), &mut graphics);
                physics_accumulator -= physics_dt;
                steps += 1;
            }

            if steps == MAX_PHYSICS_STEPS_PER_FRAME && physics_accumulator >= physics_dt {
                physics_accumulator = physics_accumulator.min(physics_dt);
            }

            let commands = scene_manager.update(previous_dt, &mut graphics, event_loop);
            scene_manager.render(&mut graphics);
            commands
        };

        self.physics_accumulator = physics_accumulator;

        self.scene_manager = scene_manager;

        self.egui_renderer.lock().end_frame_and_draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.window,
            &view,
            screen_descriptor,
        );

        let command_buffer = encoder.finish();

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.queue.submit(std::iter::once(command_buffer));
        })) {
            Ok(_) => {}
            Err(_) => {
                log::error!("Failed to submit command buffer, device may be lost");
                return Err(anyhow::anyhow!("Command buffer submission failed"));
            }
        }

        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            output.present();
        })) {
            Ok(_) => {}
            Err(_) => {
                log::error!("Failed to present frame, surface may be lost");
                return Err(anyhow::anyhow!("Frame presentation failed"));
            }
        }

        Ok(window_commands)
    }

    fn cleanup(mut self, event_loop: &ActiveEventLoop) {
        self.scene_manager.clear_all(event_loop);

        let _ = self.device.poll(wgpu::PollType::Poll);

        drop(self.egui_renderer);

        drop(self.depth_texture);
        drop(self.viewport_texture);
        drop(self.texture_bind_layout);

        drop(self.surface);

        drop(self.device);
        drop(self.queue);

        let window = self.window;
        let instance = self.instance;

        let window_count = Arc::strong_count(&window);

        if window_count > 1 {
            log::warn!("Window still has {} strong references after cleanup", window_count);
        }

        drop(window);
        drop(instance);
    }
}

/// Used to build an app ran with the dropbear engine.
///
/// It is best to use this in a "chained" manner.
///
/// ```rust
/// let app = dropbear_engine::DropbearAppBuilder::new();
/// ```
#[derive(Clone)]
pub struct DropbearAppBuilder {
    windows_to_create: Vec<WindowData>,
    future_queue: Option<Arc<FutureQueue>>,
    max_fps: u32,
    app_data: AppInfo,
}

impl DropbearAppBuilder {
    /// Initialises a new [`DropbearAppBuilder`] instance.
    ///
    /// # Defaults
    /// - `windows_to_create` - empty vector
    /// - `future_queue` - [None]
    /// - `max_fps` - [u32::MAX]
    /// - `app_data` - `<name: "unknown_dropbear_app", author: "unknown">`
    pub fn new() -> Self {
        Self {
            windows_to_create: vec![],
            future_queue: None,
            max_fps: u32::MAX,
            app_data: AppInfo { name: "unknown_dropbear_app", author: "unknown" },
        }
    }

    /// Adds a future queue.
    pub fn with_future_queue(mut self, future_queue: Arc<FutureQueue>) -> Self {
        self.future_queue = Some(future_queue);
        self
    }

    /// Creates a default [`DropbearWindowBuilder`] window.
    ///
    /// If you wish to create a custom window, use [`DropbearAppBuilder::add_window`] instead.
    pub fn create_window() -> DropbearWindowBuilder {
        DropbearWindowBuilder::new()
    }

    /// Creates a custom window as specified by the build product of [`DropbearWindowBuilder`]
    /// (in the form of [`WindowData`]).
    pub fn add_window(mut self, window_data: WindowData) -> Self {
        self.windows_to_create.push(window_data);
        self
    }

    /// Sets the maximum FPS of the app. By default, it is [`u32::MAX`]
    pub fn max_fps(mut self, max_fps: u32) -> Self {
        self.max_fps = max_fps;
        self
    }

    /// Sets a custom appdata.
    pub fn app_data(mut self, app_data: AppInfo) -> Self {
        self.app_data = app_data;
        self
    }

    /// Launches and starts the event loop for the dropbear app.
    ///
    /// This function requires you to run it asynchronously. You will require [`tokio`]
    /// to run your app.
    pub async fn run(self) -> anyhow::Result<()> {
        #[cfg(not(target_os = "android"))]
        {
            let log_dir =
                app_dirs2::app_root(AppDataType::UserData, &self.app_data)
                    .expect("Failed to get app data directory")
                    .join("logs");
            fs::create_dir_all(&log_dir).expect("Failed to create log dir");

            let datetime_str = Local::now().format("%Y-%m-%d_%H-%M-%S");
            let log_filename = format!("{}.{}.log", env!("CARGO_PKG_NAME"), datetime_str);
            let log_path = log_dir.join(log_filename);

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .expect("Failed to open log file");
            let file = Mutex::new(file);

            let app_target = "eucalyptus-editor".replace('-', "_");
            let log_config = format!("dropbear_engine=trace,{}=debug,warn", app_target);
            unsafe { std::env::set_var("RUST_LOG", log_config) };

            let _ = Builder::new()
                .format(move |buf, record| {
                    use std::io::Write;

                    let ts = chrono::offset::Local::now().format("%Y-%m-%dT%H:%M:%S");

                    let colored_level = match record.level() {
                        log::Level::Error => record.level().to_string().red().bold(),
                        log::Level::Warn => record.level().to_string().yellow().bold(),
                        log::Level::Info => record.level().to_string().green().bold(),
                        log::Level::Debug => record.level().to_string().blue().bold(),
                        log::Level::Trace => record.level().to_string().cyan().bold(),
                    };

                    let colored_timestamp = ts.to_string().bright_black();

                    let file_info = format!(
                        "{}:{}",
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0)
                    )
                        .bright_black();

                    let console_line = format!(
                        "{} {} [{}] - {}\n",
                        file_info,
                        colored_timestamp,
                        colored_level,
                        record.args()
                    );

                    let file_line = format!(
                        "{}:{} {} [{}] - {}\n",
                        record.file().unwrap_or("unknown"),
                        record.line().unwrap_or(0),
                        ts,
                        record.level(),
                        record.args()
                    );

                    write!(buf, "{}", console_line)?;

                    let mut fh = file.lock();
                    let _ = fh.write_all(file_line.as_bytes());

                    Ok(())
                })
                .filter(Some("dropbear_engine"), LevelFilter::Trace)
                .filter(
                    Some("eucalyptus-editor".replace('-', "_").as_str()),
                    LevelFilter::Debug,
                )
                .filter(Some("eucalyptus_core"), LevelFilter::Debug)
                .filter(Some("dropbear_traits"), LevelFilter::Debug)
                .try_init();
            log::info!("Initialised logger");
        }

        log::info!("======================================================================");
        log::info!(
            "dropbear-engine v{} compiled with {}",
            env!("CARGO_PKG_VERSION"),
            rustc_version_runtime::version_meta().short_version_string
        );
        log::info!("Made by tk with love at https://github.com/tirbofish/dropbear <3");
        log::info!("======================================================================");
        #[cfg(debug_assertions)]
        {
            log::warn!(
                "⚠️ Just a heads up: this is compiled with the debug profile. Expect shit to be slow..."
            );
        }
        log::info!("dropbear-engine running...");

        let event_loop = EventLoop::with_user_event().build()?;
        log::debug!("Created new event loop");

        let mut app = Box::new(App::new(self.app_data, self.future_queue));

        app.target_fps = self.max_fps;
        app.windows_to_create = self.windows_to_create;

        log::debug!("Running app");
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}

pub trait SceneWithInput: Scene + input::Keyboard + input::Mouse + input::Controller {}

impl<T> SceneWithInput for T
where
    T: Scene + input::Keyboard + input::Mouse + input::Controller
{}

#[derive(Clone)]
pub struct WindowData {
    pub attributes: WindowAttributes,
    pub scenes: HashMap<String, Rc<RwLock<dyn SceneWithInput>>>,
    pub first_scene: Option<String>,
}

pub struct DropbearWindowBuilder {
    attributes: WindowAttributes,
    scenes: HashMap<String, Rc<RwLock<dyn SceneWithInput>>>,
    first_scene: Option<String>,
}

impl DropbearWindowBuilder {
    pub fn new() -> Self {
        Self {
            attributes: WindowAttributes::default(),
            scenes: HashMap::new(),
            first_scene: None,
        }
    }

    pub fn with_attributes(mut self, window_attributes: WindowAttributes) -> Self {
        self.attributes = window_attributes;
        self
    }

    pub fn add_scene_with_input<S>(mut self, scene: Rc<RwLock<S>>, scene_name: impl ToString) -> Self
    where
        S: 'static + Scene + input::Keyboard + input::Mouse + input::Controller
    {
        let scene_name = scene_name.to_string();
        self.scenes.insert(scene_name, scene as Rc<RwLock<dyn SceneWithInput>>);
        self
    }

    pub fn set_initial_scene(mut self, scene_name: impl ToString) -> Self {
        self.first_scene = Some(scene_name.to_string());
        self
    }

    pub fn build(self) -> WindowData {
        WindowData {
            attributes: self.attributes,
            scenes: self.scenes,
            first_scene: self.first_scene,
        }
    }
}

/// A struct storing the information about the application/game that is using the engine.
pub struct App {
    #[allow(dead_code)]
    app_data: AppInfo,
    /// The input manager, manages any inputs and their actions
    input_manager: input::Manager,
    /// The amount of time it took to render the last frame.
    /// To find the FPS: just do `1.0/delta_time`.
    delta_time: f32,
    /// Internal
    next_frame_time: Option<Instant>,
    /// The fps the app should aim to hit / the max fps.
    /// It is possible to aim it at 60 fps, 120 fps, or even no limit
    /// with the const variable [`App::NO_FPS_CAP`]
    target_fps: u32,
    /// The library used for polling controllers, specifically the instance of that.
    gilrs: Gilrs,
    /// A queue that polls through futures for asynchronous functions
    ///
    /// Winit doesn't use async, so this is the next best alternative.
    future_queue: Arc<FutureQueue>,
    delta_position: Option<(f64, f64)>,

    instance: Arc<Instance>,

    // multi-window management
    windows: HashMap<WindowId, State>,
    root_window_id: Option<WindowId>,
    windows_to_create: Vec<WindowData>,
}

impl App {
    /// Creates a new instance of the application. It only sets the default for the struct + the
    /// window config.
    fn new(app_data: AppInfo, future_queue: Option<Arc<FutureQueue>>) -> Self {
        let instance = Arc::new(Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        }));

        let result = Self {
            input_manager: input::Manager::new(),
            delta_time: 1.0 / 60.0,
            next_frame_time: None,
            target_fps: u32::MAX, // assume max,
            // default settings for now
            gilrs: GilrsBuilder::new().build().unwrap(),
            future_queue: future_queue.unwrap_or_else(|| Arc::new(FutureQueue::new())),
            delta_position: None,
            instance,
            windows: Default::default(),
            root_window_id: None,
            windows_to_create: Vec::new(),
            app_data,
        };
        log::debug!("Created new instance of app");
        result
    }

    /// A constant that lets you not have any fps count.
    /// It is just the max value of an unsigned 32 bit number lol.
    pub const NO_FPS_CAP: u32 = u32::MAX_VALUE;

    /// Helper function that sets the target frames per second. Can be used mid game to increase FPS.
    pub fn set_target_fps(&mut self, fps: u32) {
        self.target_fps = fps.max(1);
    }

    /// Creates a new window and adds it to its internal window manager (its really just a hashmap).
    pub fn create_window(&mut self, event_loop: &ActiveEventLoop, attribs: WindowAttributes) -> anyhow::Result<WindowId> {
        let window = Arc::new(
            event_loop.create_window(attribs)?
        );

        let window_id = window.id();

        let mut win_state = block_on(State::new(window, self.instance.clone(), self.future_queue.clone()))?;

        let size = win_state.window.inner_size();
        win_state.resize(size.width, size.height);

        self.windows.insert(window_id, win_state);
        Ok(window_id)
    }

    fn quit(&mut self, event_loop: &ActiveEventLoop, hook: Option<fn()>) {
        if let Some(h) = hook {
            log::debug!("App has a pre-exit hook, executing...");
            h();
        }

        log::info!("Exiting app!");

        let windows = std::mem::take(&mut self.windows);
        for (_, state) in windows {
            state.cleanup(event_loop);
        }
        self.root_window_id = None;

        #[cfg(not(target_os = "linux"))]
        event_loop.exit();
        #[cfg(target_os = "linux")]
        std::process::exit(0);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            let windows_to_create = std::mem::take(&mut self.windows_to_create);

            if !windows_to_create.is_empty() {
                for window_data in windows_to_create {
                    match self.create_window(event_loop, window_data.attributes) {
                        Ok(window_id) => {
                            if let Some(state) = self.windows.get_mut(&window_id) {
                                for (scene_name, scene) in window_data.scenes {
                                    state.scene_manager.add(&scene_name, scene.clone());

                                    let keyboard_name = format!("{}_keyboard", scene_name);
                                    let mouse_name = format!("{}_mouse", scene_name);
                                    let controller_name = format!("{}_controller", scene_name);

                                    let keyboard_handler: Rc<RwLock<dyn input::Keyboard>> = scene.clone();
                                    let mouse_handler: Rc<RwLock<dyn input::Mouse>> = scene.clone();
                                    let controller_handler: Rc<RwLock<dyn input::Controller>> = scene.clone();

                                    self.input_manager.add_keyboard(&keyboard_name, keyboard_handler);
                                    self.input_manager.add_mouse(&mouse_name, mouse_handler);
                                    self.input_manager.add_controller(&controller_name, controller_handler);

                                    state.scene_manager.attach_input(&scene_name, &keyboard_name);
                                    state.scene_manager.attach_input(&scene_name, &mouse_name);
                                    state.scene_manager.attach_input(&scene_name, &controller_name);
                                }

                                if let Some(initial_scene) = window_data.first_scene {
                                    state.scene_manager.switch(&initial_scene);
                                }

                                if self.root_window_id.is_none() {
                                    self.root_window_id = Some(window_id);
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create window: {}", e);
                        }
                    }
                }
            } else {
                panic!("There must be at minimum 1 window to be able to create");
            }
        }

        self.next_frame_time = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            if Some(window_id) == self.root_window_id {
                log::info!("Root window closed, exiting app");
                self.quit(event_loop, None);
            } else {
                log::info!("Closing non-root window: {:?}", window_id);
                if let Some(state) = self.windows.remove(&window_id) {
                    state.cleanup(event_loop);
                }
            }
            return;
        }

        let state = match self.windows.get_mut(&window_id) {
            Some(canvas) => canvas,
            None => return,
        };

        state
            .egui_renderer
            .lock()
            .handle_input(&state.window, &event);

        match event {
            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                self.future_queue.poll();

                let frame_start = Instant::now();

                let active_handlers = state.scene_manager.get_active_input_handlers();
                self.input_manager.set_active_handlers(active_handlers);

                self.input_manager.update(&mut self.gilrs);

                let render_result = state.render(self.delta_time, event_loop);

                let window_commands = render_result.unwrap_or_else(|e| {
                    log::error!("Render failed: {:?}", e);
                    Vec::new()
                });

                let frame_elapsed = frame_start.elapsed();
                let target_frame_time = Duration::from_secs_f32(1.0 / self.target_fps as f32);

                if frame_elapsed < target_frame_time {
                    SpinSleeper::default().sleep(target_frame_time - frame_elapsed);
                }

                let total_frame_time = frame_start.elapsed();
                self.delta_time = total_frame_time.as_secs_f32();

                state.window.request_redraw();
                self.future_queue.cleanup();

                for command in window_commands {
                    match command {
                        scene::SceneCommand::RequestWindow(window_data) => {
                            log::info!("Scene requested new window creation");
                            match self.create_window(event_loop, window_data.attributes) {
                                Ok(new_window_id) => {
                                    if let Some(new_state) = self.windows.get_mut(&new_window_id) {
                                        for (scene_name, scene) in window_data.scenes {
                                            new_state.scene_manager.add(&scene_name, scene.clone());

                                            let keyboard_name = format!("{}_keyboard", scene_name);
                                            let mouse_name = format!("{}_mouse", scene_name);
                                            let controller_name = format!("{}_controller", scene_name);

                                            let keyboard_handler: Rc<RwLock<dyn input::Keyboard>> = scene.clone();
                                            let mouse_handler: Rc<RwLock<dyn input::Mouse>> = scene.clone();
                                            let controller_handler: Rc<RwLock<dyn input::Controller>> = scene.clone();

                                            self.input_manager.add_keyboard(&keyboard_name, keyboard_handler);
                                            self.input_manager.add_mouse(&mouse_name, mouse_handler);
                                            self.input_manager.add_controller(&controller_name, controller_handler);

                                            new_state.scene_manager.attach_input(&scene_name, &keyboard_name);
                                            new_state.scene_manager.attach_input(&scene_name, &mouse_name);
                                            new_state.scene_manager.attach_input(&scene_name, &controller_name);
                                        }

                                        if let Some(initial_scene) = window_data.first_scene {
                                            new_state.scene_manager.switch(&initial_scene);
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to create requested window: {}", e);
                                }
                            }
                        }
                        scene::SceneCommand::CloseWindow(target_window_id) => {
                            log::info!("Scene requested closing window: {:?}", target_window_id);
                            if Some(target_window_id) == self.root_window_id {
                                self.quit(event_loop, None);
                            } else {
                                self.windows.remove(&target_window_id);
                            }
                        }
                        scene::SceneCommand::Quit(hook) => {
                            log::debug!("Caught SceneCommand::Quit command!");
                            self.quit(event_loop, hook);
                        }
                        scene::SceneCommand::SetFPS(new_fps) => {
                            self.set_target_fps(new_fps);
                        }
                        _ => {}
                    }
                }

                for state in self.windows.values() {
                    state.window.request_redraw();
                }

                return;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                self.input_manager
                    .handle_key_input(code, key_state.is_pressed(), event_loop);
            }
            WindowEvent::MouseInput {
                button,
                state: button_state,
                ..
            } => {
                self.input_manager
                    .handle_mouse_input(button, button_state.is_pressed());
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input_manager.handle_mouse_movement(position, None);
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        #[allow(clippy::single_match)]
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.delta_position = Some(delta);
                self.input_manager
                    .handle_mouse_movement(self.input_manager.get_mouse_position(), Some(delta));
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for window_state in self.windows.values() {
            window_state.window.request_redraw();
        }
    }
}