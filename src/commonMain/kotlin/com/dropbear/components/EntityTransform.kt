package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Transform

/**
 * A component that contains the `local` and `world` [Transform] of an entity.
 *
 * This entity must contain the `EntityTransform` component to be queryable.
 */
class EntityTransform(val id: EntityId): Component(id, "EntityTransform") {
    /**
     * The local transform.
     *
     * This is the transform that is relative to the entity.
     */
    var local: Transform
        get() = getLocalTransform(id)
        set(value) = setLocalTransform(id, value)

    /**
     * The world transform.
     *
     * This is the transform that is relative to the world.
     */
    var world: Transform
        get() = getWorldTransform(id)
        set(value) = setWorldTransform(id, value)

    override fun toString(): String {
        return "EntityTransform(id: $id, local=$local, world=$world)"
    }

    /**
     * Walks up the world hierarchy to find the transform of the parent, and the transform of
     * the parent's parent, then multiply/add to create a propagated [Transform].
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

internal expect fun EntityTransform.getLocalTransform(entityId: EntityId): Transform
internal expect fun EntityTransform.setLocalTransform(entityId: EntityId, transform: Transform)
internal expect fun EntityTransform.getWorldTransform(entityId: EntityId): Transform
internal expect fun EntityTransform.setWorldTransform(entityId: EntityId, transform: Transform)
internal expect fun EntityTransform.propagateTransform(entityId: EntityId): Transform?

internal expect fun entityTransformExistsForEntity(entityId: EntityId): Boolean