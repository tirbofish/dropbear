package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Quaterniond
import com.dropbear.math.Vector3d
import com.dropbear.math.degreesToRadians
import kotlin.math.cos

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

    /**
     * Sets the kinematic rotation for this tick.
     *
     * This is useful for facing the character toward the camera direction while still
     * using KCC movement.
     */
    fun setRotation(rotation: Quaterniond) {
        setRotationNative(rotation)
    }

    /**
     * This function fetches the hits that are cached in the `KCC` struct.
     *
     * # Note
     * Calling this function immediately may not return any character collisions. It takes time for `rapier3d` to simulate
     * the hit, and it does take some time for an impact (as shown in [CharacterCollision.timeOfImpact]).
     */
    fun getHits(): List<CharacterCollision> {
        return getHitsNative()
    }

    /**
     * Returns true if the character is currently in contact with a "floor-like" surface.
     *
     * This uses the cached character-collision hits and checks if any collision normal points upward.
     *
     * @param minUpwardNormalY Minimum Y component of the contact normal to be considered floor.
     *        Default is ~45° slope limit (cos(45°) ~= 0.707).
     */
    fun isOnFloor(minUpwardNormalY: Double = cos(degreesToRadians(45.0))): Boolean {
        return getHits().any { hit ->
            hit.status != ShapeCastStatus.Failed && hit.normal1.y >= minUpwardNormalY
        }
    }

    companion object : ComponentType<KinematicCharacterController> {
        override fun get(entityId: EntityId): KinematicCharacterController? {
            return if (kccExistsForEntity(entityId)) KinematicCharacterController(entityId) else null
        }
    }
}

internal expect fun kccExistsForEntity(entityId: EntityId): Boolean

internal expect fun KinematicCharacterController.moveCharacter(dt: Double, translation: Vector3d)
internal expect fun KinematicCharacterController.setRotationNative(rotation: Quaterniond)
internal expect fun KinematicCharacterController.getHitsNative(): List<CharacterCollision>