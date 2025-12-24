#ifndef DROPBEAR_SCENE_H
#define DROPBEAR_SCENE_H

#include "dropbear_common.h"
#include "dropbear_utils.h"

/**
 * @brief The sister to `eucalyptus_core::scene::loading::SceneLoadResult`, which provides C-compatible enum values.
 */
typedef enum {
    SCENE_LOAD_PENDING,
    SCENE_LOAD_SUCCESS,
    SCENE_LOAD_ERROR
} SceneLoadResult;

/**
 * @brief The sister handle to `eucalyptus_core::scene::loading::SceneLoadHandle`, which provides C-compatible values.
 */
typedef struct {
    HANDLE id;
    const char* name;
} SceneLoadHandle;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * @brief Loads a scene asynchronously.
 *
 * Returns a handle (in the form of an integer)
 * to the scene load operation.
 */
DROPBEAR_NATIVE dropbear_load_scene_async_1(const CommandBuffer* command_ptr, const SceneLoader* scene_loader_ptr, const char* name, SceneLoadHandle* sceneLoadHandle);

/**
 * @brief Loads a scene asynchronously. Allows you to include a loading_scene_name,
 * which will be displayed while the scene is loading.
 *
 * Returns a handle (in the form of an integer)
 * to the scene load operation.
 */
DROPBEAR_NATIVE dropbear_load_scene_async_2(const CommandBuffer* command_ptr, const SceneLoader* scene_loader_ptr, const char* name, const char* loadingScene, SceneLoadHandle* sceneLoadHandle);

/**
 * @brief Switches to a scene asynchronously.
 *
 * This must be run after you initialise the scene loading (using `dropbear_load_scene_async_1`
 * or `dropbear_load_scene_async_2`). If this function is called before you have checked the progress
 * (with the `dropbear_get_scene_load_status` function), it will return `-10` or `DropbearNativeError::PrematureSceneSwitch`.
 */
DROPBEAR_NATIVE dropbear_switch_to_scene_async(const CommandBuffer* command_ptr, SceneLoadHandle handle);

/**
 * @brief Switches to a scene immediately.
 *
 *
 *
 * This will block your main thread and freeze the window. It will be extremely inconvenient for
 * your players, and is recommended to use `dropbear_load_scene_async_1` or
 * `dropbear_load_scene_async_2`.
 */
DROPBEAR_NATIVE dropbear_switch_to_scene_immediate(const CommandBuffer* command_ptr, const char* name);

/**
 * @brief Gets the progress of a scene load operation.
 *
 * Returns a `Progress` and a `DropbearNativeReturn`
 */
DROPBEAR_NATIVE dropbear_get_scene_load_progress(const SceneLoader* scene_loader_ptr, SceneLoadHandle handle, Progress* progress);

/**
 * @brief Gets the status of a scene load operation
 *
 * Returns a `SceneLoadResult` and a `DropbearNativeReturn`
 */
DROPBEAR_NATIVE dropbear_get_scene_load_status(const SceneLoader* scene_loader_ptr, SceneLoadHandle handle, SceneLoadResult* result);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_SCENE_H