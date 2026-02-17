use crate::asset::{AssetRegistry, Handle};
use crate::buffer::UniformBuffer;
use crate::{
    graphics::{SharedGraphicsContext},
    utils::{ResourceReference},
    texture::{Texture, TextureWrapMode}
};
// use image::GenericImageView;
use parking_lot::{RwLock};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc};
use std::{mem, ops::Range};
use gltf::image::Format;
use gltf::texture::MinFilter;
use puffin::profile_scope;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, util::DeviceExt, BindGroup};

#[derive(Clone)]
pub struct Model {
    pub hash: u64, // also the id related to the handle
    pub label: String,
    pub path: ResourceReference,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub skins: Vec<Skin>,
    pub animations: Vec<Animation>,
    pub nodes: Vec<Node>,
}

#[derive(Clone)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
    pub vertices: Vec<ModelVertex>,
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    pub normal_texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub tint: [f32; 4],
    pub emissive_factor: [f32; 3],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: bool,
    pub occlusion_strength: f32,
    pub normal_scale: f32,
    pub uv_tiling: [f32; 2],
    pub tint_buffer: UniformBuffer<MaterialUniform>,
    pub texture_tag: Option<String>,
    pub wrap_mode: TextureWrapMode,
    pub emissive_texture: Option<Texture>,
    pub metallic_roughness_texture: Option<Texture>,
    pub occlusion_texture: Option<Texture>,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum AlphaMode {
    #[default]
    Opaque = 1,
    Mask,
    Blend,
}

impl Into<AlphaMode> for gltf::material::AlphaMode {
    fn into(self) -> AlphaMode {
        match self {
            gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
            gltf::material::AlphaMode::Mask => AlphaMode::Mask,
            gltf::material::AlphaMode::Blend => AlphaMode::Blend,
        }
    }
}

/// Represents a node in the scene graph (can be a joint/bone or a mesh)
#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub transform: NodeTransform,
}

