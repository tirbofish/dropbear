//! Starter objects like planes and primitive objects are that made during runtime with
//! vertices rather than from a model.

use crate::asset::{AssetRegistry, Handle};
use crate::graphics::SharedGraphicsContext;
use crate::model::{Material, Mesh, Model};
use crate::utils::ResourceReference;
use crate::model::ModelVertex;
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

pub mod cube;
// pub mod plane;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize
)]
pub enum ProcObj {
    /// A parameterized cuboid (box) generated at runtime.
    ///
    /// Stored as IEEE-754 `f32` bit patterns so the reference remains hashable.
    /// Values can be reconstructed with `f32::from_bits`.
    ///
    /// The `size_bits` represent the full extents (width, height, depth).
    Cuboid { size_bits: [u32; 3] },
}

/// An object that comes with a template, and is generated through parameter input. 
pub struct ProcedurallyGeneratedObject {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

impl ProcedurallyGeneratedObject {
    /// Builds a GPU-backed model from this procedural mesh.
    ///
    /// - `material`: optional material; when `None`, a cached 1x1 grey texture is used.
    /// - `label`: optional cache label; when `None`, a stable hash-based label is generated.
    pub fn build_model(
        self,
        graphics: Arc<SharedGraphicsContext>,
        material: Option<Material>,
        label: Option<&str>,
        registry: Arc<RwLock<AssetRegistry>>,
    ) -> Handle<Model> {
        puffin::profile_function!();
        let mut hasher = DefaultHasher::new();
        hasher.write(bytemuck::cast_slice(&self.vertices));
        hasher.write(bytemuck::cast_slice(&self.indices));
        let hash = hasher.finish();

        let label = label
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("procedural_{hash:016x}"));

        let mut _rguard = registry.write();

        if let Some(handle) = _rguard.model_handle_by_hash(hash) {
            return handle;
        }

        let vertices = self.vertices;
        let indices = self.indices;

        let vertex_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{label} Vertex Buffer")),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = graphics
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{label} Index Buffer")),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let mesh = Mesh {
            name: label.clone(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
            vertices,
        };

        let material = material.unwrap_or_else(|| {
            let grey_handle = _rguard.grey_texture(graphics.clone());
            let flat_normal_handle =
                _rguard.solid_texture_rgba8(graphics.clone(), [128, 128, 255, 255]);
            let grey = _rguard
                .get_texture(grey_handle)
                .expect("Grey texture handle missing")
                .clone();
            let flat_normal = _rguard
                .get_texture(flat_normal_handle)
                .expect("Flat normal texture handle missing")
                .clone();
            Material::new(
                graphics.clone(),
                "procedural_material",
                grey,
                flat_normal,
                [1.0, 1.0, 1.0, 1.0],
                Some("procedural_material".to_string()),
            )
        });

        let model = Model {
            label: label.clone(),
            hash,
            path: ResourceReference::from_bytes(hash.to_le_bytes()),
            meshes: vec![mesh],
            materials: vec![material],
            skins: Vec::new(),
            animations: Vec::new(),
            nodes: Vec::new(),
        };

        _rguard.add_model_with_label(label, model)
    }
}