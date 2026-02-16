use crate::attenuation::{Attenuation, RANGE_50};
use crate::buffer::{ResizableBuffer, UniformBuffer};
use crate::graphics::SharedGraphicsContext;
use crate::pipelines::light_cube::InstanceInput;
use crate::{
    entity::Transform,
    model::Model,
};
use glam::{DMat4, DVec3};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use wgpu::{BindGroup};
use crate::asset::{Handle, ASSET_REGISTRY};
use crate::procedural::{ProcedurallyGeneratedObject};

pub const MAX_LIGHTS: usize = 10;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    pub cube_model: Handle<Model>,
    pub label: String,
    pub buffer: UniformBuffer<LightUniform>,
    pub bind_group: BindGroup,
    pub instance_buffer: ResizableBuffer<InstanceInput>,
}

impl Light {
    pub const LIGHT_BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'_> = 
        wgpu::BindGroupLayoutDescriptor {
            // @binding(0)
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX.union(wgpu::ShaderStages::FRAGMENT),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("light bind group layout descriptor"),
        };

    pub async fn new(
        graphics: Arc<SharedGraphicsContext>,
        light: LightComponent,
        transform: Transform,
        label: Option<&str>,
    ) -> Self {
        puffin::profile_function!();
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
        };

        log::trace!("Created new light uniform");

        let cube_model = ProcedurallyGeneratedObject::cuboid(DVec3::ONE)
            .build_model(
                graphics.clone(),
                None,
                Some("light cube"),
                ASSET_REGISTRY.clone()
            );

        let label_str = label.unwrap_or("Light").to_string();

        let buffer = UniformBuffer::new(&graphics.device, &label_str);

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &graphics.layouts.light_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.buffer().as_entire_binding(),
                }],
                label,
            });

        let instance: InstanceInput = DMat4::from_scale_rotation_translation(transform.scale, transform.rotation, transform.position).into();

        let mut instance_buffer = ResizableBuffer::new(
            &graphics.device,
            1,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            "Light Instance Buffer"
        );

        instance_buffer.write(&graphics.device, &graphics.queue, &[instance]);

        log::debug!("Created new light [{}]", label_str);

        Self {
            uniform,
            cube_model,
            label: label_str,
            buffer,
            bind_group,
            instance_buffer,
        }
    }

    pub fn update(&mut self, graphics: &SharedGraphicsContext, light: &mut LightComponent, transform: &Transform) {
        puffin::profile_function!();
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

        self.buffer.write(&graphics.queue, &self.uniform);
    }

    pub fn uniform(&self) -> &LightUniform {
        &self.uniform
    }

    pub fn model(&self) -> Handle<Model> {
        self.cube_model
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}