#ifndef DROPBEAR_PROPERTIES_H
#define DROPBEAR_PROPERTIES_H

#include "../dropbear_common.h"
#include "../dropbear_math.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

// getters
DROPBEAR_NATIVE dropbear_get_string_property(const World* world_ptr, HANDLE entity_handle, const char* label, const char** out_value);
DROPBEAR_NATIVE dropbear_get_int_property(const World* world_ptr, HANDLE entity_handle, const char* label, int32_t* out_value);
DROPBEAR_NATIVE dropbear_get_long_property(const World* world_ptr, HANDLE entity_handle, const char* label, int64_t* out_value);
DROPBEAR_NATIVE dropbear_get_float_property(const World* world_ptr, HANDLE entity_handle, const char* label, double* out_value);
DROPBEAR_NATIVE dropbear_get_bool_property(const World* world_ptr, HANDLE entity_handle, const char* label, BOOL* out_value);
DROPBEAR_NATIVE dropbear_get_vec3_property(const World* world_ptr, HANDLE entity_handle, const char* label, Vector3D* out_value);

// setters
DROPBEAR_NATIVE dropbear_set_string_property(const World* world_ptr, HANDLE entity_handle, const char* label, const char* value);
DROPBEAR_NATIVE dropbear_set_int_property(const World* world_ptr, HANDLE entity_handle, const char* label, int32_t value);
DROPBEAR_NATIVE dropbear_set_long_property(const World* world_ptr, HANDLE entity_handle, const char* label, int64_t value);
DROPBEAR_NATIVE dropbear_set_float_property(const World* world_ptr, HANDLE entity_handle, const char* label, double value);
DROPBEAR_NATIVE dropbear_set_bool_property(const World* world_ptr, HANDLE entity_handle, const char* label, BOOL value);
DROPBEAR_NATIVE dropbear_set_vec3_property(const World* world_ptr, HANDLE entity_handle, const char* label, Vector3D value);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_PROPERTIES_H