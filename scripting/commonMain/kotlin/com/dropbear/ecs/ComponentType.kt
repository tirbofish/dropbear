package com.dropbear.ecs

import com.dropbear.EntityId

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