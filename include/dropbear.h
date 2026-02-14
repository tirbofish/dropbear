#ifndef DROPBEAR_H
#define DROPBEAR_H

#include <stdbool.h>
#include <stdint.h>

#include <stddef.h>

typedef void* WorldPtr;

typedef struct NVector4 {
    double x;
    double y;
    double z;
    double w;
} NVector4;

typedef struct NVector3 {
    double x;
    double y;
    double z;
} NVector3;

typedef struct Option Option;// opaque

typedef struct NVector2 {
    double x;
    double y;
} NVector2;

typedef struct NMaterial {
    const char* name;
    uint64_t diffuse_texture;
    uint64_t normal_texture;
    NVector4 tint;
    NVector3 emissive_factor;
    float metallic_factor;
    float roughness_factor;
    Option alpha_cutoff;
    bool double_sided;
    float occlusion_strength;
    float normal_scale;
    NVector2 uv_tiling;
    Option emissive_texture;
    Option metallic_roughness_texture;
    Option occlusion_texture;
} NMaterial;

typedef struct NMaterialArrayFfi {
    const NMaterial* ptr;
    size_t len;
} NMaterialArrayFfi;

typedef void* AssetRegistryPtr;

typedef void* SceneLoaderPtr;

typedef struct size_t size_t;// opaque

typedef struct Progress {
    size_t current;
    size_t total;
    const char* message;
} Progress;

typedef struct i32ArrayFfi {
    const int32_t* ptr;
    size_t len;
} i32ArrayFfi;

typedef struct f64ArrayFfiArrayFfi {
    const double* ptr;
    size_t len;
} f64ArrayFfiArrayFfi;

typedef struct NSkin {
    const char* name;
    i32ArrayFfi joints;
    f64ArrayFfiArrayFfi inverse_bind_matrices;
    Option skeleton_root;
} NSkin;

typedef struct NSkinArrayFfi {
    const NSkin* ptr;
    size_t len;
} NSkinArrayFfi;

typedef struct f64ArrayFfi {
    const double* ptr;
    size_t len;
} f64ArrayFfi;

typedef struct NChannelValues NChannelValues;// opaque

typedef struct NAnimationInterpolation NAnimationInterpolation;// opaque

typedef struct NAnimationChannel {
    int32_t target_node;
    f64ArrayFfi times;
    NChannelValues values;
    NAnimationInterpolation interpolation;
} NAnimationChannel;

typedef struct NAnimationChannelArrayFfi {
    const NAnimationChannel* ptr;
    size_t len;
} NAnimationChannelArrayFfi;

typedef struct NAnimation {
    const char* name;
    NAnimationChannelArrayFfi channels;
    float duration;
} NAnimation;

typedef struct NAnimationArrayFfi {
    const NAnimation* ptr;
    size_t len;
} NAnimationArrayFfi;

typedef struct AssetKind AssetKind;// opaque

typedef struct NQuaternion {
    double x;
    double y;
    double z;
    double w;
} NQuaternion;

typedef struct NNodeTransform {
    NVector3 translation;
    NQuaternion rotation;
    NVector3 scale;
} NNodeTransform;

typedef struct NNode {
    const char* name;
    Option parent;
    i32ArrayFfi children;
    NNodeTransform transform;
} NNode;

typedef struct NNodeArrayFfi {
    const NNode* ptr;
    size_t len;
} NNodeArrayFfi;

typedef struct NModelVertex {
    NVector3 position;
    NVector3 normal;
    NVector4 tangent;
    NVector2 tex_coords0;
    NVector2 tex_coords1;
    NVector4 colour0;
    i32ArrayFfi joints0;
    NVector4 weights0;
} NModelVertex;

typedef struct NModelVertexArrayFfi {
    const NModelVertex* ptr;
    size_t len;
} NModelVertexArrayFfi;

typedef struct NMesh {
    const char* name;
    int32_t num_elements;
    int32_t material_index;
    NModelVertexArrayFfi vertices;
} NMesh;

typedef struct NMeshArrayFfi {
    const NMesh* ptr;
    size_t len;
} NMeshArrayFfi;

int32_t dropbear_scene_get_scene_load_progress(SceneLoaderPtr scene_loader, uint64_t scene_id, Progress* out0);
int32_t dropbear_get_entity(WorldPtr world, const char* label, uint64_t* out0);
int32_t dropbear_engine_get_asset(AssetRegistryPtr asset, const char* label, const AssetKind* kind, uint64_t* out0, bool* out0_present);
int32_t dropbear_asset_model_get_label(AssetRegistryPtr asset, uint64_t model_handle, char** out0);
int32_t dropbear_asset_model_get_meshes(AssetRegistryPtr asset, uint64_t model_handle, NMeshArrayFfi* out0);
int32_t dropbear_asset_model_get_materials(AssetRegistryPtr asset, uint64_t model_handle, NMaterialArrayFfi* out0);
int32_t dropbear_asset_model_get_skins(AssetRegistryPtr asset, uint64_t model_handle, NSkinArrayFfi* out0);
int32_t dropbear_asset_model_get_animations(AssetRegistryPtr asset, uint64_t model_handle, NAnimationArrayFfi* out0);
int32_t dropbear_asset_model_get_nodes(AssetRegistryPtr asset, uint64_t model_handle, NNodeArrayFfi* out0);

#endif /* DROPBEAR_H */
