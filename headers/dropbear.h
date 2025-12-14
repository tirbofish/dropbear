/**
 * dropbear-engine native header definitions. Created by tirbofish as part of the dropbear project.
 *
 * Primarily used for Kotlin/Native, however nothing is stopping you from implementing it to your own language.
 * Exports are located at `eucalyptus_core::scripting::native::exports`.
 *
 * Note: This does not include JNI definitions, only native exports from the eucalyptus-core dynamic library.
 *       For JNI definitions, take a look at `eucalyptus_core::scripting::jni::exports` or even better, take a
 *       look at the JNINative class for all JNI functions that exist.
 *
 * Warning: This header file is not always up to date with the existing JNI functions (some funcs may not be implemented),
 *          So please open a issue if there is something missing, or help us by creating a PR implementing them.
 *
 * Licensed under MIT or Apache 2.0 depending on your mood.
 */

#ifndef DROPBEAR_H
#define DROPBEAR_H

#include "dropbear_common.h"
#include "dropbear_math.h"

#include "dropbear_camera.h"
#include "dropbear_input.h"
#include "dropbear_scene.h"
#include "dropbear_engine.h"

#include "components/dropbear_entitytransform.h"
#include "components/dropbear_hierarchy.h"
#include "components/dropbear_label.h"
#include "components/dropbear_meshrenderer.h"
#include "components/dropbear_properties.h"

#endif // DROPBEAR_H