// Machine generated header bindings by goanna-gen.
// DO NOT EDIT UNLESS YOU KNOW WHAT YOU ARE DOING (it will get regenerated anyways with a modification to eucalyptus-core/src).
// Licensed under MIT or Apache 2.0 depending on your mood.
// part of the dropbear project, by tirbofish

#ifndef DROPBEAR_H
#define DROPBEAR_H

#include <stdbool.h>
#include <stdint.h>

#include <stddef.h>

typedef enum AssetKind {
    AssetKind_Texture = 0,
    AssetKind_Model = 1,
} AssetKind;

typedef enum ColliderShapeTag {
    ColliderShapeTag_Box = 0,
    ColliderShapeTag_Sphere = 1,
    ColliderShapeTag_Capsule = 2,
    ColliderShapeTag_Cylinder = 3,
    ColliderShapeTag_Cone = 4,
} ColliderShapeTag;

typedef struct ColliderShapeBox {
    void half_extents;
} ColliderShapeBox;

typedef struct ColliderShapeSphere {
    float radius;
} ColliderShapeSphere;

typedef struct ColliderShapeCapsule {
    float half_height;
    float radius;
} ColliderShapeCapsule;

typedef struct ColliderShapeCylinder {
    float half_height;
    float radius;
} ColliderShapeCylinder;

typedef struct ColliderShapeCone {
    float half_height;
    float radius;
} ColliderShapeCone;

typedef union ColliderShapeData {
    ColliderShapeBox Box;
    ColliderShapeSphere Sphere;
    ColliderShapeCapsule Capsule;
    ColliderShapeCylinder Cylinder;
    ColliderShapeCone Cone;
} ColliderShapeData;

typedef struct ColliderShapeFfi {
    ColliderShapeTag tag;
    ColliderShapeData data;
} ColliderShapeFfi;

typedef ColliderShapeFfi ColliderShape;

typedef enum NShapeCastStatusTag {
    NShapeCastStatusTag_OutOfIterations = 0,
    NShapeCastStatusTag_Converged = 1,
    NShapeCastStatusTag_Failed = 2,
    NShapeCastStatusTag_PenetratingOrWithinTargetDist = 3,
} NShapeCastStatusTag;

typedef struct NShapeCastStatusOutOfIterations {
} NShapeCastStatusOutOfIterations;

typedef struct NShapeCastStatusConverged {
} NShapeCastStatusConverged;

typedef struct NShapeCastStatusFailed {
} NShapeCastStatusFailed;

typedef struct NShapeCastStatusPenetratingOrWithinTargetDist {
} NShapeCastStatusPenetratingOrWithinTargetDist;

typedef union NShapeCastStatusData {
    NShapeCastStatusOutOfIterations OutOfIterations;
    NShapeCastStatusConverged Converged;
    NShapeCastStatusFailed Failed;
    NShapeCastStatusPenetratingOrWithinTargetDist PenetratingOrWithinTargetDist;
} NShapeCastStatusData;

typedef struct NShapeCastStatusFfi {
    NShapeCastStatusTag tag;
    NShapeCastStatusData data;
} NShapeCastStatusFfi;

typedef NShapeCastStatusFfi NShapeCastStatus;

typedef enum NAnimationInterpolationTag {
    NAnimationInterpolationTag_Linear = 0,
    NAnimationInterpolationTag_Step = 1,
    NAnimationInterpolationTag_CubicSpline = 2,
} NAnimationInterpolationTag;

typedef struct NAnimationInterpolationLinear {
} NAnimationInterpolationLinear;

typedef struct NAnimationInterpolationStep {
} NAnimationInterpolationStep;

typedef struct NAnimationInterpolationCubicSpline {
} NAnimationInterpolationCubicSpline;

typedef union NAnimationInterpolationData {
    NAnimationInterpolationLinear Linear;
    NAnimationInterpolationStep Step;
    NAnimationInterpolationCubicSpline CubicSpline;
} NAnimationInterpolationData;

