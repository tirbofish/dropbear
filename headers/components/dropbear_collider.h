#ifndef DROPBEAR_COLLIDER_H
#define DROPBEAR_COLLIDER_H

#include "../dropbear_common.h"
#include "../dropbear_math.h"

typedef enum ColliderShapeTag {
    ColliderShape_Box = 0,
    ColliderShape_Sphere = 1,
    ColliderShape_Capsule = 2,
    ColliderShape_Cylinder = 3,
    ColliderShape_Cone = 4
} ColliderShapeTag;

// -------------------------------------------------------------- //

typedef struct ColliderShapeBody_Box {
    Vector3D half_extents;
} ColliderShapeBody_Box;

typedef struct ColliderShapeBody_Sphere {
    float radius;
} ColliderShapeBody_Sphere;

typedef struct ColliderShapeBody_Capsule {
    float half_height;
    float radius;
} ColliderShapeBody_Capsule;

typedef struct ColliderShapeBody_Cylinder {
    float half_height;
    float radius;
} ColliderShapeBody_Cylinder;

typedef struct ColliderShapeBody_Cone {
    float half_height;
    float radius;
} ColliderShapeBody_Cone;

typedef struct ColliderShape {
    ColliderShapeTag tag;

    union {
        ColliderShapeBody_Box box;
        ColliderShapeBody_Sphere sphere;
        ColliderShapeBody_Capsule capsule;
        ColliderShapeBody_Cylinder cylinder;
        ColliderShapeBody_Cone cone;
    } data;
} ColliderShape;

// -------------------------------------------------------------- //

typedef struct {
    Index index;
    HANDLE entity;
    ColliderShape collider_shape;
    double density;
    double friction;
    double restitution;
    bool is_sensor;
    Vector3D translation;
    Vector3D rotation;
} Collider;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_free_colliders(Collider* colliders, unsigned int count);

DROPBEAR_NATIVE dropbear_set_collider(PhysicsEngine* physics_engine, Collider collider);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_COLLIDER_H