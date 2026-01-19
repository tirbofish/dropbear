package com.dropbear.physics

import com.dropbear.EntityRef
import com.dropbear.math.Vector3d

internal actual fun getGravity(): Vector3d {
    TODO("Not yet implemented")
}

internal actual fun setGravity(gravity: Vector3d) {
}

internal actual fun raycast(
    origin: Vector3d,
    direction: Vector3d,
    toi: Double,
    solid: Boolean
): RayHit? {
    TODO("Not yet implemented")
}

internal actual fun isOverlapping(collider1: Collider, collider2: Collider): Boolean {
    TODO("Not implemented yet")
}

internal actual fun isTriggering(collider1: Collider, collider2: Collider): Boolean {
    TODO("Not implemented yet")
}

internal actual fun isTouching(entity1: EntityRef, entity2: EntityRef): Boolean {
    TODO("Not implemented yet")
}

internal actual fun shapeCast(
    origin: Vector3d,
    direction: Vector3d,
    shape: ColliderShape,
    toi: Double,
    solid: Boolean
): ShapeCastHit? {
    TODO("Not yet implemented")
}