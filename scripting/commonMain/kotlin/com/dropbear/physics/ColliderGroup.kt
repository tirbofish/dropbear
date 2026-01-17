package com.dropbear.physics

import com.dropbear.ecs.Component
import com.dropbear.EntityId
import com.dropbear.ecs.ComponentType

/**
 * A component that can added to an entity that defines all the colliders.
 *
 * This entity requires you to have the `ColliderGroup` component attached to the entity.
 */
class ColliderGroup(
    internal val entity: EntityId,
) : Component(entity, "ColliderGroup") {

    /**
     * Fetches all colliders in the group.
     *
     * It's the only way to access all the colliders in the group from this component.
     */
    fun getColliders(): List<Collider> {
        return getColliderGroupColliders(this)
    }

    companion object : ComponentType<ColliderGroup> {
        override fun get(entityId: EntityId): ColliderGroup? {
            return if (colliderGroupExistsForEntity(entityId)) ColliderGroup(entityId) else null
        }
    }
}

internal expect fun ColliderGroup.getColliderGroupColliders(colliderGroup: ColliderGroup): List<Collider>

internal expect fun colliderGroupExistsForEntity(entityId: EntityId): Boolean