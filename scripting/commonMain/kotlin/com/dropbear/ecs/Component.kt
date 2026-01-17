package com.dropbear.ecs

import com.dropbear.EntityId

/**
 * A custom property that *can* be attached to an entity in the dropbear ECS system.
 *
 * # Note
 * A [ComponentType] `companion object` is required to be implemented to be queryable. Take a look at
 * the documentation for the object.
 *
 * @property parentEntity The [EntityId] to which this component is attached.
 * @property typeName The string name of the component type as defined in Rust. The name is
 *                   found as the type name, such as `component_registry.register_with_default::<Camera3D>();`,
 *                   where the type name would be `"Camera3D"`.
 */
abstract class Component(
    private val parentEntity: EntityId,
    internal val typeName: String,
) {
    override fun toString(): String {
        return "Component(parentEntity: ${this.parentEntity}, typeName: $typeName)"
    }
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