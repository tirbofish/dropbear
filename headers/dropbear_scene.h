#ifndef DROPBEAR_SCENE_H
#define DROPBEAR_SCENE_H

#include "dropbear_common.h"
#include "dropbear_utils.h"

typedef enum {
    SCENE_LOAD_PENDING,
    SCENE_LOAD_SUCCESS,
    SCENE_LOAD_ERROR
} SceneLoadResult;

typedef struct {
    HANDLE id;
    const char* name;
    SceneLoadResult result;
} SceneLoadHandle;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_load_scene_async_1(const CommandBuffer* command_ptr, const char* name, SceneLoadHandle* sceneLoadHandle);
DROPBEAR_NATIVE dropbear_load_scene_async_2(const CommandBuffer* command_ptr, const char* name, const char* loadingScene, SceneLoadHandle* sceneLoadHandle);
DROPBEAR_NATIVE dropbear_switch_to_scene_async(const CommandBuffer* command_ptr, SceneLoadHandle handle);
DROPBEAR_NATIVE dropbear_switch_to_scene_immediate(const CommandBuffer* command_ptr, const char* name);
DROPBEAR_NATIVE dropbear_get_scene_load_progress(const CommandBuffer* command_ptr, SceneLoadHandle handle, Progress* progress);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_SCENE_H