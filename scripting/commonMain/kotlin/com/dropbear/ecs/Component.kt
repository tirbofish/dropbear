package com.dropbear.ecs

import com.dropbear.EntityId

/**
 * Marks a class as a user-defined ECS component for the dropbear engine.
 *
 * Annotated classes must extend [Component]. The magna-carta tool scans for this annotation at
 * build time to generate a [ComponentManager] that registers the component type with the native
 * engine and dispatches lifecycle callbacks from Rust.
 *
 * Retention is [AnnotationRetention.RUNTIME] so JNI reflection can discover types at startup
 * without relying on build-time scanning alone.
 */
@Target(AnnotationTarget.CLASS)
@Retention(AnnotationRetention.RUNTIME)
annotation class EcsComponent
/**
 * A custom property that *can* be attached to an entity in the dropbear ECS system.
 *
 * # Note
 * A [ComponentType] `companion object` is required to be implemented to be queryable. Take a look at
 * the documentation for the object.
 *
 *
 */
abstract class Component(
    val fullyQualifiedTypeName: String?,
    val typeName: String?,
    val category: String?,
    val description: String?,
) {
    /**
     * Ran on a component being attached to an entity.
     */
    abstract fun onAttach()

    /**
     * Ran on a component being updated.
     */
    abstract fun update()

    /**
     * Ran on a component being detached from an entity.
     */
    abstract fun onDetach()
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
     * Uses the FFI to check if the entity is a components.
     *
     * Since most components are technically just classes with a ctor containing the parentEntity's id,
     * and getters and setters, it just needs to query the world.
     *
     * @return The components type ([T]) if it exists or `null` if not
     */
    fun get(entityId: EntityId): T?
}