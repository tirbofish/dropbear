package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityRef
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

internal actual fun isOverlapping(collider1: Collider, collider2: Collider): Boolean {
    return PhysicsNative.isOverlapping(DropbearEngine.native.physicsEngineHandle, collider1, collider2)
}

internal actual fun isTriggering(collider1: Collider, collider2: Collider): Boolean {
    return PhysicsNative.isTriggering(DropbearEngine.native.physicsEngineHandle, collider1, collider2)
}

internal actual fun isTouching(entity1: EntityRef, entity2: EntityRef): Boolean {
    return PhysicsNative.isTouching(DropbearEngine.native.physicsEngineHandle, entity1.id.raw, entity2.id.raw)
}

internal actual fun shapeCast(
    origin: Vector3d,
    direction: Vector3d,
    shape: ColliderShape,
    toi: Double,
    solid: Boolean
): ShapeCastHit? {
    return PhysicsNative.shapeCast(DropbearEngine.native.physicsEngineHandle, origin, direction, shape, toi, solid)
}