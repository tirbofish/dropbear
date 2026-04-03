use dropbear_engine::asset::{ASSET_REGISTRY, Handle};
use dropbear_engine::buffer::DynamicBuffer;
use dropbear_engine::graphics::SharedGraphicsContext;
use dropbear_engine::model::{
    AlphaMode, Animation, Material, Mesh, Model, ModelVertex, Node, Skin,
};
use dropbear_engine::texture::{Texture, TextureWrapMode};
use dropbear_engine::utils::ResourceReference;
use dropbear_engine::wgpu;
use dropbear_engine::wgpu::util::DeviceExt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use uuid::Uuid;

use crate::uuid::UuidV4;

/// How a texture is referenced inside a compiled model (`.eucmdl`).
///
/// `AssetUuid` is the canonical form for any texture that lives on disk and has
/// a `.eucmeta` sidecar. `Embedded` is used for textures that were packed
/// directly inside a source file (e.g. GLTF-embedded data) and have not yet
/// been extracted as standalone assets.
#[derive(
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum EucalyptusTextureRef {
    /// UUID of a file-backed texture tracked by a `.eucmeta` sidecar.
    AssetUuid(UuidV4),
    /// Raw image bytes embedded directly in the model file.
    Embedded(Arc<[u8]>),
}

impl EucalyptusTextureRef {
    /// Constructs from a `uuid::Uuid`.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self::AssetUuid(UuidV4::from(uuid))
    }
}

/// The serialized format for a Model without all the buffers and stuff.
///
/// This is stored in the file system as `*.eucmdl`.
#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct EucalyptusModel {
    pub label: String,
    pub meshes: Vec<EucalyptusMesh>, // this needs to be custom type because of wgpu buffers
    pub materials: Vec<EucalyptusMaterial>, // same here
    pub skins: Vec<Skin>,
    pub animations: Vec<Animation>,
    pub nodes: Vec<Node>,
    pub morph_deltas: Vec<f32>,
}

impl EucalyptusModel {
    /// Loads the [`EucalyptusModel`] as a [`Model`] by loading the buffers.
    pub fn load(&self, source: ResourceReference, graphics: Arc<SharedGraphicsContext>) -> Model {
        let materials = self
            .materials
            .iter()
            .map(|material| material.load(graphics.clone()))
            .collect::<Vec<_>>();

        let meshes = self
            .meshes
            .iter()
            .map(|mesh| mesh.load(graphics.clone()))
            .collect::<Vec<_>>();

        let morph_deltas_buffer = if self.morph_deltas.is_empty() {
            None
        } else {
            Some(
                graphics
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("model morph deltas buffer"),
                        contents: bytemuck::cast_slice(&self.morph_deltas),
                        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                    }),
            )
        };

        Model {
            hash: self.runtime_hash(&source),
            label: self.label.clone(),
            path: source,
            meshes,
            materials,
            skins: self.skins.clone(),
            animations: self.animations.clone(),
            nodes: self.nodes.clone(),
            morph_deltas_buffer,
        }
    }

    fn runtime_hash(&self, source: &ResourceReference) -> u64 {
        let mut hasher = DefaultHasher::default();
        source.hash(&mut hasher);
        self.label.hash(&mut hasher);
        self.meshes.len().hash(&mut hasher);
        self.materials.len().hash(&mut hasher);
        self.nodes.len().hash(&mut hasher);

        for mesh in &self.meshes {
            mesh.name.hash(&mut hasher);
            mesh.num_elements.hash(&mut hasher);
            mesh.vertices.len().hash(&mut hasher);
            mesh.material.hash(&mut hasher);
        }

        hasher.finish()
    }
}

impl From<Model> for EucalyptusModel {
    fn from(value: Model) -> Self {
        Self {
            label: value.label.clone(),
            meshes: value.meshes.into_iter().map(EucalyptusMesh::from).collect(),
            materials: value
                .materials
                .into_iter()
                .map(EucalyptusMaterial::from)
                .collect(),
            skins: value.skins,
            animations: value.animations,
            nodes: value.nodes,
            morph_deltas: Vec::new(),
        }
    }
}

