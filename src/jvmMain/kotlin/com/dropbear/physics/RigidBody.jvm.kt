package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Vector3d

actual fun RigidBody.getRigidbodyMode(rigidBody: RigidBody): RigidBodyMode {
    val result = RigidBodyNative.getRigidBodyMode(DropbearEngine.native.physicsEngineHandle, this)
    return RigidBodyMode.entries[result]
}

actual fun RigidBody.setRigidbodyMode(
    rigidBody: RigidBody,
    mode: RigidBodyMode
) {
    RigidBodyNative.setRigidBodyMode(DropbearEngine.native.physicsEngineHandle, this, mode.ordinal)
}

actual fun RigidBody.getRigidbodyGravityScale(rigidBody: RigidBody): Double {
    return RigidBodyNative.getRigidBodyGravityScale(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun RigidBody.setRigidbodyGravityScale(
    rigidBody: RigidBody,
    gravityScale: Double
) {
    RigidBodyNative.setRigidBodyGravityScale(DropbearEngine.native.physicsEngineHandle, this, gravityScale)
}

actual fun RigidBody.getRigidbodyCanSleep(rigidBody: RigidBody): Boolean {
    return RigidBodyNative.getRigidBodyCanSleep(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun RigidBody.setRigidbodyCanSleep(
    rigidBody: RigidBody,
    canSleep: Boolean
) {
    RigidBodyNative.setRigidBodyCanSleep(DropbearEngine.native.physicsEngineHandle, this, canSleep)
}

actual fun RigidBody.getRigidbodyCcdEnabled(rigidBody: RigidBody): Boolean {
    return RigidBodyNative.getRigidBodyCcdEnabled(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun RigidBody.setRigidbodyCcdEnabled(
    rigidBody: RigidBody,
    ccdEnabled: Boolean
) {
    RigidBodyNative.setRigidBodyCcdEnabled(DropbearEngine.native.physicsEngineHandle, this, ccdEnabled)
}

actual fun RigidBody.getRigidbodyLinearVelocity(rigidBody: RigidBody): Vector3d {
    return RigidBodyNative.getRigidBodyLinearVelocity(DropbearEngine.native.physicsEngineHandle, this)
        ?: Vector3d.zero()
}

actual fun RigidBody.setRigidbodyLinearVelocity(
    rigidBody: RigidBody,
    linearVelocity: Vector3d
) {
    RigidBodyNative.setRigidBodyLinearVelocity(DropbearEngine.native.physicsEngineHandle, this, linearVelocity)
}

actual fun RigidBody.getRigidbodyAngularVelocity(rigidBody: RigidBody): Vector3d {
    return RigidBodyNative.getRigidBodyAngularVelocity(DropbearEngine.native.physicsEngineHandle, this)
        ?: Vector3d.zero()
}

actual fun RigidBody.setRigidbodyAngularVelocity(
    rigidBody: RigidBody,
    angularVelocity: Vector3d
) {
    RigidBodyNative.setRigidBodyAngularVelocity(DropbearEngine.native.physicsEngineHandle, this, angularVelocity)
}

actual fun RigidBody.getRigidbodyLinearDamping(rigidBody: RigidBody): Double {
    return RigidBodyNative.getRigidBodyLinearDamping(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun RigidBody.setRigidbodyLinearDamping(
    rigidBody: RigidBody,
    linearDamping: Double
) {
    RigidBodyNative.setRigidBodyLinearDamping(DropbearEngine.native.physicsEngineHandle, this, linearDamping)
}

actual fun RigidBody.getRigidbodyAngularDamping(rigidBody: RigidBody): Double {
    return RigidBodyNative.getRigidBodyAngularDamping(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun RigidBody.setRigidbodyAngularDamping(
    rigidBody: RigidBody,
    angularDamping: Double
) {
    RigidBodyNative.setRigidBodyAngularDamping(DropbearEngine.native.physicsEngineHandle, this, angularDamping)
}

actual fun RigidBody.getRigidbodyLockTranslation(rigidBody: RigidBody): AxisLock {
    return RigidBodyNative.getRigidBodyLockTranslation(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun RigidBody.setRigidbodyLockTranslation(
    rigidBody: RigidBody,
    lockTranslation: AxisLock
) {
    RigidBodyNative.setRigidBodyLockTranslation(DropbearEngine.native.physicsEngineHandle, this, lockTranslation)
}

actual fun RigidBody.getRigidbodyLockRotation(rigidBody: RigidBody): AxisLock {
    return RigidBodyNative.getRigidBodyLockRotation(DropbearEngine.native.physicsEngineHandle, this)
}

actual fun RigidBody.setRigidbodyLockRotation(
    rigidBody: RigidBody,
    lockRotation: AxisLock
) {
    RigidBodyNative.setRigidBodyLockRotation(DropbearEngine.native.physicsEngineHandle, this, lockRotation)
}

actual fun RigidBody.getRigidbodyChildren(rigidBody: RigidBody): List<EntityId> {
    val result = RigidBodyNative.getRigidBodyChildren(DropbearEngine.native.physicsEngineHandle, this)
    return result.map { EntityId(it) }
}

actual fun RigidBody.setRigidbodyChildren(
    rigidBody: RigidBody,
    children: List<EntityId>
) {
    val ids = children.map { it.raw }.toLongArray()
    RigidBodyNative.setRigidBodyChildren(DropbearEngine.native.physicsEngineHandle, this, ids)
}

actual fun RigidBody.applyImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
    RigidBodyNative.applyImpulse(DropbearEngine.native.physicsEngineHandle, this, x, y, z)
}

actual fun RigidBody.applyTorqueImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
    RigidBodyNative.applyTorqueImpulse(DropbearEngine.native.physicsEngineHandle, this, x, y, z)
}

actual fun rigidBodyExistsForEntity(entityId: EntityId): Index? {
    return RigidBodyNative.rigidBodyExistsForEntity(DropbearEngine.native.physicsEngineHandle, entityId.raw)
}