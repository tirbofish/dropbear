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
pub mod collidergroup;

use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::states::Label;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::wgpu::util::{BufferInitDescriptor, DeviceExt};
use dropbear_engine::wgpu::{Buffer, BufferUsages};
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use ::jni::objects::{JObject, JValue};
use ::jni::JNIEnv;
use rapier3d::na::Vector3;
use rapier3d::prelude::ColliderBuilder;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Default, Serialize, Deserialize, Clone, SerializableComponent)]
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
                    half_extents[0].to_bits(),
                    half_extents[1].to_bits(),
                    half_extents[2].to_bits(),
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
pub enum ColliderShape {
    /// Box shape with half-extents (half-width, half-height, half-depth).
    Box { half_extents: [f32; 3] },

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
            half_extents: [0.5, 0.5, 0.5]
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
                        JValue::Double(half_extents[0] as f64),
                        JValue::Double(half_extents[1] as f64),
                        JValue::Double(half_extents[2] as f64)
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
                half_extents: [x as f32, y as f32, z as f32]
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
            shape: ColliderShape::Box { half_extents },
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
                ColliderBuilder::cuboid(half_extents[0], half_extents[1], half_extents[2])
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
            .translation(Vector3::from_column_slice(&self.translation))
            .rotation(Vector3::from_column_slice(&self.rotation))
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
    use crate::types::ColliderFFI;
    use rapier3d::prelude::ColliderHandle;

    pub fn get_collider_mut<'a>(
        physics: &'a mut PhysicsState,
        ffi: &ColliderFFI
    ) -> Option<&'a mut rapier3d::prelude::Collider> {
        let handle = ColliderHandle::from_raw_parts(ffi.index.index, ffi.index.generation);
        physics.colliders.get_mut(handle)
    }

    pub fn get_collider<'a>(
        physics: &'a PhysicsState,
        ffi: &ColliderFFI
    ) -> Option<&'a rapier3d::prelude::Collider> {
        let handle = ColliderHandle::from_raw_parts(ffi.index.index, ffi.index.generation);
        physics.colliders.get(handle)
    }
}

