package com.dropbear.components.camera

import com.dropbear.math.Vector3d
import com.dropbear.physics.ColliderShape
import com.dropbear.physics.Physics

/**
 * A springy camera that is used for any camera. It uses ray casting to check if the distance from the player to the
 * camera is obstructed by a physics object, and if so set the distance to the last "safe" spot.
 *
 * Inspired by Godot's SpringyCameraController (other engines probably have their own, never used them before).
 */
class SpringyCameraController {
    private var currentDistance: Double = 5.0
    private val margin = 0.3
    private val sphereRadius = 0.2

    fun getSpringyPosition(
        playerPos: Vector3d,
        targetPos: Vector3d,
        deltaTime: Double
    ): Vector3d {
        val vectorToCam = targetPos - playerPos
        val maxDist = vectorToCam.length()
        val dir = vectorToCam.normalize()

        val hit = Physics.shapeCast(
            origin = playerPos,
            shape = ColliderShape.Sphere(sphereRadius.toFloat()),
            direction = dir,
            maxDistance = maxDist,
            solid = false
        )

        val targetDist = if (hit != null) {
            (hit.distance - margin).coerceAtLeast(0.1)
        } else {
            maxDist
        }

        if (targetDist < currentDistance) {
            currentDistance = targetDist
        } else {
            val returnSpeed = 5.0
            currentDistance += (targetDist - currentDistance) * (returnSpeed * deltaTime).coerceIn(0.0, 1.0)
        }

        return playerPos + (dir * currentDistance)
    }
}