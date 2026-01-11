use crate::attenuation::{Attenuation, RANGE_50};
use crate::buffer::ResizableBuffer;
use crate::graphics::SharedGraphicsContext;
use crate::shader::Shader;
use crate::{
    camera::Camera,
    entity::{EntityTransform, Transform},
    model::{self, Model, Vertex},
};
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use glam::{DMat4, DQuat, DVec3};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use hecs::Entity;
use wgpu::{BindGroup, BindGroupLayout, Buffer, BufferAddress, CompareFunction, DepthBiasState, RenderPipeline, StencilState, VertexBufferLayout, FilterMode};

pub const MAX_LIGHTS: usize = 8;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// ENSURE THAT THE SIZE OF THE UNIFORM IS OF A MULTIPLE OF 16. USE `size_of::<LightUniform>()`
pub struct LightUniform {
    pub position: [f32; 4],
    pub direction: [f32; 4], // outer cutoff is .w value
    pub colour: [f32; 4],    // last value is the light type
    // pub light_type: u32,
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
    pub cutoff: f32,

    pub shadow_index: i32,
    pub _padding: [u32; 3],

    pub(crate) proj: [[f32; 4]; 4],
}

fn dvec3_to_uniform_array(vec: DVec3) -> [f32; 4] {
    [vec.x as f32, vec.y as f32, vec.z as f32, 1.0]
}

fn dvec3_colour_to_uniform_array(vec: DVec3, light_type: LightType) -> [f32; 4] {
    [
        vec.x as f32,
        vec.y as f32,
        vec.z as f32,
        light_type as u32 as f32,
    ]
}

fn dvec3_direction_to_uniform_array(vec: DVec3, outer_cutoff_angle: f32) -> [f32; 4] {
    [
        vec.x as f32,
        vec.y as f32,
        vec.z as f32,
        f32::cos(outer_cutoff_angle.to_radians()),
    ]
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 1.0],
            direction: [0.0, 0.0, -1.0, 1.0],
            colour: [1.0, 1.0, 1.0, 1.0],
            // light_type: 0,
            constant: 0.0,
            linear: 0.0,
            quadratic: 0.0,
            cutoff: f32::cos(12.5_f32.to_radians()),
            shadow_index: -1,
            _padding: [0; 3],
            proj: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightArrayUniform {
    pub lights: [LightUniform; MAX_LIGHTS],
    pub light_count: u32,
    pub ambient_strength: f32,
    pub _padding: [u32; 2],
}

impl Default for LightArrayUniform {
    fn default() -> Self {
        Self {
            lights: [LightUniform::default(); MAX_LIGHTS],
            light_count: 0,
            ambient_strength: 0.1,
            _padding: [0; 2],
        }
    }
}

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum LightType {
    #[default]
    // Example: Sunlight
    Directional = 0,
    // Example: Lamp
    Point = 1,
    // Example: Flashlight
    Spot = 2,
}

impl Display for LightType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LightType::Directional => write!(f, "Directional"),
            LightType::Point => write!(f, "Point"),
            LightType::Spot => write!(f, "Spot"),
        }
    }
}

impl From<LightType> for u32 {
    fn from(val: LightType) -> Self {
        match val {
            LightType::Directional => 0,
            LightType::Point => 1,
            LightType::Spot => 2,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SerializableComponent)]
pub struct LightComponent {
    #[serde(default)]
    pub position: DVec3,          // point, spot

    #[serde(default)]
    pub direction: DVec3,         // directional, spot

    #[serde(default)]
    pub colour: DVec3,            // all

    #[serde(default)]
    pub light_type: LightType,    // all

    #[serde(default)]
    pub intensity: f32,           // all

    #[serde(default)]
    pub attenuation: Attenuation, // point, spot

    #[serde(default)]
    pub enabled: bool,            // all - light

    #[serde(default)]
    pub visible: bool,            // all - cube

    #[serde(default)]
    pub cutoff_angle: f32,        // spot

    #[serde(default)]
    pub outer_cutoff_angle: f32,  // spot

    #[serde(default)]
    pub cast_shadows: bool,

    #[serde(default)]
    pub depth: std::ops::Range<f32>, // all
}

impl Default for LightComponent {
    fn default() -> Self {
        Self {
            position: DVec3::ZERO,
            direction: DVec3::new(0.0, 0.0, -1.0),
            colour: DVec3::ONE,
            light_type: LightType::Point,
            intensity: 1.0,
            attenuation: RANGE_50,
            enabled: true,
            cutoff_angle: 12.5,
            outer_cutoff_angle: 17.5,
            visible: true,
            cast_shadows: true,
            depth: 0.1..100.0,
        }
    }
}

impl LightComponent {
    pub fn default_direction() -> DVec3 {
        let dir = DVec3::new(-0.35, -1.0, -0.25);
        dir.normalize()
    }

