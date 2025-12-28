#ifndef DROPBEAR_MESHRENDERER_H
#define DROPBEAR_MESHRENDERER_H

#include "../dropbear_common.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

// model
DROPBEAR_NATIVE dropbear_get_model(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, HANDLE* out_model_id);
DROPBEAR_NATIVE dropbear_set_model(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, HANDLE model_id);
DROPBEAR_NATIVE dropbear_is_model_handle(const AssetRegistry* asset_ptr, HANDLE handle, BOOL* out_is_model);
DROPBEAR_NATIVE dropbear_is_using_model(const World* world_ptr, HANDLE entity_handle, HANDLE model_handle, BOOL* out_is_using);

// textures
DROPBEAR_NATIVE dropbear_get_texture(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, const char* name, HANDLE* out_texture_id);
DROPBEAR_NATIVE dropbear_get_texture_name(const AssetRegistry* asset_ptr, HANDLE texture_handle, const char** out_name);
DROPBEAR_NATIVE dropbear_set_texture(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, const char* old_material_name, HANDLE texture_id);
DROPBEAR_NATIVE dropbear_is_texture_handle(const AssetRegistry* asset_ptr, HANDLE handle, BOOL* out_is_texture);
DROPBEAR_NATIVE dropbear_is_using_texture(const World* world_ptr, HANDLE entity_handle, HANDLE texture_handle, BOOL* out_is_using);
DROPBEAR_NATIVE dropbear_get_all_textures(const World* world_ptr, const AssetRegistry* asset_ptr, HANDLE entity_handle, const char*** out_textures, size_t* out_count);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_MESHRENDERER_H