#ifndef DROPBEAR_ENTITYTRANSFORM_H
#define DROPBEAR_ENTITYTRANSFORM_H

#include "../dropbear_common.h"
#include "../dropbear_math.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_get_transform(const World* world_ptr, HANDLE entity_handle, NativeEntityTransform* out_transform);
DROPBEAR_NATIVE dropbear_propagate_transform(const World* world_ptr, HANDLE entity_id, NativeTransform* out_transform);
DROPBEAR_NATIVE dropbear_set_transform(const World* world_ptr, HANDLE entity_id, NativeEntityTransform transform);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_ENTITYTRANSFORM_H