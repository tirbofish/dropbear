package com.dropbear.physics

import com.dropbear.EntityRef
import com.dropbear.math.Vector3d

/**
 * Helpful physics utilities.
 */
class Physics {
    companion object {
        /**
         * The globally set gravity as a [Vector3d].
         *
         * By default, it is set to `Vector3d(0.0, -9.81, 0.0)`, however can be customised.
         */
        var gravity: Vector3d
            get() = getGravity()
            set(value) = setGravity(value)

        /**
         * Casts a ray from the [origin] in a specific [direction]. Returns a nullable [RayHit] object.
         *
         * @param origin The origin of the ray. Could be the player synced position?
         * @param direction The direction of the ray in the form of a unit circle, such as
         *                  `downwards` being `-Vector3d.up()`.
         * @param maxDistance The maximum distance before quitting operation. Set to `null` if no maxDistance.
         * @param solid If true, detects hits even if the ray starts inside a shape.
         *              If false, the ray "passes through" from the inside until it exits.
         * @return A [RayHit] object if hit or `null` if not.
         */
        fun raycast(origin: Vector3d, direction: Vector3d, maxDistance: Double?, solid: Boolean): RayHit? {
            return if (maxDistance != null) {
                raycast(origin, direction, toi = maxDistance, solid)
            } else {
                raycast(origin, direction, toi = Double.MAX_VALUE, solid)
            }
        }

        /**
         * Checks if two colliders are intersecting each other.
         */
        fun overlapping(collider1: Collider, collider2: Collider): Boolean {
            return isOverlapping(collider1, collider2)
        }

        /**
         * Checks if one collider is overlapping another colliders area, where at least
         * one collider is a sensor.
         */
        fun triggering(collider1: Collider, collider2: Collider): Boolean {
            return isTriggering(collider1, collider2)
        }

        /**
         * Checks if two **non-sensor** entities (with at least 1 collider per entity) are touching each other.
         */
        fun touching(entity1: EntityRef, entity2: EntityRef): Boolean {
            return isTouching(entity1, entity2)
        }

        /**
         * Casts a [shape] from the [origin] in a specific [direction]. Returns a nullable [ShapeCastHit] object.
         *
         * @param origin The origin of the cast.
         * @param direction The direction of the cast. Prefer unit vectors.
         * @param shape The shape to cast.
         * @param maxDistance The maximum distance before quitting operation. Set to `null` if no maxDistance.
         * @param solid If true, detects hits even if the cast starts inside a shape.
         *              If false, the cast "passes through" from the inside until it exits.
         */
        fun shapeCast(
            origin: Vector3d,
            direction: Vector3d,
            shape: ColliderShape,
            maxDistance: Double?,
            solid: Boolean,
        ): ShapeCastHit? {
            return if (maxDistance != null) {
                shapeCast(origin, direction, shape, toi = maxDistance, solid)
            } else {
                shapeCast(origin, direction, shape, toi = Double.MAX_VALUE, solid)
            }
        }
    }
}

internal expect fun getGravity(): Vector3d
internal expect fun setGravity(gravity: Vector3d)

internal expect fun raycast(origin: Vector3d, direction: Vector3d, toi: Double, solid: Boolean): RayHit?
internal expect fun isOverlapping(collider1: Collider, collider2: Collider): Boolean
internal expect fun isTriggering(collider1: Collider, collider2: Collider): Boolean
internal expect fun isTouching(entity1: EntityRef, entity2: EntityRef): Boolean

internal expect fun shapeCast(origin: Vector3d, direction: Vector3d, shape: ColliderShape, toi: Double, solid: Boolean): ShapeCastHit?