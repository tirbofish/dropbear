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

use std::sync::Arc;
use rapier3d::na::Vector3;
use rapier3d::prelude::{ColliderBuilder};
use serde::{Deserialize, Serialize};
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::wgpu::{Buffer, BufferUsages};
use dropbear_engine::wgpu::util::{BufferInitDescriptor, DeviceExt};
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use crate::states::Label;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collider {
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

    // /// Convex hull from a mesh.
    // ConvexHull { mesh_path: String },
    //
    // /// Triangle mesh for static/complex geometry.
    // TriMesh { mesh_path: String },
    //
    // /// Heightfield for terrain.
    // HeightField {
    //     heights: Vec<f32>,
    //     num_rows: usize,
    //     num_cols: usize,
    //     scale: [f32; 3],
    // },
    //
    // /// Compound shape (multiple shapes combined).
    // Compound { shapes: Vec<CompoundShape> },
}

// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
// pub struct CompoundShape {
//     pub shape: ColliderShape,
//     pub translation: [f32; 3],
//     pub rotation: [f32; 3],
// }

impl Collider {
    fn default_density() -> f32 { 1.0 }
    fn default_friction() -> f32 { 0.5 }

    pub fn new() -> Self {
        Self {
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
            // ColliderShape::ConvexHull { .. } => {
            //     todo!("Load mesh from path and create convex hull")
            // }
            // ColliderShape::TriMesh { .. } => {
            //     todo!("Load mesh from path and create trimesh")
            // }
            // ColliderShape::HeightField { heights, num_rows, num_cols, scale } => {
            //     ColliderBuilder::heightfield(
            //         DMatrix::from_vec(*num_rows, *num_cols, heights.clone()),
            //         Vector3::new(scale[0], scale[1], scale[2])
            //     )
            // }
            // ColliderShape::Compound { .. } => {
            //     todo!("Build compound collider")
            // }
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

impl Default for ColliderShape {
    fn default() -> Self {
        ColliderShape::Box {
            half_extents: [0.5, 0.5, 0.5]
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