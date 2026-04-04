// Machine generated header bindings by goanna-gen.
// DO NOT EDIT UNLESS YOU KNOW WHAT YOU ARE DOING (it will get regenerated anyways with a modification to eucalyptus-core/src).
// Licensed under MIT or Apache 2.0 depending on your mood.
// part of the dropbear project, by tirbofish

#ifndef DROPBEAR_H
#define DROPBEAR_H

#include <stdbool.h>
#include <stdint.h>

#include <stddef.h>

typedef enum AssetKindTag {
    AssetKindTag_Texture = 0,
    AssetKindTag_Model = 1,
} AssetKindTag;

typedef AssetKindTag AssetKind;

typedef struct Vec3 {
    float x;
    float y;
    float z;
} Vec3;

typedef enum ColliderShapeTag {
    ColliderShapeTag_Box = 0,
    ColliderShapeTag_Sphere = 1,
    ColliderShapeTag_Capsule = 2,
    ColliderShapeTag_Cylinder = 3,
    ColliderShapeTag_Cone = 4,
} ColliderShapeTag;

typedef struct ColliderShapeBox {
    Vec3 half_extents;
} ColliderShapeBox;

typedef struct ColliderShapeSphere {
    float radius;
} ColliderShapeSphere;

typedef struct ColliderShapeCapsule {
    float half_height;
    float radius;
} ColliderShapeCapsule;

typedef struct ColliderShapeCylinder {
    float half_height;
    float radius;
} ColliderShapeCylinder;

typedef struct ColliderShapeCone {
    float half_height;
    float radius;
} ColliderShapeCone;

typedef union ColliderShapeData {
    ColliderShapeBox Box;
    ColliderShapeSphere Sphere;
    ColliderShapeCapsule Capsule;
    ColliderShapeCylinder Cylinder;
    ColliderShapeCone Cone;
} ColliderShapeData;

typedef struct ColliderShapeFfi {
    ColliderShapeTag tag;
    ColliderShapeData data;
} ColliderShapeFfi;

typedef ColliderShapeFfi ColliderShape;

typedef enum NAnimationInterpolationTag {
    NAnimationInterpolationTag_Linear = 0,
    NAnimationInterpolationTag_Step = 1,
    NAnimationInterpolationTag_CubicSpline = 2,
} NAnimationInterpolationTag;

typedef NAnimationInterpolationTag NAnimationInterpolation;

typedef struct NVector3 {
    double x;
    double y;
    double z;
} NVector3;

typedef struct NVector3Array {
    NVector3* values;
    size_t length;
    size_t capacity;
} NVector3Array;

typedef struct NQuaternion {
    double x;
    double y;
    double z;
    double w;
} NQuaternion;

typedef struct NQuaternionArray {
    NQuaternion* values;
    size_t length;
    size_t capacity;
} NQuaternionArray;

typedef struct f64ArrayArray {
    double* values;
    size_t length;
    size_t capacity;
} f64ArrayArray;

typedef enum NChannelValuesTag {
    NChannelValuesTag_Translations = 0,
    NChannelValuesTag_Rotations = 1,
    NChannelValuesTag_Scales = 2,
    NChannelValuesTag_MorphWeights = 3,
} NChannelValuesTag;

typedef struct NChannelValuesTranslations {
    NVector3Array values;
} NChannelValuesTranslations;

typedef struct NChannelValuesRotations {
    NQuaternionArray values;
} NChannelValuesRotations;

typedef struct NChannelValuesScales {
    NVector3Array values;
} NChannelValuesScales;

typedef struct NChannelValuesMorphWeights {
    f64ArrayArray values;
} NChannelValuesMorphWeights;

typedef union NChannelValuesData {
    NChannelValuesTranslations Translations;
    NChannelValuesRotations Rotations;
    NChannelValuesScales Scales;
    NChannelValuesMorphWeights MorphWeights;
} NChannelValuesData;

typedef struct NChannelValuesFfi {
    NChannelValuesTag tag;
    NChannelValuesData data;
} NChannelValuesFfi;

typedef NChannelValuesFfi NChannelValues;

typedef enum NShapeCastStatusTag {
    NShapeCastStatusTag_OutOfIterations = 0,
    NShapeCastStatusTag_Converged = 1,
    NShapeCastStatusTag_Failed = 2,
    NShapeCastStatusTag_PenetratingOrWithinTargetDist = 3,
} NShapeCastStatusTag;

typedef NShapeCastStatusTag NShapeCastStatus;

typedef void* WorldPtr;

int32_t surface_nets_plugin_exists_for_entity(WorldPtr world, uint64_t entity, bool* out0);

#endif /* DROPBEAR_H */
