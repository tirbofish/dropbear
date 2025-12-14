#ifndef DROPBEAR_LABEL_H
#define DROPBEAR_LABEL_H

#include "../dropbear_common.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_get_entity_name(const World* world_ptr, HANDLE entity_id, char* out_name, size_t max_len);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_LABEL_H