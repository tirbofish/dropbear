use crate::asset::AssetRegistry;
use crate::{
    asset::{ASSET_REGISTRY, AssetHandle},
    graphics::{SharedGraphicsContext, Texture},
    utils::{ResourceReference, TextureWrapMode},
};
use image::GenericImageView;
use parking_lot::Mutex;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, LazyLock};
use std::time::Instant;
use std::{mem, ops::Range, path::PathBuf};
use glam::{Vec2, Vec3};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, util::DeviceExt, BindGroup};

pub static MODEL_CACHE: LazyLock<Mutex<HashMap<String, Arc<Model>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub u64);

impl ModelId {
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[derive(Clone)]
pub struct Model {
    pub label: String,
    pub path: ResourceReference,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub id: ModelId,
}

/// A shared, GPU-backed renderable shape.
///
/// This is intentionally a lightweight wrapper around an `Arc<Model>`.
/// It exists so other systems can treat both model-loaded and procedurally
/// generated geometry uniformly, while retaining clone-on-write semantics.
#[derive(Clone)]
pub struct SharedShape {
    model: Arc<Model>,
}

impl SharedShape {
    pub fn new(model: Arc<Model>) -> Self {
        Self { model }
    }

    pub fn get(&self) -> Arc<Model> {
        Arc::clone(&self.model)
    }

    pub fn make_mut(&mut self) -> &mut Model {
        Arc::make_mut(&mut self.model)
    }
}

impl Deref for SharedShape {
    type Target = Model;

    fn deref(&self) -> &Self::Target {
        self.model.as_ref()
    }
}

#[derive(Clone)]
pub struct LoadedModel {
    pub(crate) inner: Arc<SharedShape>,
    handle: AssetHandle,
}

impl LoadedModel {
    pub fn new(inner: Arc<Model>) -> Self {
        Self::new_raw(&ASSET_REGISTRY, inner)
    }

    pub fn new_raw(registry: &AssetRegistry, inner: Arc<Model>) -> Self {
        let reference = inner.path.clone();
        let handle = registry.register_model(reference, Arc::clone(&inner));
        let inner = Arc::new(SharedShape::new(inner));
        Self { inner, handle }
    }

    pub fn from_registered(handle: AssetHandle, inner: Arc<Model>) -> Self {
        let inner = Arc::new(SharedShape::new(inner));
        Self { inner, handle }
    }

    pub fn from_asset_handle_raw(registry: &AssetRegistry, handle: AssetHandle) -> Option<Self> {
        registry
            .get_model(handle)
            .map(|model| Self::from_registered(handle, model))
    }

    pub fn from_asset_handle(handle: AssetHandle) -> Option<Arc<LoadedModel>> {
        Self::from_asset_handle_raw(&ASSET_REGISTRY, handle).map(|model| Arc::new(model))
    }

    /// Returns the unique identifier of the underlying model asset.
    pub fn id(&self) -> ModelId {
        self.inner.id
    }

    /// Returns the asset handle associated with the underlying model.
    pub fn asset_handle(&self) -> AssetHandle {
        self.handle
    }

    pub fn matches_resource(&self, reference: &ResourceReference) -> bool {
        self.inner.matches_resource(reference)
    }

    /// Provides shared access to the underlying model.
    pub fn get(&self) -> Arc<Model> {
        self.inner.get()
    }

    /// Provides mutable access to the underlying model data, cloning if shared.
    pub fn make_mut(&mut self) -> &mut Model {
        let shape = Arc::make_mut(&mut self.inner);
        shape.make_mut()
    }

    /// Re-registers the model with the global asset registry, ensuring cached
    /// sub-assets stay in sync after mutations.
    pub fn refresh_registry(&mut self) {
        self.refresh_registry_raw(&ASSET_REGISTRY);
    }

    pub fn refresh_registry_raw(&mut self, registry: &AssetRegistry) {
        let reference = self.inner.path.clone();
        let updated_handle = registry.register_model(reference, self.get());
        self.handle = updated_handle;
    }

    pub fn contains_material_handle(&self, handle: AssetHandle) -> bool {
        self.contains_material_handle_raw(&ASSET_REGISTRY, handle)
    }

    pub fn contains_material_handle_raw(
        &self,
        registry: &AssetRegistry,
        handle: AssetHandle,
    ) -> bool {
        self.inner.contains_material_handle_raw(registry, handle)
    }