#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct EucalyptusMesh {
    pub name: String,
    pub num_elements: u32,
    pub material: usize,
    pub vertices: Vec<ModelVertex>,
    pub morph_deltas_offset: u32,
    pub morph_target_count: u32,
    pub morph_vertex_count: u32,
    pub morph_default_weights: Vec<f32>,
}

impl From<Mesh> for EucalyptusMesh {
    fn from(value: Mesh) -> Self {
        Self {
            name: value.name,
            num_elements: value.num_elements,
            material: value.material,
            vertices: value.vertex_buffer.into_data(),
            morph_deltas_offset: value.morph_deltas_offset,
            morph_target_count: value.morph_target_count,
            morph_vertex_count: value.morph_vertex_count,
            morph_default_weights: value.morph_default_weights,
        }
    }
}

impl EucalyptusMesh {
    fn load(&self, graphics: Arc<SharedGraphicsContext>) -> Mesh {
        let vertex_buffer = DynamicBuffer::from_slice(
            &graphics.device,
            &self.vertices,
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            &format!("{} Vertex Buffer", self.name),
        );

        let index_count = self.num_elements.min(self.vertices.len() as u32);
        let indices = (0..index_count).collect::<Vec<u32>>();
        let index_buffer = DynamicBuffer::from_slice(
            &graphics.device,
            &indices,
            wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            &format!("{} Index Buffer", self.name),
        );

        Mesh {
            name: self.name.clone(),
            vertex_buffer,
            index_buffer,
            num_elements: index_count,
            material: self.material,
            morph_deltas_offset: self.morph_deltas_offset,
            morph_target_count: self.morph_target_count,
            morph_vertex_count: self.morph_vertex_count,
            morph_default_weights: self.morph_default_weights.clone(),
        }
    }
}

#[derive(
    rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug, serde::Serialize, serde::Deserialize,
)]
pub struct EucalyptusMaterial {
    pub name: String,
    pub diffuse_texture: Option<EucalyptusTextureRef>,
    pub normal_texture: Option<EucalyptusTextureRef>,
    pub emissive_texture: Option<EucalyptusTextureRef>,
    pub metallic_roughness_texture: Option<EucalyptusTextureRef>,
    pub occlusion_texture: Option<EucalyptusTextureRef>,
    pub tint: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: Option<f32>,
    pub occlusion_strength: f32,
    pub normal_scale: f32,
    pub uv_tiling: [f32; 2],
    pub texture_tag: Option<String>,
    pub wrap_mode: TextureWrapMode,
}

impl From<Material> for EucalyptusMaterial {
    fn from(value: Material) -> Self {
        let project_root = crate::states::PROJECT.read().project_path.clone();

        let get_texture = |tex: Option<Handle<Texture>>| -> Option<EucalyptusTextureRef> {
            let tex = tex?;
            let registry = ASSET_REGISTRY.read();
            let t = registry.get_texture(tex)?;
            match t.reference.clone()? {
                ResourceReference::File(rel) if !rel.is_empty() => {
                    let abs = project_root.join("resources").join(&rel);
                    crate::metadata::generate_eucmeta(&abs, &project_root)
                        .ok()
                        .map(|entry| EucalyptusTextureRef::from_uuid(entry.uuid))
                }
                ResourceReference::Embedded(bytes) => Some(EucalyptusTextureRef::Embedded(bytes)),
                _ => None,
            }
        };

        Self {
            name: value.name,
            diffuse_texture: get_texture(Some(value.diffuse_texture)),
            normal_texture: get_texture(value.normal_texture),
            emissive_texture: get_texture(value.emissive_texture),
            metallic_roughness_texture: get_texture(value.metallic_roughness_texture),
            occlusion_texture: get_texture(value.occlusion_texture),
            tint: value.base_colour,
            emissive_factor: value.emissive_factor,
            metallic_factor: value.metallic_factor,
            roughness_factor: value.roughness_factor,
            alpha_mode: value.alpha_mode,
            alpha_cutoff: value.alpha_cutoff,
            occlusion_strength: value.occlusion_strength,
            normal_scale: value.normal_scale,
            uv_tiling: value.uv_tiling,
            texture_tag: value.texture_tag,
            wrap_mode: value.wrap_mode,
        }
    }
}

