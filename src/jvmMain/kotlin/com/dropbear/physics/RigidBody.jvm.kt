package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.math.Vector3D

actual fun RigidBody.getRigidbodyMode(rigidBody: RigidBody): RigidBodyMode {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyMode(
    rigidBody: RigidBody,
    mode: RigidBodyMode
) {
}

actual fun RigidBody.getRigidbodyGravityScale(rigidBody: RigidBody): Double {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyGravityScale(
    rigidBody: RigidBody,
    gravityScale: Double
) {
}

actual fun RigidBody.getRigidbodyCanSleep(rigidBody: RigidBody): Boolean {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyCanSleep(
    rigidBody: RigidBody,
    canSleep: Boolean
) {
}

actual fun RigidBody.getRigidbodyCcdEnabled(rigidBody: RigidBody): Boolean {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyCcdEnabled(
    rigidBody: RigidBody,
    ccdEnabled: Boolean
) {
}

actual fun RigidBody.getRigidbodyLinearVelocity(rigidBody: RigidBody): Vector3D {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyLinearVelocity(
    rigidBody: RigidBody,
    linearVelocity: Vector3D
) {
}

actual fun RigidBody.getRigidbodyAngularVelocity(rigidBody: RigidBody): Vector3D {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyAngularVelocity(
    rigidBody: RigidBody,
    angularVelocity: Vector3D
) {
}

actual fun RigidBody.getRigidbodyLinearDamping(rigidBody: RigidBody): Double {
    TODO("Not yet implemented")
}

actual fun RigidBody.getRigidbodyAngularDamping(rigidBody: RigidBody): Double {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyAngularDamping(
    rigidBody: RigidBody,
    angularDamping: Double
) {
}

actual fun RigidBody.getRigidbodyLockTranslation(rigidBody: RigidBody): AxisLock {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyLockTranslation(
    rigidBody: RigidBody,
    lockTranslation: AxisLock
) {
}

actual fun RigidBody.getRigidbodyLockRotation(rigidBody: RigidBody): AxisLock {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyLockRotation(
    rigidBody: RigidBody,
    lockRotation: AxisLock
) {
}

actual fun RigidBody.getRigidbodyChildren(rigidBody: RigidBody): List<EntityId> {
    TODO("Not yet implemented")
}

actual fun RigidBody.setRigidbodyChildren(
    rigidBody: RigidBody,
    children: List<EntityId>
) {
}

actual fun RigidBody.applyImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
}

actual fun RigidBody.applyTorqueImpulse(
    index: Index,
    x: Double,
    y: Double,
    z: Double
) {
}

actual fun RigidBody.
        setRigidbodyLinearDamping(
    rigidBody: RigidBody,
    linearDamping: Double
) {
}

actual fun rigidBodyExistsForEntity(entityId: EntityId): Index? {
    TODO("Not yet implemented")
}