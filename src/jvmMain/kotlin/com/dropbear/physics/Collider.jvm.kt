package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.math.Vector3d

actual fun Collider.getColliderShape(collider: Collider): ColliderShape {
    return ColliderNative.getColliderShape(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun Collider.setColliderShape(
    collider: Collider,
    shape: ColliderShape
) {
    ColliderNative.setColliderShape(DropbearEngine.native.physicsEngineHandle, this, shape)
}

actual fun Collider.getColliderDensity(collider: Collider): Double {
    return ColliderNative.getColliderDensity(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun Collider.setColliderDensity(
    collider: Collider,
    density: Double
) {
    ColliderNative.setColliderDensity(DropbearEngine.native.physicsEngineHandle, this, density)
}

actual fun Collider.getColliderFriction(collider: Collider): Double {
    return ColliderNative.getColliderFriction(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun Collider.setColliderFriction(
    collider: Collider,
    friction: Double
) {
    ColliderNative.setColliderFriction(DropbearEngine.native.physicsEngineHandle, this, friction)
}

actual fun Collider.getColliderRestitution(collider: Collider): Double {
    return ColliderNative.getColliderRestitution(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun Collider.setColliderRestitution(
    collider: Collider,
    restitution: Double
) {
    ColliderNative.setColliderRestitution(DropbearEngine.native.physicsEngineHandle, this, restitution)
}

actual fun Collider.getColliderIsSensor(collider: Collider): Boolean {
    return ColliderNative.getColliderIsSensor(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun Collider.setColliderIsSensor(
    collider: Collider,
    isSensor: Boolean
) {
    ColliderNative.setColliderIsSensor(DropbearEngine.native.physicsEngineHandle, this, isSensor)
}

actual fun Collider.getColliderTranslation(collider: Collider): Vector3d {
    return ColliderNative.getColliderTranslation(DropbearEngine.native.physicsEngineHandle, this)
        ?: Vector3d.zero()
}

actual fun Collider.setColliderTranslation(
    collider: Collider,
    translation: Vector3d
) {
    ColliderNative.setColliderTranslation(DropbearEngine.native.physicsEngineHandle, this, translation)
}

actual fun Collider.getColliderRotation(collider: Collider): Vector3d {
    return ColliderNative.getColliderRotation(DropbearEngine.native.physicsEngineHandle, this)
        ?: Vector3d.zero()
}

actual fun Collider.setColliderRotation(
    collider: Collider,
    rotation: Vector3d
) {
    ColliderNative.setColliderRotation(DropbearEngine.native.physicsEngineHandle, this, rotation)
}

actual fun Collider.getColliderMass(collider: Collider): Double {
    return ColliderNative.getColliderMass(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun Collider.setColliderMass(collider: Collider, mass: Double) {
    ColliderNative.setColliderMass(DropbearEngine.native.physicsEngineHandle, this, mass)
}