/// Local transform of a node relative to its parent
#[derive(Clone, Debug)]
pub struct NodeTransform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl NodeTransform {
    pub fn to_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn identity() -> Self {
        Self {
            translation: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

/// A skin defines how a mesh is bound to a skeleton
#[derive(Clone)]
pub struct Skin {
    pub name: String,
    /// Indices of joints (nodes) in the Model's nodes array
    pub joints: Vec<usize>,
    /// Inverse bind matrices - one per joint
    pub inverse_bind_matrices: Vec<glam::Mat4>,
    /// Optional root joint index
    pub skeleton_root: Option<usize>,
}

/// An animation that can be played on a skeleton
#[derive(Clone)]
pub struct Animation {
    pub name: String,
    pub channels: Vec<AnimationChannel>,
    pub duration: f32,
}

/// Describes how an animation affects a specific node
#[derive(Clone)]
pub struct AnimationChannel {
    /// Target node index in the Model's nodes array
    pub target_node: usize,
    /// Keyframe times
    pub times: Vec<f32>,
    /// Animation data
    pub values: ChannelValues,
    /// Interpolation method
    pub interpolation: AnimationInterpolation,
}

#[derive(Clone)]
pub enum ChannelValues {
    Translations(Vec<glam::Vec3>),
    Rotations(Vec<glam::Quat>),
    Scales(Vec<glam::Vec3>),
}

impl Material {
    pub fn new(
        graphics: Arc<SharedGraphicsContext>,
        name: impl Into<String>,
        diffuse_texture: Texture,
        normal_texture: Texture,
        tint: [f32; 4],
        texture_tag: Option<String>,
    ) -> Self {
        puffin::profile_function!();
        let name = name.into();

        let uv_tiling = [1.0, 1.0];
        let uniform = MaterialUniform {
            base_colour: tint,
            emissive: [0.0, 0.0, 0.0],
            emissive_strength: 1.0,
            metallic: 1.0,
            roughness: 1.0,
            normal_scale: 1.0,
            occlusion_strength: 1.0,
            alpha_cutoff: 0.5,
            uv_tiling,
            _pad: 0.0,
        };

        let tint_buffer = UniformBuffer::new(&graphics.device, "material_tint_uniform");
        tint_buffer.write(&graphics.queue, &uniform);
        
        let bind_group = Self::create_bind_group(&graphics, &diffuse_texture, &normal_texture, &tint_buffer, &name);

        Self {
            name,
            diffuse_texture,
            normal_texture,
            bind_group,
            tint,
            emissive_factor: [0.0, 0.0, 0.0],
            metallic_factor: 1.0,
            roughness_factor: 1.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: None,
            double_sided: false,
            occlusion_strength: 1.0,
            normal_scale: 1.0,
            uv_tiling,
            tint_buffer,
            texture_tag,
            wrap_mode: TextureWrapMode::Repeat,
            emissive_texture: None,
            metallic_roughness_texture: None,
            occlusion_texture: None,
        }
    }

    pub fn create_bind_group(
        graphics: &SharedGraphicsContext,
        diffuse: &Texture,
        normal: &Texture,
        uniform_buffer: &UniformBuffer<MaterialUniform>,
        name: &str,
    ) -> BindGroup {
        puffin::profile_function!();
        graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("{} texture bind group", name).as_str()),
            layout: &graphics.layouts.material_bind_layout,
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
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: uniform_buffer.buffer().as_entire_binding(),
                },
            ],
        })
    }

    pub fn sync_uniform(&self, graphics: &SharedGraphicsContext) {
        let uniform = MaterialUniform {
            base_colour: self.tint,
            emissive: self.emissive_factor,
            emissive_strength: 1.0,
            metallic: self.metallic_factor,
            roughness: self.roughness_factor,
            normal_scale: self.normal_scale,
            occlusion_strength: self.occlusion_strength,
            alpha_cutoff: self.alpha_cutoff.unwrap_or(0.5),
            uv_tiling: self.uv_tiling,
            _pad: 0.0,
        };

        self.tint_buffer.write(&graphics.queue, &uniform);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationInterpolation {
    /// The animated values are linearly interpolated between keyframes
    Linear,
    /// The animated values remain constant between keyframes
    Step,
    /// The animated values are interpolated using a cubic spline
    CubicSpline,
}

struct GLTFTextureInformation {
    sampler: wgpu::SamplerDescriptor<'static>,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    #[allow(dead_code)]
    mip_level_count: u32,
    #[allow(dead_code)]
    format: wgpu::TextureFormat,
}

impl GLTFTextureInformation {
    fn fetch(tex: &gltf::Texture<'_>, images: &Vec<gltf::image::Data>) -> GLTFTextureInformation {
        let sampler = tex.sampler();

        let mag_filter = match sampler.mag_filter() {
            Some(gltf::texture::MagFilter::Nearest) => wgpu::FilterMode::Nearest,
            _ => wgpu::FilterMode::Linear,
        };

        let (min_filter, mipmap_filter) = match sampler.min_filter() {
            Some(MinFilter::Nearest) => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
            Some(MinFilter::Linear) => (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest),
            Some(MinFilter::NearestMipmapNearest) => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Nearest),
            Some(MinFilter::LinearMipmapNearest) => (wgpu::FilterMode::Linear, wgpu::FilterMode::Nearest),
            Some(MinFilter::NearestMipmapLinear) => (wgpu::FilterMode::Nearest, wgpu::FilterMode::Linear),
            Some(MinFilter::LinearMipmapLinear) => (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear),
            None => (wgpu::FilterMode::Linear, wgpu::FilterMode::Linear),
        };

        fn map_wrap(wrap: gltf::texture::WrappingMode) -> wgpu::AddressMode {
            match wrap {
                gltf::texture::WrappingMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
                gltf::texture::WrappingMode::MirroredRepeat => wgpu::AddressMode::MirrorRepeat,
                gltf::texture::WrappingMode::Repeat => wgpu::AddressMode::Repeat,
            }
        }

        let sampler = wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: map_wrap(sampler.wrap_s()),
            address_mode_v: map_wrap(sampler.wrap_t()),
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter,
            min_filter,
            mipmap_filter,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        };

        let image_index = tex.source().index();
        let image_data = &images[image_index];

        let width = image_data.width;
        let height = image_data.height;

        let mip_level_count = (width.max(height) as f32).log2().floor() as u32 + 1;

        let (pixels, format) = match image_data.format {
            Format::R8 => (image_data.pixels.clone(), wgpu::TextureFormat::R8Unorm),
            Format::R8G8 => (image_data.pixels.clone(), wgpu::TextureFormat::Rg8Unorm),
            Format::R8G8B8 => {
                let mut rgba = Vec::with_capacity(image_data.pixels.len() / 3 * 4);
                for chunk in image_data.pixels.chunks(3) {
                    rgba.extend_from_slice(chunk);
                    rgba.push(255);
                }
                (rgba, wgpu::TextureFormat::Rgba8Unorm)
            },
            Format::R8G8B8A8 => (image_data.pixels.clone(), wgpu::TextureFormat::Rgba8Unorm),
            _ => panic!("Unsupported format"),
        };

        GLTFTextureInformation {
            sampler,
            mip_level_count,
            pixels,
            format,
            width,
            height,
        }
    }
}