typedef struct NAnimationInterpolationFfi {
    NAnimationInterpolationTag tag;
    NAnimationInterpolationData data;
} NAnimationInterpolationFfi;

typedef NAnimationInterpolationFfi NAnimationInterpolation;

typedef struct NVector3 {
    double x;
    double y;
    double z;
} NVector3;

typedef struct NVector3Array {
    NVector3* values;
    size_t length;
    size_t capacity;
} NVector3Array;

typedef struct NQuaternion {
    double x;
    double y;
    double z;
    double w;
} NQuaternion;

typedef struct NQuaternionArray {
    NQuaternion* values;
    size_t length;
    size_t capacity;
} NQuaternionArray;

typedef enum NChannelValuesTag {
    NChannelValuesTag_Translations = 0,
    NChannelValuesTag_Rotations = 1,
    NChannelValuesTag_Scales = 2,
} NChannelValuesTag;

typedef struct NChannelValuesTranslations {
    NVector3Array values;
} NChannelValuesTranslations;

typedef struct NChannelValuesRotations {
    NQuaternionArray values;
} NChannelValuesRotations;

typedef struct NChannelValuesScales {
    NVector3Array values;
} NChannelValuesScales;

typedef union NChannelValuesData {
    NChannelValuesTranslations Translations;
    NChannelValuesRotations Rotations;
    NChannelValuesScales Scales;
} NChannelValuesData;

typedef struct NChannelValuesFfi {
    NChannelValuesTag tag;
    NChannelValuesData data;
} NChannelValuesFfi;

typedef NChannelValuesFfi NChannelValues;

typedef void* SceneLoaderPtr;

typedef void* AssetRegistryPtr;

typedef struct NVector2 {
    double x;
    double y;
} NVector2;

typedef struct Progress {
    size_t current;
    size_t total;
    const char* message;
} Progress;

typedef struct u64Array {
    uint64_t* values;
    size_t length;
    size_t capacity;
} u64Array;

typedef struct ConnectedGamepadIds {
    u64Array ids;
} ConnectedGamepadIds;

typedef struct NAttenuation {
    float constant;
    float linear;
    float quadratic;
} NAttenuation;

typedef struct NRange {
    float start;
    float end;
} NRange;

typedef struct IndexNative {
    uint32_t index;
    uint32_t generation;
} IndexNative;

typedef struct NCollider {
    IndexNative index;
    uint64_t entity_id;
    uint32_t id;
} NCollider;

typedef struct NColliderArray {
    NCollider* values;
    size_t length;
    size_t capacity;
} NColliderArray;

typedef struct NTransform {
    NVector3 position;
    NQuaternion rotation;
    NVector3 scale;
} NTransform;

typedef void* InputStatePtr;

typedef struct IndexNativeArray {
    IndexNative* values;
    size_t length;
    size_t capacity;
} IndexNativeArray;

typedef struct CharacterCollisionArray {
    uint64_t entity_id;
    IndexNativeArray collisions;
} CharacterCollisionArray;

typedef struct NShapeCastHit {
    NCollider collider;
    double distance;
    NVector3 witness1;
    NVector3 witness2;
    NVector3 normal1;
    NVector3 normal2;
    NShapeCastStatus status;
} NShapeCastHit;

typedef struct AxisLock {
    bool x;
    bool y;
    bool z;
} AxisLock;

typedef void* CommandBufferPtr;

typedef struct NVector4 {
    double x;
    double y;
    double z;
    double w;
} NVector4;

typedef struct NMaterial {
    const char* name;
    uint64_t diffuse_texture;
    uint64_t normal_texture;
    NVector4 tint;
    NVector3 emissive_factor;
    float metallic_factor;
    float roughness_factor;
    const float* alpha_cutoff;
    bool double_sided;
    float occlusion_strength;
    float normal_scale;
    NVector2 uv_tiling;
    const uint64_t* emissive_texture;
    const uint64_t* metallic_roughness_texture;
    const uint64_t* occlusion_texture;
} NMaterial;

typedef struct NMaterialArray {
    NMaterial* values;
    size_t length;
    size_t capacity;
} NMaterialArray;

