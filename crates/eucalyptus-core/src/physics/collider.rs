//! Colliders and shapes that allow for entities to collide with other entities.
//!
//! ### TODO: Implement collision type detection:
//! - Unreal
//!     - `UCX_` = Convex collision (most common)
//!     - `UBX_` = Box collision
//!     - `USP_` = Sphere collision
//!     - `UCP_` = Capsule collision
//!     - `MCDCX_` = Complex collision as simple
//! - Unity
//!     - Suffix or contains: `_collider`, `_collision`, `_col`
//! - Godot
//!     - `col-` or `-col`
//!     - `-colonly` (invisible collision mesh)

pub mod shader;
pub mod collider_group;

use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::states::Label;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::wgpu::util::{BufferInitDescriptor, DeviceExt};
use dropbear_engine::wgpu::{Buffer, BufferUsages};
use std::any::Any;
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;
use rapier3d::prelude::ColliderBuilder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use egui::{CollapsingHeader, Ui};
use crate::physics::collider::shared::{get_collider, get_collider_mut};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{NCollider, NVector3};
use glam::DQuat;
use hecs::{Entity, World};
use rapier3d::prelude::{Rotation, SharedShape, TypedShape, Vector};
use dropbear_engine::animation::AnimationComponent;
use crate::component::{Component, ComponentDescriptor, InspectableComponent, SerializedComponent};
use crate::physics::PhysicsState;
use crate::ptr::PhysicsStatePtr;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ColliderGroup {
    #[serde(default)]
    pub colliders: Vec<Collider>,
}

impl ColliderGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, collider: Collider) {
        self.colliders.push(collider);
    }
}

#[typetag::serde]
impl SerializedComponent for ColliderGroup {}