struct GLTFMeshInformation {
    name: String,
    primitive_index: usize,
    material_index: usize,
    mode: gltf::mesh::Mode,
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
    normals: Vec<[f32; 3]>,
    tangents: Vec<[f32; 4]>,
    colors: Vec<[f32; 4]>,
    joints: Vec<[u16; 4]>,
    weights: Vec<[f32; 4]>,
    tex_coords0: Vec<[f32; 2]>,
    tex_coords1: Vec<[f32; 2]>,
}

struct GLTFMaterialInformation {
    name: String,
    diffuse_texture: Option<GLTFTextureInformation>,
    normal_texture: Option<GLTFTextureInformation>,
    emissive_texture: Option<GLTFTextureInformation>,
    metallic_roughness_texture: Option<GLTFTextureInformation>,
    occlusion_texture: Option<GLTFTextureInformation>,
    tint: [f32; 4],
    emissive_factor: [f32; 3],
    metallic_factor: f32,
    roughness_factor: f32,
    alpha_mode: gltf::material::AlphaMode,
    alpha_cutoff: Option<f32>,
    double_sided: bool,
    occlusion_strength: f32,
    normal_scale: f32,
}

struct ProcessedMaterialTextures {
    name: String,
    diffuse: Option<(Vec<u8>, (u32, u32))>,
    normal: Option<(Vec<u8>, (u32, u32))>,
    emissive: Option<(Vec<u8>, (u32, u32))>,
    metallic_roughness: Option<(Vec<u8>, (u32, u32))>,
    occlusion: Option<(Vec<u8>, (u32, u32))>,
    diffuse_sampler: Option<wgpu::SamplerDescriptor<'static>>,
    normal_sampler: Option<wgpu::SamplerDescriptor<'static>>,
    emissive_sampler: Option<wgpu::SamplerDescriptor<'static>>,
    metallic_roughness_sampler: Option<wgpu::SamplerDescriptor<'static>>,
    occlusion_sampler: Option<wgpu::SamplerDescriptor<'static>>,
    tint: [f32; 4],
    emissive_factor: [f32; 3],
    metallic_factor: f32,
    roughness_factor: f32,
    alpha_mode: gltf::material::AlphaMode,
    alpha_cutoff: Option<f32>,
    double_sided: bool,
    occlusion_strength: f32,
    normal_scale: f32,
}

impl Model {
    fn load_materials(
        gltf: &gltf::Document,
        _buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) -> Vec<GLTFMaterialInformation> {
        puffin::profile_function!();
        let process_texture = |texture: gltf::Texture<'_>| -> Option<GLTFTextureInformation> {
            puffin::profile_scope!("reading texture bytes", texture.name().unwrap_or("Unnamed Texture"));
            Some(GLTFTextureInformation::fetch(&texture, images))
        };

        let mut material_data = Vec::new();

        for material in gltf.materials() {
            let material_name = material.name().unwrap_or("Unnamed Material").to_string();
            puffin::profile_scope!("loading material", &material_name);

            let tint = material.pbr_metallic_roughness().base_color_factor();
            let tint = [tint[0], tint[1], tint[2], tint[3]];

            let pbr = material.pbr_metallic_roughness();
            let diffuse_texture = pbr.base_color_texture();
            let metallic_roughness_texture = pbr.metallic_roughness_texture();
            let normal_texture = material.normal_texture();
            let occlusion_texture = material.occlusion_texture();
            let emissive_texture = material.emissive_texture();

            let diffuse_texture_info = diffuse_texture
                .as_ref()
                .and_then(|info| process_texture(info.texture()));
            let metallic_roughness_texture_info = metallic_roughness_texture
                .as_ref()
                .and_then(|info| process_texture(info.texture()));

            let normal_texture_info = normal_texture
                .as_ref()
                .and_then(|info| process_texture(info.texture()));
            let occlusion_texture_info = occlusion_texture
                .as_ref()
                .and_then(|info| process_texture(info.texture()));
            let emissive_texture_info = emissive_texture
                .as_ref()
                .and_then(|info| process_texture(info.texture()));

            let emissive_factor = material.emissive_factor();
            let emissive_factor = [
                emissive_factor[0],
                emissive_factor[1],
                emissive_factor[2],
            ];
            let metallic_factor = pbr.metallic_factor();
            let roughness_factor = pbr.roughness_factor();
            let alpha_mode = material.alpha_mode();
            let alpha_cutoff = material.alpha_cutoff();
            let double_sided = material.double_sided();
            let occlusion_strength = occlusion_texture
                .as_ref()
                .map(|info| info.strength())
                .unwrap_or(1.0);
            let normal_scale = normal_texture
                .as_ref()
                .map(|info| info.scale())
                .unwrap_or(1.0);

            material_data.push(GLTFMaterialInformation {
                name: material_name,
                diffuse_texture: diffuse_texture_info,
                normal_texture: normal_texture_info,
                emissive_texture: emissive_texture_info,
                metallic_roughness_texture: metallic_roughness_texture_info,
                occlusion_texture: occlusion_texture_info,
                tint,
                emissive_factor,
                metallic_factor,
                roughness_factor,
                alpha_mode,
                alpha_cutoff,
                double_sided,
                occlusion_strength,
                normal_scale,
            });
        }

