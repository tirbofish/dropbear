pub mod model {
    use crate::types::{NQuaternion, NVector2, NVector3, NVector4};
    use dropbear_engine::model::{
        Animation, AnimationChannel, AnimationInterpolation, ChannelValues, Material,
        Mesh, ModelVertex, Node, NodeTransform, Skin,
    };
    use dropbear_engine::texture::Texture;
    pub use ffi::*;
    use ffi_impl::{
        NMaterialInner, NMeshInner, NModelVertexInner, NNodeInner, NNodeTransformInner,
        NSkinInner,
    };
    #[allow(dead_code)]
    #[allow(unused_imports)]
    mod ffi_impl {
        use super::*;
        use super::ffi::*;
        use dropbear_engine::asset::Handle;
        use crate::asset::model::{
            map_animation, map_material, map_mesh, map_node, map_skin,
        };
        use crate::ptr::AssetRegistryUnwrapped;
        pub struct NModelVertexInner {
            pub position: crate::types::NVector3,
            pub normal: crate::types::NVector3,
            pub tangent: crate::types::NVector4,
            pub tex_coords0: crate::types::NVector2,
            pub tex_coords1: crate::types::NVector2,
            pub colour0: crate::types::NVector4,
            pub joints0: Vec<i32>,
            pub weights0: crate::types::NVector4,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NModelVertexInner {
            #[inline]
            fn clone(&self) -> NModelVertexInner {
                NModelVertexInner {
                    position: ::core::clone::Clone::clone(&self.position),
                    normal: ::core::clone::Clone::clone(&self.normal),
                    tangent: ::core::clone::Clone::clone(&self.tangent),
                    tex_coords0: ::core::clone::Clone::clone(&self.tex_coords0),
                    tex_coords1: ::core::clone::Clone::clone(&self.tex_coords1),
                    colour0: ::core::clone::Clone::clone(&self.colour0),
                    joints0: ::core::clone::Clone::clone(&self.joints0),
                    weights0: ::core::clone::Clone::clone(&self.weights0),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NModelVertexInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "position",
                    "normal",
                    "tangent",
                    "tex_coords0",
                    "tex_coords1",
                    "colour0",
                    "joints0",
                    "weights0",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.position,
                    &self.normal,
                    &self.tangent,
                    &self.tex_coords0,
                    &self.tex_coords1,
                    &self.colour0,
                    &self.joints0,
                    &&self.weights0,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "NModelVertexInner",
                    names,
                    values,
                )
            }
        }
        pub struct NMeshInner {
            pub name: String,
            pub num_elements: i32,
            pub material_index: i32,
            pub vertices: Vec<NModelVertex>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NMeshInner {
            #[inline]
            fn clone(&self) -> NMeshInner {
                NMeshInner {
                    name: ::core::clone::Clone::clone(&self.name),
                    num_elements: ::core::clone::Clone::clone(&self.num_elements),
                    material_index: ::core::clone::Clone::clone(&self.material_index),
                    vertices: ::core::clone::Clone::clone(&self.vertices),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NMeshInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "NMeshInner",
                    "name",
                    &self.name,
                    "num_elements",
                    &self.num_elements,
                    "material_index",
                    &self.material_index,
                    "vertices",
                    &&self.vertices,
                )
            }
        }
        pub struct NMaterialInner {
            pub name: String,
            pub diffuse_texture: u64,
            pub normal_texture: u64,
            pub tint: crate::types::NVector4,
            pub emissive_factor: crate::types::NVector3,
            pub metallic_factor: f32,
            pub roughness_factor: f32,
            pub alpha_cutoff: Option<f32>,
            pub double_sided: bool,
            pub occlusion_strength: f32,
            pub normal_scale: f32,
            pub uv_tiling: crate::types::NVector2,
            pub emissive_texture: Option<u64>,
            pub metallic_roughness_texture: Option<u64>,
            pub occlusion_texture: Option<u64>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NMaterialInner {
            #[inline]
            fn clone(&self) -> NMaterialInner {
                NMaterialInner {
                    name: ::core::clone::Clone::clone(&self.name),
                    diffuse_texture: ::core::clone::Clone::clone(&self.diffuse_texture),
                    normal_texture: ::core::clone::Clone::clone(&self.normal_texture),
                    tint: ::core::clone::Clone::clone(&self.tint),
                    emissive_factor: ::core::clone::Clone::clone(&self.emissive_factor),
                    metallic_factor: ::core::clone::Clone::clone(&self.metallic_factor),
                    roughness_factor: ::core::clone::Clone::clone(
                        &self.roughness_factor,
                    ),
                    alpha_cutoff: ::core::clone::Clone::clone(&self.alpha_cutoff),
                    double_sided: ::core::clone::Clone::clone(&self.double_sided),
                    occlusion_strength: ::core::clone::Clone::clone(
                        &self.occlusion_strength,
                    ),
                    normal_scale: ::core::clone::Clone::clone(&self.normal_scale),
                    uv_tiling: ::core::clone::Clone::clone(&self.uv_tiling),
                    emissive_texture: ::core::clone::Clone::clone(
                        &self.emissive_texture,
                    ),
                    metallic_roughness_texture: ::core::clone::Clone::clone(
                        &self.metallic_roughness_texture,
                    ),
                    occlusion_texture: ::core::clone::Clone::clone(
                        &self.occlusion_texture,
                    ),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NMaterialInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "name",
                    "diffuse_texture",
                    "normal_texture",
                    "tint",
                    "emissive_factor",
                    "metallic_factor",
                    "roughness_factor",
                    "alpha_cutoff",
                    "double_sided",
                    "occlusion_strength",
                    "normal_scale",
                    "uv_tiling",
                    "emissive_texture",
                    "metallic_roughness_texture",
                    "occlusion_texture",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.name,
                    &self.diffuse_texture,
                    &self.normal_texture,
                    &self.tint,
                    &self.emissive_factor,
                    &self.metallic_factor,
                    &self.roughness_factor,
                    &self.alpha_cutoff,
                    &self.double_sided,
                    &self.occlusion_strength,
                    &self.normal_scale,
                    &self.uv_tiling,
                    &self.emissive_texture,
                    &self.metallic_roughness_texture,
                    &&self.occlusion_texture,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "NMaterialInner",
                    names,
                    values,
                )
            }
        }
        pub struct NNodeTransformInner {
            pub translation: crate::types::NVector3,
            pub rotation: crate::types::NQuaternion,
            pub scale: crate::types::NVector3,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NNodeTransformInner {
            #[inline]
            fn clone(&self) -> NNodeTransformInner {
                NNodeTransformInner {
                    translation: ::core::clone::Clone::clone(&self.translation),
                    rotation: ::core::clone::Clone::clone(&self.rotation),
                    scale: ::core::clone::Clone::clone(&self.scale),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NNodeTransformInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "NNodeTransformInner",
                    "translation",
                    &self.translation,
                    "rotation",
                    &self.rotation,
                    "scale",
                    &&self.scale,
                )
            }
        }
        pub struct NNodeInner {
            pub name: String,
            pub parent: Option<i32>,
            pub children: Vec<i32>,
            pub transform: NNodeTransform,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NNodeInner {
            #[inline]
            fn clone(&self) -> NNodeInner {
                NNodeInner {
                    name: ::core::clone::Clone::clone(&self.name),
                    parent: ::core::clone::Clone::clone(&self.parent),
                    children: ::core::clone::Clone::clone(&self.children),
                    transform: ::core::clone::Clone::clone(&self.transform),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NNodeInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "NNodeInner",
                    "name",
                    &self.name,
                    "parent",
                    &self.parent,
                    "children",
                    &self.children,
                    "transform",
                    &&self.transform,
                )
            }
        }
        pub struct NSkinInner {
            pub name: String,
            pub joints: Vec<i32>,
            pub inverse_bind_matrices: Vec<Vec<f64>>,
            pub skeleton_root: Option<i32>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NSkinInner {
            #[inline]
            fn clone(&self) -> NSkinInner {
                NSkinInner {
                    name: ::core::clone::Clone::clone(&self.name),
                    joints: ::core::clone::Clone::clone(&self.joints),
                    inverse_bind_matrices: ::core::clone::Clone::clone(
                        &self.inverse_bind_matrices,
                    ),
                    skeleton_root: ::core::clone::Clone::clone(&self.skeleton_root),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NSkinInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "NSkinInner",
                    "name",
                    &self.name,
                    "joints",
                    &self.joints,
                    "inverse_bind_matrices",
                    &self.inverse_bind_matrices,
                    "skeleton_root",
                    &&self.skeleton_root,
                )
            }
        }
        pub struct NAnimationInner {
            pub name: String,
            pub channels: Vec<Box<NAnimationChannel>>,
            pub duration: f32,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NAnimationInner {
            #[inline]
            fn clone(&self) -> NAnimationInner {
                NAnimationInner {
                    name: ::core::clone::Clone::clone(&self.name),
                    channels: ::core::clone::Clone::clone(&self.channels),
                    duration: ::core::clone::Clone::clone(&self.duration),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NAnimationInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field3_finish(
                    f,
                    "NAnimationInner",
                    "name",
                    &self.name,
                    "channels",
                    &self.channels,
                    "duration",
                    &&self.duration,
                )
            }
        }
        pub struct NAnimationChannelInner {
            pub target_node: i32,
            pub times: Vec<f64>,
            pub values: Box<NChannelValues>,
            pub interpolation: Box<NAnimationInterpolation>,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for NAnimationChannelInner {
            #[inline]
            fn clone(&self) -> NAnimationChannelInner {
                NAnimationChannelInner {
                    target_node: ::core::clone::Clone::clone(&self.target_node),
                    times: ::core::clone::Clone::clone(&self.times),
                    values: ::core::clone::Clone::clone(&self.values),
                    interpolation: ::core::clone::Clone::clone(&self.interpolation),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NAnimationChannelInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_struct_field4_finish(
                    f,
                    "NAnimationChannelInner",
                    "target_node",
                    &self.target_node,
                    "times",
                    &self.times,
                    "values",
                    &self.values,
                    "interpolation",
                    &&self.interpolation,
                )
            }
        }
        #[allow(non_camel_case_types)]
        pub enum NAnimationInterpolationInner {
            Linear,
            Step,
            CubicSpline,
        }
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        impl ::core::clone::Clone for NAnimationInterpolationInner {
            #[inline]
            fn clone(&self) -> NAnimationInterpolationInner {
                match self {
                    NAnimationInterpolationInner::Linear => {
                        NAnimationInterpolationInner::Linear
                    }
                    NAnimationInterpolationInner::Step => {
                        NAnimationInterpolationInner::Step
                    }
                    NAnimationInterpolationInner::CubicSpline => {
                        NAnimationInterpolationInner::CubicSpline
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        impl ::core::fmt::Debug for NAnimationInterpolationInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        NAnimationInterpolationInner::Linear => "Linear",
                        NAnimationInterpolationInner::Step => "Step",
                        NAnimationInterpolationInner::CubicSpline => "CubicSpline",
                    },
                )
            }
        }
        #[allow(non_camel_case_types)]
        pub enum NChannelValuesInner {
            Translations { values: Vec<crate::types::NVector3> },
            Rotations { values: Vec<crate::types::NQuaternion> },
            Scales { values: Vec<crate::types::NVector3> },
        }
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        impl ::core::clone::Clone for NChannelValuesInner {
            #[inline]
            fn clone(&self) -> NChannelValuesInner {
                match self {
                    NChannelValuesInner::Translations { values: __self_0 } => {
                        NChannelValuesInner::Translations {
                            values: ::core::clone::Clone::clone(__self_0),
                        }
                    }
                    NChannelValuesInner::Rotations { values: __self_0 } => {
                        NChannelValuesInner::Rotations {
                            values: ::core::clone::Clone::clone(__self_0),
                        }
                    }
                    NChannelValuesInner::Scales { values: __self_0 } => {
                        NChannelValuesInner::Scales {
                            values: ::core::clone::Clone::clone(__self_0),
                        }
                    }
                }
            }
        }
        #[automatically_derived]
        #[allow(non_camel_case_types)]
        impl ::core::fmt::Debug for NChannelValuesInner {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    NChannelValuesInner::Translations { values: __self_0 } => {
                        ::core::fmt::Formatter::debug_struct_field1_finish(
                            f,
                            "Translations",
                            "values",
                            &__self_0,
                        )
                    }
                    NChannelValuesInner::Rotations { values: __self_0 } => {
                        ::core::fmt::Formatter::debug_struct_field1_finish(
                            f,
                            "Rotations",
                            "values",
                            &__self_0,
                        )
                    }
                    NChannelValuesInner::Scales { values: __self_0 } => {
                        ::core::fmt::Formatter::debug_struct_field1_finish(
                            f,
                            "Scales",
                            "values",
                            &__self_0,
                        )
                    }
                }
            }
        }
    }
    pub mod ffi {
        use super::ffi_impl::{
            NModelVertexInner, NMeshInner, NMaterialInner, NNodeTransformInner,
            NNodeInner, NSkinInner, NAnimationInner, NAnimationChannelInner,
            NAnimationInterpolationInner, NChannelValuesInner,
        };
        use crate::scripting::native::DropbearNativeError;
        use crate::pointer_convert;
        use dropbear_engine::asset::Handle;
        use crate::asset::model::{
            map_animation, map_material, map_mesh, map_node, map_skin,
        };
        use crate::ptr::AssetRegistryUnwrapped;
        pub struct NModelVertex(pub NModelVertexInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NModelVertex {
            #[inline]
            fn clone(&self) -> NModelVertex {
                NModelVertex(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NModelVertex {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "NModelVertex",
                    &&self.0,
                )
            }
        }
        impl NModelVertex {
            pub fn new(
                position: crate::types::NVector3,
                normal: crate::types::NVector3,
                tangent: crate::types::NVector4,
                tex_coords0: crate::types::NVector2,
                tex_coords1: crate::types::NVector2,
                colour0: crate::types::NVector4,
                joints0: &[i32],
                weights0: crate::types::NVector4,
            ) -> Box<Self> {
                Box::new(
                    Self(NModelVertexInner {
                        position,
                        normal,
                        tangent,
                        tex_coords0,
                        tex_coords1,
                        colour0,
                        joints0: joints0.to_vec(),
                        weights0,
                    }),
                )
            }
            pub fn get_position(&self) -> crate::types::NVector3 {
                self.0.position.clone()
            }
            pub fn set_position(&mut self, value: crate::types::NVector3) {
                self.0.position = value;
            }
            pub fn get_normal(&self) -> crate::types::NVector3 {
                self.0.normal.clone()
            }
            pub fn set_normal(&mut self, value: crate::types::NVector3) {
                self.0.normal = value;
            }
            pub fn get_tangent(&self) -> crate::types::NVector4 {
                self.0.tangent.clone()
            }
            pub fn set_tangent(&mut self, value: crate::types::NVector4) {
                self.0.tangent = value;
            }
            pub fn get_tex_coords0(&self) -> crate::types::NVector2 {
                self.0.tex_coords0.clone()
            }
            pub fn set_tex_coords0(&mut self, value: crate::types::NVector2) {
                self.0.tex_coords0 = value;
            }
            pub fn get_tex_coords1(&self) -> crate::types::NVector2 {
                self.0.tex_coords1.clone()
            }
            pub fn set_tex_coords1(&mut self, value: crate::types::NVector2) {
                self.0.tex_coords1 = value;
            }
            pub fn get_colour0(&self) -> crate::types::NVector4 {
                self.0.colour0.clone()
            }
            pub fn set_colour0(&mut self, value: crate::types::NVector4) {
                self.0.colour0 = value;
            }
            pub fn joints0_len(&self) -> usize {
                self.0.joints0.len()
            }
            pub fn joints0_get(&self, index: usize) -> Option<i32> {
                self.0.joints0.get(index).cloned()
            }
            pub fn joints0_push(&mut self, value: i32) {
                self.0.joints0.push(value);
            }
            pub fn get_weights0(&self) -> crate::types::NVector4 {
                self.0.weights0.clone()
            }
            pub fn set_weights0(&mut self, value: crate::types::NVector4) {
                self.0.weights0 = value;
            }
        }
        pub struct NMesh(pub NMeshInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NMesh {
            #[inline]
            fn clone(&self) -> NMesh {
                NMesh(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NMesh {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "NMesh", &&self.0)
            }
        }
        impl NMesh {
            pub fn new(
                name: String,
                num_elements: i32,
                material_index: i32,
                vertices: &[NModelVertex],
            ) -> Box<Self> {
                Box::new(
                    Self(NMeshInner {
                        name,
                        num_elements,
                        material_index,
                        vertices: vertices.to_vec(),
                    }),
                )
            }
            pub fn get_name(&self) -> String {
                self.0.name.clone()
            }
            pub fn set_name(&mut self, value: String) {
                self.0.name = value;
            }
            pub fn get_num_elements(&self) -> i32 {
                self.0.num_elements.clone()
            }
            pub fn set_num_elements(&mut self, value: i32) {
                self.0.num_elements = value;
            }
            pub fn get_material_index(&self) -> i32 {
                self.0.material_index.clone()
            }
            pub fn set_material_index(&mut self, value: i32) {
                self.0.material_index = value;
            }
            pub fn vertices_len(&self) -> usize {
                self.0.vertices.len()
            }
            pub fn vertices_get(&self, index: usize) -> Option<NModelVertex> {
                self.0.vertices.get(index).cloned()
            }
            pub fn vertices_push(&mut self, value: NModelVertex) {
                self.0.vertices.push(value);
            }
        }
        pub struct NMaterial(pub NMaterialInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NMaterial {
            #[inline]
            fn clone(&self) -> NMaterial {
                NMaterial(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NMaterial {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "NMaterial",
                    &&self.0,
                )
            }
        }
        impl NMaterial {
            pub fn new(
                name: String,
                diffuse_texture: u64,
                normal_texture: u64,
                tint: crate::types::NVector4,
                emissive_factor: crate::types::NVector3,
                metallic_factor: f32,
                roughness_factor: f32,
                alpha_cutoff: Option<f32>,
                double_sided: bool,
                occlusion_strength: f32,
                normal_scale: f32,
                uv_tiling: crate::types::NVector2,
                emissive_texture: Option<u64>,
                metallic_roughness_texture: Option<u64>,
                occlusion_texture: Option<u64>,
            ) -> Box<Self> {
                Box::new(
                    Self(NMaterialInner {
                        name,
                        diffuse_texture,
                        normal_texture,
                        tint,
                        emissive_factor,
                        metallic_factor,
                        roughness_factor,
                        alpha_cutoff,
                        double_sided,
                        occlusion_strength,
                        normal_scale,
                        uv_tiling,
                        emissive_texture,
                        metallic_roughness_texture,
                        occlusion_texture,
                    }),
                )
            }
            pub fn get_name(&self) -> String {
                self.0.name.clone()
            }
            pub fn set_name(&mut self, value: String) {
                self.0.name = value;
            }
            pub fn get_diffuse_texture(&self) -> u64 {
                self.0.diffuse_texture.clone()
            }
            pub fn set_diffuse_texture(&mut self, value: u64) {
                self.0.diffuse_texture = value;
            }
            pub fn get_normal_texture(&self) -> u64 {
                self.0.normal_texture.clone()
            }
            pub fn set_normal_texture(&mut self, value: u64) {
                self.0.normal_texture = value;
            }
            pub fn get_tint(&self) -> crate::types::NVector4 {
                self.0.tint.clone()
            }
            pub fn set_tint(&mut self, value: crate::types::NVector4) {
                self.0.tint = value;
            }
            pub fn get_emissive_factor(&self) -> crate::types::NVector3 {
                self.0.emissive_factor.clone()
            }
            pub fn set_emissive_factor(&mut self, value: crate::types::NVector3) {
                self.0.emissive_factor = value;
            }
            pub fn get_metallic_factor(&self) -> f32 {
                self.0.metallic_factor.clone()
            }
            pub fn set_metallic_factor(&mut self, value: f32) {
                self.0.metallic_factor = value;
            }
            pub fn get_roughness_factor(&self) -> f32 {
                self.0.roughness_factor.clone()
            }
            pub fn set_roughness_factor(&mut self, value: f32) {
                self.0.roughness_factor = value;
            }
            pub fn get_alpha_cutoff(&self) -> Option<f32> {
                self.0.alpha_cutoff.clone()
            }
            pub fn set_alpha_cutoff(&mut self, value: Option<f32>) {
                self.0.alpha_cutoff = value;
            }
            pub fn get_double_sided(&self) -> bool {
                self.0.double_sided.clone()
            }
            pub fn set_double_sided(&mut self, value: bool) {
                self.0.double_sided = value;
            }
            pub fn get_occlusion_strength(&self) -> f32 {
                self.0.occlusion_strength.clone()
            }
            pub fn set_occlusion_strength(&mut self, value: f32) {
                self.0.occlusion_strength = value;
            }
            pub fn get_normal_scale(&self) -> f32 {
                self.0.normal_scale.clone()
            }
            pub fn set_normal_scale(&mut self, value: f32) {
                self.0.normal_scale = value;
            }
            pub fn get_uv_tiling(&self) -> crate::types::NVector2 {
                self.0.uv_tiling.clone()
            }
            pub fn set_uv_tiling(&mut self, value: crate::types::NVector2) {
                self.0.uv_tiling = value;
            }
            pub fn get_emissive_texture(&self) -> Option<u64> {
                self.0.emissive_texture.clone()
            }
            pub fn set_emissive_texture(&mut self, value: Option<u64>) {
                self.0.emissive_texture = value;
            }
            pub fn get_metallic_roughness_texture(&self) -> Option<u64> {
                self.0.metallic_roughness_texture.clone()
            }
            pub fn set_metallic_roughness_texture(&mut self, value: Option<u64>) {
                self.0.metallic_roughness_texture = value;
            }
            pub fn get_occlusion_texture(&self) -> Option<u64> {
                self.0.occlusion_texture.clone()
            }
            pub fn set_occlusion_texture(&mut self, value: Option<u64>) {
                self.0.occlusion_texture = value;
            }
        }
        pub struct NNodeTransform(pub NNodeTransformInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NNodeTransform {
            #[inline]
            fn clone(&self) -> NNodeTransform {
                NNodeTransform(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NNodeTransform {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "NNodeTransform",
                    &&self.0,
                )
            }
        }
        impl NNodeTransform {
            pub fn new(
                translation: crate::types::NVector3,
                rotation: crate::types::NQuaternion,
                scale: crate::types::NVector3,
            ) -> Box<Self> {
                Box::new(
                    Self(NNodeTransformInner {
                        translation,
                        rotation,
                        scale,
                    }),
                )
            }
            pub fn get_translation(&self) -> crate::types::NVector3 {
                self.0.translation.clone()
            }
            pub fn set_translation(&mut self, value: crate::types::NVector3) {
                self.0.translation = value;
            }
            pub fn get_rotation(&self) -> crate::types::NQuaternion {
                self.0.rotation.clone()
            }
            pub fn set_rotation(&mut self, value: crate::types::NQuaternion) {
                self.0.rotation = value;
            }
            pub fn get_scale(&self) -> crate::types::NVector3 {
                self.0.scale.clone()
            }
            pub fn set_scale(&mut self, value: crate::types::NVector3) {
                self.0.scale = value;
            }
        }
        pub struct NNode(pub NNodeInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NNode {
            #[inline]
            fn clone(&self) -> NNode {
                NNode(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NNode {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "NNode", &&self.0)
            }
        }
        impl NNode {
            pub fn new(
                name: String,
                parent: Option<i32>,
                children: &[i32],
                transform: NNodeTransform,
            ) -> Box<Self> {
                Box::new(
                    Self(NNodeInner {
                        name,
                        parent,
                        children: children.to_vec(),
                        transform,
                    }),
                )
            }
            pub fn get_name(&self) -> String {
                self.0.name.clone()
            }
            pub fn set_name(&mut self, value: String) {
                self.0.name = value;
            }
            pub fn get_parent(&self) -> Option<i32> {
                self.0.parent.clone()
            }
            pub fn set_parent(&mut self, value: Option<i32>) {
                self.0.parent = value;
            }
            pub fn children_len(&self) -> usize {
                self.0.children.len()
            }
            pub fn children_get(&self, index: usize) -> Option<i32> {
                self.0.children.get(index).cloned()
            }
            pub fn children_push(&mut self, value: i32) {
                self.0.children.push(value);
            }
            pub fn get_transform(&self) -> NNodeTransform {
                self.0.transform.clone()
            }
            pub fn set_transform(&mut self, value: NNodeTransform) {
                self.0.transform = value;
            }
        }
        pub struct NSkin(pub NSkinInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NSkin {
            #[inline]
            fn clone(&self) -> NSkin {
                NSkin(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NSkin {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "NSkin", &&self.0)
            }
        }
        impl NSkin {
            pub fn get_name(&self) -> String {
                self.0.name.clone()
            }
            pub fn set_name(&mut self, value: String) {
                self.0.name = value;
            }
            pub fn joints_len(&self) -> usize {
                self.0.joints.len()
            }
            pub fn joints_get(&self, index: usize) -> Option<i32> {
                self.0.joints.get(index).cloned()
            }
            pub fn joints_push(&mut self, value: i32) {
                self.0.joints.push(value);
            }
            pub fn get_skeleton_root(&self) -> Option<i32> {
                self.0.skeleton_root.clone()
            }
            pub fn set_skeleton_root(&mut self, value: Option<i32>) {
                self.0.skeleton_root = value;
            }
        }
        pub struct NAnimation(pub NAnimationInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NAnimation {
            #[inline]
            fn clone(&self) -> NAnimation {
                NAnimation(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NAnimation {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "NAnimation",
                    &&self.0,
                )
            }
        }
        impl NAnimation {
            pub fn new(
                name: String,
                channels: &[Box<NAnimationChannel>],
                duration: f32,
            ) -> Box<Self> {
                Box::new(
                    Self(NAnimationInner {
                        name,
                        channels: channels.to_vec(),
                        duration,
                    }),
                )
            }
            pub fn get_name(&self) -> String {
                self.0.name.clone()
            }
            pub fn set_name(&mut self, value: String) {
                self.0.name = value;
            }
            pub fn channels_len(&self) -> usize {
                self.0.channels.len()
            }
            pub fn channels_get(&self, index: usize) -> Option<Box<NAnimationChannel>> {
                self.0.channels.get(index).cloned()
            }
            pub fn channels_push(&mut self, value: Box<NAnimationChannel>) {
                self.0.channels.push(value);
            }
            pub fn get_duration(&self) -> f32 {
                self.0.duration.clone()
            }
            pub fn set_duration(&mut self, value: f32) {
                self.0.duration = value;
            }
        }
        pub struct NAnimationChannel(pub NAnimationChannelInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NAnimationChannel {
            #[inline]
            fn clone(&self) -> NAnimationChannel {
                NAnimationChannel(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NAnimationChannel {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "NAnimationChannel",
                    &&self.0,
                )
            }
        }
        impl NAnimationChannel {
            pub fn new(
                target_node: i32,
                times: &[f64],
                values: Box<NChannelValues>,
                interpolation: Box<NAnimationInterpolation>,
            ) -> Box<Self> {
                Box::new(
                    Self(NAnimationChannelInner {
                        target_node,
                        times: times.to_vec(),
                        values,
                        interpolation,
                    }),
                )
            }
            pub fn get_target_node(&self) -> i32 {
                self.0.target_node.clone()
            }
            pub fn set_target_node(&mut self, value: i32) {
                self.0.target_node = value;
            }
            pub fn times_len(&self) -> usize {
                self.0.times.len()
            }
            pub fn times_get(&self, index: usize) -> Option<f64> {
                self.0.times.get(index).cloned()
            }
            pub fn times_push(&mut self, value: f64) {
                self.0.times.push(value);
            }
            pub fn get_values(&self) -> Box<NChannelValues> {
                self.0.values.clone()
            }
            pub fn set_values(&mut self, value: Box<NChannelValues>) {
                self.0.values = value;
            }
            pub fn get_interpolation(&self) -> Box<NAnimationInterpolation> {
                self.0.interpolation.clone()
            }
            pub fn set_interpolation(&mut self, value: Box<NAnimationInterpolation>) {
                self.0.interpolation = value;
            }
        }
        pub struct NAnimationInterpolation(NAnimationInterpolationInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NAnimationInterpolation {
            #[inline]
            fn clone(&self) -> NAnimationInterpolation {
                NAnimationInterpolation(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NAnimationInterpolation {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "NAnimationInterpolation",
                    &&self.0,
                )
            }
        }
        impl NAnimationInterpolation {
            pub fn new_linear() -> Box<NAnimationInterpolation> {
                Box::new(NAnimationInterpolation(NAnimationInterpolationInner::Linear))
            }
            pub fn new_step() -> Box<NAnimationInterpolation> {
                Box::new(NAnimationInterpolation(NAnimationInterpolationInner::Step))
            }
            pub fn new_cubicspline() -> Box<NAnimationInterpolation> {
                Box::new(
                    NAnimationInterpolation(NAnimationInterpolationInner::CubicSpline),
                )
            }
        }
        pub struct NChannelValues(NChannelValuesInner);
        #[automatically_derived]
        impl ::core::clone::Clone for NChannelValues {
            #[inline]
            fn clone(&self) -> NChannelValues {
                NChannelValues(::core::clone::Clone::clone(&self.0))
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for NChannelValues {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "NChannelValues",
                    &&self.0,
                )
            }
        }
        impl NChannelValues {
            pub fn new_translations(
                values: &[crate::types::NVector3],
            ) -> Box<NChannelValues> {
                Box::new(
                    NChannelValues(NChannelValuesInner::Translations {
                        values: values.to_vec(),
                    }),
                )
            }
            pub fn new_rotations(
                values: &[crate::types::NQuaternion],
            ) -> Box<NChannelValues> {
                Box::new(
                    NChannelValues(NChannelValuesInner::Rotations {
                        values: values.to_vec(),
                    }),
                )
            }
            pub fn new_scales(values: &[crate::types::NVector3]) -> Box<NChannelValues> {
                Box::new(
                    NChannelValues(NChannelValuesInner::Scales {
                        values: values.to_vec(),
                    }),
                )
            }
            pub fn translations_values_len(&self) -> usize {
                if let NChannelValuesInner::Translations { values, .. } = &self.0 {
                    values.len()
                } else {
                    0
                }
            }
            pub fn translations_values_get(
                &self,
                index: usize,
            ) -> Option<crate::types::NVector3> {
                if let NChannelValuesInner::Translations { values, .. } = &self.0 {
                    values.get(index).cloned()
                } else {
                    None
                }
            }
            pub fn translations_values_push(&mut self, value: crate::types::NVector3) {
                if let NChannelValuesInner::Translations { values, .. } = &mut self.0 {
                    values.push(value);
                }
            }
            pub fn rotations_values_len(&self) -> usize {
                if let NChannelValuesInner::Rotations { values, .. } = &self.0 {
                    values.len()
                } else {
                    0
                }
            }
            pub fn rotations_values_get(
                &self,
                index: usize,
            ) -> Option<crate::types::NQuaternion> {
                if let NChannelValuesInner::Rotations { values, .. } = &self.0 {
                    values.get(index).cloned()
                } else {
                    None
                }
            }
            pub fn rotations_values_push(&mut self, value: crate::types::NQuaternion) {
                if let NChannelValuesInner::Rotations { values, .. } = &mut self.0 {
                    values.push(value);
                }
            }
            pub fn scales_values_len(&self) -> usize {
                if let NChannelValuesInner::Scales { values, .. } = &self.0 {
                    values.len()
                } else {
                    0
                }
            }
            pub fn scales_values_get(
                &self,
                index: usize,
            ) -> Option<crate::types::NVector3> {
                if let NChannelValuesInner::Scales { values, .. } = &self.0 {
                    values.get(index).cloned()
                } else {
                    None
                }
            }
            pub fn scales_values_push(&mut self, value: crate::types::NVector3) {
                if let NChannelValuesInner::Scales { values, .. } = &mut self.0 {
                    values.push(value);
                }
            }
        }
        pub fn dropbear_asset_model_get_label(
            asset_registry: u64,
            model_handle: u64,
        ) -> Result<String, crate::scripting::native::DropbearNativeError> {
            let asset = {
                let ptr = asset_registry as *const AssetRegistryUnwrapped;
                if ptr.is_null() {
                    return Err(
                        crate::scripting::native::DropbearNativeError::NullPointer,
                    );
                }
                let addr = ptr as usize;
                let align = std::mem::align_of::<AssetRegistryUnwrapped>();
                if addr % align != 0 {
                    return Err(
                        crate::scripting::native::DropbearNativeError::InvalidArgument,
                    );
                }
                unsafe { &*ptr }
            };
            let label = asset
                .read()
                .get_label_from_model_handle(Handle::new(model_handle))
                .ok_or_else(|| DropbearNativeError::InvalidHandle)?;
            Ok(label)
        }
        pub struct NMeshList(pub Vec<NMesh>);
        impl NMeshList {
            pub fn len(&self) -> usize {
                self.0.len()
            }
            pub fn get(&self, i: usize) -> Option<&NMesh> {
                self.0.get(i)
            }
        }
        pub fn dropbear_asset_model_get_meshes(
            asset_registry: u64,
            model_handle: u64,
        ) -> Result<Box<NMeshList>, crate::scripting::native::DropbearNativeError> {
            let logic = || -> Result<Vec<NMesh>, _> {
                {
                    let asset = {
                        let ptr = asset_registry as *const AssetRegistryUnwrapped;
                        if ptr.is_null() {
                            return Err(
                                crate::scripting::native::DropbearNativeError::NullPointer,
                            );
                        }
                        let addr = ptr as usize;
                        let align = std::mem::align_of::<AssetRegistryUnwrapped>();
                        if addr % align != 0 {
                            return Err(
                                crate::scripting::native::DropbearNativeError::InvalidArgument,
                            );
                        }
                        unsafe { &*ptr }
                    };
                    let reader = asset.read();
                    let model = reader
                        .get_model(Handle::new(model_handle))
                        .ok_or(DropbearNativeError::InvalidHandle)?;
                    Ok(model.meshes.iter().map(map_mesh).collect())
                }
            };
            logic().map(|v| Box::new(NMeshList(v)))
        }
        pub struct NMaterialList(pub Vec<NMaterial>);
        impl NMaterialList {
            pub fn len(&self) -> usize {
                self.0.len()
            }
            pub fn get(&self, i: usize) -> Option<&NMaterial> {
                self.0.get(i)
            }
        }
        pub fn dropbear_asset_model_get_materials(
            asset_registry: u64,
            model_handle: u64,
        ) -> Result<Box<NMaterialList>, crate::scripting::native::DropbearNativeError> {
            let logic = || -> Result<Vec<NMaterial>, _> {
                {
                    let asset = {
                        let ptr = asset_registry as *const AssetRegistryUnwrapped;
                        if ptr.is_null() {
                            return Err(
                                crate::scripting::native::DropbearNativeError::NullPointer,
                            );
                        }
                        let addr = ptr as usize;
                        let align = std::mem::align_of::<AssetRegistryUnwrapped>();
                        if addr % align != 0 {
                            return Err(
                                crate::scripting::native::DropbearNativeError::InvalidArgument,
                            );
                        }
                        unsafe { &*ptr }
                    };
                    let reader = asset.read();
                    let model = reader
                        .get_model(Handle::new(model_handle))
                        .ok_or(DropbearNativeError::InvalidHandle)?;
                    Ok(
                        model
                            .materials
                            .iter()
                            .map(|mat| map_material(&reader, mat))
                            .collect(),
                    )
                }
            };
            logic().map(|v| Box::new(NMaterialList(v)))
        }
        pub struct NSkinList(pub Vec<NSkin>);
        impl NSkinList {
            pub fn len(&self) -> usize {
                self.0.len()
            }
            pub fn get(&self, i: usize) -> Option<&NSkin> {
                self.0.get(i)
            }
        }
        pub fn dropbear_asset_model_get_skins(
            asset_registry: u64,
            model_handle: u64,
        ) -> Result<Box<NSkinList>, crate::scripting::native::DropbearNativeError> {
            let logic = || -> Result<Vec<NSkin>, _> {
                {
                    let asset = {
                        let ptr = asset_registry as *const AssetRegistryUnwrapped;
                        if ptr.is_null() {
                            return Err(
                                crate::scripting::native::DropbearNativeError::NullPointer,
                            );
                        }
                        let addr = ptr as usize;
                        let align = std::mem::align_of::<AssetRegistryUnwrapped>();
                        if addr % align != 0 {
                            return Err(
                                crate::scripting::native::DropbearNativeError::InvalidArgument,
                            );
                        }
                        unsafe { &*ptr }
                    };
                    let reader = asset.read();
                    let model = reader
                        .get_model(Handle::new(model_handle))
                        .ok_or(DropbearNativeError::InvalidHandle)?;
                    Ok(model.skins.iter().map(map_skin).collect())
                }
            };
            logic().map(|v| Box::new(NSkinList(v)))
        }
        pub struct NAnimationList(pub Vec<Box<NAnimation>>);
        impl NAnimationList {
            pub fn len(&self) -> usize {
                self.0.len()
            }
            pub fn get(&self, i: usize) -> Option<&Box<NAnimation>> {
                self.0.get(i)
            }
        }
        pub fn dropbear_asset_model_get_animations(
            asset_registry: u64,
            model_handle: u64,
        ) -> Result<Box<NAnimationList>, crate::scripting::native::DropbearNativeError> {
            let logic = || -> Result<Vec<Box<NAnimation>>, _> {
                {
                    let asset = {
                        let ptr = asset_registry as *const AssetRegistryUnwrapped;
                        if ptr.is_null() {
                            return Err(
                                crate::scripting::native::DropbearNativeError::NullPointer,
                            );
                        }
                        let addr = ptr as usize;
                        let align = std::mem::align_of::<AssetRegistryUnwrapped>();
                        if addr % align != 0 {
                            return Err(
                                crate::scripting::native::DropbearNativeError::InvalidArgument,
                            );
                        }
                        unsafe { &*ptr }
                    };
                    let reader = asset.read();
                    let model = reader
                        .get_model(Handle::new(model_handle))
                        .ok_or(DropbearNativeError::InvalidHandle)?;
                    Ok(model.animations.iter().map(map_animation).collect())
                }
            };
            logic().map(|v| Box::new(NAnimationList(v)))
        }
        pub struct NNodeList(pub Vec<NNode>);
        impl NNodeList {
            pub fn len(&self) -> usize {
                self.0.len()
            }
            pub fn get(&self, i: usize) -> Option<&NNode> {
                self.0.get(i)
            }
        }
        pub fn dropbear_asset_model_get_nodes(
            asset_registry: u64,
            model_handle: u64,
        ) -> Result<Box<NNodeList>, crate::scripting::native::DropbearNativeError> {
            let logic = || -> Result<Vec<NNode>, _> {
                {
                    let asset = {
                        let ptr = asset_registry as *const AssetRegistryUnwrapped;
                        if ptr.is_null() {
                            return Err(
                                crate::scripting::native::DropbearNativeError::NullPointer,
                            );
                        }
                        let addr = ptr as usize;
                        let align = std::mem::align_of::<AssetRegistryUnwrapped>();
                        if addr % align != 0 {
                            return Err(
                                crate::scripting::native::DropbearNativeError::InvalidArgument,
                            );
                        }
                        unsafe { &*ptr }
                    };
                    let reader = asset.read();
                    let model = reader
                        .get_model(Handle::new(model_handle))
                        .ok_or(DropbearNativeError::InvalidHandle)?;
                    Ok(model.nodes.iter().map(map_node).collect())
                }
            };
            logic().map(|v| Box::new(NNodeList(v)))
        }
        use diplomat_runtime::*;
        use core::ffi::c_void;
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_new(
            name: String,
            channels: &[Box<NAnimationChannel>],
            duration: f32,
        ) -> Box<NAnimation> {
            NAnimation::new(name, channels, duration)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_get_name(this: &NAnimation) -> String {
            this.get_name()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_set_name(this: &mut NAnimation, value: String) {
            this.set_name(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_channels_len(this: &NAnimation) -> usize {
            this.channels_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_channels_get(
            this: &NAnimation,
            index: usize,
        ) -> Option<Box<NAnimationChannel>> {
            this.channels_get(index)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_channels_push(
            this: &mut NAnimation,
            value: Box<NAnimationChannel>,
        ) {
            this.channels_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_get_duration(this: &NAnimation) -> f32 {
            this.get_duration()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_set_duration(this: &mut NAnimation, value: f32) {
            this.set_duration(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimation_destroy(this: Box<NAnimation>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_new(
            target_node: i32,
            times: diplomat_runtime::DiplomatSlice<f64>,
            values: Box<NChannelValues>,
            interpolation: Box<NAnimationInterpolation>,
        ) -> Box<NAnimationChannel> {
            let times = times.into();
            NAnimationChannel::new(target_node, times, values, interpolation)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_get_target_node(
            this: &NAnimationChannel,
        ) -> i32 {
            this.get_target_node()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_set_target_node(
            this: &mut NAnimationChannel,
            value: i32,
        ) {
            this.set_target_node(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_times_len(this: &NAnimationChannel) -> usize {
            this.times_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_times_get(
            this: &NAnimationChannel,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<f64, ()> {
            this.times_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_times_push(
            this: &mut NAnimationChannel,
            value: f64,
        ) {
            this.times_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_get_values(
            this: &NAnimationChannel,
        ) -> Box<NChannelValues> {
            this.get_values()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_set_values(
            this: &mut NAnimationChannel,
            value: Box<NChannelValues>,
        ) {
            this.set_values(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_get_interpolation(
            this: &NAnimationChannel,
        ) -> Box<NAnimationInterpolation> {
            this.get_interpolation()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_set_interpolation(
            this: &mut NAnimationChannel,
            value: Box<NAnimationInterpolation>,
        ) {
            this.set_interpolation(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationChannel_destroy(this: Box<NAnimationChannel>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationInterpolation_new_linear() -> Box<
            NAnimationInterpolation,
        > {
            NAnimationInterpolation::new_linear()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationInterpolation_new_step() -> Box<
            NAnimationInterpolation,
        > {
            NAnimationInterpolation::new_step()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationInterpolation_new_cubicspline() -> Box<
            NAnimationInterpolation,
        > {
            NAnimationInterpolation::new_cubicspline()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationInterpolation_destroy(
            this: Box<NAnimationInterpolation>,
        ) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationList_len(this: &NAnimationList) -> usize {
            this.len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationList_get(
            this: &NAnimationList,
            i: usize,
        ) -> Option<&Box<NAnimation>> {
            this.get(i)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NAnimationList_destroy(this: Box<NAnimationList>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_new_translations(
            values: &[crate::types::NVector3],
        ) -> Box<NChannelValues> {
            NChannelValues::new_translations(values)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_new_rotations(
            values: &[crate::types::NQuaternion],
        ) -> Box<NChannelValues> {
            NChannelValues::new_rotations(values)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_new_scales(
            values: &[crate::types::NVector3],
        ) -> Box<NChannelValues> {
            NChannelValues::new_scales(values)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_translations_values_len(
            this: &NChannelValues,
        ) -> usize {
            this.translations_values_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_translations_values_get(
            this: &NChannelValues,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<crate::types::NVector3, ()> {
            this.translations_values_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_translations_values_push(
            this: &mut NChannelValues,
            value: crate::types::NVector3,
        ) {
            this.translations_values_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_rotations_values_len(
            this: &NChannelValues,
        ) -> usize {
            this.rotations_values_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_rotations_values_get(
            this: &NChannelValues,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<crate::types::NQuaternion, ()> {
            this.rotations_values_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_rotations_values_push(
            this: &mut NChannelValues,
            value: crate::types::NQuaternion,
        ) {
            this.rotations_values_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_scales_values_len(this: &NChannelValues) -> usize {
            this.scales_values_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_scales_values_get(
            this: &NChannelValues,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<crate::types::NVector3, ()> {
            this.scales_values_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_scales_values_push(
            this: &mut NChannelValues,
            value: crate::types::NVector3,
        ) {
            this.scales_values_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NChannelValues_destroy(this: Box<NChannelValues>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_new(
            name: String,
            diffuse_texture: u64,
            normal_texture: u64,
            tint: crate::types::NVector4,
            emissive_factor: crate::types::NVector3,
            metallic_factor: f32,
            roughness_factor: f32,
            alpha_cutoff: diplomat_runtime::DiplomatOption<f32>,
            double_sided: bool,
            occlusion_strength: f32,
            normal_scale: f32,
            uv_tiling: crate::types::NVector2,
            emissive_texture: diplomat_runtime::DiplomatOption<u64>,
            metallic_roughness_texture: diplomat_runtime::DiplomatOption<u64>,
            occlusion_texture: diplomat_runtime::DiplomatOption<u64>,
        ) -> Box<NMaterial> {
            let alpha_cutoff: Option<f32> = alpha_cutoff.into();
            let emissive_texture: Option<u64> = emissive_texture.into();
            let metallic_roughness_texture: Option<u64> = metallic_roughness_texture
                .into();
            let occlusion_texture: Option<u64> = occlusion_texture.into();
            NMaterial::new(
                name,
                diffuse_texture,
                normal_texture,
                tint,
                emissive_factor,
                metallic_factor,
                roughness_factor,
                alpha_cutoff,
                double_sided,
                occlusion_strength,
                normal_scale,
                uv_tiling,
                emissive_texture,
                metallic_roughness_texture,
                occlusion_texture,
            )
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_name(this: &NMaterial) -> String {
            this.get_name()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_name(this: &mut NMaterial, value: String) {
            this.set_name(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_diffuse_texture(this: &NMaterial) -> u64 {
            this.get_diffuse_texture()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_diffuse_texture(this: &mut NMaterial, value: u64) {
            this.set_diffuse_texture(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_normal_texture(this: &NMaterial) -> u64 {
            this.get_normal_texture()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_normal_texture(this: &mut NMaterial, value: u64) {
            this.set_normal_texture(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_tint(this: &NMaterial) -> crate::types::NVector4 {
            this.get_tint()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_tint(
            this: &mut NMaterial,
            value: crate::types::NVector4,
        ) {
            this.set_tint(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_emissive_factor(
            this: &NMaterial,
        ) -> crate::types::NVector3 {
            this.get_emissive_factor()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_emissive_factor(
            this: &mut NMaterial,
            value: crate::types::NVector3,
        ) {
            this.set_emissive_factor(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_metallic_factor(this: &NMaterial) -> f32 {
            this.get_metallic_factor()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_metallic_factor(this: &mut NMaterial, value: f32) {
            this.set_metallic_factor(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_roughness_factor(this: &NMaterial) -> f32 {
            this.get_roughness_factor()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_roughness_factor(this: &mut NMaterial, value: f32) {
            this.set_roughness_factor(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_alpha_cutoff(
            this: &NMaterial,
        ) -> diplomat_runtime::DiplomatResult<f32, ()> {
            this.get_alpha_cutoff().ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_alpha_cutoff(
            this: &mut NMaterial,
            value: diplomat_runtime::DiplomatOption<f32>,
        ) {
            let value: Option<f32> = value.into();
            this.set_alpha_cutoff(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_double_sided(this: &NMaterial) -> bool {
            this.get_double_sided()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_double_sided(this: &mut NMaterial, value: bool) {
            this.set_double_sided(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_occlusion_strength(this: &NMaterial) -> f32 {
            this.get_occlusion_strength()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_occlusion_strength(
            this: &mut NMaterial,
            value: f32,
        ) {
            this.set_occlusion_strength(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_normal_scale(this: &NMaterial) -> f32 {
            this.get_normal_scale()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_normal_scale(this: &mut NMaterial, value: f32) {
            this.set_normal_scale(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_uv_tiling(
            this: &NMaterial,
        ) -> crate::types::NVector2 {
            this.get_uv_tiling()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_uv_tiling(
            this: &mut NMaterial,
            value: crate::types::NVector2,
        ) {
            this.set_uv_tiling(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_emissive_texture(
            this: &NMaterial,
        ) -> diplomat_runtime::DiplomatResult<u64, ()> {
            this.get_emissive_texture().ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_emissive_texture(
            this: &mut NMaterial,
            value: diplomat_runtime::DiplomatOption<u64>,
        ) {
            let value: Option<u64> = value.into();
            this.set_emissive_texture(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_metallic_roughness_texture(
            this: &NMaterial,
        ) -> diplomat_runtime::DiplomatResult<u64, ()> {
            this.get_metallic_roughness_texture().ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_metallic_roughness_texture(
            this: &mut NMaterial,
            value: diplomat_runtime::DiplomatOption<u64>,
        ) {
            let value: Option<u64> = value.into();
            this.set_metallic_roughness_texture(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_get_occlusion_texture(
            this: &NMaterial,
        ) -> diplomat_runtime::DiplomatResult<u64, ()> {
            this.get_occlusion_texture().ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_set_occlusion_texture(
            this: &mut NMaterial,
            value: diplomat_runtime::DiplomatOption<u64>,
        ) {
            let value: Option<u64> = value.into();
            this.set_occlusion_texture(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterial_destroy(this: Box<NMaterial>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterialList_len(this: &NMaterialList) -> usize {
            this.len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterialList_get(
            this: &NMaterialList,
            i: usize,
        ) -> Option<&NMaterial> {
            this.get(i)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMaterialList_destroy(this: Box<NMaterialList>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_new(
            name: String,
            num_elements: i32,
            material_index: i32,
            vertices: &[NModelVertex],
        ) -> Box<NMesh> {
            NMesh::new(name, num_elements, material_index, vertices)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_get_name(this: &NMesh) -> String {
            this.get_name()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_set_name(this: &mut NMesh, value: String) {
            this.set_name(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_get_num_elements(this: &NMesh) -> i32 {
            this.get_num_elements()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_set_num_elements(this: &mut NMesh, value: i32) {
            this.set_num_elements(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_get_material_index(this: &NMesh) -> i32 {
            this.get_material_index()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_set_material_index(this: &mut NMesh, value: i32) {
            this.set_material_index(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_vertices_len(this: &NMesh) -> usize {
            this.vertices_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_vertices_get(
            this: &NMesh,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<NModelVertex, ()> {
            this.vertices_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_vertices_push(this: &mut NMesh, value: NModelVertex) {
            this.vertices_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMesh_destroy(this: Box<NMesh>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMeshList_len(this: &NMeshList) -> usize {
            this.len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMeshList_get(this: &NMeshList, i: usize) -> Option<&NMesh> {
            this.get(i)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NMeshList_destroy(this: Box<NMeshList>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_new(
            position: crate::types::NVector3,
            normal: crate::types::NVector3,
            tangent: crate::types::NVector4,
            tex_coords0: crate::types::NVector2,
            tex_coords1: crate::types::NVector2,
            colour0: crate::types::NVector4,
            joints0: diplomat_runtime::DiplomatSlice<i32>,
            weights0: crate::types::NVector4,
        ) -> Box<NModelVertex> {
            let joints0 = joints0.into();
            NModelVertex::new(
                position,
                normal,
                tangent,
                tex_coords0,
                tex_coords1,
                colour0,
                joints0,
                weights0,
            )
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_get_position(
            this: &NModelVertex,
        ) -> crate::types::NVector3 {
            this.get_position()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_set_position(
            this: &mut NModelVertex,
            value: crate::types::NVector3,
        ) {
            this.set_position(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_get_normal(
            this: &NModelVertex,
        ) -> crate::types::NVector3 {
            this.get_normal()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_set_normal(
            this: &mut NModelVertex,
            value: crate::types::NVector3,
        ) {
            this.set_normal(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_get_tangent(
            this: &NModelVertex,
        ) -> crate::types::NVector4 {
            this.get_tangent()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_set_tangent(
            this: &mut NModelVertex,
            value: crate::types::NVector4,
        ) {
            this.set_tangent(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_get_tex_coords0(
            this: &NModelVertex,
        ) -> crate::types::NVector2 {
            this.get_tex_coords0()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_set_tex_coords0(
            this: &mut NModelVertex,
            value: crate::types::NVector2,
        ) {
            this.set_tex_coords0(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_get_tex_coords1(
            this: &NModelVertex,
        ) -> crate::types::NVector2 {
            this.get_tex_coords1()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_set_tex_coords1(
            this: &mut NModelVertex,
            value: crate::types::NVector2,
        ) {
            this.set_tex_coords1(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_get_colour0(
            this: &NModelVertex,
        ) -> crate::types::NVector4 {
            this.get_colour0()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_set_colour0(
            this: &mut NModelVertex,
            value: crate::types::NVector4,
        ) {
            this.set_colour0(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_joints0_len(this: &NModelVertex) -> usize {
            this.joints0_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_joints0_get(
            this: &NModelVertex,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<i32, ()> {
            this.joints0_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_joints0_push(this: &mut NModelVertex, value: i32) {
            this.joints0_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_get_weights0(
            this: &NModelVertex,
        ) -> crate::types::NVector4 {
            this.get_weights0()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_set_weights0(
            this: &mut NModelVertex,
            value: crate::types::NVector4,
        ) {
            this.set_weights0(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NModelVertex_destroy(this: Box<NModelVertex>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_new(
            name: String,
            parent: diplomat_runtime::DiplomatOption<i32>,
            children: diplomat_runtime::DiplomatSlice<i32>,
            transform: NNodeTransform,
        ) -> Box<NNode> {
            let parent: Option<i32> = parent.into();
            let children = children.into();
            NNode::new(name, parent, children, transform)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_get_name(this: &NNode) -> String {
            this.get_name()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_set_name(this: &mut NNode, value: String) {
            this.set_name(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_get_parent(
            this: &NNode,
        ) -> diplomat_runtime::DiplomatResult<i32, ()> {
            this.get_parent().ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_set_parent(
            this: &mut NNode,
            value: diplomat_runtime::DiplomatOption<i32>,
        ) {
            let value: Option<i32> = value.into();
            this.set_parent(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_children_len(this: &NNode) -> usize {
            this.children_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_children_get(
            this: &NNode,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<i32, ()> {
            this.children_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_children_push(this: &mut NNode, value: i32) {
            this.children_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_get_transform(this: &NNode) -> NNodeTransform {
            this.get_transform()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_set_transform(this: &mut NNode, value: NNodeTransform) {
            this.set_transform(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNode_destroy(this: Box<NNode>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeList_len(this: &NNodeList) -> usize {
            this.len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeList_get(this: &NNodeList, i: usize) -> Option<&NNode> {
            this.get(i)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeList_destroy(this: Box<NNodeList>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_new(
            translation: crate::types::NVector3,
            rotation: crate::types::NQuaternion,
            scale: crate::types::NVector3,
        ) -> Box<NNodeTransform> {
            NNodeTransform::new(translation, rotation, scale)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_get_translation(
            this: &NNodeTransform,
        ) -> crate::types::NVector3 {
            this.get_translation()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_set_translation(
            this: &mut NNodeTransform,
            value: crate::types::NVector3,
        ) {
            this.set_translation(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_get_rotation(
            this: &NNodeTransform,
        ) -> crate::types::NQuaternion {
            this.get_rotation()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_set_rotation(
            this: &mut NNodeTransform,
            value: crate::types::NQuaternion,
        ) {
            this.set_rotation(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_get_scale(
            this: &NNodeTransform,
        ) -> crate::types::NVector3 {
            this.get_scale()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_set_scale(
            this: &mut NNodeTransform,
            value: crate::types::NVector3,
        ) {
            this.set_scale(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NNodeTransform_destroy(this: Box<NNodeTransform>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_get_name(this: &NSkin) -> String {
            this.get_name()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_set_name(this: &mut NSkin, value: String) {
            this.set_name(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_joints_len(this: &NSkin) -> usize {
            this.joints_len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_joints_get(
            this: &NSkin,
            index: usize,
        ) -> diplomat_runtime::DiplomatResult<i32, ()> {
            this.joints_get(index).ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_joints_push(this: &mut NSkin, value: i32) {
            this.joints_push(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_get_skeleton_root(
            this: &NSkin,
        ) -> diplomat_runtime::DiplomatResult<i32, ()> {
            this.get_skeleton_root().ok_or(()).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_set_skeleton_root(
            this: &mut NSkin,
            value: diplomat_runtime::DiplomatOption<i32>,
        ) {
            let value: Option<i32> = value.into();
            this.set_skeleton_root(value)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkin_destroy(this: Box<NSkin>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkinList_len(this: &NSkinList) -> usize {
            this.len()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkinList_get(this: &NSkinList, i: usize) -> Option<&NSkin> {
            this.get(i)
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn NSkinList_destroy(this: Box<NSkinList>) {}
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn diplomat_external_dropbear_asset_model_get_animations(
            asset_registry: u64,
            model_handle: u64,
        ) -> diplomat_runtime::DiplomatResult<
            Box<NAnimationList>,
            crate::scripting::native::DropbearNativeError,
        > {
            dropbear_asset_model_get_animations(asset_registry, model_handle).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn diplomat_external_dropbear_asset_model_get_label(
            asset_registry: u64,
            model_handle: u64,
        ) -> diplomat_runtime::DiplomatResult<
            String,
            crate::scripting::native::DropbearNativeError,
        > {
            dropbear_asset_model_get_label(asset_registry, model_handle).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn diplomat_external_dropbear_asset_model_get_materials(
            asset_registry: u64,
            model_handle: u64,
        ) -> diplomat_runtime::DiplomatResult<
            Box<NMaterialList>,
            crate::scripting::native::DropbearNativeError,
        > {
            dropbear_asset_model_get_materials(asset_registry, model_handle).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn diplomat_external_dropbear_asset_model_get_meshes(
            asset_registry: u64,
            model_handle: u64,
        ) -> diplomat_runtime::DiplomatResult<
            Box<NMeshList>,
            crate::scripting::native::DropbearNativeError,
        > {
            dropbear_asset_model_get_meshes(asset_registry, model_handle).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn diplomat_external_dropbear_asset_model_get_nodes(
            asset_registry: u64,
            model_handle: u64,
        ) -> diplomat_runtime::DiplomatResult<
            Box<NNodeList>,
            crate::scripting::native::DropbearNativeError,
        > {
            dropbear_asset_model_get_nodes(asset_registry, model_handle).into()
        }
        #[no_mangle]
        #[allow(deprecated)]
        extern "C" fn diplomat_external_dropbear_asset_model_get_skins(
            asset_registry: u64,
            model_handle: u64,
        ) -> diplomat_runtime::DiplomatResult<
            Box<NSkinList>,
            crate::scripting::native::DropbearNativeError,
        > {
            dropbear_asset_model_get_skins(asset_registry, model_handle).into()
        }
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
    fn map_vertex(vertex: &ModelVertex) -> NModelVertex {
        NModelVertex {
            0: NModelVertexInner {
                position: NVector3::from(vertex.position),
                normal: NVector3::from(vertex.normal),
                tangent: NVector4::from(vertex.tangent),
                tex_coords0: NVector2::from(vertex.tex_coords0),
                tex_coords1: NVector2::from(vertex.tex_coords1),
                colour0: NVector4::from(vertex.colour0),
                joints0: vertex.joints0.iter().map(|v| *v as i32).collect(),
                weights0: NVector4::from(vertex.weights0),
            },
        }
    }
    fn map_mesh(mesh: &Mesh) -> NMesh {
        NMesh {
            0: NMeshInner {
                name: mesh.name.clone(),
                num_elements: mesh.num_elements as i32,
                material_index: mesh.material as i32,
                vertices: mesh.vertices.iter().map(map_vertex).collect(),
            },
        }
    }
    fn map_material(
        registry: &dropbear_engine::asset::AssetRegistry,
        material: &Material,
    ) -> NMaterial {
        NMaterial {
            0: NMaterialInner {
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
            },
        }
    }
    fn map_node_transform(transform: &NodeTransform) -> NNodeTransform {
        NNodeTransform {
            0: NNodeTransformInner {
                translation: NVector3::from(transform.translation),
                rotation: NQuaternion::from(transform.rotation),
                scale: NVector3::from(transform.scale),
            },
        }
    }
    fn map_node(node: &Node) -> NNode {
        NNode {
            0: NNodeInner {
                name: node.name.clone(),
                parent: node.parent.map(|v| v as i32),
                children: node.children.iter().map(|v| *v as i32).collect(),
                transform: map_node_transform(&node.transform),
            },
        }
    }
    fn map_skin(skin: &Skin) -> NSkin {
        let inverse_bind_matrices = skin
            .inverse_bind_matrices
            .iter()
            .map(|matrix| matrix.to_cols_array().iter().map(|v| *v as f64).collect())
            .collect();
        NSkin {
            0: NSkinInner {
                name: skin.name.clone(),
                joints: skin.joints.iter().map(|v| *v as i32).collect(),
                inverse_bind_matrices,
                skeleton_root: skin.skeleton_root.map(|v| v as i32),
            },
        }
    }
    fn map_interpolation(
        value: &AnimationInterpolation,
    ) -> Box<NAnimationInterpolation> {
        match value {
            AnimationInterpolation::Linear => NAnimationInterpolation::new_linear(),
            AnimationInterpolation::Step => NAnimationInterpolation::new_step(),
            AnimationInterpolation::CubicSpline => {
                NAnimationInterpolation::new_cubicspline()
            }
        }
    }
    fn map_channel_values(values: &ChannelValues) -> Box<NChannelValues> {
        match values {
            ChannelValues::Translations(list) => {
                let values: Vec<NVector3> = list
                    .iter()
                    .map(|v| NVector3::from(*v))
                    .collect();
                NChannelValues::new_translations(&values)
            }
            ChannelValues::Rotations(list) => {
                let values: Vec<NQuaternion> = list
                    .iter()
                    .map(|v| NQuaternion::from(*v))
                    .collect();
                NChannelValues::new_rotations(&values)
            }
            ChannelValues::Scales(list) => {
                let values: Vec<NVector3> = list
                    .iter()
                    .map(|v| NVector3::from(*v))
                    .collect();
                NChannelValues::new_scales(&values)
            }
        }
    }
    fn map_animation_channel(channel: &AnimationChannel) -> Box<NAnimationChannel> {
        let times: Vec<f64> = channel.times.iter().map(|v| *v as f64).collect();
        NAnimationChannel::new(
            channel.target_node as i32,
            &times,
            map_channel_values(&channel.values),
            map_interpolation(&channel.interpolation),
        )
    }
    fn map_animation(animation: &Animation) -> Box<NAnimation> {
        let channels: Vec<Box<NAnimationChannel>> = animation
            .channels
            .iter()
            .map(map_animation_channel)
            .collect();
        NAnimation::new(animation.name.clone(), &channels, animation.duration)
    }
}