typedef struct i32Array {
    int32_t* values;
    size_t length;
    size_t capacity;
} i32Array;

typedef struct f64ArrayArray {
    double* values;
    size_t length;
    size_t capacity;
} f64ArrayArray;

typedef struct NSkin {
    const char* name;
    i32Array joints;
    f64ArrayArray inverse_bind_matrices;
    const int32_t* skeleton_root;
} NSkin;

typedef struct NSkinArray {
    NSkin* values;
    size_t length;
    size_t capacity;
} NSkinArray;

typedef struct f64Array {
    double* values;
    size_t length;
    size_t capacity;
} f64Array;

typedef struct NAnimationChannel {
    int32_t target_node;
    f64Array times;
    NChannelValues values;
    NAnimationInterpolation interpolation;
} NAnimationChannel;

typedef struct NAnimationChannelArray {
    NAnimationChannel* values;
    size_t length;
    size_t capacity;
} NAnimationChannelArray;

typedef struct NAnimation {
    const char* name;
    NAnimationChannelArray channels;
    float duration;
} NAnimation;

typedef struct NAnimationArray {
    NAnimation* values;
    size_t length;
    size_t capacity;
} NAnimationArray;

typedef struct NColour {
    uint8_t r;
    uint8_t g;
    uint8_t b;
    uint8_t a;
} NColour;

typedef void* PhysicsStatePtr;

typedef struct NNodeTransform {
    NVector3 translation;
    NQuaternion rotation;
    NVector3 scale;
} NNodeTransform;

typedef struct NNode {
    const char* name;
    const int32_t* parent;
    i32Array children;
    NNodeTransform transform;
} NNode;

typedef struct NNodeArray {
    NNode* values;
    size_t length;
    size_t capacity;
} NNodeArray;

typedef void* GraphicsContextPtr;

typedef struct NModelVertex {
    NVector3 position;
    NVector3 normal;
    NVector4 tangent;
    NVector2 tex_coords0;
    NVector2 tex_coords1;
    NVector4 colour0;
    i32Array joints0;
    NVector4 weights0;
} NModelVertex;

typedef struct NModelVertexArray {
    NModelVertex* values;
    size_t length;
    size_t capacity;
} NModelVertexArray;

typedef struct NMesh {
    const char* name;
    int32_t num_elements;
    int32_t material_index;
    NModelVertexArray vertices;
} NMesh;

typedef struct NMeshArray {
    NMesh* values;
    size_t length;
    size_t capacity;
} NMeshArray;

typedef void* WorldPtr;

typedef struct RayHit {
    NCollider collider;
    double distance;
} RayHit;

typedef struct RigidBodyContext {
    IndexNative index;
    uint64_t entity_id;
} RigidBodyContext;

