use crate::ptr::{AssetRegistryPtr, AssetRegistryUnwrapped};
use crate::scripting::jni::utils::ToJObject;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use crate::types::{NQuaternion, NVector2, NVector3, NVector4};
use dropbear_engine::asset::Handle;
use dropbear_engine::model::{
    Animation, AnimationChannel, AnimationInterpolation, ChannelValues, Material, Mesh,
    ModelVertex, Node, NodeTransform, Skin,
};
use dropbear_engine::texture::Texture;
use jni::JNIEnv;
use jni::objects::{JObject, JValue};

#[repr(C)]
#[derive(Clone, Debug)]
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

impl ToJObject for NModelVertex {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/ModelVertex")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let position = self.position.to_jobject(env)?;
        let normal = self.normal.to_jobject(env)?;
        let tangent = self.tangent.to_jobject(env)?;
        let tex_coords0 = self.tex_coords0.to_jobject(env)?;
        let tex_coords1 = self.tex_coords1.to_jobject(env)?;
        let colour0 = self.colour0.to_jobject(env)?;
        let joints0 = self.joints0.as_slice().to_jobject(env)?;
        let weights0 = self.weights0.to_jobject(env)?;

        let args = [
            JValue::Object(&position),
            JValue::Object(&normal),
            JValue::Object(&tangent),
            JValue::Object(&tex_coords0),
            JValue::Object(&tex_coords1),
            JValue::Object(&colour0),
            JValue::Object(&joints0),
            JValue::Object(&weights0),
        ];

