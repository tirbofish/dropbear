package com.dropbear.physics

import com.dropbear.ecs.Component
import com.dropbear.EntityId
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3d

/**
 * A rigidbody is a component that determines the velocity and movement of a body.
 *
 * To use the rigidbody, you are required to have the [ColliderGroup] component attached
 * to the entity.
 */
class RigidBody(
    internal val index: Index,
    internal val entity: EntityId,
) : Component(entity, "RigidBody") {
    /**
     * The mode of the rigidbody, as determined through [RigidBodyMode].
     */
    var rigidBodyMode: RigidBodyMode
        get() = getRigidbodyMode(this)
        set(value) = setRigidbodyMode(this, value)

    /**
     * The scale of gravity. The global gravity is set through [Physics.gravity],
     * however can be altered for different entities.
     *
     * Default: `1.0`
     */
    var gravityScale: Double
        get() = getRigidbodyGravityScale(this)
        set(value) = setRigidbodyGravityScale(this, value)

    /**
     * Is this rigidbody sleeping?
     *
     * For an object to sleep is to not be aware of whether it is colliding with something or not. It is
     * used to ensure CPU bandwidth is optimised, but can cause issues if is sleeping.
     *
     * By setting this value to `true`, you are essentially "waking up" the entity.
     */
    var sleeping: Boolean
        get() = getRigidBodySleep(this)
        set(value) = setRigidBodySleep(this, value)

    /**
     * Enables or disables Continuous Collision Detection for this body.
     *
     * CCD prevents fast-moving objects from tunnelling through thin walls, but costs more CPU.
     * Enable for bullets, fast projectiles, or any object that must never pass through geometry.
     */
    var ccdEnabled: Boolean
        get() = getRigidbodyCcdEnabled(this)
        set(value) = setRigidbodyCcdEnabled(this, value)

    /**
     * The current linear velocity (speed and direction of movement). This is how fast the body is moving 
     * in units per second.
     */
    var linearVelocity: Vector3d
        get() = getRigidbodyLinearVelocity(this)
        set(value) = setRigidbodyLinearVelocity(this, value)

    /**
     * The current angular velocity (rotation speed) in 3D.
     *
     * Returns a vector in radians per second around each axis (X, Y, Z).
     */
    var angularVelocity: Vector3d
        get() = getRigidbodyAngularVelocity(this)
        set(value) = setRigidbodyAngularVelocity(this, value)

    /**
     * The linear damping coefficient (velocity reduction over time).
     *
     * Damping gradually slows down moving objects.
     *
     * - 0.0 = no slowdown (space/frictionless)
     * - 0.1 = gradual slowdown (air resistance)
     * - 1.0+ = rapid slowdown (thick fluid)
     */
    var linearDamping: Double
        get() = getRigidbodyLinearDamping(this)
        set(value) = setRigidbodyLinearDamping(this, value)

    /**
     * The angular damping coefficient (rotation slowdown over time).
     *
     * Like linear damping but for rotation. Higher values make spinning objects stop faster.
     */
    var angularDamping: Double
        get() = getRigidbodyAngularDamping(this)
        set(value) = setRigidbodyAngularDamping(this, value)

    /**
     * Locks or unlocks rotations of this rigid-body along each cartesian axes.
     */
    var lockTranslation: AxisLock
        get() = getRigidbodyLockTranslation(this)
        set(value) = setRigidbodyLockTranslation(this, value)

    /**
     * Locks or unlocks all rotational movement for this body.
     * 
     * When locked, the body cannot rotate at all (useful for keeping objects upright). 
     * Use for characters that shouldn’t tip over, or objects that should only slide.
     */
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

internal expect fun RigidBody.getRigidbodyMode(rigidBody: RigidBody): RigidBodyMode
internal expect fun RigidBody.setRigidbodyMode(rigidBody: RigidBody, mode: RigidBodyMode)
internal expect fun RigidBody.getRigidbodyGravityScale(rigidBody: RigidBody): Double
internal expect fun RigidBody.setRigidbodyGravityScale(rigidBody: RigidBody, gravityScale: Double)
internal expect fun RigidBody.getRigidBodySleep(rigidBody: RigidBody): Boolean
internal expect fun RigidBody.setRigidBodySleep(rigidBody: RigidBody, canSleep: Boolean)
internal expect fun RigidBody.getRigidbodyCcdEnabled(rigidBody: RigidBody): Boolean
internal expect fun RigidBody.setRigidbodyCcdEnabled(rigidBody: RigidBody, ccdEnabled: Boolean)
internal expect fun RigidBody.getRigidbodyLinearVelocity(rigidBody: RigidBody): Vector3d
internal expect fun RigidBody.setRigidbodyLinearVelocity(rigidBody: RigidBody, linearVelocity: Vector3d)
internal expect fun RigidBody.getRigidbodyAngularVelocity(rigidBody: RigidBody): Vector3d
internal expect fun RigidBody.setRigidbodyAngularVelocity(rigidBody: RigidBody, angularVelocity: Vector3d)
internal expect fun RigidBody.getRigidbodyLinearDamping(rigidBody: RigidBody): Double
internal expect fun RigidBody.setRigidbodyLinearDamping(rigidBody: RigidBody, linearDamping: Double)
internal expect fun RigidBody.getRigidbodyAngularDamping(rigidBody: RigidBody): Double
internal expect fun RigidBody.setRigidbodyAngularDamping(rigidBody: RigidBody, angularDamping: Double)
internal expect fun RigidBody.getRigidbodyLockTranslation(rigidBody: RigidBody): AxisLock
internal expect fun RigidBody.setRigidbodyLockTranslation(rigidBody: RigidBody, lockTranslation: AxisLock)
internal expect fun RigidBody.getRigidbodyLockRotation(rigidBody: RigidBody): AxisLock
internal expect fun RigidBody.setRigidbodyLockRotation(rigidBody: RigidBody, lockRotation: AxisLock)
internal expect fun RigidBody.getRigidbodyChildren(rigidBody: RigidBody): List<Collider>
internal expect fun RigidBody.applyImpulse(index: Index, x: Double, y: Double, z: Double)
internal expect fun RigidBody.applyTorqueImpulse(index: Index, x: Double, y: Double, z: Double)

internal expect fun rigidBodyExistsForEntity(entityId: EntityId): Index?