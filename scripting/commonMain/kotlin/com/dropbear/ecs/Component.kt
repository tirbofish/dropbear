package com.dropbear.ecs

import com.dropbear.EntityId

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

internal expect fun registerKotlinComponentType(
    fqcn: String,
    typeName: String?,
    category: String?,
    description: String?,
)

/**
 * The base class of any component.
 *
 * Extend [NativeComponent] or [ExternalComponent], not this class directly.
 */
sealed class Component

/**
 * States a Kotlin-defined ECS component.
 *
 * Annotate subclasses with [@EcsComponent][EcsComponent] so that `magna-carta` can discover them
 * at build time and emit a `ComponentManager` that calls [register] for every discovered type.
 *
 * @param fullyQualifiedTypeName The fully qualified class name used as the stable key inside the
 *   Rust ComponentRegistry (e.g. `"com.example.PlayerHealth"`). Prefer using the literal string
 *   directly so the value survives code shrinking / obfuscation.
 * @param typeName Human-readable short name shown in the editor (e.g. `"PlayerHealth"`).
 * @param category Optional grouping shown in the "Add Component" picker (e.g. `"Gameplay"`).
 * @param description Optional one-line description shown as a tooltip in the editor.
 */
abstract class NativeComponent(
    val fullyQualifiedTypeName: String,
    val typeName: String,
    val category: String? = null,
    val description: String? = null,
): Component() {
    /**
     * Registers this component type with the engine's ComponentRegistry.
     *
     * This is called once per type at startup
     */
    fun register() {
        registerKotlinComponentType(fullyQualifiedTypeName, typeName, category, description)
    }

    /**
     * Renders the inspector UI for this component in the editor.
     *
     * Called by the editor for every entity that has this component type attached.
     */
    abstract fun inspect()

    /**
     * Updates the component for all entities that contain this component.
     */
    abstract fun updateComponent()
}

/**
 * Proxy for an ECS component that is defined outside Kotlin, typically in a Rust crate or
 * another native shared library, and registered with the engine's ComponentRegistry under a
 * known [fullyQualifiedTypeName].
 *
 * Use this when you need to query whether an entity carries a non-Kotlin component, without
 * owning or duplicating its data on the Kotlin side.
 *
 * @param fullyQualifiedTypeName The fully qualified type name that the external component was
 *   registered under in the ComponentRegistry (e.g. `"eucalyptus_core::components::RigidBody"`).
 */
abstract class ExternalComponent(
    val fullyQualifiedTypeName: String,
): Component() {
}

/**
 * An interface that all components must include to be queryable.
 *
 * Basically looks like this:
 * ```
 * companion object : ComponentType<T> {
 *     override fun get(entityId: EntityId): T? {
 *         // chuck in whatever validation stuff you want idk as long as it works...
 *     }
 * }
 * ```
 *
 * @param T The component that can be queried. It must extend the [Component] class.
 */
interface ComponentType<T : Component> {
    /**
     * Uses the FFI to check if the entity is a component.
     *
     * Since most components are technically just classes with a ctor containing the parentEntity's id,
     * and getters and setters, it just needs to query the world.
     *
     * @return The components type ([T]) if it exists or `null` if not
     */
    fun get(entityId: EntityId): T?
}