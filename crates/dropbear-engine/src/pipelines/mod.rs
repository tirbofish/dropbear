use std::path::PathBuf;
use crate::graphics::SharedGraphicsContext;
use crate::shader::Shader;
use std::sync::Arc;
use arc_swap::ArcSwap;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub mod globals;
pub mod hdr;
pub mod light_cube;
pub mod shader;
pub mod animation;
pub mod builder;

pub use globals::{Globals, GlobalsUniform};

/// A render pipeline that hot-swaps itself when shader source files change.
pub struct HotPipeline {
    pipeline: Arc<ArcSwap<wgpu::RenderPipeline>>,
}

impl HotPipeline {
    pub fn new<F>(
        device: Arc<wgpu::Device>,
        watch_dir: PathBuf,
        factory: F,
    ) -> anyhow::Result<Self>
    where
        F: Fn(&wgpu::Device) -> anyhow::Result<wgpu::RenderPipeline> + Send + Sync + 'static,
    {
        let factory = Arc::new(factory);

        let initial = factory(&device)?;
        let pipeline = Arc::new(ArcSwap::from_pointee(initial));

        if !watch_dir.exists() {
            log::warn!("HotPipeline: watch directory does not exist: {watch_dir:?}");
            return Ok(Self { pipeline });
        }

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        watcher.watch(&watch_dir, RecursiveMode::Recursive)?;

        let pipeline_ref = pipeline.clone();
        std::thread::spawn(move || {
            let _watcher = watcher; // keep the watcher alive inside the thread
            for event in rx {
                match event {
                    Ok(ev) => {
                        use notify::EventKind::*;
                        match ev.kind {
                            Modify(_) | Create(_) => {
                                log::info!("Shader change detected: {:?}", ev.paths);
                                let result = std::panic::catch_unwind(
                                    std::panic::AssertUnwindSafe(|| factory(&device)),
                                );
                                match result {
                                    Ok(Ok(new_pipeline)) => {
                                        pipeline_ref.store(Arc::new(new_pipeline));
                                        log::info!("Pipeline hot-reloaded successfully");
                                    }
                                    Ok(Err(e)) => {
                                        log::error!(
                                            "Shader compile error, keeping old pipeline: {e}"
                                        );
                                    }
                                    Err(_) => {
                                        log::error!(
                                            "Shader factory panicked during hot reload, keeping old pipeline"
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => log::error!("Shader watcher error: {e}"),
                }
            }
        });

        Ok(Self { pipeline })
    }

    pub fn get(&self) -> arc_swap::Guard<Arc<wgpu::RenderPipeline>> {
        self.pipeline.load()
    }
}

/// A helper in defining a pipelines required information, as well as getters.
///
/// This contains the bare minimum for any pipeline.
pub trait DropbearShaderPipeline {
    /// Creates a new instance of a pipeline.
    fn new(graphics: Arc<SharedGraphicsContext>) -> Self;
    /// Fetches the shader property
    fn shader(&self) -> &Shader;
    /// Fetches the pipeline layout
    fn pipeline_layout(&self) -> &wgpu::PipelineLayout;
    /// Fetches the pipeline
    fn pipeline(&self) -> &wgpu::RenderPipeline;
}

pub fn create_render_pipeline(
    label: Option<&str>,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology,
    shader: wgpu::ShaderModuleDescriptor,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    create_render_pipeline_ex(
        label,
        device,
        layout,
        color_format,
        depth_format,
        vertex_layouts,
        topology,
        shader,
        true, // depth_write_enabled
        wgpu::CompareFunction::LessEqual,
        sample_count,
    )
}

pub fn create_render_pipeline_ex(
    label: Option<&str>,
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    topology: wgpu::PrimitiveTopology,
    shader: wgpu::ShaderModuleDescriptor,
    depth_write_enabled: bool,
    depth_compare: wgpu::CompareFunction,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label,
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology, // NEW!
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: Some(depth_write_enabled),
            depth_compare: Some(depth_compare),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        cache: None,
        multiview_mask: None,
    })
}
