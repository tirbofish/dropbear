package com.dropbear.physics

import com.dropbear.ecs.Component
import com.dropbear.EntityId
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3d

class RigidBody(
    internal val index: Index,
    internal val entity: EntityId,
) : Component(entity, "RigidBody") {
    var rigidBodyMode: RigidBodyMode
        get() = getRigidbodyMode(this)
        set(value) = setRigidbodyMode(this, value)
    var gravityScale: Double
        get() = getRigidbodyGravityScale(this)
        set(value) = setRigidbodyGravityScale(this, value)
    var canSleep: Boolean
        get() = getRigidBodySleep(this)
        set(value) = setRigidBodySleep(this, value)
    var ccdEnabled: Boolean
        get() = getRigidbodyCcdEnabled(this)
        set(value) = setRigidbodyCcdEnabled(this, value)
    var linearVelocity: Vector3d
        get() = getRigidbodyLinearVelocity(this)
        set(value) = setRigidbodyLinearVelocity(this, value)
    var angularVelocity: Vector3d
        get() = getRigidbodyAngularVelocity(this)
        set(value) = setRigidbodyAngularVelocity(this, value)
    var linearDamping: Double
        get() = getRigidbodyLinearDamping(this)
        set(value) = setRigidbodyLinearDamping(this, value)
    var angularDamping: Double
        get() = getRigidbodyAngularDamping(this)
        set(value) = setRigidbodyAngularDamping(this, value)
    var lockTranslation: AxisLock
        get() = getRigidbodyLockTranslation(this)
        set(value) = setRigidbodyLockTranslation(this, value)
    var lockRotation: AxisLock
        get() = getRigidbodyLockRotation(this)
        set(value) = setRigidbodyLockRotation(this, value)
    val childColliders: List<Collider>
        get() = getRigidbodyChildren(this)

    /**
     * Applies an instant force.
     *
     * Typically used for jumping or explosions.
     */
    fun applyImpulse(impulse: Vector3d) {
        applyImpulse(impulse.x, impulse.y, impulse.z)
    }

    /**
     * Applies an instant force.
     *
     * Typically used for jumping or explosions.
     */
    fun applyImpulse(x: Double, y: Double, z: Double) {
        return applyImpulse(index, x, y, z)
    }

    /**
     * Applies an instant torque/rotational impulse.
     *
     * Typically used for spinning objects.
     */
    fun applyTorqueImpulse(impulse: Vector3d) {
        applyTorqueImpulse(impulse.x, impulse.y, impulse.z)
    }

    /**
     * Applies an instant torque/rotational impulse.
     *
     * Typically used for spinning objects.
     */
    fun applyTorqueImpulse(x: Double, y: Double, z: Double) {
        return applyTorqueImpulse(index, x, y, z)
    }

    companion object : ComponentType<RigidBody> {
        override fun get(entityId: EntityId): RigidBody? {
            val index = rigidBodyExistsForEntity(entityId)
            return if (index != null) RigidBody(index, entityId) else null
        }
    }
}

expect fun RigidBody.getRigidbodyMode(rigidBody: RigidBody): RigidBodyMode
expect fun RigidBody.setRigidbodyMode(rigidBody: RigidBody, mode: RigidBodyMode)
expect fun RigidBody.getRigidbodyGravityScale(rigidBody: RigidBody): Double
expect fun RigidBody.setRigidbodyGravityScale(rigidBody: RigidBody, gravityScale: Double)
expect fun RigidBody.getRigidBodySleep(rigidBody: RigidBody): Boolean
expect fun RigidBody.setRigidBodySleep(rigidBody: RigidBody, canSleep: Boolean)
expect fun RigidBody.getRigidbodyCcdEnabled(rigidBody: RigidBody): Boolean
expect fun RigidBody.setRigidbodyCcdEnabled(rigidBody: RigidBody, ccdEnabled: Boolean)
expect fun RigidBody.getRigidbodyLinearVelocity(rigidBody: RigidBody): Vector3d
expect fun RigidBody.setRigidbodyLinearVelocity(rigidBody: RigidBody, linearVelocity: Vector3d)
expect fun RigidBody.getRigidbodyAngularVelocity(rigidBody: RigidBody): Vector3d
expect fun RigidBody.setRigidbodyAngularVelocity(rigidBody: RigidBody, angularVelocity: Vector3d)
expect fun RigidBody.getRigidbodyLinearDamping(rigidBody: RigidBody): Double
expect fun RigidBody.setRigidbodyLinearDamping(rigidBody: RigidBody, linearDamping: Double)
expect fun RigidBody.getRigidbodyAngularDamping(rigidBody: RigidBody): Double
expect fun RigidBody.setRigidbodyAngularDamping(rigidBody: RigidBody, angularDamping: Double)
expect fun RigidBody.getRigidbodyLockTranslation(rigidBody: RigidBody): AxisLock
expect fun RigidBody.setRigidbodyLockTranslation(rigidBody: RigidBody, lockTranslation: AxisLock)
expect fun RigidBody.getRigidbodyLockRotation(rigidBody: RigidBody): AxisLock
expect fun RigidBody.setRigidbodyLockRotation(rigidBody: RigidBody, lockRotation: AxisLock)
expect fun RigidBody.getRigidbodyChildren(rigidBody: RigidBody): List<Collider>
expect fun RigidBody.applyImpulse(index: Index, x: Double, y: Double, z: Double)
expect fun RigidBody.applyTorqueImpulse(index: Index, x: Double, y: Double, z: Double)

expect fun rigidBodyExistsForEntity(entityId: EntityId): Index?