package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3d

class KinematicCharacterController(
    val entity: EntityId,
) : Component(entity, "KCC") {
    /**
     * Moves the character by a translation (displacement) for this tick.
     *
     * Notes:
     * - This is intended to be called from [com.dropbear.ecs.System.physicsUpdate], not the per-frame
     *   [com.dropbear.ecs.System.update]. Physics runs at a fixed tick rate and will sync transforms from the
     *   physics body each step.
     * - If the entity has a `RigidBody` + `KCC`, you generally should not directly modify `EntityTransform` for movement.
     *   Use this API so the physics body (and collisions/steps/sliding) remain authoritative.
     *
     * Since the KinematicCharacterController uses [RigidBodyMode.KinematicPosition], it uses position manipulation
     * instead of impulse/force based movement.
     */
    fun move(dt: Double, translation: Vector3d) {
        moveCharacter(dt, translation)
    }

    companion object : ComponentType<KinematicCharacterController> {
        override fun get(entityId: EntityId): KinematicCharacterController? {
            return if (kccExistsForEntity(entityId)) KinematicCharacterController(entityId) else null
        }
    }
}

internal expect fun kccExistsForEntity(entityId: EntityId): Boolean

internal expect fun KinematicCharacterController.moveCharacter(dt: Double, translation: Vector3d)