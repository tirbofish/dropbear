#ifndef DROPBEAR_CAMERA_H
#define DROPBEAR_CAMERA_H

#include "../dropbear_common.h"
#include "../dropbear_math.h"

typedef struct {
    const char* label;
    int64_t entity_id;

    Vector3D eye;
    Vector3D target;
    Vector3D up;

    double aspect;
    double fov_y;
    double znear;
    double zfar;

    double yaw;
    double pitch;
    double speed;
    double sensitivity;
} NativeCamera;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_get_camera(const World* world_ptr, const char* label, NativeCamera* out_camera);
DROPBEAR_NATIVE dropbear_get_attached_camera(const World* world_ptr, HANDLE entity_handle, NativeCamera* out_camera);
DROPBEAR_NATIVE dropbear_set_camera(const World* world_ptr, NativeCamera camera);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_CAMERA_H