        if material_data.is_empty() {
            material_data.push(GLTFMaterialInformation {
                name: "Default".to_string(),
                diffuse_texture: None,
                normal_texture: None,
                emissive_texture: None,
                metallic_roughness_texture: None,
                occlusion_texture: None,
                tint: [1.0, 1.0, 1.0, 1.0],
                emissive_factor: [0.0, 0.0, 0.0],
                metallic_factor: 1.0,
                roughness_factor: 1.0,
                alpha_mode: gltf::material::AlphaMode::Opaque,
                alpha_cutoff: None,
                double_sided: false,
                occlusion_strength: 1.0,
                normal_scale: 1.0,
            });
        }

        material_data
    }

    fn load_meshes(
        mesh: &gltf::Mesh,
        buffers: &Vec<gltf::buffer::Data>,
        mesh_collector: &mut Vec<GLTFMeshInformation>,
    ) -> anyhow::Result<()> {
        let mesh_name = mesh.name().unwrap_or("Unnamed Mesh").to_string();
        puffin::profile_function!(&mesh_name);

        for (primitive_index, primitive) in mesh.primitives().enumerate() {
            puffin::profile_scope!("reading primitive", &format!("{}[{}]", &mesh_name, primitive_index));
            
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| anyhow::anyhow!("Mesh missing positions"))?
                .collect();

            let indices: Vec<u32> = reader
                .read_indices()
                .ok_or_else(|| anyhow::anyhow!("Mesh missing indices"))?
                .into_u32()
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            let tangents: Vec<[f32; 4]> = reader
                .read_tangents()
                .map(|iter| iter.collect())
                .unwrap_or_else(|| vec![[0.0, 0.0, 0.0, 1.0]; positions.len()]);

            let colors: Vec<[f32; 4]> = reader
                .read_colors(0)
                .map(|iter| iter.into_rgba_f32().collect())
                .unwrap_or_else(|| vec![[1.0; 4]; positions.len()]);

            let joints: Vec<[u16; 4]> = reader
                .read_joints(0)
                .map(|iter| iter.into_u16().collect())
                .unwrap_or_else(|| vec![[0u16; 4]; positions.len()]);

            let mut weights: Vec<[f32; 4]> = reader
                .read_weights(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]);

            for weight in &mut weights {
                let sum = weight[0] + weight[1] + weight[2] + weight[3];
                if sum > 0.0 {
                    weight[0] /= sum;
                    weight[1] /= sum;
                    weight[2] /= sum;
                    weight[3] /= sum;
                }
            }

            let tex_coords: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let tex_coords1: Vec<[f32; 2]> = reader
                .read_tex_coords(1)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let expected_len = positions.len();
            let check_len = |label: &str, len: usize| -> anyhow::Result<()> {
                if len == expected_len {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!(
                        "Mesh attribute length mismatch for {}: expected {}, got {}",
                        label,
                        expected_len,
                        len
                    ))
                }
            };

            check_len("normals", normals.len())?;
            check_len("tangents", tangents.len())?;
            check_len("colors", colors.len())?;
            check_len("joints", joints.len())?;
            check_len("weights", weights.len())?;
            check_len("tex_coords0", tex_coords.len())?;
            check_len("tex_coords1", tex_coords1.len())?;

            mesh_collector.push(GLTFMeshInformation {
                name: mesh_name.clone(),
                primitive_index,
                material_index: primitive.material().index().unwrap_or(0),
                mode: primitive.mode(),
                positions,
                indices,
                normals,
                tangents,
                colors,
                joints,
                weights,
                tex_coords0: tex_coords,
                tex_coords1,
            });
        }

        Ok(())
    }

    fn load_nodes(gltf: &gltf::Document) -> Vec<Node> {
        puffin::profile_function!("loading nodes");
        let mut nodes = Vec::new();
        
        for node in gltf.nodes() {
            profile_scope!("reading node", node.name().unwrap_or("Unnamed Node"));
            let (translation, rotation, scale) = node.transform().decomposed();
            
            let transform = NodeTransform {
                translation: glam::Vec3::from(translation),
                rotation: glam::Quat::from_array(rotation),
                scale: glam::Vec3::from(scale),
            };
            
            nodes.push(Node {
                name: node.name().unwrap_or("Unnamed Node").to_string(),
                parent: None,
                children: node.children().map(|n| n.index()).collect(),
                transform,
            });
        }
        
        for (node_index, node) in gltf.nodes().enumerate() {
            profile_scope!("second pass enumerating children", node.name().unwrap_or("Unnamed Node"));
            for child in node.children() {
                if let Some(child_node) = nodes.get_mut(child.index()) {
                    child_node.parent = Some(node_index);
                }
            }
        }
        
        nodes
    }

    fn load_skins(gltf: &gltf::Document, buffers: &[gltf::buffer::Data]) -> Vec<Skin> {
        puffin::profile_function!("loading skins");
        let mut skins = Vec::new();
        
        for skin in gltf.skins() {
            puffin::profile_scope!("reading skin", skin.name().unwrap_or("Unnamed Skin"));
            let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();
            
            let inverse_bind_matrices = if let Some(accessor) = skin.inverse_bind_matrices() {
                let view = accessor.view().expect("Accessor must have a buffer view");
                let buffer_data = &buffers[view.buffer().index()];
                let start = view.offset() + accessor.offset();
                let stride = view.stride().unwrap_or(accessor.size());
                
                let mut matrices = Vec::with_capacity(accessor.count());
                for i in 0..accessor.count() {
                    let offset = start + i * stride;
                    let matrix_bytes = &buffer_data[offset..offset + 64];
                    
                    let mut floats = [0f32; 16];
                    for (j, chunk) in matrix_bytes.chunks_exact(4).enumerate() {
                        floats[j] = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    }
                    
                    matrices.push(glam::Mat4::from_cols_array(&floats));
                }
                matrices
            } else {
                vec![glam::Mat4::IDENTITY; joints.len()]
            };
            
            skins.push(Skin {
                name: skin.name().unwrap_or("Unnamed Skin").to_string(),
                joints,
                inverse_bind_matrices,
                skeleton_root: skin.skeleton().map(|n| n.index()),
            });
        }
        
        skins
    }

    fn load_animations(gltf: &gltf::Document, buffers: &[gltf::buffer::Data]) -> Vec<Animation> {
        puffin::profile_function!("loading animations");
        let mut animations = Vec::new();
        
        for animation in gltf.animations() {
            puffin::profile_scope!("reading animation", animation.name().unwrap_or("Unnamed Animation"));
            let mut channels = Vec::new();
            let mut max_time = 0.0f32;
            
            for channel in animation.channels() {
                let target = channel.target();
                let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
                let interpolation_mode = channel.sampler().interpolation();
                
                let times: Vec<f32> = if let Some(inputs) = reader.read_inputs() {
                    inputs.collect()
                } else {
                    continue;
                };
                
                if let Some(&last_time) = times.last() {
                    max_time = max_time.max(last_time);
                }
                
                let values = match target.property() {
                    gltf::animation::Property::Translation => {
                        puffin::profile_scope!("reading translation values");
                        if let Some(outputs) = reader.read_outputs() {
                            match outputs {
                                gltf::animation::util::ReadOutputs::Translations(iter) => {
                                    let translations: Vec<glam::Vec3> = iter
                                        .map(|t| glam::Vec3::from(t))
                                        .collect();
                                    ChannelValues::Translations(translations)
                                }
                                _ => continue,
                            }
                        } else {
                            continue;
                        }
                    }
                    gltf::animation::Property::Rotation => {
                        puffin::profile_scope!("reading rotation values");
                        if let Some(outputs) = reader.read_outputs() {
                            match outputs {
                                gltf::animation::util::ReadOutputs::Rotations(iter) => {
                                    let rotations: Vec<glam::Quat> = if interpolation_mode == gltf::animation::Interpolation::CubicSpline {
                                        iter.into_f32()
                                            .enumerate()
                                            .map(|(i, r)| {
                                                let q = glam::Quat::from_array(r);
                                                if i % 3 == 1 {
                                                    q.normalize()
                                                } else {
                                                    q
                                                }
                                            })
                                            .collect()
                                    } else {
                                        iter.into_f32()
                                            .map(|r| glam::Quat::from_array(r).normalize())
                                            .collect()
                                    };
                                    ChannelValues::Rotations(rotations)
                                }
                                _ => continue,
                            }
                        } else {
                            continue;
                        }
                    }
                    gltf::animation::Property::Scale => {
                        puffin::profile_scope!("reading scale values");
                        if let Some(outputs) = reader.read_outputs() {
                            match outputs {
                                gltf::animation::util::ReadOutputs::Scales(iter) => {
                                    let scales: Vec<glam::Vec3> = iter
                                        .map(|s| glam::Vec3::from(s))
                                        .collect();
                                    ChannelValues::Scales(scales)
                                }
                                _ => continue,
                            }
                        } else {
                            continue;
                        }
                    }
                    gltf::animation::Property::MorphTargetWeights => {
                        puffin::profile_scope!("reading morph target weights");
                        // Skip morph targets for now
                        continue;
                    }
                };
                
                let interpolation = match channel.sampler().interpolation() {
                    gltf::animation::Interpolation::Linear => AnimationInterpolation::Linear,
                    gltf::animation::Interpolation::Step => AnimationInterpolation::Step,
                    gltf::animation::Interpolation::CubicSpline => AnimationInterpolation::CubicSpline,
                };
                
                channels.push(AnimationChannel {
                    target_node: target.node().index(),
                    times,
                    values,
                    interpolation,
                });
            }
            
            animations.push(Animation {
                name: animation.name().unwrap_or("Unnamed Animation").to_string(),
                channels,
                duration: max_time,
            });
        }
        
        animations
    }

    pub async fn load_from_memory_raw<B>(
        graphics: Arc<SharedGraphicsContext>,
        buffer: B,
        label: Option<&str>,
        registry: Arc<RwLock<AssetRegistry>>,
    ) -> anyhow::Result<Handle<Model>>
    where
        B: AsRef<[u8]>,
    {
        puffin::profile_function!(label.unwrap_or("unlabelled model"));
        let mut registry = registry.write();

        let model_label = label.unwrap_or("No named model");
        let hash = {
            puffin::profile_scope!("hashing model");
            let mut hasher = DefaultHasher::default();
            if let Some(label) = label {
                label.hash(&mut hasher);
            } else {
                buffer.as_ref().hash(&mut hasher);
            };
            hasher.finish()
        };

        if let Some(model) = registry.model_handle_by_hash(hash) {
            return Ok(model);
        }

        let (gltf, buffers, images) = gltf::import_slice(buffer.as_ref())?;

        let mut meshes = Vec::new();
        for mesh in gltf.meshes() {
            Self::load_meshes(&mesh, &buffers, &mut meshes)?;
        }

        let nodes = Self::load_nodes(&gltf);
        
        let skins = Self::load_skins(&gltf, &buffers);
        
        let animations = Self::load_animations(&gltf, &buffers);
        
        log::debug!(
            "Loaded {} nodes, {} skins, {} animations for model [{:?}]",
            nodes.len(),
            skins.len(),
            animations.len(),
            label
        );

        let material_data = Self::load_materials(&gltf, &buffers, &images);

        let processed_textures: Vec<ProcessedMaterialTextures> = material_data
            .into_par_iter()
            .map(|material_info| {
                puffin::profile_scope!("processing material textures");
                let material_name = material_info.name;

                let extract = |info: Option<GLTFTextureInformation>| -> (Option<(Vec<u8>, (u32, u32))>, Option<wgpu::SamplerDescriptor<'static>>) {
                    if let Some(info) = info {
                         (Some((info.pixels, (info.width, info.height))), Some(info.sampler))
                    } else {
                         (None, None)
                    }
                };

                let (processed_diffuse, diffuse_sampler) = extract(material_info.diffuse_texture);
                let (processed_normal, normal_sampler) = extract(material_info.normal_texture);
                let (processed_emissive, emissive_sampler) = extract(material_info.emissive_texture);
                let (processed_metallic_roughness, metallic_roughness_sampler) = extract(material_info.metallic_roughness_texture);
                let (processed_occlusion, occlusion_sampler) = extract(material_info.occlusion_texture);

                let tint = material_info.tint;
                let emissive_factor = material_info.emissive_factor;
                let metallic_factor = material_info.metallic_factor;
                let roughness_factor = material_info.roughness_factor;
                let alpha_mode = material_info.alpha_mode;
                let alpha_cutoff = material_info.alpha_cutoff;
                let double_sided = material_info.double_sided;
                let occlusion_strength = material_info.occlusion_strength;
                let normal_scale = material_info.normal_scale;

                ProcessedMaterialTextures {
                    name: material_name,
                    diffuse: processed_diffuse,
                    normal: processed_normal,
                    emissive: processed_emissive,
                    metallic_roughness: processed_metallic_roughness,
                    occlusion: processed_occlusion,
                    diffuse_sampler,
                    normal_sampler,
                    emissive_sampler,
                    metallic_roughness_sampler,
                    occlusion_sampler,
                    tint,
                    emissive_factor,
                    metallic_factor,
                    roughness_factor,
                    alpha_mode,
                    alpha_cutoff,
                    double_sided,
                    occlusion_strength,
                    normal_scale,
                }
            })
            .collect();

        let mut materials = Vec::new();

        let grey_texture = registry.grey_texture(graphics.clone());
        let flat_normal_texture =
            registry.solid_texture_rgba8(graphics.clone(), [128, 128, 255, 255]);

        for processed in processed_textures {
            puffin::profile_scope!("creating material");

            let material_name = processed.name;
            let processed_diffuse = processed.diffuse;
            let processed_normal = processed.normal;
            let processed_emissive = processed.emissive;
            let processed_metallic_roughness = processed.metallic_roughness;
            let processed_occlusion = processed.occlusion;
            let diffuse_sampler = processed.diffuse_sampler;
            let normal_sampler = processed.normal_sampler;
            let emissive_sampler = processed.emissive_sampler;
            let metallic_roughness_sampler = processed.metallic_roughness_sampler;
            let occlusion_sampler = processed.occlusion_sampler;

            let diffuse_texture = if let Some((rgba_data, dimensions)) = processed_diffuse {
                Texture::from_bytes_verbose_mipmapped(
                    graphics.clone(),
                    &rgba_data,
                    Some(dimensions),
                    None,
                    diffuse_sampler.clone(),
                    Some(material_name.as_str())
                )
            } else if let Some(grey) = registry.get_texture(grey_texture) {
                (*grey).clone()
            } else {
                anyhow::bail!("Unable to find processed diffuse or fetch fallback texture for model {:?}", label);
            };

            let normal_texture = if let Some((rgba_data, dimensions)) = processed_normal {
                Texture::from_bytes_verbose_mipmapped(
                    graphics.clone(),
                    &rgba_data,
                    Some(dimensions),
                    None,
                    normal_sampler.clone(),
                    Some(material_name.as_str())
                )
            } else if let Some(tex) = registry.get_texture(flat_normal_texture) {
                (*tex).clone()
            } else {
                anyhow::bail!("Unable to find processed normal or fetch fallback texture for model {:?}", label);
            };

            let emissive_texture = processed_emissive.map(|(rgba_data, dimensions)| {
                Texture::from_bytes_verbose_mipmapped(
                    graphics.clone(),
                    &rgba_data,
                    Some(dimensions),
                    None,
                    emissive_sampler.clone(),
                    Some(material_name.as_str())
                )
            });
            let metallic_roughness_texture =
                processed_metallic_roughness.map(|(rgba_data, dimensions)| {
                    Texture::from_bytes_verbose_mipmapped(
                        graphics.clone(),
                        &rgba_data,
                        Some(dimensions),
                        None,
                        metallic_roughness_sampler.clone(),
                        Some(material_name.as_str())
                    )
                });
            let occlusion_texture = processed_occlusion.map(|(rgba_data, dimensions)| {
                Texture::from_bytes_verbose_mipmapped(
                    graphics.clone(),
                    &rgba_data,
                    Some(dimensions),
                    None,
                    occlusion_sampler.clone(),
                    Some(material_name.as_str())
                )
            });
            let texture_tag = Some(material_name.clone());

            let mut material = Material::new(
                graphics.clone(),
                material_name,
                diffuse_texture,
                normal_texture,
                processed.tint,
                texture_tag,
            );

            material.emissive_factor = processed.emissive_factor;
            material.metallic_factor = processed.metallic_factor;
            material.roughness_factor = processed.roughness_factor;
            material.alpha_mode = processed.alpha_mode.into();
            material.alpha_cutoff = processed.alpha_cutoff;
            material.double_sided = processed.double_sided;
            material.occlusion_strength = processed.occlusion_strength;
            material.normal_scale = processed.normal_scale;
            material.emissive_texture = emissive_texture;
            material.metallic_roughness_texture = metallic_roughness_texture;
            material.occlusion_texture = occlusion_texture;
            material.sync_uniform(&graphics);

            materials.push(material);
        }

        let mut gpu_meshes = Vec::new();
        for mesh_info in meshes {
            if mesh_info.mode != gltf::mesh::Mode::Triangles {
                return Err(anyhow::anyhow!(
                    "Unsupported primitive mode {:?} for mesh '{}' (primitive {})",
                    mesh_info.mode,
                    mesh_info.name,
                    mesh_info.primitive_index
                ));
            }

            let mut vertices = Vec::with_capacity(mesh_info.positions.len());
            for index in 0..mesh_info.positions.len() {
                vertices.push(ModelVertex {
                    position: mesh_info.positions[index],
                    normal: mesh_info.normals[index],
                    tangent: mesh_info.tangents[index],
                    tex_coords0: mesh_info.tex_coords0[index],
                    tex_coords1: mesh_info.tex_coords1[index],
                    colour0: mesh_info.colors[index],
                    joints0: mesh_info.joints[index],
                    weights0: mesh_info.weights[index],
                });
            }

            let vertex_buffer =
                graphics
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("{} Vertex Buffer", model_label)),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

            let index_buffer =
                graphics
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&format!("{} Index Buffer", model_label)),
                        contents: bytemuck::cast_slice(&mesh_info.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

            gpu_meshes.push(Mesh {
                name: mesh_info.name,
                vertex_buffer,
                index_buffer,
                vertices,
                num_elements: mesh_info.indices.len() as u32,
                material: mesh_info.material_index,
            });
        }

        log::debug!("Successfully loaded model [{:?}]", label);

        let model = Model {
            label: model_label.to_string(),
            hash,
            path: ResourceReference::from_bytes(buffer.as_ref()),
            meshes: gpu_meshes,
            materials,
            skins,
            animations,
            nodes,
        };

        let handle = if let Some(label) = label {
            registry.add_model_with_label(label, model)
        } else {
            registry.add_model(model)
        };

        Ok(handle)
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
        skin_bind_group: Option<&'a wgpu::BindGroup>,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: Range<u32>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        skin_bind_group: Option<&'a wgpu::BindGroup>,
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
        skin_bind_group: Option<&'a wgpu::BindGroup>,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, camera_bind_group, light_bind_group, skin_bind_group);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: Range<u32>,
        camera_bind_group: &'b wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        skin_bind_group: Option<&'a wgpu::BindGroup>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.set_bind_group(2, light_bind_group, &[]);
        
        if let Some(skin_bg) = skin_bind_group {
            self.set_bind_group(4, skin_bg, &[]);
        }

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
                None, // Provide an AnimationComponent if available in a future update
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

