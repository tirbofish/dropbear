use bytemuck::{Pod, Zeroable};
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::TextureId;
use glam::{Mat4, Vec2, Vec4};
use kino_ui::camera::Camera2D;
use std::sync::Arc;
use wgpu::TextureFormat;
use dropbear_engine::pipelines::HotPipeline;

pub mod inspector;
pub mod viewport;
pub mod widget_tree;

pub struct UiEditor {
    pub grid_pipeline: Option<UIGridPipeline>,

    pub active_entity: Option<hecs::Entity>,
    pub camera: Camera2D,
    zoom: f32,
    camera_position: Vec2,
}

impl UiEditor {
    pub fn new() -> Self {
        let mut camera = Camera2D::default();
        camera.set_zoom(1.0);
        camera.target(Vec2::ZERO);

        Self {
            grid_pipeline: None,
            active_entity: None,
            camera,
            zoom: 1.0,
            camera_position: Vec2::ZERO,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self, graphics: Arc<SharedGraphicsContext>, width: u32, height: u32) {
        if self.grid_pipeline.is_none() {
            self.setup(graphics.clone());
        }

        let screen_size = Vec2::new(width.max(1) as f32, height.max(1) as f32);
        let view_proj = self.view_proj(screen_size);
        let inv_view_proj = self.inv_view_proj(screen_size);

        if let Some(grid) = self.grid_pipeline.as_mut() {
            grid.render_to_texture(graphics, width, height, view_proj, inv_view_proj);
        }
    }

    pub fn zoom_by(&mut self, delta: f32) {
        self.zoom = (self.zoom + delta).clamp(0.1, 10.0);
        self.camera.set_zoom(self.zoom);
        self.camera.target(self.camera_position);
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn pan_by_pixels(&mut self, delta_pixels: Vec2) {
        self.camera_position -= delta_pixels / self.zoom.max(0.1);
        self.camera.target(self.camera_position);
    }

    pub fn view_proj(&self, screen_size: Vec2) -> Mat4 {
        let width = screen_size.x.max(1.0);
        let height = screen_size.y.max(1.0);
        let half_w = width / (2.0 * self.zoom.max(0.1));
        let half_h = height / (2.0 * self.zoom.max(0.1));

        let view = Mat4::from_translation((-self.camera_position).extend(0.0));
        let proj = Mat4::orthographic_rh(-half_w, half_w, half_h, -half_h, -1.0, 1.0);

        proj * view
    }

    pub fn inv_view_proj(&self, screen_size: Vec2) -> Mat4 {
        self.view_proj(screen_size).inverse()
    }

    pub fn world_from_screen_pixels(&self, pixel: Vec2, viewport_size: Vec2) -> Vec2 {
        let viewport_size = Vec2::new(viewport_size.x.max(1.0), viewport_size.y.max(1.0));
        let ndc = Vec2::new(
            (pixel.x / viewport_size.x) * 2.0 - 1.0,
            1.0 - (pixel.y / viewport_size.y) * 2.0,
        );

        let world = self.inv_view_proj(viewport_size) * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
        Vec2::new(world.x, world.y) / world.w
    }

    pub fn texture_id(&self) -> Option<TextureId> {
        self.grid_pipeline
            .as_ref()
            .and_then(|pipeline| pipeline.texture_id)
    }

    fn setup(&mut self, graphics: Arc<SharedGraphicsContext>) {
        self.grid_pipeline = Some(UIGridPipeline::new(graphics.clone()));
    }
}

pub struct UIGridPipeline {
    size: wgpu::Extent3d,
    sample_count: u32,
    format: TextureFormat,

    texture_id: Option<TextureId>,
    resolve_texture: Option<wgpu::Texture>,
    resolve_view: Option<wgpu::TextureView>,
    msaa_texture: Option<wgpu::Texture>,
    msaa_view: Option<wgpu::TextureView>,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    pipeline: HotPipeline,
}

impl UIGridPipeline {
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let camera_bind_group_layout =
            graphics
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ui grid camera bind group layout"),
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
                });

