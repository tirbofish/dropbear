#ifndef DROPBEAR_COMMON_H
#define DROPBEAR_COMMON_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

/**
* @brief Return function for a DropbearNativeError. 0 on success, otherwise look
*        at `eucalyptus_core::scripting::native::DropbearNativeError`
*/
#define DROPBEAR_NATIVE int

/**
* @brief The handle/id of an object, as a long.
*
* Kotlin/Native requires
* this to have an int64_t as a Long (or use a long long).
*/
#define HANDLE int64_t

/**
* @brief A helper type that defines a value that can either be a 0 or 1
*/
#define BOOL int

typedef struct World World; // opaque pointer
typedef struct InputState InputState; // opaque pointer
typedef struct CommandBuffer CommandBuffer; // opaque pointer
typedef struct AssetRegistry AssetRegistry; // opaque pointer
typedef struct SceneLoader SceneLoader; // opaque pointer
typedef struct PhysicsEngine PhysicsEngine; // opaque pointer

/// Describes all the different pointers that can be passed into a scripting
/// module.
typedef struct {
    World* world;
    InputState* input;
    CommandBuffer* graphics;
    AssetRegistry* assets;
    SceneLoader* scene_loader;
    PhysicsEngine* physics_engine;
} DropbearContext;

typedef struct {
    unsigned int index;
    unsigned int generation;
} Index;

typedef struct {
    bool x;
    bool y;
    bool z;
} AxisLock;

#endif // DROPBEAR_COMMON_H