impl Component for ColliderGroup {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self, );

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "eucalyptus_core::physics::collider::ColliderGroup".to_string(),
            type_name: "ColliderGroup".to_string(),
            category: Some("Physics".to_string()),
            description: Some("A group of colliders".to_string()),
        }
    }

    async fn first_time(_: Arc<SharedGraphicsContext>) -> anyhow::Result<Self::RequiredComponentTypes>
    where
        Self: Sized
    {
        Ok((Self::new(), ))
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> crate::component::ComponentInitFuture<'a, Self> {
        Box::pin(async move { Ok((ser.clone(), )) })
    }

    fn update_component(&mut self, _world: &World, _entity: Entity, _dt: f32, _graphics: Arc<SharedGraphicsContext>) {}

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for ColliderGroup {
    fn inspect(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Colliders").default_open(true).show(ui, |ui| {
            ui.label("Not implemented yet!");
        });
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collider {
    /// A unique identifier index for this collider.
    #[serde(default)]
    pub id: u32,

    /// The entity this component is attached to.
    #[serde(default)]
    pub entity: Label,

    /// The collision shape.
    pub shape: ColliderShape,

    /// Density of the collider (used to calculate mass).
    #[serde(default = "Collider::default_density")]
    pub density: f32,

    /// Friction coefficient (0.0 = no friction, 1.0 = high friction).
    #[serde(default = "Collider::default_friction")]
    pub friction: f32,

    /// Restitution/bounciness (0.0 = no bounce, 1.0 = perfect bounce).
    #[serde(default)]
    pub restitution: f32,

    /// Whether this is a sensor (triggers collision events but no physical response).
    #[serde(default)]
    pub is_sensor: bool,

    /// Local translation offset from the rigid body.
    #[serde(default)]
    pub translation: [f32; 3],

    /// Local rotation offset from the rigid body (Euler angles in radians).
    #[serde(default)]
    pub rotation: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColliderShapeType {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Cone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColliderShapeKey {
    Box { half_extents_bits: [u32; 3] },
    Sphere { radius_bits: u32 },
    Capsule { half_height_bits: u32, radius_bits: u32 },
    Cylinder { half_height_bits: u32, radius_bits: u32 },
    Cone { half_height_bits: u32, radius_bits: u32 },
}

impl From<&ColliderShape> for ColliderShapeKey {
    fn from(shape: &ColliderShape) -> Self {
        match *shape {
            ColliderShape::Box { half_extents } => Self::Box {
                half_extents_bits: [
                    half_extents.x.to_bits() as u32,
                    half_extents.y.to_bits() as u32,
                    half_extents.z.to_bits() as u32,
                ],
            },
            ColliderShape::Sphere { radius } => Self::Sphere {
                radius_bits: radius.to_bits(),
            },
            ColliderShape::Capsule { half_height, radius } => Self::Capsule {
                half_height_bits: half_height.to_bits(),
                radius_bits: radius.to_bits(),
            },
            ColliderShape::Cylinder { half_height, radius } => Self::Cylinder {
                half_height_bits: half_height.to_bits(),
                radius_bits: radius.to_bits(),
            },
            ColliderShape::Cone { half_height, radius } => Self::Cone {
                half_height_bits: half_height.to_bits(),
                radius_bits: radius.to_bits(),
            },
        }
    }
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[dropbear_macro::repr_c_enum]
pub enum ColliderShape {
    /// Box shape with half-extents (half-width, half-height, half-depth).
    Box { half_extents: NVector3 },

    /// Sphere shape with radius.
    Sphere { radius: f32 },

    /// Capsule shape along Y-axis.
    Capsule { half_height: f32, radius: f32 },

    /// Cylinder shape along Y-axis.
    Cylinder { half_height: f32, radius: f32 },

    /// Cone shape along Y-axis.
    Cone { half_height: f32, radius: f32 },
}

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Box {
            half_extents: NVector3::from([0.5, 0.5, 0.5])
        }
    }
}

impl ToJObject for ColliderShape {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            ColliderShape::Box { half_extents } => {
                let vec_cls = env.find_class("com/dropbear/math/Vector3d")
                    .map_err(|e| {
                        eprintln!("[JNI Error] Vector3d class not found: {:?}", e);
                        DropbearNativeError::JNIClassNotFound
                    })?;

                let vec_obj = env.new_object(
                    &vec_cls,
                    "(DDD)V",
                    &[
                        JValue::Double(half_extents.x),
                        JValue::Double(half_extents.y),
                        JValue::Double(half_extents.z)
                    ]
                ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                let cls = env.find_class("com/dropbear/physics/ColliderShape$Box")
                    .map_err(|e| {
                        eprintln!("[JNI Error] ColliderShape$Box class not found: {:?}", e);
                        DropbearNativeError::JNIClassNotFound
                    })?;

                let obj = env.new_object(
                    &cls,
                    "(Lcom/dropbear/math/Vector3d;)V",
                    &[JValue::Object(&vec_obj)]
                ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            },
            ColliderShape::Sphere { radius } => {
                let cls = env.find_class("com/dropbear/physics/ColliderShape$Sphere")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                let obj = env.new_object(
                    &cls,
                    "(F)V",
                    &[JValue::Float(*radius)]
                ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            },
            ColliderShape::Capsule { half_height, radius } => {
                let cls = env.find_class("com/dropbear/physics/ColliderShape$Capsule")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                let obj = env.new_object(
                    &cls,
                    "(FF)V",
                    &[JValue::Float(*half_height), JValue::Float(*radius)]
                ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            },
            ColliderShape::Cylinder { half_height, radius } => {
                let cls = env.find_class("com/dropbear/physics/ColliderShape$Cylinder")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                // let ctor = env.get_method_id(&cls, "<init>", "(FF)V")
                //     .map_err(|_| DropbearNativeError::JNIMethodNotFound)?;

                let obj = env.new_object(
                    &cls,
                    "(FF)V",
                    &[JValue::Float(*half_height), JValue::Float(*radius)]
                ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            },
            ColliderShape::Cone { half_height, radius } => {
                let cls = env.find_class("com/dropbear/physics/ColliderShape$Cone")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

                let obj = env.new_object(
                    &cls,
                    "(FF)V",
                    &[JValue::Float(*half_height), JValue::Float(*radius)]
                ).map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

                Ok(obj)
            },
        }
    }
}

impl FromJObject for ColliderShape {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let is_instance = |env: &mut JNIEnv, obj: &JObject, class_name: &str| -> bool {
            env.is_instance_of(obj, class_name).unwrap_or(false)
        };

        if is_instance(env, obj, "com/dropbear/physics/ColliderShape$Box") {
            let vec_obj_val = env.get_field(obj, "halfExtents", "Lcom/dropbear/math/Vector3d;")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?;
            let vec_obj = vec_obj_val.l()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

            let x = env.get_field(&vec_obj, "x", "D")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.d().unwrap_or(0.0);
            let y = env.get_field(&vec_obj, "y", "D")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.d().unwrap_or(0.0);
            let z = env.get_field(&vec_obj, "z", "D")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.d().unwrap_or(0.0);

            return Ok(ColliderShape::Box {
                half_extents: NVector3::from([x as f32, y as f32, z as f32])
            });
        }

        if is_instance(env, obj, "com/dropbear/physics/ColliderShape$Sphere") {
            let radius = env.get_field(obj, "radius", "F")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.f().unwrap_or(0.0);

            return Ok(ColliderShape::Sphere { radius });
        }

        if is_instance(env, obj, "com/dropbear/physics/ColliderShape$Capsule") {
            let hh = env.get_field(obj, "halfHeight", "F")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.f().unwrap_or(0.0);
            let r = env.get_field(obj, "radius", "F")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.f().unwrap_or(0.0);

            return Ok(ColliderShape::Capsule { half_height: hh, radius: r });
        }

        if is_instance(env, obj, "com/dropbear/physics/ColliderShape$Cylinder") {
            let hh = env.get_field(obj, "halfHeight", "F")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.f().unwrap_or(0.0);
            let r = env.get_field(obj, "radius", "F")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.f().unwrap_or(0.0);

            return Ok(ColliderShape::Cylinder { half_height: hh, radius: r });
        }

        if is_instance(env, obj, "com/dropbear/physics/ColliderShape$Cone") {
            let hh = env.get_field(obj, "halfHeight", "F")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.f().unwrap_or(0.0);
            let r = env.get_field(obj, "radius", "F")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?.f().unwrap_or(0.0);

            return Ok(ColliderShape::Cone { half_height: hh, radius: r });
        }

        Err(DropbearNativeError::GenericError)
    }
}

impl Collider {
    fn default_density() -> f32 { 1.0 }
    fn default_friction() -> f32 { 0.5 }

    pub fn new() -> Self {
        Self {
            id: 0 as u32,
            entity: Label::default(),
            shape: ColliderShape::default(),
            density: Self::default_density(),
            friction: Self::default_friction(),
            restitution: 0.0,
            is_sensor: false,
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
        }
    }

    /// Create a box collider
    pub fn box_collider(half_extents: [f32; 3]) -> Self {
        Self {
            shape: ColliderShape::Box { half_extents: NVector3::from(half_extents) },
            ..Self::new()
        }
    }

    /// Create a sphere collider
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            ..Self::new()
        }
    }

    /// Create a capsule collider
    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule { half_height, radius },
            ..Self::new()
        }
    }

    /// Create a cylinder collider
    pub fn cylinder(half_height: f32, radius: f32) -> Self {
        Self {
            shape: ColliderShape::Cylinder { half_height, radius },
            ..Self::new()
        }
    }

    /// Set the collider as a sensor (no physical response, only triggers events)
    pub fn with_sensor(mut self) -> Self {
        self.is_sensor = true;
        self
    }

    /// Set the translation offset
    pub fn with_translation(mut self, translation: [f32; 3]) -> Self {
        self.translation = translation;
        self
    }

    /// Set the rotation offset (in radians)
    pub fn with_rotation(mut self, rotation: [f32; 3]) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the friction coefficient
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    /// Set the restitution (bounciness)
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;
        self
    }

    /// Set the density
    pub fn with_density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    pub fn to_rapier(&self) -> rapier3d::prelude::Collider {
        let shape: ColliderBuilder = match &self.shape {
            ColliderShape::Box { half_extents } => {
                ColliderBuilder::cuboid(half_extents.x as f32, half_extents.y as f32, half_extents.z as f32)
            }
            ColliderShape::Sphere { radius } => {
                ColliderBuilder::ball(*radius)
            }
            ColliderShape::Capsule { half_height, radius } => {
                ColliderBuilder::capsule_y(*half_height, *radius)
            }
            ColliderShape::Cylinder { half_height, radius } => {
                ColliderBuilder::cylinder(*half_height, *radius)
            }
            ColliderShape::Cone { half_height, radius } => {
                ColliderBuilder::cone(*half_height, *radius)
            }
        };

        shape
            .density(self.density)
            .friction(self.friction)
            .restitution(self.restitution)
            .sensor(self.is_sensor)
            .translation(Vector::from_array(self.translation))
            .rotation(Vector::from_array(self.rotation))
            .build()
    }

    pub fn shape_type_name(&self) -> &'static str {
        match self.shape {
            ColliderShape::Box { .. } => "Box",
            ColliderShape::Sphere { .. } => "Sphere",
            ColliderShape::Capsule { .. } => "Capsule",
            ColliderShape::Cylinder { .. } => "Cylinder",
            ColliderShape::Cone { .. } => "Cone",
            // ColliderShape::ConvexHull { .. } => "ConvexHull",
            // ColliderShape::TriMesh { .. } => "TriMesh",
            // ColliderShape::HeightField { .. } => "HeightField",
            // ColliderShape::Compound { .. } => "Compound",
        }
    }
}

