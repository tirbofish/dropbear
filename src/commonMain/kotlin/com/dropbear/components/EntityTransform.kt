package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Transform

/**
 * A component that contains the local and world [Transform] of an entity.
 */
class EntityTransform(val id: EntityId): Component(id, "EntityTransform") {
    var local: Transform
        get() = getLocalTransform(id)
        set(value) = setLocalTransform(id, value)
    var world: Transform
        get() = getWorldTransform(id)
        set(value) = setWorldTransform(id, value)

    override fun toString(): String {
        return "EntityTransform(id: $id, local=$local, world=$world)"
    }

    /**
     * Walks up the world hierarchy to find the transform of the parent, then multiply/add
     * to create a propagated [Transform].
     */
    fun propagate(): Transform? {
        return propagateTransform(id)
    }

    /**
     * Merges the local and world transforms and return the merged [Transform]
     */
    fun sync(): Transform {
        val scaledPos = local.position * world.scale
        val rotatedPos = world.rotation * scaledPos
        val finalPos = world.position + rotatedPos
        val finalRot = world.rotation * local.rotation
        val finalScale = world.scale * local.scale
        return Transform(finalPos, finalRot, finalScale)
    }

    companion object : ComponentType<EntityTransform> {
        override fun get(entityId: EntityId): EntityTransform? {
            return if (entityTransformExistsForEntity(entityId)) {
                EntityTransform(entityId)
            } else {
                null
            }
        }
    }
}

expect fun EntityTransform.getLocalTransform(entityId: EntityId): Transform
expect fun EntityTransform.setLocalTransform(entityId: EntityId, transform: Transform)
expect fun EntityTransform.getWorldTransform(entityId: EntityId): Transform
expect fun EntityTransform.setWorldTransform(entityId: EntityId, transform: Transform)
expect fun EntityTransform.propagateTransform(entityId: EntityId): Transform?

expect fun entityTransformExistsForEntity(entityId: EntityId): Boolean