        let obj = env
            .new_object(
                &class,
                "(Lcom/dropbear/math/Vector3f;Lcom/dropbear/math/Vector3f;Lcom/dropbear/math/Vector4f;Lcom/dropbear/math/Vector2f;Lcom/dropbear/math/Vector2f;Lcom/dropbear/math/Vector4f;[ILcom/dropbear/math/Vector4f;)V",
                &args,
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct NMesh {
    pub name: String,
    pub num_elements: i32,
    pub material_index: i32,
    pub vertices: Vec<NModelVertex>,
}

impl ToJObject for NMesh {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/Mesh")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = env
            .new_string(&self.name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let vertices = self.vertices.to_jobject(env)?;

        let args = [
            JValue::Object(&name),
            JValue::Int(self.num_elements),
            JValue::Int(self.material_index),
            JValue::Object(&vertices),
        ];

        let obj = env
            .new_object(&class, "(Ljava/lang/String;IILjava/util/List;)V", &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
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

impl ToJObject for NMaterial {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/Material")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = env
            .new_string(&self.name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let diffuse_texture = new_texture(env, self.diffuse_texture)?;
        let normal_texture = new_texture(env, self.normal_texture)?;
        let tint = self.tint.to_jobject(env)?;
        let emissive_factor = self.emissive_factor.to_jobject(env)?;
        let uv_tiling = self.uv_tiling.to_jobject(env)?;
        let alpha_cutoff = self.alpha_cutoff.to_jobject(env)?;
        let emissive_texture = match self.emissive_texture {
            Some(id) => new_texture(env, id)?,
            None => JObject::null(),
        };
        let metallic_roughness_texture = match self.metallic_roughness_texture {
            Some(id) => new_texture(env, id)?,
            None => JObject::null(),
        };
        let occlusion_texture = match self.occlusion_texture {
            Some(id) => new_texture(env, id)?,
            None => JObject::null(),
        };

        let args = [
            JValue::Object(&name),
            JValue::Object(&diffuse_texture),
            JValue::Object(&normal_texture),
            JValue::Object(&tint),
            JValue::Object(&emissive_factor),
            JValue::Double(self.metallic_factor as f64),
            JValue::Double(self.roughness_factor as f64),
            JValue::Object(&alpha_cutoff),
            JValue::Bool(if self.double_sided { 1 } else { 0 }),
            JValue::Double(self.occlusion_strength as f64),
            JValue::Double(self.normal_scale as f64),
            JValue::Object(&uv_tiling),
            JValue::Object(&emissive_texture),
            JValue::Object(&metallic_roughness_texture),
            JValue::Object(&occlusion_texture),
        ];

        let obj = env
            .new_object(
                &class,
                "(Ljava/lang/String;Lcom/dropbear/asset/Texture;Lcom/dropbear/asset/Texture;Lcom/dropbear/math/Vector4d;Lcom/dropbear/math/Vector3d;DDLjava/lang/Double;ZDDLcom/dropbear/math/Vector2d;Lcom/dropbear/asset/Texture;Lcom/dropbear/asset/Texture;Lcom/dropbear/asset/Texture;)V",
                &args,
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct NNodeTransform {
    pub translation: NVector3,
    pub rotation: NQuaternion,
    pub scale: NVector3,
}

impl ToJObject for NNodeTransform {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/NodeTransform")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let translation = self.translation.to_jobject(env)?;
        let rotation = self.rotation.to_jobject(env)?;
        let scale = self.scale.to_jobject(env)?;

        let args = [
            JValue::Object(&translation),
            JValue::Object(&rotation),
            JValue::Object(&scale),
        ];

        let obj = env
            .new_object(
                &class,
                "(Lcom/dropbear/math/Vector3d;Lcom/dropbear/math/Quaterniond;Lcom/dropbear/math/Vector3d;)V",
                &args,
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct NNode {
    pub name: String,
    pub parent: Option<i32>,
    pub children: Vec<i32>,
    pub transform: NNodeTransform,
}

impl ToJObject for NNode {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/Node")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = env
            .new_string(&self.name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let parent = self.parent.to_jobject(env)?;
        let children = self.children.as_slice().to_jobject(env)?;
        let transform = self.transform.to_jobject(env)?;

        let args = [
            JValue::Object(&name),
            JValue::Object(&parent),
            JValue::Object(&children),
            JValue::Object(&transform),
        ];

        let obj = env
            .new_object(
                &class,
                "(Ljava/lang/String;Ljava/lang/Integer;Ljava/util/List;Lcom/dropbear/asset/model/NodeTransform;)V",
                &args,
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct NSkin {
    pub name: String,
    pub joints: Vec<i32>,
    pub inverse_bind_matrices: Vec<Vec<f64>>,
    pub skeleton_root: Option<i32>,
}

impl ToJObject for NSkin {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/Skin")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = env
            .new_string(&self.name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let joints = self.joints.as_slice().to_jobject(env)?;
        let inverse_bind_matrices = self.inverse_bind_matrices.as_slice().to_jobject(env)?;
        let skeleton_root = self.skeleton_root.to_jobject(env)?;

        let args = [
            JValue::Object(&name),
            JValue::Object(&joints),
            JValue::Object(&inverse_bind_matrices),
            JValue::Object(&skeleton_root),
        ];

        let obj = env
            .new_object(
                &class,
                "(Ljava/lang/String;Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;)V",
                &args,
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct NAnimation {
    pub name: String,
    pub channels: Vec<NAnimationChannel>,
    pub duration: f32,
}

impl ToJObject for NAnimation {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/Animation")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let name = env
            .new_string(&self.name)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
        let channels = self.channels.to_jobject(env)?;

        let args = [
            JValue::Object(&name),
            JValue::Object(&channels),
            JValue::Double(self.duration as f64),
        ];

        let obj = env
            .new_object(&class, "(Ljava/lang/String;Ljava/util/List;D)V", &args)
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct NAnimationChannel {
    pub target_node: i32,
    pub times: Vec<f64>,
    pub values: NChannelValues,
    pub interpolation: NAnimationInterpolation,
}

impl ToJObject for NAnimationChannel {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/AnimationChannel")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let times = self.times.as_slice().to_jobject(env)?;
        let values = self.values.to_jobject(env)?;
        let interpolation = self.interpolation.to_jobject(env)?;

        let args = [
            JValue::Int(self.target_node),
            JValue::Object(&times),
            JValue::Object(&values),
            JValue::Object(&interpolation),
        ];

        let obj = env
            .new_object(
                &class,
                "(I[DLcom/dropbear/asset/model/ChannelValues;Lcom/dropbear/asset/model/AnimationInterpolation;)V",
                &args,
            )
            .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;

        Ok(obj)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
#[dropbear_macro::repr_c_enum]
pub enum NAnimationInterpolation {
    Linear,
    Step,
    CubicSpline,
}

impl ToJObject for NAnimationInterpolation {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        let class = env
            .find_class("com/dropbear/asset/model/AnimationInterpolation")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        let field_name = match self {
            NAnimationInterpolation::Linear => "LINEAR",
            NAnimationInterpolation::Step => "STEP",
            NAnimationInterpolation::CubicSpline => "CUBIC_SPLINE",
        };

        let value = env
            .get_static_field(
                class,
                field_name,
                "Lcom/dropbear/asset/model/AnimationInterpolation;",
            )
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        Ok(value)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
#[dropbear_macro::repr_c_enum]
pub enum NChannelValues {
    Translations { values: Vec<NVector3> },
    Rotations { values: Vec<NQuaternion> },
    Scales { values: Vec<NVector3> },
}

impl ToJObject for NChannelValues {
    fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        match self {
            NChannelValues::Translations { values } => {
                let class = env
                    .find_class("com/dropbear/asset/model/ChannelValues$Translations")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;
                let list = values.to_jobject(env)?;
                let obj = env
                    .new_object(&class, "(Ljava/util/List;)V", &[JValue::Object(&list)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
                Ok(obj)
            }
            NChannelValues::Rotations { values } => {
                let class = env
                    .find_class("com/dropbear/asset/model/ChannelValues$Rotations")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;
                let list = values.to_jobject(env)?;
                let obj = env
                    .new_object(&class, "(Ljava/util/List;)V", &[JValue::Object(&list)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
                Ok(obj)
            }
            NChannelValues::Scales { values } => {
                let class = env
                    .find_class("com/dropbear/asset/model/ChannelValues$Scales")
                    .map_err(|_| DropbearNativeError::JNIClassNotFound)?;
                let list = values.to_jobject(env)?;
                let obj = env
                    .new_object(&class, "(Ljava/util/List;)V", &[JValue::Object(&list)])
                    .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)?;
                Ok(obj)
            }
        }
    }
}

fn new_texture<'a>(env: &mut JNIEnv<'a>, texture_id: u64) -> DropbearNativeResult<JObject<'a>> {
    let class = env
        .find_class("com/dropbear/asset/Texture")
        .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

    env.new_object(&class, "(J)V", &[JValue::Long(texture_id as i64)])
        .map_err(|_| DropbearNativeError::JNIFailedToCreateObject)
}

fn texture_handle_id(registry: &dropbear_engine::asset::AssetRegistry, texture: &Texture) -> u64 {
    texture
        .hash
        .and_then(|hash| registry.texture_handle_by_hash(hash).map(|h| h.id))
        .unwrap_or(0)
}

fn map_vertex(vertex: &ModelVertex) -> NModelVertex {
    NModelVertex {
        position: NVector3::from(vertex.position),
        normal: NVector3::from(vertex.normal),
        tangent: NVector4::from(vertex.tangent),
        tex_coords0: NVector2::from(vertex.tex_coords0),
        tex_coords1: NVector2::from(vertex.tex_coords1),
        colour0: NVector4::from(vertex.colour0),
        joints0: vertex.joints0.iter().map(|v| *v as i32).collect(),
        weights0: NVector4::from(vertex.weights0),
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
        tint: NVector4::from(material.tint),
        emissive_factor: NVector3::from(material.emissive_factor),
        metallic_factor: material.metallic_factor,
        roughness_factor: material.roughness_factor,
        alpha_cutoff: material.alpha_cutoff,
        double_sided: material.double_sided,
        occlusion_strength: material.occlusion_strength,
        normal_scale: material.normal_scale,
        uv_tiling: NVector2::from(material.uv_tiling),
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
        translation: NVector3::from(transform.translation),
        rotation: NQuaternion::from(transform.rotation),
        scale: NVector3::from(transform.scale),
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
            values: list.iter().map(|v| NVector3::from(*v)).collect(),
        },
        ChannelValues::Rotations(list) => NChannelValues::Rotations {
            values: list.iter().map(|v| NQuaternion::from(*v)).collect(),
        },
        ChannelValues::Scales(list) => NChannelValues::Scales {
            values: list.iter().map(|v| NVector3::from(*v)).collect(),
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
        channels: animation
            .channels
            .iter()
            .map(map_animation_channel)
            .collect(),
        duration: animation.duration,
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getLabel"),
    c(name = "dropbear_asset_model_get_label")
)]
fn dropbear_asset_model_get_label(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<String> {
    let label = asset
        .read()
        .get_label_from_model_handle(Handle::new(model_handle))
        .ok_or_else(|| DropbearNativeError::InvalidHandle)?;
    Ok(label)
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getMeshes"),
    c(name = "dropbear_asset_model_get_meshes")
)]
fn dropbear_asset_model_get_meshes(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NMesh>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.meshes.iter().map(map_mesh).collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getMaterials"),
    c(name = "dropbear_asset_model_get_materials")
)]
fn dropbear_asset_model_get_materials(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NMaterial>> {
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

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getSkins"),
    c(name = "dropbear_asset_model_get_skins")
)]
pub fn dropbear_asset_model_get_skins(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NSkin>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.skins.iter().map(map_skin).collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getAnimations"),
    c(name = "dropbear_asset_model_get_animations")
)]
pub fn dropbear_asset_model_get_animations(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NAnimation>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.animations.iter().map(map_animation).collect())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.asset.ModelNative", func = "getNodes"),
    c(name = "dropbear_asset_model_get_nodes")
)]
pub fn dropbear_asset_model_get_nodes(
    #[dropbear_macro::define(AssetRegistryPtr)] asset: &AssetRegistryUnwrapped,
    model_handle: u64,
) -> DropbearNativeResult<Vec<NNode>> {
    let reader = asset.read();
    let model = reader
        .get_model(Handle::new(model_handle))
        .ok_or(DropbearNativeError::InvalidHandle)?;

    Ok(model.nodes.iter().map(map_node).collect())
}
