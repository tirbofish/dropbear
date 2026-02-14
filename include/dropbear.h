#ifndef DROPBEAR_H
#define DROPBEAR_H

#include <stdbool.h>
#include <stdint.h>

typedef struct Progress {
    uintptr_t current;
    uintptr_t total;
    const char* message;
} Progress;

int32_t dropbear_scene_get_scene_load_progress(SceneLoaderPtr scene_loader, uint64_t scene_id, Progress* out0);
int32_t dropbear_get_entity(WorldPtr world, const char* label, uint64_t* out0);

#endif /* DROPBEAR_H */