    pub fn contains_material_reference(&self, reference: &ResourceReference) -> bool {
        self.contains_material_reference_raw(&ASSET_REGISTRY, reference)
    }

    pub fn contains_material_reference_raw(
        &self,
        registry: &AssetRegistry,
        reference: &ResourceReference,
    ) -> bool {
        self.inner
            .contains_material_reference_raw(registry, reference)
    }
}

impl std::ops::Deref for LoadedModel {
    type Target = Model;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub normal_texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub tint: [f32; 4],
    pub uv_tiling: [f32; 2],
    pub tint_buffer: wgpu::Buffer,
    pub tint_bind_group: wgpu::BindGroup,
    pub texture_tag: Option<String>,
    pub wrap_mode: TextureWrapMode,
}

impl Material {
    pub fn new(
        graphics: Arc<SharedGraphicsContext>,
        name: impl Into<String>,
        diffuse_texture: Texture,
        normal_texture: Texture,
    ) -> Self {
        Self::new_with_tint(graphics, name, diffuse_texture, normal_texture, [1.0, 1.0, 1.0, 1.0], None)
    }

    pub fn new_with_tint(
        graphics: Arc<SharedGraphicsContext>,
        name: impl Into<String>,
        diffuse_texture: Texture,
        normal_texture: Texture,
        tint: [f32; 4],
        texture_tag: Option<String>,
    ) -> Self {
        let name: String = name.into();

        let uv_tiling = [1.0, 1.0];
        let uniform = MaterialUniform {
            colour: tint,
            uv_tiling,
            _pad: [0.0, 0.0],
        };
        let tint_buffer = graphics.create_uniform(uniform, Some("material_tint_uniform"));
        let tint_bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &graphics.material_tint_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: tint_buffer.as_entire_binding(),
            }],
            label: Some("material_tint_bind_group"),
        });

        let bind_group = Self::create_bind_group(&graphics, &diffuse_texture, &normal_texture, &name);

        Self {
            name,
            diffuse_texture,
            normal_texture,
            bind_group,
            tint,
            uv_tiling,
            tint_buffer,
            tint_bind_group,
            texture_tag,
            wrap_mode: TextureWrapMode::Repeat,
        }
    }

    pub fn create_bind_group(
        graphics: &SharedGraphicsContext,
        diffuse: &Texture,
        normal: &Texture,
        name: &str,
    ) -> BindGroup {
        graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &graphics.texture_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal.sampler),
                },
            ],
            label: Some(name),
        })
    }

    pub fn set_tint(&mut self, graphics: &SharedGraphicsContext, tint: [f32; 4]) {
        self.tint = tint;
        let uniform = MaterialUniform {
            colour: tint,
            uv_tiling: self.uv_tiling,
            _pad: [0.0, 0.0],
        };
        graphics
            .queue
            .write_buffer(&self.tint_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn set_uv_tiling(&mut self, graphics: &SharedGraphicsContext, tiling: [f32; 2]) {
        self.uv_tiling = tiling;
        let uniform = MaterialUniform {
            colour: self.tint,
            uv_tiling: tiling,
            _pad: [0.0, 0.0],
        };
        graphics
            .queue
            .write_buffer(&self.tint_buffer, 0, bytemuck::bytes_of(&uniform));
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

impl Model {
    /// Replaces the material textures (diffuse + normal) for the material identified by `material_name`.
    ///
    /// Note: `bind_group` must match the provided textures.
    pub fn set_material_texture(
        &mut self,
        material_name: &str,
        diffuse_texture: Texture,
        normal_texture: Texture,
        bind_group: wgpu::BindGroup,
        texture_tag: Option<String>,
    ) -> bool {
        if let Some(material) = self
            .materials
            .iter_mut()
            .find(|mat| mat.name == material_name)
        {
            material.diffuse_texture = diffuse_texture;
            material.normal_texture = normal_texture;
            material.bind_group = bind_group;
            material.texture_tag = texture_tag;
            true
        } else {
            false
        }
    }

    /// Removes any stored texture tag for the supplied material.
    pub fn clear_material_texture_tag(&mut self, material_name: &str) -> bool {
        if let Some(material) = self
            .materials
            .iter_mut()
            .find(|mat| mat.name == material_name)
        {
            material.texture_tag = None;
            true
        } else {
            false
        }
    }

    /// Returns `true` if a material with `material_name` exists within this model.
    pub fn contains_material(&self, material_name: &str) -> bool {
        self.materials.iter().any(|mat| mat.name == material_name)
    }

    /// Returns the registered asset handle for this model, if available.
    pub fn asset_handle(&self) -> Option<AssetHandle> {
        self.asset_handle_raw(&ASSET_REGISTRY)
    }

    pub fn asset_handle_raw(&self, registry: &AssetRegistry) -> Option<AssetHandle> {
        registry.model_handle_from_reference(&self.path)
    }

    /// Returns `true` if this model was loaded from the specified resource reference.
    pub fn matches_resource(&self, reference: &ResourceReference) -> bool {
        &self.path == reference
    }

    /// Returns `true` if this model owns the supplied material handle.
    pub fn contains_material_handle(&self, material_handle: AssetHandle) -> bool {
        self.contains_material_handle_raw(&ASSET_REGISTRY, material_handle)
    }

    pub fn contains_material_handle_raw(
        &self,
        registry: &AssetRegistry,
        material_handle: AssetHandle,
    ) -> bool {
        registry.material_owner(material_handle) == Some(self.id)
    }

    /// Returns `true` if this model owns a material registered under the provided resource reference.
    pub fn contains_material_reference(&self, reference: &ResourceReference) -> bool {
        self.contains_material_reference_raw(&ASSET_REGISTRY, reference)
    }

    pub fn contains_material_reference_raw(
        &self,
        registry: &AssetRegistry,
        reference: &ResourceReference,
    ) -> bool {
        registry
            .material_handle_from_reference(reference)
            .map_or(false, |handle| {
                self.contains_material_handle_raw(registry, handle)
            })
    }

    /// Returns `true` if any material on this model is tagged with `texture_tag`.
    pub fn contains_texture_tag(&self, texture_tag: &str) -> bool {
        self.materials
            .iter()
            .any(|mat| mat.texture_tag.as_deref() == Some(texture_tag))
    }

    /// Returns `true` if the specified material currently carries `texture_tag`.
    pub fn material_has_texture_tag(&self, material_name: &str, texture_tag: &str) -> bool {
        self.materials
            .iter()
            .find(|mat| mat.name == material_name)
            .and_then(|mat| mat.texture_tag.as_deref())
            == Some(texture_tag)
    }

    pub async fn load_from_memory<B>(
        graphics: Arc<SharedGraphicsContext>,
        buffer: B,
        label: Option<&str>,
    ) -> anyhow::Result<LoadedModel>
    where
        B: AsRef<[u8]>,
    {
        Self::load_from_memory_raw(
            graphics,
            buffer,
            label,
            &ASSET_REGISTRY,
            LazyLock::force(&MODEL_CACHE),
        )
        .await
    }

    pub async fn load_from_memory_raw<B>(
        graphics: Arc<SharedGraphicsContext>,
        buffer: B,
        label: Option<&str>,
        registry: &AssetRegistry,
        cache: &Mutex<HashMap<String, Arc<Model>>>,
    ) -> anyhow::Result<LoadedModel>
    where
        B: AsRef<[u8]>,
    {
        let start = Instant::now();
        let mut hasher = DefaultHasher::new();

        let cache_key = label.unwrap_or("default").to_string();

        if let Some(cached_model) = {
            let cache_guard = cache.lock();
            cache_guard.get(&cache_key).cloned()
        } {
            log::debug!("Model loaded from memory cache: {:?}", cache_key);
            return Ok(LoadedModel::new_raw(registry, cached_model));
        }

        log::trace!(
            "========== Benchmarking speed of loading {:?} ==========",
            label
        );
        log::debug!("Loading from memory");
        let res_ref = ResourceReference::from_bytes(buffer.as_ref());

        let (gltf, buffers, _images) = gltf::import_slice(buffer.as_ref())?;
        let mut meshes = Vec::new();

        let mut texture_data: Vec<(String, Option<Vec<u8>>, Option<Vec<u8>>, [f32; 4])> = Vec::new();
        for material in gltf.materials() {
            log::debug!("Processing material: {:?}", material.name());
            let material_name = material.name().unwrap_or("Unnamed Material").to_string();

            let tint = material
                .pbr_metallic_roughness()
                .base_color_factor();

            let tint = [tint[0], tint[1], tint[2], tint[3]];

            let diffuse_bytes = if let Some(pbr) = material.pbr_metallic_roughness().base_color_texture() {
                let texture_info = pbr.texture();
                let image = texture_info.source();
                match image.source() {
                    gltf::image::Source::View { view, mime_type: _ } => {
                        let buffer_data = &buffers[view.buffer().index()];
                        let start = view.offset();
                        let end = start + view.length();
                        Some(buffer_data[start..end].to_vec())
                    }
                    gltf::image::Source::Uri { uri, mime_type: _ } => {
                        log::warn!("External URI textures not supported: {}", uri);
                        None
                    }
                }
            } else {
                None
            };

            let normal_bytes = if let Some(info) = material.normal_texture() {
                let image = info.texture().source();
                match image.source() {
                    gltf::image::Source::View { view, mime_type: _ } => {
                        let buffer_data = &buffers[view.buffer().index()];
                        let start = view.offset();
                        let end = start + view.length();
                        Some(buffer_data[start..end].to_vec())
                    }
                    gltf::image::Source::Uri { uri, mime_type: _ } => {
                        log::warn!("External URI textures not supported: {}", uri);
                        None
                    }
                }
            } else {
                None
            };

            texture_data.push((material_name, diffuse_bytes, normal_bytes, tint));
        }

        if texture_data.is_empty() {
            texture_data.push((
                "Default".to_string(),
                None,
                None,
                [1.0, 1.0, 1.0, 1.0],
            ));
        }

        let parallel_start = Instant::now();
        let processed_textures: Vec<_> = texture_data
            .into_par_iter()
            .map(|(material_name, diffuse_bytes, normal_bytes, tint)| {
                let material_start = Instant::now();

                let processed_diffuse = diffuse_bytes.as_ref().map(|bytes| {
                    let load_start = Instant::now();
                    let image = image::load_from_memory(bytes).unwrap();
                    log::trace!("Loading diffuse image to memory: {:?}", load_start.elapsed());

                    let rgba_start = Instant::now();
                    let rgba = image.to_rgba8();
                    log::trace!(
                        "Converting diffuse image to rgba8 took {:?}",
                        rgba_start.elapsed()
                    );

                    let dimensions = image.dimensions();
                    (rgba.into_raw(), dimensions)
                });

                let processed_normal = normal_bytes.as_ref().map(|bytes| {
                    let load_start = Instant::now();
                    let image = image::load_from_memory(bytes).unwrap();
                    log::trace!("Loading normal image to memory: {:?}", load_start.elapsed());

                    let rgba_start = Instant::now();
                    let rgba = image.to_rgba8();
                    log::trace!(
                        "Converting normal image to rgba8 took {:?}",
                        rgba_start.elapsed()
                    );

                    let dimensions = image.dimensions();
                    (rgba.into_raw(), dimensions)
                });

                log::trace!(
                    "Parallel processing of material '{}' took: {:?}",
                    material_name,
                    material_start.elapsed()
                );

                (material_name, processed_diffuse, processed_normal, tint)
            })
            .collect();

        log::trace!(
            "Total parallel image processing took: {:?}",
            parallel_start.elapsed()
        );

        let mut materials = Vec::new();

        let grey_texture = registry.grey_texture(graphics.clone());
        let flat_normal_texture =
            registry.solid_texture_rgba8(graphics.clone(), [128, 128, 255, 255]);

        for (material_name, processed_diffuse, processed_normal, tint) in processed_textures {
            let start = Instant::now();

            let diffuse_texture = if let Some((rgba_data, dimensions)) = processed_diffuse {
                Texture::from_rgba_buffer(graphics.clone(), &rgba_data, dimensions)
            } else {
                (*grey_texture).clone()
            };

            let normal_texture = if let Some((rgba_data, dimensions)) = processed_normal {
                Texture::from_rgba_buffer(graphics.clone(), &rgba_data, dimensions)
            } else {
                (*flat_normal_texture).clone()
            };
            let texture_tag = Some(material_name.clone());

            materials.push(Material::new_with_tint(
                graphics.clone(),
                material_name,
                diffuse_texture,
                normal_texture,
                tint,
                texture_tag,
            ));

            log::trace!("Time to create GPU texture: {:?}", start.elapsed());
        }

        for mesh in gltf.meshes() {
            log::debug!("Processing mesh: {:?}", mesh.name());
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .ok_or_else(|| anyhow::anyhow!("Mesh missing positions"))?
                    .collect();

                let normals: Vec<[f32; 3]> = reader
                    .read_normals()
                    .map(|iter| iter.collect())
                    .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

                let tex_coords: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|iter| iter.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                let mut vertices: Vec<ModelVertex> = positions
                    .iter()
                    .zip(normals.iter())
                    .zip(tex_coords.iter())
                    .map(|((pos, norm), tex)| ModelVertex {
                        position: *pos,
                        normal: *norm,
                        tex_coords: *tex,
                        tangent: [0.0; 3],
                        bitangent: [0.0; 3],
                    })
                    .collect();
                for v in &vertices {
                    let _ = v.position.iter().map(|v| (*v as i32).hash(&mut hasher));
                    let _ = v.normal.iter().map(|v| (*v as i32).hash(&mut hasher));
                    let _ = v.tex_coords.iter().map(|v| (*v as i32).hash(&mut hasher));
                }

                let indices: Vec<u32> = reader
                    .read_indices()
                    .ok_or_else(|| anyhow::anyhow!("Mesh missing indices"))?
                    .into_u32()
                    .collect();
                indices.hash(&mut hasher);

                let mut triangles_included = vec![0; vertices.len()];
                for c in indices.chunks(3) {
                    let v0 = vertices[c[0] as usize];
                    let v1 = vertices[c[1] as usize];
                    let v2 = vertices[c[2] as usize];

                    let pos0: Vec3 = v0.position.into();
                    let pos1: Vec3 = v1.position.into();
                    let pos2: Vec3 = v2.position.into();

                    let uv0: Vec2 = v0.tex_coords.into();
                    let uv1: Vec2 = v1.tex_coords.into();
                    let uv2: Vec2 = v2.tex_coords.into();

                    // Calculate the edges of the triangle
                    let delta_pos1 = pos1 - pos0;
                    let delta_pos2 = pos2 - pos0;

                    // This will give us a direction to calculate the
                    // tangent and bitangent
                    let delta_uv1 = uv1 - uv0;
                    let delta_uv2 = uv2 - uv0;

                    // Solving the following system of equations will
                    // give us the tangent and bitangent.
                    //     delta_pos1 = delta_uv1.x * T + delta_u.y * B
                    //     delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
                    // Luckily, the place I found this equation provided
                    // the solution!
                    let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                    let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
                    // We flip the bitangent to enable right-handed normal
                    // maps with wgpu texture coordinate system
                    let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                    // We'll use the same tangent/bitangent for each vertex in the triangle
                    vertices[c[0] as usize].tangent =
                        (tangent + Vec3::from(vertices[c[0] as usize].tangent)).into();
                    vertices[c[1] as usize].tangent =
                        (tangent + Vec3::from(vertices[c[1] as usize].tangent)).into();
                    vertices[c[2] as usize].tangent =
                        (tangent + Vec3::from(vertices[c[2] as usize].tangent)).into();
                    vertices[c[0] as usize].bitangent =
                        (bitangent + Vec3::from(vertices[c[0] as usize].bitangent)).into();
                    vertices[c[1] as usize].bitangent =
                        (bitangent + Vec3::from(vertices[c[1] as usize].bitangent)).into();
                    vertices[c[2] as usize].bitangent =
                        (bitangent + Vec3::from(vertices[c[2] as usize].bitangent)).into();

                    // Used to average the tangents/bitangents
                    triangles_included[c[0] as usize] += 1;
                    triangles_included[c[1] as usize] += 1;
                    triangles_included[c[2] as usize] += 1;
                }

                for (i, n) in triangles_included.into_iter().enumerate() {
                    let denom = 1.0 / n as f32;
                    let v = &mut vertices[i];
                    v.tangent = (Vec3::from(v.tangent) * denom).into();
                    v.bitangent = (Vec3::from(v.bitangent) * denom).into();
                }

                let vertex_buffer =
                    graphics
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Vertex Buffer", label)),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                let index_buffer =
                    graphics
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Index Buffer", label)),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                let material_index = primitive.material().index().unwrap_or(0);

                meshes.push(Mesh {
                    name: mesh.name().unwrap_or("Unnamed Mesh").to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: material_index,
                });
            }
        }

        log::debug!("Successfully loaded model [{:?}]", label);

        let model = Arc::new(Model {
            meshes,
            materials,
            label: label.unwrap_or("No named model").to_string(),
            path: res_ref,
            id: ModelId(hasher.finish()),
        });

        let loaded = LoadedModel::new_raw(registry, Arc::clone(&model));

        {
            let mut cache_guard = cache.lock();
            cache_guard.insert(cache_key.clone(), model);
        }
        log::trace!("==================== DONE ====================");
        log::debug!("Model cached from memory: {:?}", label);
        log::debug!("Took {:?} to load model: {:?}", start.elapsed(), label);
        log::trace!("==============================================");
        Ok(loaded)
    }

    pub async fn load(
        graphics: Arc<SharedGraphicsContext>,
        path: &PathBuf,
        label: Option<&str>,
    ) -> anyhow::Result<LoadedModel> {
        Self::load_raw(
            graphics,
            path,
            label,
            &ASSET_REGISTRY,
            LazyLock::force(&MODEL_CACHE),
        )
        .await
    }

    pub async fn load_raw(
        graphics: Arc<SharedGraphicsContext>,
        path: &PathBuf,
        label: Option<&str>,
        registry: &AssetRegistry,
        cache: &Mutex<HashMap<String, Arc<Model>>>,
    ) -> anyhow::Result<LoadedModel> {
        let file_name = path.file_name();
        log::debug!("Loading model [{:?}]", file_name);

        let path_str = path.to_string_lossy().to_string();

        log::debug!("Checking if model exists in cache");
        if let Some(cached_model) = {
            let cache_guard = cache.lock();
            cache_guard.get(&path_str).cloned()
        } {
            log::debug!("Model loaded from cache: {:?}", path_str);
            return Ok(LoadedModel::new_raw(registry, cached_model));
        }
        log::debug!("Model does not exist in cache, loading memory...");

        log::debug!("Path of model: {}", path.display());

        let buffer = std::fs::read(path)?;
        let loaded = Self::load_from_memory_raw(graphics, buffer, label, registry, cache).await?;

        let mut model_clone: Model = (*loaded).clone();
        if let Ok(reference) = ResourceReference::from_path(path) {
            model_clone.path = reference;
        }
        if let Some(custom_label) = label {
            model_clone.label = custom_label.to_string();
        }

        let updated = Arc::new(model_clone);
        {
            let mut cache_guard = cache.lock();
            cache_guard.insert(path_str.clone(), Arc::clone(&updated));
            if let Some(custom_label) = label {
                cache_guard.insert(custom_label.to_string(), Arc::clone(&updated));
            }
        }

        log::debug!("Model cached and loaded: {:?}", file_name);
        Ok(LoadedModel::new_raw(registry, updated))
    }

}

pub trait DrawModel<'a> {
    #[allow(unused)]
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    #[allow(unused)]
    fn draw_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        self.set_bind_group(3, &material.tint_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'b Model,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.draw_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(
                mesh,
                material,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
            );
        }
    }
}

