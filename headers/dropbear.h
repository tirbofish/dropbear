#ifndef DROPBEAR_H
#define DROPBEAR_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum dropbear_ColliderShapeType {
    Box = 0,
    Sphere = 1,
    Capsule = 2,
    Cylinder = 3,
    Cone = 4,
} dropbear_ColliderShapeType;

typedef struct dropbear_CommandBuffer dropbear_CommandBuffer;

/**
 * Shows the information about the input at that current time.
 */
typedef struct dropbear_InputState dropbear_InputState;

/**
 * A serializable [rapier3d] state that shows all the different actions and types related
 * to physics rendering.
 */
typedef struct dropbear_PhysicsState dropbear_PhysicsState;

/**
 * A mutable pointer to a [`World`].
 *
 * Defined in `dropbear_common.h` as `World`
 */
typedef dropbear_World *dropbear_WorldPtr;

typedef struct dropbear_Vector3 {
    double x;
    double y;
    double z;
} dropbear_Vector3;

/**
 * A mutable pointer to an [`InputState`].
 *
 * Defined in `dropbear_common.h` as `InputState`
 */
typedef struct dropbear_InputState *dropbear_InputStatePtr;

typedef struct dropbear_Vector2 {
    double x;
    double y;
} dropbear_Vector2;

/**
 * A non-mutable pointer to a [`crossbeam_channel::Sender`] that sends
 * [`CommandBuffer`] signals.
 *
 * Defined in `dropbear_common.h` as `CommandBuffer`
 */
typedef const dropbear_Sender<dropbear_CommandBuffer> *dropbear_CommandBufferPtr;

/**
 * A mutable pointer to a [`PhysicsState`].
 *
 * Defined in `dropbear_common.h` as `PhysicsEngine`
 */
typedef struct dropbear_PhysicsState *dropbear_PhysicsStatePtr;

typedef struct dropbear_IndexNative {
    uint32_t index;
    uint32_t generation;
} dropbear_IndexNative;

typedef struct dropbear_ColliderFFI {
    struct dropbear_IndexNative index;
    uint64_t entity_id;
    uint32_t id;
} dropbear_ColliderFFI;

typedef struct dropbear_ColliderShapeFFI {
    enum dropbear_ColliderShapeType shape_type;
    float radius;
    float half_height;
    float half_extents_x;
    float half_extents_y;
    float half_extents_z;
} dropbear_ColliderShapeFFI;

/**
 * A non-mutable pointer to the [`AssetRegistry`].
 *
 * Defined in `dropbear_common.h` as `AssetRegistry`
 */
typedef const dropbear_AssetRegistry *dropbear_AssetRegistryPtr;

const uint8_t *get_rustc_version(void);

int32_t dropbear_camera_exists_for_entity(dropbear_WorldPtr world_ptr,
                                          uint64_t entity_id,
                                          bool *out_result);

int32_t dropbear_get_camera_eye(dropbear_WorldPtr world_ptr,
                                uint64_t entity_id,
                                struct dropbear_Vector3 *out_result);

int32_t dropbear_set_camera_eye(dropbear_WorldPtr world_ptr,
                                uint64_t entity_id,
                                struct dropbear_Vector3 value);

int32_t dropbear_get_camera_target(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   struct dropbear_Vector3 *out_result);

int32_t dropbear_set_camera_target(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   struct dropbear_Vector3 value);

int32_t dropbear_get_camera_up(dropbear_WorldPtr world_ptr,
                               uint64_t entity_id,
                               struct dropbear_Vector3 *out_result);

int32_t dropbear_set_camera_up(dropbear_WorldPtr world_ptr,
                               uint64_t entity_id,
                               struct dropbear_Vector3 value);

int32_t dropbear_get_camera_aspect(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   double *out_result);

int32_t dropbear_get_camera_fov_y(dropbear_WorldPtr world_ptr,
                                  uint64_t entity_id,
                                  double *out_result);

int32_t dropbear_set_camera_fov_y(dropbear_WorldPtr world_ptr, uint64_t entity_id, double value);

