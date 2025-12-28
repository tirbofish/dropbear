package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector3D

class Collider(
    internal val index: Index,
    internal val entity: EntityId,
    internal val id: UInt,
    internal val native: NativeEngine
) {
    var colliderShape: ColliderShape
        get() = native.getColliderShape(this)
        set(value) = native.setColliderShape(this, value)
    var density: Double
        get() = native.getColliderDensity(this)
        set(value) = native.setColliderDensity(this, value)
    var friction: Double
        get() = native.getColliderFriction(this)
        set(value) = native.setColliderFriction(this, value)
    var restitution: Double
        get() = native.getColliderRestitution(this)
        set(value) = native.setColliderRestitution(this, value)
    var isSensor: Boolean
        get() = native.getColliderIsSensor(this)
        set(value) = native.setColliderIsSensor(this, value)
    var translation: Vector3D
        get() = native.getColliderTranslation(this)
        set(value) = native.setColliderTranslation(this, value)
    var rotation: Vector3D
        get() = native.getColliderRotation(this)
        set(value) = native.setColliderRotation(this, value)
    var mass: Double
        get() = native.getColliderMass(this)
        set(value) = native.setColliderMass(this, value)
}