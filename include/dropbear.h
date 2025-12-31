#pragma once

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
