#ifndef DROPBEAR_COMMON_H
#define DROPBEAR_COMMON_H

#include <stddef.h>
#include <stdint.h>

/**
* @brief Return function for a DropbearNativeError. 0 on success, otherwise look
*        at `eucalyptus_core::scripting::native::DropbearNativeError`
*/
#define DROPBEAR_NATIVE int

/**
* @brief The handle/id of an object, as a long. Kotlin/Native requires
*        me to have an int64_t as a Long (or use a long long).
*/
#define HANDLE int64_t

/**
* @brief A helper type that defines a value that can either be a 0 or 1
*/
#define BOOL int

typedef struct World World; // opaque pointer
typedef struct InputState InputState; // opaque pointer
typedef struct GraphicsCommandQueue GraphicsCommandQueue; // opaque pointer
typedef struct AssetRegistry AssetRegistry; // opaque pointer

#endif // DROPBEAR_COMMON_H