pub mod jni {
    #![allow(non_snake_case)]
    use crate::physics::collider::shared::{get_collider, get_collider_mut};
    use crate::physics::collider::ColliderShape;
    use crate::physics::PhysicsState;
    use crate::scripting::jni::utils::{FromJObject, ToJObject};
    use crate::types::ColliderFFI;
    use glam::DQuat;
    use jni::objects::{JClass, JObject};
    use jni::sys::{jboolean, jdouble, jlong, jobject};
    use jni::JNIEnv;
    use rapier3d::na::{UnitQuaternion, Vector3};
    use rapier3d::prelude::{ColliderHandle, SharedShape, TypedShape};

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderShape(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jobject {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);

        let ffi = match ColliderFFI::from_jobject(&mut env, &collider_obj) {
            Ok(v) => v,
            Err(_) => return std::ptr::null_mut(),
        };

        if let Some(collider) = get_collider(&physics, &ffi) {
            let rapier_shape = collider.shape();

            let my_shape = match rapier_shape.as_typed_shape() {
                TypedShape::Cuboid(c) => {
                    let he = c.half_extents;
                    ColliderShape::Box {
                        half_extents: [he.x, he.y, he.z]
                    }
                },
                TypedShape::Ball(b) => {
                     ColliderShape::Sphere {
                        radius: b.radius
                     }
                },
                TypedShape::Capsule(c) => {
                    let height = c.segment.length();
                    ColliderShape::Capsule {
                        half_height: height * 0.5,
                        radius: c.radius
                    }
                },
                TypedShape::Cylinder(c) => {
                    ColliderShape::Cylinder {
                        half_height: c.half_height,
                        radius: c.radius
                    }
                },
                TypedShape::Cone(c) => {
                    ColliderShape::Cone {
                        half_height: c.half_height,
                        radius: c.radius
                    }
                },
                _ => {
                    eprintln!("Unsupported collider shape type found.");
                    return std::ptr::null_mut();
                }
            };

            match my_shape.to_jobject(&mut env) {
                Ok(obj) => obj.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }

        } else {
            let _ = env.throw_new("java/lang/RuntimeException", "Collider handle invalid");
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderShape(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        shape_obj: JObject,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);

        let ffi = match ColliderFFI::from_jobject(&mut env, &collider_obj) {
            Ok(v) => v,
            Err(_) => return,
        };

        let handle = ColliderHandle::from_raw_parts(ffi.index.index, ffi.index.generation);

        let Some(collider) = physics.colliders.get_mut(handle) else {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "Collider handle invalid");
            return;
        };

        let Ok(shape) = ColliderShape::from_jobject(&mut env, &shape_obj) else {
            let _ = env.throw_new("java/lang/IllegalArgumentException", "Collider shape is invalid");
            return;
        };

        let new_shape = match shape {
            ColliderShape::Box { half_extents } => {
                SharedShape::cuboid(half_extents[0], half_extents[1], half_extents[2])
            }
            ColliderShape::Sphere { radius } => {
                SharedShape::ball(radius)
            }
            ColliderShape::Capsule { half_height, radius } => {
                SharedShape::capsule_y(half_height, radius)
            }
            ColliderShape::Cylinder { half_height, radius } => {
                SharedShape::cylinder(half_height, radius)
            }
            ColliderShape::Cone { half_height, radius } => {
                SharedShape::cone(half_height, radius)
            }
        };

        collider.set_shape(new_shape);
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderDensity(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jdouble {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);
        let ffi = ColliderFFI::from_jobject(&mut env, &collider_obj).ok().unwrap();

        if let Some(col) = get_collider(&physics, &ffi) {
            col.density() as jdouble
        } else {
            0.0
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderDensity(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        density: jdouble,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);
        if let Ok(ffi) = ColliderFFI::from_jobject(&mut env, &collider_obj) {
            if let Some(col) = get_collider_mut(physics, &ffi) {
                col.set_density(density as f32);
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderFriction(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jdouble {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);
        let ffi = ColliderFFI::from_jobject(&mut env, &collider_obj).ok().unwrap();
        if let Some(col) = get_collider(&physics, &ffi) {
            col.friction() as jdouble
        } else { 0.0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderFriction(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        friction: jdouble,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);
        if let Ok(ffi) = ColliderFFI::from_jobject(&mut env, &collider_obj) {
            if let Some(col) = get_collider_mut(physics, &ffi) {
                col.set_friction(friction as f32);
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderRestitution(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jdouble {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);
        let ffi = ColliderFFI::from_jobject(&mut env, &collider_obj).ok().unwrap();
        if let Some(col) = get_collider(&physics, &ffi) {
            col.restitution() as jdouble
        } else { 0.0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderRestitution(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        restitution: jdouble,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);
        if let Ok(ffi) = ColliderFFI::from_jobject(&mut env, &collider_obj) {
            if let Some(col) = get_collider_mut(physics, &ffi) {
                col.set_restitution(restitution as f32);
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderMass(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jdouble {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);
        let ffi = ColliderFFI::from_jobject(&mut env, &collider_obj).ok().unwrap();
        if let Some(col) = get_collider(&physics, &ffi) {
            col.mass() as jdouble
        } else { 0.0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderMass(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        mass: jdouble,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);
        if let Ok(ffi) = ColliderFFI::from_jobject(&mut env, &collider_obj) {
            if let Some(col) = get_collider_mut(physics, &ffi) {
                col.set_mass(mass as f32);
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderIsSensor(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jboolean {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);
        let ffi = ColliderFFI::from_jobject(&mut env, &collider_obj).ok().unwrap();
        if let Some(col) = get_collider(&physics, &ffi) {
            if col.is_sensor() { 1 } else { 0 }
        } else { 0 }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderIsSensor(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        is_sensor: jboolean,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);
        if let Ok(ffi) = ColliderFFI::from_jobject(&mut env, &collider_obj) {
            if let Some(col) = get_collider_mut(physics, &ffi) {
                col.set_sensor(is_sensor != 0);
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderTranslation(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jobject {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);
        let ffi = match ColliderFFI::from_jobject(&mut env, &collider_obj) {
            Ok(v) => v,
            Err(_) => return std::ptr::null_mut(),
        };

        if let Some(col) = get_collider(&physics, &ffi) {
            let t: &Vector3<f32> = col.translation();
            let vec = crate::types::Vector3::new(t.x as f64, t.y as f64, t.z as f64);
            match vec.to_jobject(&mut env) {
                Ok(o) => o.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderTranslation(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        vec_obj: JObject,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);
        if let Ok(ffi) = ColliderFFI::from_jobject(&mut env, &collider_obj) {
            if let Ok(vec) = crate::types::Vector3::from_jobject(&mut env, &vec_obj) {
                if let Some(col) = get_collider_mut(physics, &ffi) {
                    let t = Vector3::new(vec.x as f32, vec.y as f32, vec.z as f32);
                    col.set_translation(t);
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_getColliderRotation(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
    ) -> jobject {
        let physics = crate::convert_ptr!(physics_ptr => PhysicsState);
        let ffi = match ColliderFFI::from_jobject(&mut env, &collider_obj) {
            Ok(v) => v,
            Err(_) => return std::ptr::null_mut(),
        };

        if let Some(col) = get_collider(&physics, &ffi) {
            let r: &UnitQuaternion<f32> = col.rotation();
            let q = DQuat::from_xyzw(r.i as f64, r.j as f64, r.k as f64, r.w as f64);
            let euler = q.to_euler(glam::EulerRot::XYZ);
            let vec = crate::types::Vector3::new(euler.0, euler.1, euler.2);

            match vec.to_jobject(&mut env) {
                Ok(o) => o.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    }

    #[unsafe(no_mangle)]
    pub fn Java_com_dropbear_physics_ColliderNative_setColliderRotation(
        mut env: JNIEnv,
        _class: JClass,
        physics_ptr: jlong,
        collider_obj: JObject,
        vec_obj: JObject,
    ) {
        let physics = crate::convert_ptr!(mut physics_ptr => PhysicsState);
        if let Ok(ffi) = ColliderFFI::from_jobject(&mut env, &collider_obj) {
            if let Ok(vec) = crate::types::Vector3::from_jobject(&mut env, &vec_obj) {
                if let Some(col) = get_collider_mut(physics, &ffi) {
                    let q = DQuat::from_euler(glam::EulerRot::XYZ, vec.x, vec.y, vec.z);
                    let r = rapier3d::na::UnitQuaternion::new_normalize(
                        rapier3d::na::Quaternion::new(q.w as f32, q.x as f32, q.y as f32, q.z as f32)
                    );
                    col.set_rotation(r);
                }
            }
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use crate::convert_ptr;
    use crate::physics::collider::shared::{get_collider, get_collider_mut};
    use crate::physics::PhysicsState;
    use crate::ptr::PhysicsStatePtr;
    use crate::types::Vector3;

    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::{ColliderFFI, ColliderShapeFFI, ColliderShapeType};
    use glam::DQuat;
    use rapier3d::na::{Quaternion, UnitQuaternion};
    use rapier3d::prelude::{SharedShape, TypedShape};

    pub fn dropbear_get_collider_shape(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<ColliderShapeFFI> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);

        if let Some(collider) = get_collider(physics, &ffi) {
            let rapier_shape = collider.shape();
            let mut result = ColliderShapeFFI {
                shape_type: ColliderShapeType::Box,
                radius: 0.0, half_height: 0.0,
                half_extents_x: 0.0, half_extents_y: 0.0, half_extents_z: 0.0,
            };

            match rapier_shape.as_typed_shape() {
                TypedShape::Cuboid(c) => {
                    result.shape_type = ColliderShapeType::Box;
                    result.half_extents_x = c.half_extents.x;
                    result.half_extents_y = c.half_extents.y;
                    result.half_extents_z = c.half_extents.z;
                },
                TypedShape::Ball(b) => {
                    result.shape_type = ColliderShapeType::Sphere;
                    result.radius = b.radius;
                },
                TypedShape::Capsule(c) => {
                    result.shape_type = ColliderShapeType::Capsule;
                    result.radius = c.radius;
                    result.half_height = c.segment.length() * 0.5;
                },
                TypedShape::Cylinder(c) => {
                    result.shape_type = ColliderShapeType::Cylinder;
                    result.radius = c.radius;
                    result.half_height = c.half_height;
                },
                TypedShape::Cone(c) => {
                    result.shape_type = ColliderShapeType::Cone;
                    result.radius = c.radius;
                    result.half_height = c.half_height;
                },
                _ => return DropbearNativeResult::Err(DropbearNativeError::GenericError),
            }
            DropbearNativeResult::Ok(result)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_shape(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        shape: ColliderShapeFFI,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);

        if let Some(collider) = get_collider_mut(physics, &ffi) {
            let new_shape = match shape.shape_type {
                ColliderShapeType::Box => SharedShape::cuboid(shape.half_extents_x, shape.half_extents_y, shape.half_extents_z),
                ColliderShapeType::Sphere => SharedShape::ball(shape.radius),
                ColliderShapeType::Capsule => SharedShape::capsule_y(shape.half_height, shape.radius),
                ColliderShapeType::Cylinder => SharedShape::cylinder(shape.half_height, shape.radius),
                ColliderShapeType::Cone => SharedShape::cone(shape.half_height, shape.radius),
            };
            collider.set_shape(new_shape);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::NoSuchHandle)
        }
    }

    pub fn dropbear_get_collider_density(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<f64> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        if let Some(col) = get_collider(physics, &ffi) {
            DropbearNativeResult::Ok(col.density() as f64)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_density(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        density: f64,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);
        if let Some(col) = get_collider_mut(physics, &ffi) {
            col.set_density(density as f32);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_get_collider_friction(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<f64> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        if let Some(col) = get_collider(physics, &ffi) {
            DropbearNativeResult::Ok(col.friction() as f64)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_friction(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        friction: f64,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);
        if let Some(col) = get_collider_mut(physics, &ffi) {
            col.set_friction(friction as f32);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_get_collider_restitution(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<f64> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        if let Some(col) = get_collider(physics, &ffi) {
            DropbearNativeResult::Ok(col.restitution() as f64)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_restitution(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        restitution: f64,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);
        if let Some(col) = get_collider_mut(physics, &ffi) {
            col.set_restitution(restitution as f32);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_get_collider_mass(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<f64> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        if let Some(col) = get_collider(physics, &ffi) {
            DropbearNativeResult::Ok(col.mass() as f64)
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_mass(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        mass: f64,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);
        if let Some(col) = get_collider_mut(physics, &ffi) {
            col.set_mass(mass as f32);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_get_collider_is_sensor(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<bool> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        if let Some(col) = get_collider(physics, &ffi) {
            DropbearNativeResult::Ok(col.is_sensor())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_is_sensor(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        is_sensor: bool,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);
        if let Some(col) = get_collider_mut(physics, &ffi) {
            col.set_sensor(is_sensor);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_get_collider_translation(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<Vector3> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        if let Some(col) = get_collider(physics, &ffi) {
            let t = col.translation();
            DropbearNativeResult::Ok(Vector3 { x: t.x as f64, y: t.y as f64, z: t.z as f64 })
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_translation(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        translation: Vector3,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);
        if let Some(col) = get_collider_mut(physics, &ffi) {
            let t = rapier3d::na::Vector3::new(translation.x as f32, translation.y as f32, translation.z as f32);
            col.set_translation(t);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_get_collider_rotation(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
    ) -> DropbearNativeResult<Vector3> {
        let physics = convert_ptr!(physics_ptr => PhysicsState);
        if let Some(col) = get_collider(physics, &ffi) {
            let r = col.rotation();
            let q = DQuat::from_xyzw(r.i as f64, r.j as f64, r.k as f64, r.w as f64);
            let (x, y, z) = q.to_euler(glam::EulerRot::XYZ);
            DropbearNativeResult::Ok(Vector3 { x, y, z })
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }

    pub fn dropbear_set_collider_rotation(
        physics_ptr: PhysicsStatePtr,
        ffi: ColliderFFI,
        rotation: Vector3,
    ) -> DropbearNativeResult<()> {
        let physics = convert_ptr!(mut physics_ptr => PhysicsState);
        if let Some(col) = get_collider_mut(physics, &ffi) {
            let q = DQuat::from_euler(glam::EulerRot::XYZ, rotation.x, rotation.y, rotation.z);
            let r = UnitQuaternion::new_normalize(
                Quaternion::new(q.w as f32, q.x as f32, q.y as f32, q.z as f32)
            );
            col.set_rotation(r);
            DropbearNativeResult::Ok(())
        } else {
            DropbearNativeResult::Err(DropbearNativeError::InvalidHandle)
        }
    }
}