int32_t dropbear_get_camera_znear(dropbear_WorldPtr world_ptr,
                                  uint64_t entity_id,
                                  double *out_result);

int32_t dropbear_set_camera_znear(dropbear_WorldPtr world_ptr, uint64_t entity_id, double value);

int32_t dropbear_get_camera_zfar(dropbear_WorldPtr world_ptr,
                                 uint64_t entity_id,
                                 double *out_result);

int32_t dropbear_set_camera_zfar(dropbear_WorldPtr world_ptr, uint64_t entity_id, double value);

int32_t dropbear_get_camera_yaw(dropbear_WorldPtr world_ptr,
                                uint64_t entity_id,
                                double *out_result);

int32_t dropbear_set_camera_yaw(dropbear_WorldPtr world_ptr, uint64_t entity_id, double value);

int32_t dropbear_get_camera_pitch(dropbear_WorldPtr world_ptr,
                                  uint64_t entity_id,
                                  double *out_result);

int32_t dropbear_set_camera_pitch(dropbear_WorldPtr world_ptr, uint64_t entity_id, double value);

int32_t dropbear_get_camera_speed(dropbear_WorldPtr world_ptr,
                                  uint64_t entity_id,
                                  double *out_result);

int32_t dropbear_set_camera_speed(dropbear_WorldPtr world_ptr, uint64_t entity_id, double value);

int32_t dropbear_get_camera_sensitivity(dropbear_WorldPtr world_ptr,
                                        uint64_t entity_id,
                                        double *out_result);

int32_t dropbear_set_camera_sensitivity(dropbear_WorldPtr world_ptr,
                                        uint64_t entity_id,
                                        double value);

int32_t dropbear_is_gamepad_button_pressed(dropbear_InputStatePtr input_ptr,
                                           uint64_t gamepad_id,
                                           int32_t button_ordinal,
                                           bool *out_result);

int32_t dropbear_get_left_stick_position(dropbear_InputStatePtr input_ptr,
                                         uint64_t gamepad_id,
                                         struct dropbear_Vector2 *out_result);

int32_t dropbear_get_right_stick_position(dropbear_InputStatePtr input_ptr,
                                          uint64_t gamepad_id,
                                          struct dropbear_Vector2 *out_result);

int32_t dropbear_free_gamepads_array(uint64_t *ptr, uintptr_t count);

int32_t dropbear_print_input_state(dropbear_InputStatePtr input_ptr);

int32_t dropbear_is_key_pressed(dropbear_InputStatePtr input_ptr,
                                int32_t key_ordinal,
                                bool *out_result);

int32_t dropbear_get_mouse_position(dropbear_InputStatePtr input_ptr,
                                    struct dropbear_Vector2 *out_result);

int32_t dropbear_is_mouse_button_pressed(dropbear_InputStatePtr input_ptr,
                                         int32_t btn_ordinal,
                                         bool *out_result);

int32_t dropbear_get_mouse_delta(dropbear_InputStatePtr input_ptr,
                                 struct dropbear_Vector2 *out_result);

int32_t dropbear_is_cursor_locked(dropbear_InputStatePtr input_ptr, bool *out_result);

int32_t dropbear_set_cursor_locked(dropbear_CommandBufferPtr cmd_ptr,
                                   dropbear_InputStatePtr input_ptr,
                                   bool locked);

int32_t dropbear_get_last_mouse_pos(dropbear_InputStatePtr input_ptr,
                                    struct dropbear_Vector2 *out_result);

int32_t dropbear_is_cursor_hidden(dropbear_InputStatePtr input_ptr, bool *out_result);

int32_t dropbear_set_cursor_hidden(dropbear_CommandBufferPtr cmd_ptr,
                                   dropbear_InputStatePtr input_ptr,
                                   bool hidden);

int32_t dropbear_get_connected_gamepads(dropbear_InputStatePtr input_ptr,
                                        uintptr_t *out_count,
                                        uint64_t **out_result);

int32_t dropbear_free_string(char *ptr);

int32_t dropbear_collider_group_exists_for_entity(dropbear_WorldPtr world_ptr,
                                                  uint64_t entity_id,
                                                  bool *out_result);

