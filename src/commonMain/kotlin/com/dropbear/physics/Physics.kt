package com.dropbear.physics

import com.dropbear.math.Vector3d

/**
 * Helpful physics utilities.
 */
class Physics {
    companion object {
        /**
         * The globally set gravity set. By default, it is set to `Vector3d(0.0, -9.81, 0.0)`, however
         * can be customised.
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
            if (maxDistance != null) {
                return raycast(origin, direction, toi = maxDistance, solid)
            } else {
                return raycast(origin, direction, toi = Double.MAX_VALUE, solid)
            }
        }
    }
}

internal expect fun getGravity(): Vector3d
internal expect fun setGravity(gravity: Vector3d)

internal expect fun raycast(origin: Vector3d, direction: Vector3d, toi: Double, solid: Boolean): RayHit?