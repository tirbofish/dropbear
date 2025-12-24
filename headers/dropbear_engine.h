#ifndef DROPBEAR_ENGINE_H
#define DROPBEAR_ENGINE_H

#include "dropbear_common.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * @brief Fetches an entity from the world/current scene by its label.
 *
 * Returns the entity's id.
 */
DROPBEAR_NATIVE dropbear_get_entity(const char* label, const World* world_ptr, int64_t* out_entity);

/**
 * @brief Fetches an asset from the asset registry as by its name.
 *
 * Returns the asset's handle.
 */
DROPBEAR_NATIVE dropbear_get_asset(const AssetRegistry* asset_ptr, const char* label, HANDLE* out_asset_id);

/**
 * @brief Quits the currently running app or game.
 *
 * Does not return anything. If any issues occur, the program will terminate.
 *
 * @note Behaviours:
 * - eucalyptus-editor: When called, this exits your Play Mode session and brings
 *                      you back to EditorState::Editing
 * - redback-runtime: When called, this will exit your current process and kill
 *                    the app as is. It will also drop any pointers and do any
 *                    additional clean-up.
 *
 * @return void
 */
void dropbear_quit(const CommandBuffer* command_ptr);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_ENGINE_H