package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.math.Vector3d

internal actual fun RigidBody.setRigidbodyMode(
    rigidBody: RigidBody,
    mode: RigidBodyMode
) {
}

internal actual fun RigidBody.getRigidbodyMode(rigidBody: RigidBody): RigidBodyMode {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.getRigidbodyGravityScale(rigidBody: RigidBody): Double {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidbodyGravityScale(
    rigidBody: RigidBody,
    gravityScale: Double
) {
}

internal actual fun RigidBody.getRigidBodySleep(rigidBody: RigidBody): Boolean {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidBodySleep(
    rigidBody: RigidBody,
    canSleep: Boolean
) {
}

internal actual fun RigidBody.getRigidbodyCcdEnabled(rigidBody: RigidBody): Boolean {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidbodyCcdEnabled(
    rigidBody: RigidBody,
    ccdEnabled: Boolean
) {
}

internal actual fun RigidBody.getRigidbodyLinearVelocity(rigidBody: RigidBody): Vector3d {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidbodyLinearDamping(
    rigidBody: RigidBody,
    linearDamping: Double
) {
}

internal actual fun RigidBody.setRigidbodyLinearVelocity(
    rigidBody: RigidBody,
    linearVelocity: Vector3d
) {
}

internal actual fun RigidBody.getRigidbodyAngularVelocity(rigidBody: RigidBody): Vector3d {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidbodyAngularVelocity(
    rigidBody: RigidBody,
    angularVelocity: Vector3d
) {
}

internal actual fun RigidBody.getRigidbodyLinearDamping(rigidBody: RigidBody): Double {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.getRigidbodyAngularDamping(rigidBody: RigidBody): Double {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidbodyAngularDamping(
    rigidBody: RigidBody,
    angularDamping: Double
) {
}

internal actual fun RigidBody.getRigidbodyLockTranslation(rigidBody: RigidBody): AxisLock {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidbodyLockTranslation(
    rigidBody: RigidBody,
    lockTranslation: AxisLock
) {
}

internal actual fun RigidBody.getRigidbodyLockRotation(rigidBody: RigidBody): AxisLock {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.setRigidbodyLockRotation(
    rigidBody: RigidBody,
    lockRotation: AxisLock
) {
}

internal actual fun RigidBody.getRigidbodyChildren(rigidBody: RigidBody): List<Collider> {
    TODO("Not yet implemented")
}

internal actual fun RigidBody.applyImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
}

internal actual fun RigidBody.applyTorqueImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
}

internal actual fun rigidBodyExistsForEntity(entityId: EntityId): Index? {
    TODO("Not yet implemented")
}