package com.dropbear.physics

import com.dropbear.ecs.Component
import com.dropbear.EntityId
import com.dropbear.ecs.ComponentType

class ColliderGroup(
    entity: EntityId,
) : Component(entity, "ColliderGroup") {
    fun getColliders(): List<Collider> {
        return getColliderGroupColliders(this)
    }

    companion object : ComponentType<ColliderGroup> {
        override fun get(entityId: EntityId): ColliderGroup? {
            return if (colliderGroupExistsForEntity(entityId)) ColliderGroup(entityId) else null
        }
    }
}

expect fun ColliderGroup.getColliderGroupColliders(colliderGroup: ColliderGroup): List<Collider>

expect fun colliderGroupExistsForEntity(entityId: EntityId): Boolean