    pub fn new(
        colour: DVec3,
        light_type: LightType,
        intensity: f32,
        attenuation: Option<Attenuation>,
    ) -> Self {
        let direction = match light_type {
            LightType::Directional | LightType::Spot => Self::default_direction(),
            LightType::Point => DVec3::ZERO,
        };

        Self {
            position: Default::default(),
            direction,
            colour,
            light_type,
            intensity,
            attenuation: attenuation.unwrap_or(RANGE_50),
            enabled: true,
            cutoff_angle: 12.5,
            outer_cutoff_angle: 17.5,
            cast_shadows: true,
            visible: true,
            depth: 0.1..100.0,
        }
    }

    pub fn directional(colour: DVec3, intensity: f32) -> Self {
        Self::new(colour, LightType::Directional, intensity, None)
    }

    pub fn point(colour: DVec3, intensity: f32, attenuation: Attenuation) -> Self {
        Self::new(colour, LightType::Point, intensity, Some(attenuation))
    }

    pub fn spot(colour: DVec3, intensity: f32) -> Self {
        Self::new(colour, LightType::Spot, intensity, None)
    }

    pub fn hide_cube(&mut self) {
        self.visible = false;
    }

    pub fn show_cube(&mut self) {
        self.visible = true;
    }

    pub fn disable_light(&mut self) {
        self.enabled = false;
    }

    pub fn enable_light(&mut self) {
        self.enabled = true;
    }
}

#[derive(Clone)]
pub struct Light {
    pub uniform: LightUniform,
    pub cube_model: Arc<Model>,
    pub label: String,
    buffer: Option<Buffer>,
    layout: Option<BindGroupLayout>,
    bind_group: Option<BindGroup>,
    pub instance_buffer: ResizableBuffer<InstanceRaw>,
}

impl Light {
    pub async fn new(
        graphics: Arc<SharedGraphicsContext>,
        light: LightComponent,
        transform: Transform,
        label: Option<&str>,
    ) -> Self {
        let forward = DVec3::new(0.0, 0.0, -1.0);
        let direction = transform.rotation * forward;

        let uniform = LightUniform {
            position: dvec3_to_uniform_array(transform.position),
            direction: dvec3_direction_to_uniform_array(direction, light.outer_cutoff_angle),
            colour: dvec3_colour_to_uniform_array(
                light.colour * light.intensity as f64,
                light.light_type,
            ),
            constant: light.attenuation.constant,
            linear: light.attenuation.linear,
            quadratic: light.attenuation.quadratic,
            cutoff: f32::cos(light.cutoff_angle.to_radians()),
            shadow_index: -1,
            _padding: Default::default(),
            proj: Default::default(),
        };

        log::trace!("Created new light uniform");

        let cube_model = Model::load_from_memory(
            graphics.clone(),
            include_bytes!("../../resources/models/cube.glb").to_vec(),
            label,
        )
        .await
        .expect("failed to load light cube model")
        .get();

        let label_str = label.unwrap_or("Light").to_string();

        let buffer = graphics.create_uniform(uniform, label);

        let layout = graphics
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label,
            });

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label,
            });

        let instance = Instance::new(
            transform.position,
            transform.rotation,
            DVec3::new(0.25, 0.25, 0.25),
        );

        let mut instance_buffer = ResizableBuffer::new(
            &graphics.device,
            1,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            "Light Instance Buffer"
        );

        instance_buffer.write(&graphics.device, &graphics.queue, &[instance.to_raw()]);

        log::debug!("Created new light [{}]", label_str);

        Self {
            uniform,
            cube_model,
            label: label_str,
            buffer: Some(buffer),
            layout: Some(layout),
            bind_group: Some(bind_group),
            instance_buffer,
        }
    }

    pub fn update(&mut self, graphics: &SharedGraphicsContext, light: &mut LightComponent, transform: &Transform) {
        self.uniform.position = dvec3_to_uniform_array(transform.position);

        let forward = DVec3::new(0.0, 0.0, -1.0);
        let direction = transform.rotation * forward;
        self.uniform.direction =
            dvec3_direction_to_uniform_array(direction, light.outer_cutoff_angle);

        self.uniform.colour =
            dvec3_colour_to_uniform_array(light.colour * light.intensity as f64, light.light_type);
        self.uniform.constant = light.attenuation.constant;
        self.uniform.linear = light.attenuation.linear;
        self.uniform.quadratic = light.attenuation.quadratic;

        self.uniform.cutoff = f32::cos(light.cutoff_angle.to_radians());

        let safe_up = if direction.normalize_or_zero().dot(DVec3::Y).abs() > 0.99 {
            DVec3::Z
        } else {
            DVec3::Y
        };

        let view = glam::DMat4::look_at_lh(
            transform.position,
            transform.position + direction,
            safe_up,
        );

        let projection = match light.light_type {
            LightType::Directional => {
                let extent = 50.0;
                glam::DMat4::orthographic_lh(
                    -extent,
                    extent,
                    -extent,
                    extent,
                    light.depth.start as f64,
                    light.depth.end as f64,
                )
            }
            LightType::Spot => glam::DMat4::perspective_lh(
                light.outer_cutoff_angle.to_radians() as f64 * 2.0,
                1.0,
                light.depth.start as f64,
                light.depth.end as f64,
            ),
            // Point light shadows require cubemaps; not supported here.
            LightType::Point => glam::DMat4::IDENTITY,
        };

        let light_vp = projection * view;
        self.uniform.proj = light_vp.as_mat4().to_cols_array_2d();

        if let Some(buffer) = &self.buffer {
            graphics
                .queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        }
    }

    pub fn uniform(&self) -> &LightUniform {
        &self.uniform
    }

    pub fn model(&self) -> &Model {
        &self.cube_model
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn bind_group(&self) -> &BindGroup {
        self.bind_group.as_ref().unwrap()
    }

    pub fn layout(&self) -> &BindGroupLayout {
        self.layout.as_ref().unwrap()
    }

    pub fn buffer(&self) -> &Buffer {
        self.buffer.as_ref().unwrap()
    }
}

