package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.math.Vector3d

internal actual fun getGravity(): Vector3d {
    return PhysicsNative.getGravity(DropbearEngine.native.physicsEngineHandle) ?: Vector3d(0.0, -9.81, 0.0)
}

internal actual fun setGravity(gravity: Vector3d) {
    return PhysicsNative.setGravity(DropbearEngine.native.physicsEngineHandle, gravity)
}

internal actual fun raycast(
    origin: Vector3d,
    direction: Vector3d,
    toi: Double,
    solid: Boolean
): RayHit? {
    return PhysicsNative.raycast(DropbearEngine.native.physicsEngineHandle, origin, direction, toi, solid)
}