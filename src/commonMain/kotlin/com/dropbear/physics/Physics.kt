package com.dropbear.physics

import com.dropbear.math.Vector3
import com.dropbear.math.Vector3d

class Physics {
    var gravity: Vector3d
        get() = getGravity()
        set(value) = setGravity(value)

    fun raycast(origin: Vector3d, direction: Vector3d, maxDistance: Double?, solid: Boolean): RayHit? {
        if (maxDistance != null) {
            return raycast(origin, direction, toi = maxDistance, solid)
        } else {
            return raycast(origin, direction, toi = Double.MAX_VALUE, solid)
        }
    }
}

internal expect fun getGravity(): Vector3d
internal expect fun setGravity(gravity: Vector3d)

internal expect fun raycast(origin: Vector3d, direction: Vector3d, toi: Double, solid: Boolean): RayHit?