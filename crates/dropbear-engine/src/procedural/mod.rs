//! Starter objects like planes and primitive objects are that made during runtime with
//! vertices rather than from a model.

use crate::asset::{AssetRegistry, Handle};
use crate::graphics::SharedGraphicsContext;
use crate::model::{Material, Mesh, Model};
use crate::texture::Texture;
use crate::utils::ResourceReference;
use crate::model::ModelVertex;
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;

pub mod cube;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize
)]
pub enum ProcObjType {
    Cuboid,
}

/// An object that comes with a template, and is generated through parameter input. 
#[derive(
    Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize
)]
pub struct ProcedurallyGeneratedObject {
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub ty: ProcObjType,
}

impl ProcedurallyGeneratedObject {
    /// Constructs a [`Model`] and returns the model itself instead of adding to the registry. 
    pub fn construct(
        &self,
        graphics: Arc<SharedGraphicsContext>,
        material: Option<Material>,
        label: Option<&str>,
        hash: Option<u64>,
        registry: Arc<RwLock<AssetRegistry>>,
    ) -> Model {
        let mut _rguard = registry.write();

        let hash = if let Some(hash) = hash {
            hash
        } else {
            let mut hasher = DefaultHasher::new();
            hasher.write(bytemuck::cast_slice(&self.vertices));
            hasher.write(bytemuck::cast_slice(&self.indices));
            hasher.finish()
        };

        let label = label
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("procedural_{hash:016x}"));

        let vertices = self.vertices.clone();
        let indices = self.indices.clone();

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
            let flat_normal_handle = _rguard.solid_texture_rgba8_with_format(
                graphics.clone(),
                [128, 128, 255, 255],
                Texture::TEXTURE_FORMAT_BASE,
            );
            let white_srgb_handle = _rguard.solid_texture_rgba8_with_format(
                graphics.clone(),
                [255, 255, 255, 255],
                Texture::TEXTURE_FORMAT_BASE.add_srgb_suffix(),
            );
            let white_linear_handle = _rguard.solid_texture_rgba8_with_format(
                graphics.clone(),
                [255, 255, 255, 255],
                Texture::TEXTURE_FORMAT_BASE,
            );
            let flat_normal = _rguard
                .get_texture(flat_normal_handle)
                .expect("Flat normal texture handle missing")
                .clone();
            let white_srgb = _rguard
                .get_texture(white_srgb_handle)
                .expect("White SRGB texture handle missing")
                .clone();
            let white_linear = _rguard
                .get_texture(white_linear_handle)
                .expect("White linear texture handle missing")
                .clone();
            Material::new(
                graphics.clone(),
                "procedural_material",
                white_srgb.clone(),
                flat_normal,
                None,
                None,
                None,
                white_srgb,
                white_linear.clone(),
                white_linear,
                false,
                [1.0, 1.0, 1.0, 1.0],
                Some("procedural_material".to_string()),
            )
        });

        let model = Model {
            label: label.clone(),
            hash,
            path: ResourceReference::from_reference(crate::utils::ResourceReferenceType::ProcObj(self.clone())),
            meshes: vec![mesh],
            materials: vec![material],
            skins: Vec::new(),
            animations: Vec::new(),
            nodes: Vec::new(),
        };

        model
    }

    /// Builds a GPU-backed model from this procedural mesh.
    ///
    /// - `material`: optional material; when `None`, a cached 1x1 grey texture is used.
    /// - `label`: optional cache label; when `None`, a stable hash-based label is generated.
    pub fn build_model(
        &self,
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

        let label_str = label
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("procedural_{hash:016x}"));

        {
            let mut _rguard = registry.read();

            if let Some(handle) = _rguard.model_handle_by_hash(hash) {
                return handle;
            }
        }

        let model = Self::construct(&self, graphics, material, label, Some(hash), registry.clone());

        {
            let mut _rguard = registry.write();
            _rguard.add_model_with_label(label_str, model)
        }
    }
}