pub trait Vertex {
    fn desc() -> VertexBufferLayout<'static>;
}

/// Maps to
/// ```wgsl
/// struct VertexInput {
///     @location(0) position: vec3<f32>,
///     @location(1) normal: vec3<f32>,
///     @location(2) tangent: vec4<f32>,
///     @location(3) tex_coords0: vec2<f32>,
///     @location(4) tex_coords1: vec2<f32>,
///     @location(5) color0: vec4<f32>,
///     @location(6) joints0: vec4<u32>,
///     @location(7) weights0: vec4<f32>,
/// };
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4],      // xyz + handedness (w)
    pub tex_coords0: [f32; 2],
    pub tex_coords1: [f32; 2],  // optional, can be zeroed if missing
    pub colour0: [f32; 4],       // optional, default to white
    pub joints0: [u16; 4],
    pub weights0: [f32; 4],
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // tangent
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // tex_coords0
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 10]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // tex_coords1
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color0
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 14]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // joints0
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 18]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Uint16x4,
                },
                // weights0
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<[f32; 18]>() + mem::size_of::<[u16; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub base_colour: [f32; 4],
    pub emissive: [f32; 3],
    pub emissive_strength: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub normal_scale: f32,
    pub occlusion_strength: f32,
    pub alpha_cutoff: f32,
    pub uv_tiling: [f32; 2],
    pub _pad: f32,
}