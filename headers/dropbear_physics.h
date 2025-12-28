#ifndef DROPBEAR_PHYSICS_H
#define DROPBEAR_PHYSICS_H

#include "dropbear_common.h"
#include "dropbear_math.h"
#include "components/dropbear_rigidbody.h"
#include "components/dropbear_collider.h"

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_set_physics_enabled(World* world_handle, PhysicsEngine* physics_handle, HANDLE entity_handle, bool enabled);
DROPBEAR_NATIVE dropbear_is_physics_enabled(World* world_handle, PhysicsEngine* physics_handle, HANDLE entity_handle, bool* out_enabled);
DROPBEAR_NATIVE dropbear_get_rigidbody(World* world_handle, PhysicsEngine* physics_handle, HANDLE entity_handle, RigidBody* out_rigidbody);
DROPBEAR_NATIVE dropbear_get_all_colliders(World* world_handle, PhysicsEngine* physics_handle, HANDLE entity_handle, Collider** out_colliders, unsigned int* out_count);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_PHYSICS_H