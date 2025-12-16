#ifndef DROPBEAR_ENGINE_H
#define DROPBEAR_ENGINE_H

#include "dropbear_common.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_get_entity(const char* label, const World* world_ptr, int64_t* out_entity);
DROPBEAR_NATIVE dropbear_get_asset(const AssetRegistry* asset_ptr, const char* label, HANDLE* out_asset_id);

void dropbear_quit(const CommandBuffer* command_ptr);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_ENGINE_H