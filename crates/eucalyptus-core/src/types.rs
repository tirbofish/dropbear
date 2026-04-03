use crate::physics::PhysicsState;
use dropbear_engine::entity::Transform;
use glam::{DQuat, DVec3, Vec3};
use rapier3d::data::Index;
use rapier3d::geometry::ColliderHandle;
use serde::{Deserialize, Serialize};

// --------------------------------------------------------------- math types --

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NVector2 {
    pub x: f64,
    pub y: f64,
}

impl NVector2 {
    pub fn to_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }
}

impl From<glam::DVec2> for NVector2 {
    fn from(v: glam::DVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}
impl From<[f64; 2]> for NVector2 {
    fn from(v: [f64; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}
impl From<[f32; 2]> for NVector2 {
    fn from(v: [f32; 2]) -> Self {
        Self {
            x: v[0] as f64,
            y: v[1] as f64,
        }
    }
}
impl From<(f64, f64)> for NVector2 {
    fn from(v: (f64, f64)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}
impl From<NVector2> for glam::DVec2 {
    fn from(v: NVector2) -> Self {
        Self::new(v.x, v.y)
    }
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct NVector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl NVector3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn to_array(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    pub fn to_float_array(&self) -> [f32; 3] {
        [self.x as f32, self.y as f32, self.z as f32]
    }
}

impl From<DVec3> for NVector3 {
    fn from(v: DVec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}
impl From<&DVec3> for NVector3 {
    fn from(v: &DVec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}
impl From<&NVector3> for DVec3 {
    fn from(v: &NVector3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}
impl From<NVector3> for DVec3 {
    fn from(v: NVector3) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}
impl From<Vec3> for NVector3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x as f64,
            y: v.y as f64,
            z: v.z as f64,
        }
    }
}
impl From<NVector3> for Vec3 {
    fn from(v: NVector3) -> Self {
        Self::new(v.x as f32, v.y as f32, v.z as f32)
    }
}
impl From<[f64; 3]> for NVector3 {
    fn from(v: [f64; 3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}
impl From<[f32; 3]> for NVector3 {
    fn from(v: [f32; 3]) -> Self {
        Self {
            x: v[0] as f64,
            y: v[1] as f64,
            z: v[2] as f64,
        }
    }
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NVector4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl NVector4 {
    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }
    pub fn to_array(&self) -> [f64; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

impl From<glam::DVec4> for NVector4 {
    fn from(v: glam::DVec4) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}
impl From<NVector4> for glam::DVec4 {
    fn from(v: NVector4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}
impl From<[f64; 4]> for NVector4 {
    fn from(v: [f64; 4]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
            w: v[3],
        }
    }
}
impl From<[f32; 4]> for NVector4 {
    fn from(v: [f32; 4]) -> Self {
        Self {
            x: v[0] as f64,
            y: v[1] as f64,
            z: v[2] as f64,
            w: v[3] as f64,
        }
    }
}
impl From<NQuaternion> for NVector4 {
    fn from(q: NQuaternion) -> Self {
        Self {
            x: q.x,
            y: q.y,
            z: q.z,
            w: q.w,
        }
    }
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NQuaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl From<DQuat> for NQuaternion {
    fn from(v: DQuat) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}
impl From<NQuaternion> for DQuat {
    fn from(v: NQuaternion) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}
impl From<[f64; 4]> for NQuaternion {
    fn from(v: [f64; 4]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
            w: v[3],
        }
    }
}
impl From<glam::Quat> for NQuaternion {
    fn from(v: glam::Quat) -> Self {
        Self {
            x: v.x as f64,
            y: v.y as f64,
            z: v.z as f64,
            w: v.w as f64,
        }
    }
}
impl From<NVector4> for NQuaternion {
    fn from(v: NVector4) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NTransform {
    pub position: NVector3,
    pub rotation: NQuaternion,
    pub scale: NVector3,
}

impl From<Transform> for NTransform {
    fn from(v: Transform) -> Self {
        Self {
            position: NVector3::from(v.position),
            rotation: NQuaternion::from(v.rotation),
            scale: NVector3::from(v.scale),
        }
    }
}
impl From<NTransform> for Transform {
    fn from(v: NTransform) -> Self {
        Self {
            position: DVec3::from(v.position),
            rotation: DQuat::from(v.rotation),
            scale: DVec3::from(v.scale),
        }
    }
}

// ------------------------------------------------------------------- colour --

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl NColour {
    pub fn to_f32_array(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn to_linear_rgb(self) -> DVec3 {
        DVec3::new(
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }

    pub fn from_linear_rgb(rgb: DVec3) -> Self {
        fn clamp_u8(x: f64) -> u8 {
            (x * 255.0).round().clamp(0.0, 255.0) as u8
        }
        Self {
            r: clamp_u8(rgb.x),
            g: clamp_u8(rgb.y),
            b: clamp_u8(rgb.z),
            a: 255,
        }
    }
}

// ----------------------------------------------------------- physics types ---

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct IndexNative {
    pub index: u32,
    pub generation: u32,
}

impl From<Index> for IndexNative {
    fn from(v: Index) -> Self {
        let raw = v.into_raw_parts();
        Self {
            index: raw.0,
            generation: raw.1,
        }
    }
}
impl From<IndexNative> for Index {
    fn from(v: IndexNative) -> Self {
        Self::from_raw_parts(v.index, v.generation)
    }
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NCollider {
    pub index: IndexNative,
    pub entity_id: u64,
    pub id: u32,
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub collider: NCollider,
    pub distance: f64,
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RigidBodyContext {
    pub index: IndexNative,
    pub entity_id: u64,
}

// -----------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NShapeCastHit {
    pub collider: NCollider,
    pub distance: f64,
    pub witness1: NVector3,
    pub witness2: NVector3,
    pub normal1: NVector3,
    pub normal2: NVector3,
    pub status: NShapeCastStatus,
}

#[repr(C)]
#[dropbear_macro::repr_c_enum]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NShapeCastStatus {
    OutOfIterations,
    Converged,
    Failed,
    PenetratingOrWithinTargetDist,
}

impl From<rapier3d::parry::query::ShapeCastStatus> for NShapeCastStatus {
    fn from(v: rapier3d::parry::query::ShapeCastStatus) -> Self {
        use rapier3d::parry::query::ShapeCastStatus;
        match v {
            ShapeCastStatus::OutOfIterations => Self::OutOfIterations,
            ShapeCastStatus::Converged => Self::Converged,
            ShapeCastStatus::Failed => Self::Failed,
            ShapeCastStatus::PenetratingOrWithinTargetDist => Self::PenetratingOrWithinTargetDist,
        }
    }
}

// -------------------------------------------------------------- event types --

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum CollisionEventType {
    Started,
    Stopped,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CollisionEvent {
    pub event_type: CollisionEventType,
    pub collider1: NCollider,
    pub collider2: NCollider,
    pub flags: u64,
}

impl CollisionEvent {
    pub fn collider1_entity_id(&self) -> u64 {
        self.collider1.entity_id
    }
    pub fn collider2_entity_id(&self) -> u64 {
        self.collider2.entity_id
    }

    pub fn from_rapier3d(
        physics: &PhysicsState,
        value: rapier3d::geometry::CollisionEvent,
    ) -> Option<Self> {
        fn resolve(
            physics: &PhysicsState,
            handle: ColliderHandle,
        ) -> Option<hecs::Entity> {
            let label = physics.colliders_entity_map.iter().find_map(|(l, s)| {
                for (_, h) in s {
                    if *h == handle {
                        return Some(l.clone());
                    }
                }
                None
            })?;
            physics
                .entity_label_map
                .iter()
                .find_map(|(e, l)| if l == &label { Some(*e) } else { None })
        }

        match value {
            rapier3d::prelude::CollisionEvent::Started(col1, col2, flags) => {
                let e1 = resolve(physics, col1)?;
                let e2 = resolve(physics, col2)?;
                Some(Self {
                    event_type: CollisionEventType::Started,
                    collider1: NCollider {
                        index: IndexNative::from(col1.0),
                        entity_id: e1.to_bits().get(),
                        id: col1.into_raw_parts().0,
                    },
                    collider2: NCollider {
                        index: IndexNative::from(col2.0),
                        entity_id: e2.to_bits().get(),
                        id: col2.into_raw_parts().0,
                    },
                    flags: flags.bits() as u64,
                })
            }
            rapier3d::prelude::CollisionEvent::Stopped(col1, col2, flags) => {
                let e1 = resolve(physics, col1)?;
                let e2 = resolve(physics, col2)?;
                Some(Self {
                    event_type: CollisionEventType::Stopped,
                    collider1: NCollider {
                        index: IndexNative::from(col1.0),
                        entity_id: e1.to_bits().get(),
                        id: col1.into_raw_parts().0,
                    },
                    collider2: NCollider {
                        index: IndexNative::from(col2.0),
                        entity_id: e2.to_bits().get(),
                        id: col2.into_raw_parts().0,
                    },
                    flags: flags.bits() as u64,
                })
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ContactForceEvent {
    pub collider1: NCollider,
    pub collider2: NCollider,
    pub total_force: NVector3,
    pub total_force_magnitude: f64,
    pub max_force_direction: NVector3,
    pub max_force_magnitude: f64,
}

impl ContactForceEvent {
    pub fn collider1_entity_id(&self) -> u64 {
        self.collider1.entity_id
    }
    pub fn collider2_entity_id(&self) -> u64 {
        self.collider2.entity_id
    }

    pub fn from_rapier3d(
        physics: &PhysicsState,
        event: rapier3d::prelude::ContactForceEvent,
    ) -> Option<Self> {
        let find_entity = |handle: ColliderHandle| -> Option<hecs::Entity> {
            let label = physics.colliders_entity_map.iter().find_map(|(l, s)| {
                for (_, h) in s {
                    if *h == handle {
                        return Some(l.clone());
                    }
                }
                None
            })?;
            physics
                .entity_label_map
                .iter()
                .find_map(|(e, l)| if l == &label { Some(*e) } else { None })
        };

        Some(Self {
            collider1: NCollider {
                index: IndexNative::from(event.collider1.0),
                entity_id: find_entity(event.collider1)?.to_bits().get(),
                id: event.collider1.into_raw_parts().0,
            },
            collider2: NCollider {
                index: IndexNative::from(event.collider2.0),
                entity_id: find_entity(event.collider2)?.to_bits().get(),
                id: event.collider2.into_raw_parts().0,
            },
            total_force: NVector3::new(
                event.total_force.x as f64,
                event.total_force.y as f64,
                event.total_force.z as f64,
            ),
            total_force_magnitude: event.total_force_magnitude as f64,
            max_force_direction: NVector3::new(
                event.max_force_direction.x as f64,
                event.max_force_direction.y as f64,
                event.max_force_direction.z as f64,
            ),
            max_force_magnitude: event.max_force_magnitude as f64,
        })
    }
}
