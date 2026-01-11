//! Starter objects like planes and primitive objects are that made during runtime with
//! vertices rather than from a model.

use crate::asset::{AssetRegistry, ASSET_REGISTRY};
use crate::graphics::SharedGraphicsContext;
use crate::model::{LoadedModel, Material, Mesh, Model, ModelId, MODEL_CACHE};
use crate::utils::ResourceReference;
use crate::model::ModelVertex;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::sync::{Arc, LazyLock};
use wgpu::util::DeviceExt;

pub mod cube;
// pub mod plane;

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
    ) -> LoadedModel {
        self.build_model_raw(
            graphics,
            material,
            label,
            &ASSET_REGISTRY,
            LazyLock::force(&MODEL_CACHE),
        )
    }

    pub fn build_model_raw(
        self,
        graphics: Arc<SharedGraphicsContext>,
        material: Option<Material>,
        label: Option<&str>,
        registry: &AssetRegistry,
        cache: &Mutex<HashMap<String, Arc<Model>>>,
    ) -> LoadedModel {
        let mut hasher = DefaultHasher::new();
        hasher.write(bytemuck::cast_slice(&self.vertices));
        hasher.write(bytemuck::cast_slice(&self.indices));
        let hash = hasher.finish();

        let label = label
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("procedural_{hash:016x}"));

        if let Some(cached_model) = {
            let cache_guard = cache.lock();
            cache_guard.get(&label).cloned()
        } {
            return LoadedModel::new_raw(registry, cached_model);
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
            let grey = registry.grey_texture(graphics.clone());
            let flat_normal = registry.solid_texture_rgba8(graphics.clone(), [128, 128, 255, 255]);
            Material::new_with_tint(
                graphics.clone(),
                "procedural_material",
                (*grey).clone(),
                (*flat_normal).clone(),
                [1.0, 1.0, 1.0, 1.0],
                Some("procedural_material".to_string()),
            )
        });

        let model = Arc::new(Model {
            label: label.clone(),
            path: ResourceReference::from_bytes(hash.to_le_bytes()),
            meshes: vec![mesh],
            materials: vec![material],
            id: ModelId(hash),
        });

        {
            let mut cache_guard = cache.lock();
            cache_guard.insert(label, Arc::clone(&model));
        }

        LoadedModel::new_raw(registry, model)
    }
}