/**
 * dropbear-engine native header definitions. Created by tirbofish as part of the dropbear project. 
 * 
 * Primarily used for Kotlin/Native, however nothing is stopping you from implementing it to your own language. 
 * Exports are located at `eucalyptus_core::scripting::native::exports`. 
 * 
 * Note: This does not include JNI definitions, only native exports from the eucalyptus-core dynamic library. 
 *       For JNI definitions, take a look at `eucalyptus_core::scripting::jni::exports` or even better, take a 
 *       look at the JNINative class for all JNI functions that exist. 
 * 
 * Warning: This header file is not always up to date with the existing JNI functions (some funcs may not be implemented),
 *          So please open a issue if there is something missing, or help us by creating a PR implementing them. 
 * 
 * Licensed under MIT or Apache 2.0 depending on your mood. 
 */

#ifndef DROPBEAR_H
#define DROPBEAR_H

/**
* @brief Return function for a DropbearNativeError. 0 on success, otherwise look
*        at `eucalyptus_core::scripting::native::DropbearNativeError`
*/
#define DROPBEAR_NATIVE int

/**
* @brief The handle/id of an object, as a long. Kotlin/Native requires
*        me to have an int64_t as a Long (or use a long long).
*/
#define HANDLE int64_t

#define BOOL int // either as 0 or 1

#include <stddef.h>
#include <stdint.h>

// ===========================================

typedef struct World World; // opaque pointer
typedef struct InputState InputState; // opaque pointer
typedef struct GraphicsCommandQueue GraphicsCommandQueue; // opaque pointer
typedef struct AssetRegistry AssetRegistry; // opaque pointer

// ===========================================

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

// ===========================================

typedef struct {
    float x;
    float y;
    float z;
} Vector3D;

typedef struct {
    double position_x;
    double position_y;
    double position_z;
    double rotation_x;
    double rotation_y;
    double rotation_z;
    double rotation_w;
    double scale_x;
    double scale_y;
    double scale_z;
} NativeTransform;

typedef struct {
    NativeTransform local;
    NativeTransform world;
} NativeEntityTransform;

typedef struct {
    const char* label;
    int64_t entity_id;

    Vector3D eye;
    Vector3D target;
    Vector3D up;

    double aspect;
    double fov_y;
    double znear;
    double zfar;

    double yaw;
    double pitch;
    double speed;
    double sensitivity;
} NativeCamera;

// ===========================================

// getters
DROPBEAR_NATIVE dropbear_get_entity(const char* label, const World* world_ptr, int64_t* out_entity);
DROPBEAR_NATIVE dropbear_get_asset(const AssetRegistry* asset_ptr, const char* label, HANDLE* out_asset_id);

// entity
DROPBEAR_NATIVE dropbear_get_entity_name(const World* world_ptr, HANDLE entity_id, char* out_name, size_t max_len);

// model
DROPBEAR_NATIVE dropbear_get_model(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, HANDLE* out_model_id);
DROPBEAR_NATIVE dropbear_set_model(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, HANDLE model_id);
DROPBEAR_NATIVE dropbear_is_model_handle(const AssetRegistry* asset_ptr, HANDLE handle, BOOL* out_is_model);
DROPBEAR_NATIVE dropbear_is_using_model(const World* world_ptr, HANDLE entity_handle, HANDLE model_handle, BOOL* out_is_using);

// texture
DROPBEAR_NATIVE dropbear_get_texture(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, const char* name, HANDLE* out_texture_id);
DROPBEAR_NATIVE dropbear_get_texture_name(const AssetRegistry* asset_ptr, HANDLE texture_handle, const char** out_name);
DROPBEAR_NATIVE dropbear_set_texture(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, const char* old_material_name, HANDLE texture_id);
DROPBEAR_NATIVE dropbear_is_texture_handle(const AssetRegistry* asset_ptr, HANDLE handle, BOOL* out_is_texture);
DROPBEAR_NATIVE dropbear_is_using_texture(const World* world_ptr, HANDLE entity_handle, HANDLE texture_handle, BOOL* out_is_using);
DROPBEAR_NATIVE dropbear_get_all_textures(const World* world_ptr, HANDLE entity_handle, const char*** out_textures, size_t* out_count);