int32_t dropbear_gamepad_is_button_pressed(InputStatePtr input, uint64_t gamepad_id, int32_t button_ordinal, bool* out0);
int32_t dropbear_gamepad_get_left_stick_position(InputStatePtr input, uint64_t gamepad_id, NVector2* out0);
int32_t dropbear_gamepad_get_right_stick_position(InputStatePtr input, uint64_t gamepad_id, NVector2* out0);
int32_t dropbear_collider_group_exists_for_entity(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_collider_group_get_colliders(WorldPtr world, PhysicsStatePtr physics, uint64_t entity, NColliderArray* out0);
int32_t dropbear_character_collision_get_character_collision_collider(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NCollider* out0);
int32_t dropbear_character_collision_get_character_collision_position(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NTransform* out0);
int32_t dropbear_character_collision_get_character_collision_translation_applied(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NVector3* out0);
int32_t dropbear_character_collision_get_character_collision_translation_remaining(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NVector3* out0);
int32_t dropbear_character_collision_get_character_collision_time_of_impact(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, double* out0);
int32_t dropbear_character_collision_get_character_collision_witness1(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NVector3* out0);
int32_t dropbear_character_collision_get_character_collision_witness2(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NVector3* out0);
int32_t dropbear_character_collision_get_character_collision_normal1(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NVector3* out0);
int32_t dropbear_character_collision_get_character_collision_normal2(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NVector3* out0);
int32_t dropbear_character_collision_get_character_collision_status(WorldPtr world, uint64_t entity, const IndexNative* collision_handle, NShapeCastStatus* out0);
int32_t dropbear_collider_get_collider_shape(PhysicsStatePtr physics, const NCollider* collider, ColliderShape* out0);
int32_t dropbear_collider_set_collider_shape(PhysicsStatePtr physics, const NCollider* collider, const ColliderShape* shape);
int32_t dropbear_collider_get_collider_density(PhysicsStatePtr physics, const NCollider* collider, double* out0);
int32_t dropbear_collider_set_collider_density(PhysicsStatePtr physics, const NCollider* collider, double density);
int32_t dropbear_collider_get_collider_friction(PhysicsStatePtr physics, const NCollider* collider, double* out0);
int32_t dropbear_collider_set_collider_friction(PhysicsStatePtr physics, const NCollider* collider, double friction);
int32_t dropbear_collider_get_collider_restitution(PhysicsStatePtr physics, const NCollider* collider, double* out0);
int32_t dropbear_collider_set_collider_restitution(PhysicsStatePtr physics, const NCollider* collider, double restitution);
int32_t dropbear_collider_get_collider_mass(PhysicsStatePtr physics, const NCollider* collider, double* out0);
int32_t dropbear_collider_set_collider_mass(PhysicsStatePtr physics, const NCollider* collider, double mass);
int32_t dropbear_collider_get_collider_is_sensor(PhysicsStatePtr physics, const NCollider* collider, bool* out0);
int32_t dropbear_collider_set_collider_is_sensor(PhysicsStatePtr physics, const NCollider* collider, bool is_sensor);
int32_t dropbear_collider_get_collider_translation(PhysicsStatePtr physics, const NCollider* collider, NVector3* out0);
int32_t dropbear_collider_set_collider_translation(PhysicsStatePtr physics, const NCollider* collider, const NVector3* translation);
int32_t dropbear_collider_get_collider_rotation(PhysicsStatePtr physics, const NCollider* collider, NVector3* out0);
int32_t dropbear_collider_set_collider_rotation(PhysicsStatePtr physics, const NCollider* collider, const NVector3* rotation);
int32_t dropbear_rigidbody_rigid_body_exists_for_entity(WorldPtr world, PhysicsStatePtr physics, uint64_t entity, IndexNative* out0, bool* out0_present);
int32_t dropbear_rigidbody_get_rigidbody_mode(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, int32_t* out0);
int32_t dropbear_rigidbody_set_rigidbody_mode(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, int32_t mode);
int32_t dropbear_rigidbody_get_rigidbody_gravity_scale(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double* out0);
int32_t dropbear_rigidbody_set_rigidbody_gravity_scale(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double gravity_scale);
int32_t dropbear_rigidbody_get_rigidbody_linear_damping(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double* out0);
int32_t dropbear_rigidbody_set_rigidbody_linear_damping(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double linear_damping);
int32_t dropbear_rigidbody_get_rigidbody_angular_damping(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double* out0);
int32_t dropbear_rigidbody_set_rigidbody_angular_damping(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double angular_damping);
int32_t dropbear_rigidbody_get_rigidbody_sleep(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, bool* out0);
int32_t dropbear_rigidbody_set_rigidbody_sleep(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, bool sleep);
int32_t dropbear_rigidbody_get_rigidbody_ccd_enabled(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, bool* out0);
int32_t dropbear_rigidbody_set_rigidbody_ccd_enabled(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, bool ccd_enabled);
int32_t dropbear_rigidbody_get_rigidbody_linear_velocity(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, NVector3* out0);
int32_t dropbear_rigidbody_set_rigidbody_linear_velocity(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, const NVector3* linear_velocity);
int32_t dropbear_rigidbody_get_rigidbody_angular_velocity(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, NVector3* out0);
int32_t dropbear_rigidbody_set_rigidbody_angular_velocity(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, const NVector3* angular_velocity);
int32_t dropbear_rigidbody_get_rigidbody_lock_translation(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, AxisLock* out0);
int32_t dropbear_rigidbody_set_rigidbody_lock_translation(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, const AxisLock* lock_translation);
int32_t dropbear_rigidbody_get_rigidbody_lock_rotation(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, AxisLock* out0);
int32_t dropbear_rigidbody_set_rigidbody_lock_rotation(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, const AxisLock* lock_rotation);
int32_t dropbear_rigidbody_get_rigidbody_children(WorldPtr _world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, NColliderArray* out0);
int32_t dropbear_rigidbody_apply_impulse(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double x, double y, double z);
int32_t dropbear_rigidbody_apply_torque_impulse(WorldPtr world, PhysicsStatePtr physics, const RigidBodyContext* rigidbody, double x, double y, double z);
int32_t dropbear_kcc_kcc_exists_for_entity(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_kcc_move_character(WorldPtr world, PhysicsStatePtr physics_state, uint64_t entity, const NVector3* translation, double delta_time);
int32_t dropbear_kcc_set_rotation(WorldPtr world, PhysicsStatePtr physics_state, uint64_t entity, const NQuaternion* rotation);
int32_t dropbear_kcc_get_hit(WorldPtr world, uint64_t entity, CharacterCollisionArray* out0);
int32_t dropbear_scene_get_scene_load_progress(SceneLoaderPtr scene_loader, uint64_t scene_id, Progress* out0);
int32_t dropbear_camera_exists_for_entity(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_camera_get_eye(WorldPtr world, uint64_t entity, NVector3* out0);
int32_t dropbear_camera_set_eye(WorldPtr world, uint64_t entity, const NVector3* eye);
int32_t dropbear_camera_get_target(WorldPtr world, uint64_t entity, NVector3* out0);
int32_t dropbear_camera_set_target(WorldPtr world, uint64_t entity, const NVector3* target);
int32_t dropbear_camera_get_up(WorldPtr world, uint64_t entity, NVector3* out0);
int32_t dropbear_camera_set_up(WorldPtr world, uint64_t entity, const NVector3* up);
int32_t dropbear_camera_get_aspect(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_get_fovy(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_set_fovy(WorldPtr world, uint64_t entity, double fovy);
int32_t dropbear_camera_get_znear(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_set_znear(WorldPtr world, uint64_t entity, double znear);
int32_t dropbear_camera_get_zfar(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_set_zfar(WorldPtr world, uint64_t entity, double zfar);
int32_t dropbear_camera_get_yaw(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_set_yaw(WorldPtr world, uint64_t entity, double yaw);
int32_t dropbear_camera_get_pitch(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_set_pitch(WorldPtr world, uint64_t entity, double pitch);
int32_t dropbear_camera_get_speed(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_set_speed(WorldPtr world, uint64_t entity, double speed);
int32_t dropbear_camera_get_sensitivity(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_camera_set_sensitivity(WorldPtr world, uint64_t entity, double sensitivity);
int32_t dropbear_entity_label_exists_for_entity(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_entity_get_label(WorldPtr world, uint64_t entity, char** out0);
int32_t dropbear_entity_get_children(WorldPtr world, uint64_t entity, u64Array* out0);
int32_t dropbear_entity_get_child_by_label(WorldPtr world, uint64_t entity, const char* target, uint64_t* out0, bool* out0_present);
int32_t dropbear_entity_get_parent(WorldPtr world, uint64_t entity, uint64_t* out0, bool* out0_present);
int32_t dropbear_input_print_input_state(InputStatePtr input);
int32_t dropbear_input_is_key_pressed(InputStatePtr input, int32_t key_code, bool* out0);
int32_t dropbear_input_get_mouse_position(InputStatePtr input, NVector2* out0);
int32_t dropbear_input_is_mouse_button_pressed(InputStatePtr input, int32_t button_ordinal, bool* out0);
int32_t dropbear_input_get_mouse_delta(InputStatePtr input, NVector2* out0);
int32_t dropbear_input_is_cursor_locked(InputStatePtr input, bool* out0);
int32_t dropbear_input_set_cursor_locked(CommandBufferPtr command_buffer, InputStatePtr input, bool locked);
int32_t dropbear_input_get_last_mouse_pos(InputStatePtr input, NVector2* out0);
int32_t dropbear_input_is_cursor_hidden(InputStatePtr input, bool* out0);
int32_t dropbear_input_set_cursor_hidden(CommandBufferPtr command_buffer, InputStatePtr input, bool hidden);
int32_t dropbear_input_get_connected_gamepads(InputStatePtr input, ConnectedGamepadIds* out0);
int32_t dropbear_lighting_light_exists_for_entity(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_lighting_get_position(WorldPtr world, uint64_t entity, NVector3* out0);
int32_t dropbear_lighting_set_position(WorldPtr world, uint64_t entity, const NVector3* position);
int32_t dropbear_lighting_get_direction(WorldPtr world, uint64_t entity, NVector3* out0);
int32_t dropbear_lighting_set_direction(WorldPtr world, uint64_t entity, const NVector3* direction);
int32_t dropbear_lighting_get_colour(WorldPtr world, uint64_t entity, NColour* out0);
int32_t dropbear_lighting_set_colour(WorldPtr world, uint64_t entity, const NColour* colour);
int32_t dropbear_lighting_get_light_type(WorldPtr world, uint64_t entity, int32_t* out0);
int32_t dropbear_lighting_set_light_type(WorldPtr world, uint64_t entity, int32_t light_type);
int32_t dropbear_lighting_get_intensity(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_lighting_set_intensity(WorldPtr world, uint64_t entity, double intensity);
int32_t dropbear_lighting_get_attenuation(WorldPtr world, uint64_t entity, NAttenuation* out0);
int32_t dropbear_lighting_set_attenuation(WorldPtr world, uint64_t entity, const NAttenuation* attenuation);
int32_t dropbear_lighting_get_enabled(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_lighting_set_enabled(WorldPtr world, uint64_t entity, bool enabled);
int32_t dropbear_lighting_get_cutoff_angle(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_lighting_set_cutoff_angle(WorldPtr world, uint64_t entity, double cutoff_angle);
int32_t dropbear_lighting_get_outer_cutoff_angle(WorldPtr world, uint64_t entity, double* out0);
int32_t dropbear_lighting_set_outer_cutoff_angle(WorldPtr world, uint64_t entity, double outer_cutoff_angle);
int32_t dropbear_lighting_get_casts_shadows(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_lighting_set_casts_shadows(WorldPtr world, uint64_t entity, bool casts_shadows);
int32_t dropbear_lighting_get_depth(WorldPtr world, uint64_t entity, NRange* out0);
int32_t dropbear_lighting_set_depth(WorldPtr world, uint64_t entity, const NRange* depth);
int32_t dropbear_physics_get_gravity(PhysicsStatePtr physics, NVector3* out0);
int32_t dropbear_physics_set_gravity(PhysicsStatePtr physics, const NVector3* gravity);
int32_t dropbear_physics_raycast(PhysicsStatePtr physics, const NVector3* origin, const NVector3* dir, double time_of_impact, bool solid, RayHit* out0, bool* out0_present);
int32_t dropbear_physics_shape_cast(PhysicsStatePtr physics, const NVector3* origin, const NVector3* direction, const ColliderShape* shape, double time_of_impact, bool solid, NShapeCastHit* out0, bool* out0_present);
int32_t dropbear_physics_is_overlapping(PhysicsStatePtr physics, const NCollider* collider1, const NCollider* collider2, bool* out0);
int32_t dropbear_physics_is_triggering(PhysicsStatePtr physics, const NCollider* collider1, const NCollider* collider2, bool* out0);
int32_t dropbear_physics_is_touching(PhysicsStatePtr physics, uint64_t entity1, uint64_t entity2, bool* out0);
int32_t dropbear_properties_custom_properties_exists_for_entity(WorldPtr world, uint64_t entity, bool* out0);
int32_t dropbear_properties_get_string_property(WorldPtr world, uint64_t entity, const char* key, char** out0, bool* out0_present);
int32_t dropbear_properties_get_int_property(WorldPtr world, uint64_t entity, const char* key, int32_t* out0, bool* out0_present);
int32_t dropbear_properties_get_long_property(WorldPtr world, uint64_t entity, const char* key, int64_t* out0, bool* out0_present);
int32_t dropbear_properties_get_double_property(WorldPtr world, uint64_t entity, const char* key, double* out0, bool* out0_present);
int32_t dropbear_properties_get_float_property(WorldPtr world, uint64_t entity, const char* key, float* out0, bool* out0_present);
int32_t dropbear_properties_get_bool_property(WorldPtr world, uint64_t entity, const char* key, bool* out0, bool* out0_present);
int32_t dropbear_properties_get_vec3_property(WorldPtr world, uint64_t entity, const char* key, NVector3* out0, bool* out0_present);
int32_t dropbear_properties_set_string_property(WorldPtr world, uint64_t entity, const char* key, const char* value);
int32_t dropbear_properties_set_int_property(WorldPtr world, uint64_t entity, const char* key, int32_t value);
int32_t dropbear_properties_set_long_property(WorldPtr world, uint64_t entity, const char* key, int64_t value);
int32_t dropbear_properties_set_double_property(WorldPtr world, uint64_t entity, const char* key, double value);
int32_t dropbear_properties_set_float_property(WorldPtr world, uint64_t entity, const char* key, double value);
int32_t dropbear_properties_set_bool_property(WorldPtr world, uint64_t entity, const char* key, bool value);
int32_t dropbear_properties_set_vec3_property(WorldPtr world, uint64_t entity, const char* key, const NVector3* value);
int32_t dropbear_engine_get_entity(WorldPtr world, const char* label, uint64_t* out0);
int32_t dropbear_engine_quit(CommandBufferPtr command_buffer);
int32_t dropbear_engine_get_asset(AssetRegistryPtr asset, const char* label, const AssetKind* kind, uint64_t* out0, bool* out0_present);
int32_t dropbear_asset_model_get_label(AssetRegistryPtr asset, uint64_t model_handle, char** out0);
int32_t dropbear_asset_model_get_meshes(AssetRegistryPtr asset, uint64_t model_handle, NMeshArray* out0);
int32_t dropbear_asset_model_get_materials(AssetRegistryPtr asset, uint64_t model_handle, NMaterialArray* out0);
int32_t dropbear_asset_model_get_skins(AssetRegistryPtr asset, uint64_t model_handle, NSkinArray* out0);
int32_t dropbear_asset_model_get_animations(AssetRegistryPtr asset, uint64_t model_handle, NAnimationArray* out0);
int32_t dropbear_asset_model_get_nodes(AssetRegistryPtr asset, uint64_t model_handle, NNodeArray* out0);
int32_t dropbear_asset_texture_get_label(AssetRegistryPtr asset_manager, uint64_t texture_handle, char** out0, bool* out0_present);
int32_t dropbear_asset_texture_get_width(AssetRegistryPtr asset_manager, uint64_t texture_handle, uint32_t* out0);
int32_t dropbear_asset_texture_get_height(AssetRegistryPtr asset_manager, uint64_t texture_handle, uint32_t* out0);
int32_t dropbear_mesh_get_texture(WorldPtr world, AssetRegistryPtr asset, uint64_t entity, const char* material_name, uint64_t* out0, bool* out0_present);
int32_t dropbear_mesh_set_texture_override(WorldPtr world, AssetRegistryPtr asset, uint64_t entity, const char* material_name, uint64_t texture_handle);
int32_t dropbear_mesh_set_material_tint(WorldPtr world, AssetRegistryPtr asset, GraphicsContextPtr graphics, uint64_t entity, const char* material_name, float r, float g, float b, float a);

#endif /* DROPBEAR_H */