pub struct WireframeGeometry {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl WireframeGeometry {
    pub fn box_wireframe(graphics: Arc<SharedGraphicsContext>, half_extents: [f32; 3]) -> Self {
        let [hx, hy, hz] = half_extents;

        let vertices: Vec<[f32; 3]> = vec![
            [-hx, -hy, -hz], [-hx, -hy,  hz], [-hx,  hy, -hz], [-hx,  hy,  hz],
            [ hx, -hy, -hz], [ hx, -hy,  hz], [ hx,  hy, -hz], [ hx,  hy,  hz],
        ];

        let indices: Vec<u16> = vec![
            0, 1,  0, 2,  0, 4,  // from corner 0
            1, 3,  1, 5,          // from corner 1
            2, 3,  2, 6,          // from corner 2
            3, 7,                 // from corner 3
            4, 5,  4, 6,          // from corner 4
            5, 7,                 // from corner 5
            6, 7,                 // from corner 6
        ];

        let vertex_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("box wireframe vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("box wireframe index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn sphere_wireframe(graphics: Arc<SharedGraphicsContext>, radius: f32, lat_segments: u32, lon_segments: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for lat in 0..=lat_segments {
            let theta = std::f32::consts::PI * lat as f32 / lat_segments as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for lon in 0..=lon_segments {
                let phi = 2.0 * std::f32::consts::PI * lon as f32 / lon_segments as f32;
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();

                let x = radius * sin_theta * cos_phi;
                let y = radius * cos_theta;
                let z = radius * sin_theta * sin_phi;

                vertices.push([x, y, z]);
            }
        }

        for lat in 0..lat_segments {
            for lon in 0..lon_segments {
                let first = (lat * (lon_segments + 1) + lon) as u16;
                let second = first + lon_segments as u16 + 1;

                indices.push(first);
                indices.push(first + 1);

                indices.push(first);
                indices.push(second);
            }
        }

        let vertex_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sphere wireframe vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("sphere wireframe index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn capsule_wireframe(graphics: Arc<SharedGraphicsContext>, half_height: f32, radius: f32, segments: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=segments / 2 {
            let theta = std::f32::consts::PI * i as f32 / segments as f32;
            let y = half_height + radius * theta.cos();
            let r = radius * theta.sin();

            for j in 0..=segments {
                let phi = 2.0 * std::f32::consts::PI * j as f32 / segments as f32;
                vertices.push([r * phi.cos(), y, r * phi.sin()]);
            }
        }

        for j in 0..=segments {
            let phi = 2.0 * std::f32::consts::PI * j as f32 / segments as f32;
            vertices.push([radius * phi.cos(), half_height, radius * phi.sin()]);
            vertices.push([radius * phi.cos(), -half_height, radius * phi.sin()]);
        }

        for i in 0..=segments / 2 {
            let theta = std::f32::consts::PI * i as f32 / segments as f32;
            let y = -half_height - radius * theta.cos();
            let r = radius * theta.sin();

            for j in 0..=segments {
                let phi = 2.0 * std::f32::consts::PI * j as f32 / segments as f32;
                vertices.push([r * phi.cos(), y, r * phi.sin()]);
            }
        }

        for i in 0..(vertices.len() as u16 - 1) {
            indices.push(i);
            indices.push(i + 1);
        }

        let vertex_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("capsule wireframe vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = graphics.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("capsule wireframe index buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn cylinder_wireframe(graphics: Arc<SharedGraphicsContext>, half_height: f32, radius: f32, _segments: u32) -> Self {
        // TODO: Implement cylinder wireframe
        Self::box_wireframe(graphics, [radius, half_height, radius]) // Placeholder
    }

    pub fn cone_wireframe(graphics: Arc<SharedGraphicsContext>, half_height: f32, radius: f32, _segments: u32) -> Self {
        // TODO: Implement cone wireframe
        Self::box_wireframe(graphics, [radius, half_height, radius]) // Placeholder
    }
}

pub mod shared {
    use crate::physics::PhysicsState;
    use crate::types::NCollider;
    use rapier3d::prelude::ColliderHandle;

    pub fn get_collider_mut<'a>(
        physics: &'a mut PhysicsState,
        ffi: &NCollider
    ) -> Option<&'a mut rapier3d::prelude::Collider> {
        let handle = ColliderHandle::from_raw_parts(ffi.index.index, ffi.index.generation);
        physics.colliders.get_mut(handle)
    }

    pub fn get_collider<'a>(
        physics: &'a PhysicsState,
        ffi: &NCollider
    ) -> Option<&'a rapier3d::prelude::Collider> {
        let handle = ColliderHandle::from_raw_parts(ffi.index.index, ffi.index.generation);
        physics.colliders.get(handle)
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderShape"),
    c
)]
fn get_collider_shape(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<ColliderShape> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;

    let rapier_shape = collider.shape();
    let my_shape = match rapier_shape.as_typed_shape() {
        TypedShape::Cuboid(c) => {
            let he = c.half_extents;
            ColliderShape::Box {
                half_extents: NVector3::from([he.x, he.y, he.z]),
            }
        }
        TypedShape::Ball(b) => ColliderShape::Sphere { radius: b.radius },
        TypedShape::Capsule(c) => {
            let height = c.segment.length();
            ColliderShape::Capsule {
                half_height: height * 0.5,
                radius: c.radius,
            }
        }
        TypedShape::Cylinder(c) => ColliderShape::Cylinder {
            half_height: c.half_height,
            radius: c.radius,
        },
        TypedShape::Cone(c) => ColliderShape::Cone {
            half_height: c.half_height,
            radius: c.radius,
        },
        _ => return Err(DropbearNativeError::InvalidArgument),
    };

    Ok(my_shape)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderShape"),
    c
)]
fn set_collider_shape(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    shape: &ColliderShape,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;

    let new_shape = match shape {
        ColliderShape::Box { half_extents } => {
            SharedShape::cuboid(half_extents.x as f32, half_extents.y as f32, half_extents.z as f32)
        }
        ColliderShape::Sphere { radius } => SharedShape::ball(*radius),
        ColliderShape::Capsule { half_height, radius } => {
            SharedShape::capsule_y(*half_height, *radius)
        }
        ColliderShape::Cylinder { half_height, radius } => {
            SharedShape::cylinder(*half_height, *radius)
        }
        ColliderShape::Cone { half_height, radius } => SharedShape::cone(*half_height, *radius),
    };

    collider.set_shape(new_shape);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderDensity"),
    c
)]
fn get_collider_density(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.density() as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderDensity"),
    c
)]
fn set_collider_density(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    density: f64,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_density(density as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderFriction"),
    c
)]
fn get_collider_friction(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.friction() as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderFriction"),
    c
)]
fn set_collider_friction(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    friction: f64,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_friction(friction as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderRestitution"),
    c
)]
fn get_collider_restitution(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.restitution() as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderRestitution"),
    c
)]
fn set_collider_restitution(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    restitution: f64,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_restitution(restitution as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderMass"),
    c
)]
fn get_collider_mass(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<f64> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.mass() as f64)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderMass"),
    c
)]
fn set_collider_mass(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    mass: f64,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_mass(mass as f32);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderIsSensor"),
    c
)]
fn get_collider_is_sensor(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<bool> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    Ok(collider.is_sensor())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderIsSensor"),
    c
)]
fn set_collider_is_sensor(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    is_sensor: bool,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    collider.set_sensor(is_sensor);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderTranslation"),
    c
)]
fn get_collider_translation(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<NVector3> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let t: Vector = collider.translation();
    Ok(NVector3::new(t.x as f64, t.y as f64, t.z as f64))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderTranslation"),
    c
)]
fn set_collider_translation(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    translation: &NVector3,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let t = Vector::new(translation.x as f32, translation.y as f32, translation.z as f32);
    collider.set_translation(t);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "getColliderRotation"),
    c
)]
fn get_collider_rotation(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &PhysicsState,
    collider: &NCollider,
) -> DropbearNativeResult<NVector3> {
    let collider = get_collider(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let r: Rotation = collider.rotation();
    let q = DQuat::from_xyzw(r.x as f64, r.y as f64, r.z as f64, r.w as f64);
    let euler = q.to_euler(glam::EulerRot::XYZ);
    Ok(NVector3::new(euler.0, euler.1, euler.2))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.physics.ColliderNative", func = "setColliderRotation"),
    c
)]
fn set_collider_rotation(
    #[dropbear_macro::define(PhysicsStatePtr)]
    physics: &mut PhysicsState,
    collider: &NCollider,
    rotation: &NVector3,
) -> DropbearNativeResult<()> {
    let collider = get_collider_mut(physics, &collider)
        .ok_or(DropbearNativeError::PhysicsObjectNotFound)?;
    let q = DQuat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
    let r = Rotation::from_array([q.w as f32, q.x as f32, q.y as f32, q.z as f32]);
    collider.set_rotation(r);
    Ok(())
}