impl EucalyptusMaterial {
    fn load_texture(
        &self,
        graphics: Arc<SharedGraphicsContext>,
        reference: &EucalyptusTextureRef,
        suffix: &str,
    ) -> Option<Handle<Texture>> {
        match reference {
            EucalyptusTextureRef::AssetUuid(uuid_v4) => {
                let uuid = uuid_v4.as_uuid();
                let project_root = crate::states::PROJECT.read().project_path.clone();
                let entry = crate::metadata::find_asset_by_uuid(&project_root, uuid)
                    .map_err(|e| log::warn!("load_texture: UUID {} not found: {}", uuid, e))
                    .ok()?;
                if let crate::resource::ResourceReference::File(rel) = &entry.location {
                    let abs = project_root.join(rel);
                    // Dedup: return cached handle if already loaded.
                    if let Ok(engine_ref) = ResourceReference::from_path(&abs) {
                        let registry = ASSET_REGISTRY.read();
                        if let Some(h) = registry.get_texture_handle_by_reference(&engine_ref) {
                            return Some(h);
                        }
                    }
                    let bytes = std::fs::read(&abs)
                        .map_err(|e| {
                            log::warn!("load_texture: failed to read '{}': {}", abs.display(), e)
                        })
                        .ok()?;
                    let label = format!("{}_{}", self.name, suffix);
                    let engine_ref = ResourceReference::from_path(&abs).ok();
                    let mut texture =
                        dropbear_engine::texture::TextureBuilder::new(&graphics.device)
                            .with_bytes(graphics.clone(), bytes.as_slice())
                            .label(label.as_str())
                            .build();
                    texture.reference = engine_ref;
                    let mut registry = ASSET_REGISTRY.write();
                    Some(registry.add_texture_with_label(entry.name, texture))
                } else {
                    log::warn!("load_texture: UUID {} has no file-backed location", uuid);
                    None
                }
            }
            EucalyptusTextureRef::Embedded(bytes) => {
                let label = format!("{}_{}", self.name, suffix);
                let texture = dropbear_engine::texture::TextureBuilder::new(&graphics.device)
                    .with_bytes(graphics.clone(), bytes)
                    .label(label.as_str())
                    .build();
                let mut registry = ASSET_REGISTRY.write();
                Some(registry.add_texture(texture))
            }
        }
    }

    fn load(&self, graphics: Arc<SharedGraphicsContext>) -> Material {
        let diffuse_texture = {
            let maybe = self
                .diffuse_texture
                .as_ref()
                .and_then(|r| self.load_texture(graphics.clone(), r, "diffuse"));
            if let Some(handle) = maybe {
                handle
            } else {
                ASSET_REGISTRY.write().solid_texture_rgba8(
                    graphics.clone(),
                    [255, 255, 255, 255],
                    Some(Texture::TEXTURE_FORMAT_BASE.add_srgb_suffix()),
                )
            }
        };

        let normal_texture = self
            .normal_texture
            .as_ref()
            .and_then(|reference| self.load_texture(graphics.clone(), reference, "normal"));
        let emissive_texture = self
            .emissive_texture
            .as_ref()
            .and_then(|reference| self.load_texture(graphics.clone(), reference, "emissive"));
        let metallic_roughness_texture =
            self.metallic_roughness_texture
                .as_ref()
                .and_then(|reference| {
                    self.load_texture(graphics.clone(), reference, "metallic_roughness")
                });
        let occlusion_texture = self
            .occlusion_texture
            .as_ref()
            .and_then(|reference| self.load_texture(graphics.clone(), reference, "occlusion"));

        let mut registry = ASSET_REGISTRY.write();
        let mut material = Material::new(
            &mut registry,
            graphics.clone(),
            self.name.clone(),
            diffuse_texture,
            normal_texture,
            emissive_texture,
            metallic_roughness_texture,
            occlusion_texture,
            self.tint,
            self.texture_tag.clone(),
        );

        material.emissive_factor = self.emissive_factor;
        material.metallic_factor = self.metallic_factor;
        material.roughness_factor = self.roughness_factor;
        material.alpha_mode = self.alpha_mode;
        material.alpha_cutoff = self.alpha_cutoff;
        material.occlusion_strength = self.occlusion_strength;
        material.normal_scale = self.normal_scale;
        material.uv_tiling = self.uv_tiling;
        material.wrap_mode = self.wrap_mode;

        material.sync_uniform(&graphics);
        material
    }
}
