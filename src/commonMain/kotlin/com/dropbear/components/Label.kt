package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.EntityRef
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.getEntityLabel

/**
 * The `Label` component.
 *
 * All entities derive from this component, and any components that do not contain this component is considered
 * invalid, or does not exist. This component is added automatically to all components. It's just an excuse to create a
 * new component class.
 */
class Label(
    internal val entity: EntityId
): Component(entity, "Label") {
    val name: String
        get() = EntityRef.getEntityLabel(entity)

    companion object : ComponentType<Label> {
        override fun get(entityId: EntityId): Label? {
            return if (labelExistsForEntity(entityId)) Label(entityId) else null
        }
    }
}

internal expect fun labelExistsForEntity(entityId: EntityId): Boolean