        let camera_buffer = graphics.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui grid camera uniform buffer"),
            size: std::mem::size_of::<UIGridCameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ui grid camera bind group"),
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            });

        let layout = std::sync::Arc::new(
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ui grid pipeline layout"),
                    bind_group_layouts: &[Some(&camera_bind_group_layout)],
                    immediate_size: 0,
                }),
        );

        let sample_count: u32 = (*graphics.antialiasing.read()).into();
        let format = graphics.surface_format.add_srgb_suffix();

        let device = graphics.device.clone();
        let shader_dir = std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/editor/ui/shader"
        ));
        let shader_file = shader_dir.join("grid.wgsl");

        let pipeline = HotPipeline::new(
            device,
            shader_dir,
            move |device| {
                let source = std::fs::read_to_string(&shader_file)?;
                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("ui grid shader"),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });
                let pipeline =
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: Some("ui grid pipeline"),
                        layout: Some(&layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: Some("vs_main"),
                            compilation_options: Default::default(),
                            buffers: &[],
                        },
                        primitive: wgpu::PrimitiveState {
                            topology: wgpu::PrimitiveTopology::TriangleList,
                            strip_index_format: None,
                            front_face: wgpu::FrontFace::Ccw,
                            cull_mode: None,
                            polygon_mode: wgpu::PolygonMode::Fill,
                            unclipped_depth: false,
                            conservative: false,
                        },
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState {
                            count: sample_count,
                            mask: !0,
                            alpha_to_coverage_enabled: false,
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: Some("fs_main"),
                            compilation_options: Default::default(),
                            targets: &[Some(wgpu::ColorTargetState {
                                format,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            })],
                        }),
                        cache: None,
                        multiview_mask: None,
                    });
                Ok(pipeline)
            },
        )
        .expect("failed to build initial grid pipeline");

        Self {
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            sample_count,
            format,
            texture_id: None,
            resolve_texture: None,
            resolve_view: None,
            msaa_texture: None,
            msaa_view: None,
            camera_buffer,
            camera_bind_group,
            pipeline,
        }
    }

    fn update_camera_uniform(
        &self,
        graphics: &SharedGraphicsContext,
        view_proj: Mat4,
        inv_view_proj: Mat4,
        width: u32,
        height: u32,
    ) {
        let screen_size = Vec2::new(width.max(1) as f32, height.max(1) as f32);

        let uniform = UIGridCameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: inv_view_proj.to_cols_array_2d(),
            viewport_size: [screen_size.x, screen_size.y],
            _padding: [0.0, 0.0],
        };

        graphics
            .queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    fn create_texture(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: TextureFormat,
        sample_count: u32,
        label: &'static str,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn recreate_targets_if_needed(
        &mut self,
        graphics: &SharedGraphicsContext,
        width: u32,
        height: u32,
    ) {
        let width = width.max(1);
        let height = height.max(1);
        let new_sample_count: u32 = (*graphics.antialiasing.read()).into();

        let needs_recreate = self.size.width != width
            || self.size.height != height
            || self.sample_count != new_sample_count
            || self.resolve_view.is_none();

        if !needs_recreate {
            return;
        }

        self.size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        self.sample_count = new_sample_count;

        let (resolve_texture, resolve_view) = Self::create_texture(
            &graphics.device,
            self.size,
            self.format,
            1,
            "ui viewport resolve texture",
        );

        if let Some(texture_id) = self.texture_id {
            graphics
                .egui_renderer
                .lock()
                .renderer()
                .update_egui_texture_from_wgpu_texture(
                    &graphics.device,
                    &resolve_view,
                    wgpu::FilterMode::Linear,
                    texture_id,
                );
        } else {
            let texture_id = graphics
                .egui_renderer
                .lock()
                .renderer()
                .register_native_texture(&graphics.device, &resolve_view, wgpu::FilterMode::Linear);
            self.texture_id = Some(texture_id);
        }

        self.resolve_texture = Some(resolve_texture);
        self.resolve_view = Some(resolve_view);

        if self.sample_count > 1 {
            let (msaa_texture, msaa_view) = Self::create_texture(
                &graphics.device,
                self.size,
                self.format,
                self.sample_count,
                "ui viewport msaa texture",
            );
            self.msaa_texture = Some(msaa_texture);
            self.msaa_view = Some(msaa_view);
        } else {
            self.msaa_texture = None;
            self.msaa_view = None;
        }
    }

    pub fn render_to_texture(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        width: u32,
        height: u32,
        view_proj: Mat4,
        inv_view_proj: Mat4,
    ) {
        self.recreate_targets_if_needed(&graphics, width, height);
        self.update_camera_uniform(&graphics, view_proj, inv_view_proj, width, height);

        let Some(resolve_view) = self.resolve_view.as_ref() else {
            return;
        };

        let mut encoder = graphics
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ui viewport grid encoder"),
            });

        let (color_view, resolve_target) = if self.sample_count > 1 {
            let Some(msaa_view) = self.msaa_view.as_ref() else {
                return;
            };
            (msaa_view, Some(resolve_view))
        } else {
            (resolve_view, None)
        };

        let pipeline = self.pipeline.get();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("ui viewport grid render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: color_view,
                    depth_slice: None,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        graphics.queue.submit(std::iter::once(encoder.finish()));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct UIGridCameraUniform {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    viewport_size: [f32; 2],
    _padding: [f32; 2],
}
