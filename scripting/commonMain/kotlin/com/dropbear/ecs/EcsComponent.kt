package com.dropbear.ecs

/**
 * Marks a class as a user-defined ECS component for the dropbear engine.
 *
 * Annotated classes must extend [NativeComponent]. The `magna-carta` tool scans for this annotation
 * at build time to generate a [ComponentManager] that registers each component type with the
 * native engine's ComponentRegistry via [registerKotlinComponentType].
 *
 * Retention is [AnnotationRetention.RUNTIME] so JNI reflection can discover types at startup
 * without relying on build-time scanning alone.
 */
@Target(AnnotationTarget.CLASS)
@Retention(AnnotationRetention.RUNTIME)
annotation class EcsComponent