package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Vector3d

internal actual fun RigidBody.getRigidbodyMode(rigidBody: RigidBody): RigidBodyMode {
    val result = RigidBodyNative.getRigidBodyMode(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
    return RigidBodyMode.entries[result]
}

internal actual fun RigidBody.setRigidbodyMode(
    rigidBody: RigidBody,
    mode: RigidBodyMode
) {
    RigidBodyNative.setRigidBodyMode(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, mode.ordinal)
}

internal actual fun RigidBody.getRigidbodyGravityScale(rigidBody: RigidBody): Double {
    return RigidBodyNative.getRigidBodyGravityScale(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
}

internal actual fun RigidBody.setRigidbodyGravityScale(
    rigidBody: RigidBody,
    gravityScale: Double
) {
    RigidBodyNative.setRigidBodyGravityScale(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, gravityScale)
}

internal actual fun RigidBody.getRigidBodySleep(rigidBody: RigidBody): Boolean {
    return RigidBodyNative.getRigidBodySleep(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
}

internal actual fun RigidBody.setRigidBodySleep(
    rigidBody: RigidBody,
    canSleep: Boolean
) {
    RigidBodyNative.setRigidBodySleep(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, canSleep)
}

internal actual fun RigidBody.getRigidbodyCcdEnabled(rigidBody: RigidBody): Boolean {
    return RigidBodyNative.getRigidBodyCcdEnabled(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
}

internal actual fun RigidBody.setRigidbodyCcdEnabled(
    rigidBody: RigidBody,
    ccdEnabled: Boolean
) {
    RigidBodyNative.setRigidBodyCcdEnabled(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, ccdEnabled)
}

internal actual fun RigidBody.getRigidbodyLinearVelocity(rigidBody: RigidBody): Vector3d {
    return RigidBodyNative.getRigidBodyLinearVelocity(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
        ?: Vector3d.zero()
}

internal actual fun RigidBody.setRigidbodyLinearVelocity(
    rigidBody: RigidBody,
    linearVelocity: Vector3d
) {
    RigidBodyNative.setRigidBodyLinearVelocity(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, linearVelocity)
}

internal actual fun RigidBody.getRigidbodyAngularVelocity(rigidBody: RigidBody): Vector3d {
    return RigidBodyNative.getRigidBodyAngularVelocity(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
        ?: Vector3d.zero()
}

internal actual fun RigidBody.setRigidbodyAngularVelocity(
    rigidBody: RigidBody,
    angularVelocity: Vector3d
) {
    RigidBodyNative.setRigidBodyAngularVelocity(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, angularVelocity)
}

internal actual fun RigidBody.getRigidbodyLinearDamping(rigidBody: RigidBody): Double {
    return RigidBodyNative.getRigidBodyLinearDamping(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
}

internal actual fun RigidBody.setRigidbodyLinearDamping(
    rigidBody: RigidBody,
    linearDamping: Double
) {
    RigidBodyNative.setRigidBodyLinearDamping(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, linearDamping)
}

internal actual fun RigidBody.getRigidbodyAngularDamping(rigidBody: RigidBody): Double {
    return RigidBodyNative.getRigidBodyAngularDamping(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
}

internal actual fun RigidBody.setRigidbodyAngularDamping(
    rigidBody: RigidBody,
    angularDamping: Double
) {
    RigidBodyNative.setRigidBodyAngularDamping(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, angularDamping)
}

internal actual fun RigidBody.getRigidbodyLockTranslation(rigidBody: RigidBody): AxisLock {
    return RigidBodyNative.getRigidBodyLockTranslation(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
}

internal actual fun RigidBody.setRigidbodyLockTranslation(
    rigidBody: RigidBody,
    lockTranslation: AxisLock
) {
    RigidBodyNative.setRigidBodyLockTranslation(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, lockTranslation)
}

internal actual fun RigidBody.getRigidbodyLockRotation(rigidBody: RigidBody): AxisLock {
    return RigidBodyNative.getRigidBodyLockRotation(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
}

internal actual fun RigidBody.setRigidbodyLockRotation(
    rigidBody: RigidBody,
    lockRotation: AxisLock
) {
    RigidBodyNative.setRigidBodyLockRotation(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, lockRotation)
}

internal actual fun RigidBody.getRigidbodyChildren(rigidBody: RigidBody): List<Collider> {
    val result = RigidBodyNative.getRigidBodyChildren(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this)
    return result.toList()
}

internal actual fun RigidBody.applyImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
    RigidBodyNative.applyImpulse(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, x, y, z)
}

internal actual fun RigidBody.applyTorqueImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
    RigidBodyNative.applyTorqueImpulse(DropbearEngine.native.worldHandle, DropbearEngine.native.physicsEngineHandle, this, x, y, z)
}

internal actual fun rigidBodyExistsForEntity(entityId: EntityId): Index? {
    return RigidBodyNative.rigidBodyExistsForEntity(
        DropbearEngine.native.worldHandle,
        DropbearEngine.native.physicsEngineHandle,
        entityId.raw
    )
}