#[derive(Clone)]
pub struct LightManager {
    pub pipeline: Option<RenderPipeline>,
    pub shadow_pipeline: Option<RenderPipeline>,
    light_array_buffer: Option<Buffer>,
    light_array_bind_group: Option<BindGroup>,
    light_array_layout: Option<BindGroupLayout>,

    pub shadow_texture: Option<wgpu::Texture>,
    pub shadow_view: Option<wgpu::TextureView>,
    pub shadow_sampler: Option<wgpu::Sampler>,
    pub shadow_target_views: Vec<wgpu::TextureView>,
}

impl Default for LightManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LightManager {
    pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const SHADOW_SIZE: u32 = 2048;

    pub fn new() -> Self {
        log::info!("Initialised lighting");
        Self {
            pipeline: None,
            shadow_pipeline: None,
            light_array_buffer: None,
            light_array_bind_group: None,
            light_array_layout: None,
            shadow_texture: None,
            shadow_view: None,
            shadow_sampler: None,
            shadow_target_views: vec![],
        }
    }

    pub fn create_shadow_pipeline(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        shader_contents: &str,
        label: Option<&str>,
    ) {
        let shader = Shader::new(graphics.clone(), shader_contents, label);

        // Layout compatible with `Light::bind_group()` (single uniform buffer).
        let per_light_layout = graphics
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Shadow Per-Light Layout"),
            });

        let pipeline_layout = graphics
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label.unwrap_or("Shadow Pipeline Layout")),
                bind_group_layouts: &[&per_light_layout],
                push_constant_ranges: &[],
            });

        let pipeline = graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label.unwrap_or("Shadow Pipeline")),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: Some("vs_main"),
                    buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
                    compilation_options: Default::default(),
                },
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Front),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Self::SHADOW_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: 2,
                        slope_scale: 2.0,
                        clamp: 0.0,
                    },
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        self.shadow_pipeline = Some(pipeline);
        log::debug!("Created shadow render pipeline");
    }

    pub fn create_light_array_resources(&mut self, graphics: Arc<SharedGraphicsContext>) {
        let shadow_texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Map Array"),
            size: wgpu::Extent3d {
                width: Self::SHADOW_SIZE,
                height: Self::SHADOW_SIZE,
                depth_or_array_layers: MAX_LIGHTS as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::SHADOW_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Shadow Array View"),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        let shadow_sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        self.shadow_target_views = (0..MAX_LIGHTS)
            .map(|i| {
                shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&format!("Shadow Layer {}", i)),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: i as u32,
                    array_layer_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        let layout = graphics
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // light data
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // shadow texture array
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    // shadow sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
                label: Some("Light Array Layout"),
            });

        let buffer = graphics.create_uniform(LightArrayUniform::default(), Some("Light Array"));

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&shadow_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                    },
                ],
                label: Some("Light Array Bind Group"),
            });

        self.light_array_layout = Some(layout);
        self.light_array_buffer = Some(buffer);
        self.light_array_bind_group = Some(bind_group);

        self.shadow_texture = Some(shadow_texture);
        self.shadow_view = Some(shadow_view);
        self.shadow_sampler = Some(shadow_sampler);

        log::debug!("Created light array resources");
    }

    pub fn update(&mut self, graphics: Arc<SharedGraphicsContext>, world: &hecs::World) {
        let mut light_array = LightArrayUniform::default();
        let mut light_index = 0;

        let mut shadow_map_index = 0;

        for (light_component, s_trans, e_trans, light) in world
            .query::<(&LightComponent, Option<&Transform>, Option<&EntityTransform>, &mut Light)>()
            .iter()
        {
            let instance = if let Some(transform) = e_trans {
                let sync_transform = transform.sync();
                Instance::from_matrix(sync_transform.matrix())
            } else if let Some(transform) = s_trans {
                Instance::from_matrix(transform.matrix())
            } else {
                panic!("Unable to locate either a \"Transform\" or an \"EntityTransform\" component for the light {}", light.label);
            };

            light.instance_buffer.write(&graphics.device, &graphics.queue, &[instance.to_raw()]);

            if light_component.enabled && light_index < MAX_LIGHTS {
                let mut uniform = *light.uniform();

                if light_component.cast_shadows
                    && light_component.light_type != LightType::Point
                    && shadow_map_index < MAX_LIGHTS
                {
                    uniform.shadow_index = shadow_map_index as i32;
                    shadow_map_index += 1;
                } else {
                    uniform.shadow_index = -1;
                }

                // Keep per-light uniform in sync (used by shadow pass and light cube rendering).
                light.uniform.shadow_index = uniform.shadow_index;
                graphics
                    .queue
                    .write_buffer(light.buffer(), 0, bytemuck::cast_slice(&[light.uniform]));

                light_array.lights[light_index] = uniform;
                light_index += 1;
            }
        }

        light_array.light_count = light_index as u32;

        if let Some(buffer) = &self.light_array_buffer {
            graphics
                .queue
                .write_buffer(buffer, 0, bytemuck::cast_slice(&[light_array]));
        }

        log_once::debug_once!("LightUniform size = {}", size_of::<LightUniform>());
    }

    pub fn layout(&self) -> &BindGroupLayout {
        self.light_array_layout.as_ref().unwrap()
    }

    pub fn bind_group(&self) -> &BindGroup {
        self.light_array_bind_group.as_ref().unwrap()
    }

    pub fn create_render_pipeline(
        &mut self,
        graphics: Arc<SharedGraphicsContext>,
        shader_contents: &str,
        camera: &Camera,
        label: Option<&str>,
    ) {
        use crate::shader::Shader;

        let shader = Shader::new(graphics.clone(), shader_contents, label);

        let per_light_layout = graphics
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("Per-Light Layout"),
            });

        let pipeline = Self::create_render_pipeline_for_lighting(
            graphics,
            &shader,
            vec![camera.layout(), &per_light_layout],
            label,
        );

        self.pipeline = Some(pipeline);
        log::debug!("Created ECS light render pipeline");
    }

    fn create_render_pipeline_for_lighting(
        graphics: Arc<SharedGraphicsContext>,
        shader: &Shader,
        bind_group_layouts: Vec<&BindGroupLayout>,
        label: Option<&str>,
    ) -> RenderPipeline {
        let render_pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label.unwrap_or("Light Render Pipeline Descriptor")),
                    bind_group_layouts: bind_group_layouts.as_slice(),
                    push_constant_ranges: &[],
                });

        graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: Some("vs_main"),
                    buffers: &[model::ModelVertex::desc(), InstanceRaw::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Greater,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            })
    }
}

#[derive(Default)]
pub struct Instance {
    pub position: DVec3,
    pub rotation: DQuat,
    pub scale: DVec3,

    buffer: Option<Buffer>,
}

impl Instance {
    pub fn new(position: DVec3, rotation: DQuat, scale: DVec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            buffer: None,
        }
    }

    pub fn to_raw(&self) -> InstanceRaw {
        let model_matrix =
            DMat4::from_scale_rotation_translation(self.scale, self.rotation, self.position);
        InstanceRaw {
            model: model_matrix.as_mat4().to_cols_array_2d(),
        }
    }

    pub fn buffer(&self) -> &Buffer {
        self.buffer.as_ref().unwrap()
    }

    pub fn from_matrix(mat: DMat4) -> Self {
        let (scale, rotation, position) = mat.to_scale_rotation_translation();
        Instance {
            position,
            rotation,
            scale,
            buffer: None,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
