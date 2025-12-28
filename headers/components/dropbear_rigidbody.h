#ifndef DROPBEAR_RIGIDBODY_H
#define DROPBEAR_RIGIDBODY_H

#include "../dropbear_common.h"
#include "../dropbear_math.h"
#include "dropbear_collider.h"

typedef enum {
    RIGIDBODY_MODE_DYNAMIC,
    RIGIDBODY_MODE_FIXED,
    RIGIDBODY_MODE_KINEMATIC_POSITION,
    RIGIDBODY_MODE_KINEMATIC_VELOCITY,
} RigidBodyMode;

typedef struct {
    Index index;
    HANDLE entity;
    RigidBodyMode mode;
    double gravity_scale;
    bool can_sleep;
    bool ccd_enabled;
    Vector3D linear_velocity;
    Vector3D angualar_velocity;
    double linear_damping;
    double angular_damping;
    AxisLock lock_translation;
    AxisLock lock_rotation;
} RigidBody;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_apply_impulse(PhysicsEngine* physics_engine, Index index, Vector3D impulse);
DROPBEAR_NATIVE dropbear_apply_torque_impulse(PhysicsEngine* physics_engine, Index index, Vector3D torque_impulse);

DROPBEAR_NATIVE dropbear_set_rigidbody(World* world_handle, PhysicsEngine* physics_engine, RigidBody rigidbody);
DROPBEAR_NATIVE dropbear_get_child_colliders(World* world_handle, PhysicsEngine* physics_engine, Index parent_index, Collider** out_colliders, unsigned int* out_count);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_RIGIDBODY_H