int32_t dropbear_get_collider_group_colliders(dropbear_WorldPtr world_ptr,
                                              dropbear_PhysicsStatePtr physics_ptr,
                                              uint64_t entity_id,
                                              uintptr_t *out_count,
                                              struct dropbear_ColliderFFI **out_result);

int32_t dropbear_free_collider_array(struct dropbear_ColliderFFI *ptr, uintptr_t count);

int32_t dropbear_get_collider_shape(dropbear_PhysicsStatePtr physics_ptr,
                                    struct dropbear_ColliderFFI ffi,
                                    struct dropbear_ColliderShapeFFI *out_result);

int32_t dropbear_set_collider_shape(dropbear_PhysicsStatePtr physics_ptr,
                                    struct dropbear_ColliderFFI ffi,
                                    struct dropbear_ColliderShapeFFI shape);

int32_t dropbear_get_collider_density(dropbear_PhysicsStatePtr physics_ptr,
                                      struct dropbear_ColliderFFI ffi,
                                      double *out_result);

int32_t dropbear_set_collider_density(dropbear_PhysicsStatePtr physics_ptr,
                                      struct dropbear_ColliderFFI ffi,
                                      double density);

int32_t dropbear_get_collider_friction(dropbear_PhysicsStatePtr physics_ptr,
                                       struct dropbear_ColliderFFI ffi,
                                       double *out_result);

int32_t dropbear_set_collider_friction(dropbear_PhysicsStatePtr physics_ptr,
                                       struct dropbear_ColliderFFI ffi,
                                       double friction);

int32_t dropbear_get_collider_restitution(dropbear_PhysicsStatePtr physics_ptr,
                                          struct dropbear_ColliderFFI ffi,
                                          double *out_result);

int32_t dropbear_set_collider_restitution(dropbear_PhysicsStatePtr physics_ptr,
                                          struct dropbear_ColliderFFI ffi,
                                          double restitution);

int32_t dropbear_get_collider_mass(dropbear_PhysicsStatePtr physics_ptr,
                                   struct dropbear_ColliderFFI ffi,
                                   double *out_result);

int32_t dropbear_set_collider_mass(dropbear_PhysicsStatePtr physics_ptr,
                                   struct dropbear_ColliderFFI ffi,
                                   double mass);

int32_t dropbear_get_collider_is_sensor(dropbear_PhysicsStatePtr physics_ptr,
                                        struct dropbear_ColliderFFI ffi,
                                        bool *out_result);

int32_t dropbear_set_collider_is_sensor(dropbear_PhysicsStatePtr physics_ptr,
                                        struct dropbear_ColliderFFI ffi,
                                        bool is_sensor);

int32_t dropbear_get_collider_translation(dropbear_PhysicsStatePtr physics_ptr,
                                          struct dropbear_ColliderFFI ffi,
                                          struct dropbear_Vector3 *out_result);

int32_t dropbear_set_collider_translation(dropbear_PhysicsStatePtr physics_ptr,
                                          struct dropbear_ColliderFFI ffi,
                                          struct dropbear_Vector3 translation);

int32_t dropbear_get_collider_rotation(dropbear_PhysicsStatePtr physics_ptr,
                                       struct dropbear_ColliderFFI ffi,
                                       struct dropbear_Vector3 *out_result);

int32_t dropbear_set_collider_rotation(dropbear_PhysicsStatePtr physics_ptr,
                                       struct dropbear_ColliderFFI ffi,
                                       struct dropbear_Vector3 rotation);

int32_t dropbear_get_texture_name(dropbear_AssetRegistryPtr asset_registry_ptr,
                                  uint64_t handle,
                                  char **out_result);

int32_t dropbear_is_model_handle(dropbear_AssetRegistryPtr asset_registry_ptr,
                                 uint64_t handle,
                                 bool *out_result);

int32_t dropbear_is_texture_handle(dropbear_AssetRegistryPtr asset_registry_ptr,
                                   uint64_t handle,
                                   bool *out_result);

