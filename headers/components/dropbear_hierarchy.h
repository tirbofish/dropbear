#ifndef DROPBEAR_HIERARCHY_H
#define DROPBEAR_HIERARCHY_H

#include "../dropbear_common.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_get_children(const World* world_ptr, HANDLE entity_id, HANDLE** out_children, size_t* out_count);
DROPBEAR_NATIVE dropbear_get_child_by_label(const World* world_ptr, HANDLE entity_id, const char* label, HANDLE* out_child);
DROPBEAR_NATIVE dropbear_get_parent(const World* world_ptr, HANDLE entity_id, HANDLE* out_parent);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_HIERARCHY_H