// camera
DROPBEAR_NATIVE dropbear_get_camera(const World* world_ptr, const char* label, NativeCamera* out_camera);
DROPBEAR_NATIVE dropbear_get_attached_camera(const World* world_ptr, HANDLE entity_handle, NativeCamera* out_camera);
DROPBEAR_NATIVE dropbear_set_camera(const World* world_ptr, NativeCamera camera);

// transformations
DROPBEAR_NATIVE dropbear_get_transform(const World* world_ptr, HANDLE entity_handle, NativeEntityTransform* out_transform);
DROPBEAR_NATIVE dropbear_propagate_transform(const World* world_ptr, HANDLE entity_id, NativeTransform* out_transform);
DROPBEAR_NATIVE dropbear_set_transform(const World* world_ptr, HANDLE entity_id, NativeEntityTransform transform);

// hierarchy
DROPBEAR_NATIVE dropbear_get_children(const World* world_ptr, HANDLE entity_id, HANDLE** out_children, size_t* out_count);
DROPBEAR_NATIVE dropbear_get_child_by_label(const World* world_ptr, HANDLE entity_id, const char* label, HANDLE* out_child);
DROPBEAR_NATIVE dropbear_get_parent(const World* world_ptr, HANDLE entity_id, HANDLE* out_parent);

// properties - getters
DROPBEAR_NATIVE dropbear_get_string_property(const World* world_ptr, HANDLE entity_handle, const char* label, const char** out_value);
DROPBEAR_NATIVE dropbear_get_int_property(const World* world_ptr, HANDLE entity_handle, const char* label, int32_t* out_value);
DROPBEAR_NATIVE dropbear_get_long_property(const World* world_ptr, HANDLE entity_handle, const char* label, int64_t* out_value);
DROPBEAR_NATIVE dropbear_get_float_property(const World* world_ptr, HANDLE entity_handle, const char* label, double* out_value);
DROPBEAR_NATIVE dropbear_get_bool_property(const World* world_ptr, HANDLE entity_handle, const char* label, BOOL* out_value);
DROPBEAR_NATIVE dropbear_get_vec3_property(const World* world_ptr, HANDLE entity_handle, const char* label, Vector3D* out_value);

// properties - setters
DROPBEAR_NATIVE dropbear_set_string_property(const World* world_ptr, HANDLE entity_handle, const char* label, const char* value);
DROPBEAR_NATIVE dropbear_set_int_property(const World* world_ptr, HANDLE entity_handle, const char* label, int32_t value);
DROPBEAR_NATIVE dropbear_set_long_property(const World* world_ptr, HANDLE entity_handle, const char* label, int64_t value);
DROPBEAR_NATIVE dropbear_set_float_property(const World* world_ptr, HANDLE entity_handle, const char* label, double value);
DROPBEAR_NATIVE dropbear_set_bool_property(const World* world_ptr, HANDLE entity_handle, const char* label, BOOL value);
DROPBEAR_NATIVE dropbear_set_vec3_property(const World* world_ptr, HANDLE entity_handle, const char* label, Vector3D value);

// input
DROPBEAR_NATIVE dropbear_print_input_state(const InputState* input_ptr);
DROPBEAR_NATIVE dropbear_is_key_pressed(const InputState* input_ptr, int32_t key_ordinal, BOOL* out_pressed);
DROPBEAR_NATIVE dropbear_get_mouse_position(const InputState* input_ptr, float* out_x, float* out_y);
DROPBEAR_NATIVE dropbear_is_mouse_button_pressed(const InputState* input_ptr, int32_t button_ordinal, BOOL* out_pressed);
DROPBEAR_NATIVE dropbear_get_mouse_delta(const InputState* input_ptr, float* out_dx, float* out_dy);
DROPBEAR_NATIVE dropbear_is_cursor_locked(const InputState* input_ptr, BOOL* out_locked);
DROPBEAR_NATIVE dropbear_set_cursor_locked(InputState* input_ptr, GraphicsCommandQueue* graphics_ptr, BOOL locked);
DROPBEAR_NATIVE dropbear_get_last_mouse_pos(const InputState* input_ptr, float* out_x, float* out_y);
DROPBEAR_NATIVE dropbear_is_cursor_hidden(const InputState* input_ptr, BOOL* out_hidden);
DROPBEAR_NATIVE dropbear_set_cursor_hidden(InputState* input_ptr, GraphicsCommandQueue* graphics_ptr, BOOL hidden);

// editor
void dropbear_quit(const GraphicsCommandQueue* command_ptr);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_H