int32_t dropbear_custom_properties_exists_for_entity(dropbear_WorldPtr world_ptr,
                                                     uint64_t entity_id,
                                                     bool *out_result);

int32_t dropbear_get_string_property(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     const char *key,
                                     char **out_result);

int32_t dropbear_get_int_property(dropbear_WorldPtr world_ptr,
                                  uint64_t entity_id,
                                  const char *key,
                                  int32_t *out_result);

int32_t dropbear_get_long_property(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   const char *key,
                                   int64_t *out_result);

int32_t dropbear_get_double_property(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     const char *key,
                                     double *out_result);

int32_t dropbear_get_float_property(dropbear_WorldPtr world_ptr,
                                    uint64_t entity_id,
                                    const char *key,
                                    float *out_result);

int32_t dropbear_get_bool_property(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   const char *key,
                                   bool *out_result);

int32_t dropbear_get_vec3_property(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   const char *key,
                                   struct dropbear_Vector3 *out_result);

int32_t dropbear_set_string_property(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     const char *key,
                                     const char *value);

int32_t dropbear_set_int_property(dropbear_WorldPtr world_ptr,
                                  uint64_t entity_id,
                                  const char *key,
                                  int32_t value);

int32_t dropbear_set_long_property(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   const char *key,
                                   int64_t value);

int32_t dropbear_set_double_property(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     const char *key,
                                     double value);

int32_t dropbear_set_float_property(dropbear_WorldPtr world_ptr,
                                    uint64_t entity_id,
                                    const char *key,
                                    float value);

int32_t dropbear_set_bool_property(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   const char *key,
                                   bool value);

int32_t dropbear_set_vec3_property(dropbear_WorldPtr world_ptr,
                                   uint64_t entity_id,
                                   const char *key,
                                   struct dropbear_Vector3 value);

int32_t dropbear_mesh_renderer_exists_for_entity(dropbear_WorldPtr world_ptr,
                                                 uint64_t entity_id,
                                                 bool *out_result);

int32_t dropbear_get_model(dropbear_WorldPtr world_ptr, uint64_t entity_id, uint64_t *out_result);

int32_t dropbear_set_model(dropbear_WorldPtr world_ptr,
                           dropbear_AssetRegistryPtr asset_ptr,
                           uint64_t entity_id,
                           uint64_t model_id);

int32_t dropbear_get_all_texture_ids(dropbear_WorldPtr world_ptr,
                                     dropbear_AssetRegistryPtr asset_ptr,
                                     uint64_t entity_id,
                                     uintptr_t *out_count,
                                     uint64_t **out_result);

int32_t dropbear_get_texture(dropbear_WorldPtr world_ptr,
                             dropbear_AssetRegistryPtr asset_ptr,
                             uint64_t entity_id,
                             const char *material_name,
                             uint64_t *out_result);

int32_t dropbear_set_texture_override(dropbear_WorldPtr world_ptr,
                                      dropbear_AssetRegistryPtr asset_ptr,
                                      uint64_t entity_id,
                                      const char *material_name,
                                      uint64_t texture_handle);

int32_t dropbear_get_entity(dropbear_WorldPtr world_ptr, const char *label, uint64_t *out_result);

int32_t dropbear_get_asset(dropbear_AssetRegistryPtr asset_ptr,
                           const char *uri,
                           uint64_t *out_result);

int32_t dropbear_quit(dropbear_CommandBufferPtr command_ptr);

int32_t dropbear_entity_transform_exists_for_entity(dropbear_WorldPtr world_ptr,
                                                    uint64_t entity_id,
                                                    bool *out_result);

int32_t dropbear_get_local_transform(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     dropbear_Transform *out_result);

int32_t dropbear_set_local_transform(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     dropbear_Transform value);

int32_t dropbear_get_world_transform(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     dropbear_Transform *out_result);

int32_t dropbear_set_world_transform(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     dropbear_Transform value);

int32_t dropbear_propagate_transform(dropbear_WorldPtr world_ptr,
                                     uint64_t entity_id,
                                     dropbear_Transform *out_result);

#endif  /* DROPBEAR_H */
