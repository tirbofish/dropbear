package com.dropbear.ecs

import com.dropbear.EntityId

/**
 * A custom property that *can* be attached to an entity in the dropbear ECS system.
 *
 * @param parentEntity - The [com.dropbear.EntityId] to which this component is attached.
 * @param typeName - The string name of the component type as defined in Rust. The name is
 *                   found as the type name, such as `component_registry.register_with_default::<Camera3D>();`,
 *                   where the type name would be `"Camera3D"`.
 */
abstract class Component(
    val parentEntity: EntityId,
    val typeName: String,
) {
    override fun toString(): String {
        return "Component(parentEntity: ${this.parentEntity}, typeName: $typeName)"
    }
}

interface ComponentType<T : Component> {
    fun get(entityId: EntityId): T?
}