pub trait DrawLight<'a> {
    #[allow(unused)]
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );

    #[allow(unused)]
    fn draw_light_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
    fn draw_light_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawLight<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_light_mesh(
        &mut self,
        mesh: &'a Mesh,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.draw_light_mesh_instanced(mesh, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, camera_bind_group, &[]);
        self.set_bind_group(1, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_light_model(
        &mut self,
        model: &'a Model,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        self.draw_light_model_instanced(model, 0..1, camera_bind_group, light_bind_group);
    }

    fn draw_light_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_light_mesh_instanced(
                mesh,
                instances.clone(),
                camera_bind_group,
                light_bind_group,
            );
        }
    }
}

pub trait DrawShadow<'a> {
    fn draw_shadow_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        instances: Range<u32>,
        light_bind_group: &'a wgpu::BindGroup,
    );

    fn draw_shadow_model_instanced(
        &mut self,
        model: &'a Model,
        instances: Range<u32>,
        light_bind_group: &'a wgpu::BindGroup,
    );
}

impl<'a, 'b> DrawShadow<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_shadow_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        instances: Range<u32>,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, light_bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_shadow_model_instanced(
        &mut self,
        model: &'b Model,
        instances: Range<u32>,
        light_bind_group: &'b wgpu::BindGroup,
    ) {
        for mesh in &model.meshes {
            self.draw_shadow_mesh_instanced(mesh, instances.clone(), light_bind_group);
        }
    }
}

pub trait Vertex {
    fn desc() -> VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    /// RGBA tint multiplier applied to the sampled base colour.
    pub colour: [f32; 4],

    /// Scales incoming UVs before sampling (repeat counts when using Repeat wrap mode).
    pub uv_tiling: [f32; 2],

    pub _pad: [f32; 2],
}

impl MaterialUniform {
    pub fn new(colour: [f32; 4]) -> Self {
        Self {
            colour,
            uv_tiling: [1.0, 1.0],
            _pad: [0.0, 0.0],
        }
    }
}