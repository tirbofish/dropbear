#ifndef DROPBEAR_MATH_H
#define DROPBEAR_MATH_H

#include "dropbear_common.h"

/**
 * @brief Represents a `glam::Vec2` in a C struct form.
 */
typedef struct {
    double x;
    double y;
} Vector2D;

/**
 * @brief Represents a `glam::Vec3` in a C struct form.
 */
typedef struct {
    float x;
    float y;
    float z;
} Vector3D;

/**
 * @brief Represents a `dropbear_engine::entity::Transform` in a C struct form.
 */
typedef struct {
    double position_x;
    double position_y;
    double position_z;
    double rotation_x;
    double rotation_y;
    double rotation_z;
    double rotation_w;
    double scale_x;
    double scale_y;
    double scale_z;
} NativeTransform;

/**
 * @brief Represents an `dropbear_engine::entity::EntityTransform` in a C struct form.
 */
typedef struct {
    NativeTransform local;
    NativeTransform world;
} NativeEntityTransform;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_MATH_H