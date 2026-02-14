use crate::pointer_convert;
use crate::ptr::AssetRegistryUnwrapped;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{NQuaternion, NVector2, NVector3, NVector4};
use dropbear_engine::asset::Handle;
use dropbear_engine::model::{Animation, AnimationChannel, AnimationInterpolation, ChannelValues, Material, Mesh, ModelVertex, Node, NodeTransform, Skin};
use dropbear_engine::texture::Texture;

#[derive(Clone, Debug, uniffi::Record)]
pub struct NModelVertex {
    pub position: NVector3,
    pub normal: NVector3,
    pub tangent: NVector4,
    pub tex_coords0: NVector2,
    pub tex_coords1: NVector2,
    pub colour0: NVector4,
    pub joints0: Vec<i32>,
    pub weights0: NVector4,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NMesh {
    pub name: String,
    pub num_elements: i32,
    pub material_index: i32,
    pub vertices: Vec<NModelVertex>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NMaterial {
    pub name: String,
    pub diffuse_texture: u64,
    pub normal_texture: u64,
    pub tint: NVector4,
    pub emissive_factor: NVector3,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub alpha_cutoff: Option<f32>,
    pub double_sided: bool,
    pub occlusion_strength: f32,
    pub normal_scale: f32,
    pub uv_tiling: NVector2,
    pub emissive_texture: Option<u64>,
    pub metallic_roughness_texture: Option<u64>,
    pub occlusion_texture: Option<u64>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NNodeTransform {
    pub translation: NVector3,
    pub rotation: NQuaternion,
    pub scale: NVector3,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NNode {
    pub name: String,
    pub parent: Option<i32>,
    pub children: Vec<i32>,
    pub transform: NNodeTransform,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NSkin {
    pub name: String,
    pub joints: Vec<i32>,
    pub inverse_bind_matrices: Vec<Vec<f64>>,
    pub skeleton_root: Option<i32>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NAnimation {
    pub name: String,
    pub channels: Vec<NAnimationChannel>,
    pub duration: f32,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct NAnimationChannel {
    pub target_node: i32,
    pub times: Vec<f64>,
    pub values: NChannelValues,
    pub interpolation: NAnimationInterpolation,
}

#[derive(Clone, Debug, uniffi::Enum)]
pub enum NAnimationInterpolation {
    Linear,
    Step,
    CubicSpline,
}

#[derive(Clone, Debug, uniffi::Enum)]
pub enum NChannelValues {
    Translations { values: Vec<NVector3> },
    Rotations { values: Vec<NQuaternion> },
    Scales { values: Vec<NVector3> },
}

fn texture_handle_id(
    registry: &dropbear_engine::asset::AssetRegistry,
    texture: &Texture,
) -> u64 {
    texture
        .hash()
        .and_then(|hash| registry.texture_handle_by_hash(hash).map(|h| h.id))
        .unwrap_or(0)
}

fn to_nvector3(v: glam::Vec3) -> NVector3 {
    NVector3::from([v.x, v.y, v.z])
}

fn to_nvector2(v: [f32; 2]) -> NVector2 {
    NVector2::from(v)
}

fn to_nvector4(v: [f32; 4]) -> NVector4 {
    NVector4::from(v)
}

fn to_nquaternion(q: glam::Quat) -> NQuaternion {
    NQuaternion {
        x: q.x as f64,
        y: q.y as f64,
        z: q.z as f64,
        w: q.w as f64,
    }
}

fn map_vertex(vertex: &ModelVertex) -> NModelVertex {
    NModelVertex {
        position: NVector3::from(vertex.position),
        normal: NVector3::from(vertex.normal),
        tangent: to_nvector4(vertex.tangent),
        tex_coords0: to_nvector2(vertex.tex_coords0),
        tex_coords1: to_nvector2(vertex.tex_coords1),
        colour0: to_nvector4(vertex.colour0),
        joints0: vertex.joints0.iter().map(|v| *v as i32).collect(),
        weights0: to_nvector4(vertex.weights0),
    }
}

fn map_mesh(mesh: &Mesh) -> NMesh {
    NMesh {
        name: mesh.name.clone(),
        num_elements: mesh.num_elements as i32,
        material_index: mesh.material as i32,
        vertices: mesh.vertices.iter().map(map_vertex).collect(),
    }
}

fn map_material(
    registry: &dropbear_engine::asset::AssetRegistry,
    material: &Material,
) -> NMaterial {
    NMaterial {
        name: material.name.clone(),
        diffuse_texture: texture_handle_id(registry, &material.diffuse_texture),
        normal_texture: texture_handle_id(registry, &material.normal_texture),
        tint: to_nvector4(material.tint),
        emissive_factor: NVector3::from(material.emissive_factor),
        metallic_factor: material.metallic_factor,
        roughness_factor: material.roughness_factor,
        alpha_cutoff: material.alpha_cutoff,
        double_sided: material.double_sided,
        occlusion_strength: material.occlusion_strength,
        normal_scale: material.normal_scale,
        uv_tiling: to_nvector2(material.uv_tiling),
        emissive_texture: material
            .emissive_texture
            .as_ref()
            .map(|tex| texture_handle_id(registry, tex))
            .filter(|id| *id != 0),
        metallic_roughness_texture: material
            .metallic_roughness_texture
            .as_ref()
            .map(|tex| texture_handle_id(registry, tex))
            .filter(|id| *id != 0),
        occlusion_texture: material
            .occlusion_texture
            .as_ref()
            .map(|tex| texture_handle_id(registry, tex))
            .filter(|id| *id != 0),
    }
}

fn map_node_transform(transform: &NodeTransform) -> NNodeTransform {
    NNodeTransform {
        translation: to_nvector3(transform.translation),
        rotation: to_nquaternion(transform.rotation),
        scale: to_nvector3(transform.scale),
    }
}

fn map_node(node: &Node) -> NNode {
    NNode {
        name: node.name.clone(),
        parent: node.parent.map(|v| v as i32),
        children: node.children.iter().map(|v| *v as i32).collect(),
        transform: map_node_transform(&node.transform),
    }
}

fn map_skin(skin: &Skin) -> NSkin {
    let inverse_bind_matrices = skin
        .inverse_bind_matrices
        .iter()
        .map(|matrix| matrix.to_cols_array().iter().map(|v| *v as f64).collect())
        .collect();

    NSkin {
        name: skin.name.clone(),
        joints: skin.joints.iter().map(|v| *v as i32).collect(),
        inverse_bind_matrices,
        skeleton_root: skin.skeleton_root.map(|v| v as i32),
    }
}

fn map_interpolation(value: &AnimationInterpolation) -> NAnimationInterpolation {
    match value {
        AnimationInterpolation::Linear => NAnimationInterpolation::Linear,
        AnimationInterpolation::Step => NAnimationInterpolation::Step,
        AnimationInterpolation::CubicSpline => NAnimationInterpolation::CubicSpline,
    }
}

fn map_channel_values(values: &ChannelValues) -> NChannelValues {
    match values {
        ChannelValues::Translations(list) => NChannelValues::Translations {
            values: list.iter().map(|v| to_nvector3(*v)).collect(),
        },
        ChannelValues::Rotations(list) => NChannelValues::Rotations {
            values: list.iter().map(|v| to_nquaternion(*v)).collect(),
        },
        ChannelValues::Scales(list) => NChannelValues::Scales {
            values: list.iter().map(|v| to_nvector3(*v)).collect(),
        },
    }
}

fn map_animation_channel(channel: &AnimationChannel) -> NAnimationChannel {
    NAnimationChannel {
        target_node: channel.target_node as i32,
        times: channel.times.iter().map(|v| *v as f64).collect(),
        values: map_channel_values(&channel.values),
        interpolation: map_interpolation(&channel.interpolation),
    }
}

fn map_animation(animation: &Animation) -> NAnimation {
    NAnimation {
        name: animation.name.clone(),
        channels: animation.channels.iter().map(map_animation_channel).collect(),
        duration: animation.duration,
    }
}

#[uniffi::export]
pub fn dropbear_asset_model_get_label(
    asset_registry: u64,
    model_handle: u64,
) -> DropbearNativeResult<String> {
    let asset = pointer_convert!(asset_registry => AssetRegistryUnwrapped);
    let label = asset
        .read()
        .get_label_from_model_handle(Handle::new(model_handle))
        .ok_or_else(|| DropbearNativeError::InvalidHandle)?;
    Ok(label)
}

#[uniffi::export]
pub fn dropbear_asset_model_get_meshes(
    asset_registry: u64,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NMesh>> {
    let asset = pointer_convert!(asset_registry => AssetRegistryUnwrapped);
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.meshes.iter().map(map_mesh).collect())
}

#[uniffi::export]
pub fn dropbear_asset_model_get_materials(
    asset_registry: u64,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NMaterial>> {
    let asset = pointer_convert!(asset_registry => AssetRegistryUnwrapped);
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model
        .materials
        .iter()
        .map(|mat| map_material(&reader, mat))
        .collect())
}

#[uniffi::export]
pub fn dropbear_asset_model_get_skins(
    asset_registry: u64,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NSkin>> {
    let asset = pointer_convert!(asset_registry => AssetRegistryUnwrapped);
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.skins.iter().map(map_skin).collect())
}

#[uniffi::export]
pub fn dropbear_asset_model_get_animations(
    asset_registry: u64,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NAnimation>> {
    let asset = pointer_convert!(asset_registry => AssetRegistryUnwrapped);
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.animations.iter().map(map_animation).collect())
}

#[uniffi::export]
pub fn dropbear_asset_model_get_nodes(
    asset_registry: u64,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NNode>> {
    let asset = pointer_convert!(asset_registry => AssetRegistryUnwrapped);
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.nodes.iter().map(map_node).collect())
}