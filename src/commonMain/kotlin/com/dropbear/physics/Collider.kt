package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.math.Vector3d

class Collider(
    internal val index: Index,
    internal val entity: EntityId,
    internal val id: UInt,
) {
    var colliderShape: ColliderShape
        get() = getColliderShape(this)
        set(value) = setColliderShape(this, value)
    var density: Double
        get() = getColliderDensity(this)
        set(value) = setColliderDensity(this, value)
    var friction: Double
        get() = getColliderFriction(this)
        set(value) = setColliderFriction(this, value)
    var restitution: Double
        get() = getColliderRestitution(this)
        set(value) = setColliderRestitution(this, value)
    var isSensor: Boolean
        get() = getColliderIsSensor(this)
        set(value) = setColliderIsSensor(this, value)
    var translation: Vector3d
        get() = getColliderTranslation(this)
        set(value) = setColliderTranslation(this, value)
    var rotation: Vector3d
        get() = getColliderRotation(this)
        set(value) = setColliderRotation(this, value)
    var mass: Double
        get() = getColliderMass(this)
        set(value) = setColliderMass(this, value)
}

expect fun Collider.getColliderShape(collider: Collider): ColliderShape
expect fun Collider.setColliderShape(collider: Collider, shape: ColliderShape)
expect fun Collider.getColliderDensity(collider: Collider): Double
expect fun Collider.setColliderDensity(collider: Collider, density: Double)
expect fun Collider.getColliderFriction(collider: Collider): Double
expect fun Collider.setColliderFriction(collider: Collider, friction: Double)
expect fun Collider.getColliderRestitution(collider: Collider): Double
expect fun Collider.setColliderRestitution(collider: Collider, restitution: Double)
expect fun Collider.getColliderIsSensor(collider: Collider): Boolean
expect fun Collider.setColliderIsSensor(collider: Collider, isSensor: Boolean)
expect fun Collider.getColliderTranslation(collider: Collider): Vector3d
expect fun Collider.setColliderTranslation(collider: Collider, translation: Vector3d)
expect fun Collider.getColliderRotation(collider: Collider): Vector3d
expect fun Collider.setColliderRotation(collider: Collider, rotation: Vector3d)
expect fun Collider.getColliderMass(collider: Collider): Double
expect fun Collider.setColliderMass(collider: Collider, mass: Double)