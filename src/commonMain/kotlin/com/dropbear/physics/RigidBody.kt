package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.exceptionOnError
import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector3D

class RigidBody(
    internal val index: Index,
    internal val entity: EntityId,
    internal var native: NativeEngine,
) {
    var rigidBodyMode: RigidBodyMode
        get() = native.getRigidbodyMode(this)
        set(value) = native.setRigidbodyMode(this, value)
    var gravityScale: Double
        get() = native.getRigidbodyGravityScale(this)
        set(value) = native.setRigidbodyGravityScale(this, value)
    var canSleep: Boolean
        get() = native.getRigidbodyCanSleep(this)
        set(value) = native.setRigidbodyCanSleep(this, value)
    var ccdEnabled: Boolean
        get() = native.getRigidbodyCcdEnabled(this)
        set(value) = native.setRigidbodyCcdEnabled(this, value)
    var linearVelocity: Vector3D
        get() = native.getRigidbodyLinearVelocity(this)
        set(value) = native.setRigidbodyLinearVelocity(this, value)
    var angularVelocity: Vector3D
        get() = native.getRigidbodyAngularVelocity(this)
        set(value) = native.setRigidbodyAngularVelocity(this, value)
    var linearDamping: Double
        get() = native.getRigidbodyLinearDamping(this)
        set(value) = native.setRigidbodyLinearDamping(this, value)
    var angularDamping: Double
        get() = native.getRigidbodyAngularDamping(this)
        set(value) = native.setRigidbodyAngularDamping(this, value)
    var lockTranslation: AxisLock
        get() = native.getRigidbodyLockTranslation(this)
        set(value) = native.setRigidbodyLockTranslation(this, value)
    var lockRotation: AxisLock
        get() = native.getRigidbodyLockRotation(this)
        set(value) = native.setRigidbodyLockRotation(this, value)

    /**
     * Applies an instant force.
     *
     * Typically used for jumping or explosions.
     */
    fun applyImpulse(impulse: Vector3D) {
        applyImpulse(impulse.x, impulse.y, impulse.z)
    }

    /**
     * Applies an instant force.
     *
     * Typically used for jumping or explosions.
     */
    fun applyImpulse(x: Double, y: Double, z: Double) {
        val native = native ?: if (exceptionOnError) {
            throw IllegalStateException("Native engine is not initialised")
        } else {
            return
        }
        return native.applyImpulse(index, x, y, z)
    }
    /**
     * Applies an instant torque/rotational impulse.
     *
     * Typically used for spinning objects.
     */
    fun applyTorqueImpulse(impulse: Vector3D) {
        applyTorqueImpulse(impulse.x, impulse.y, impulse.z)
    }

    /**
     * Applies an instant torque/rotational impulse.
     *
     * Typically used for spinning objects.
     */
    fun applyTorqueImpulse(x: Double, y: Double, z: Double) {
        val native = native ?: if (exceptionOnError) {
            throw IllegalStateException("Native engine is not initialised")
        } else {
            return
        }
        return native.applyTorqueImpulse(index, x, y, z)
    }

    /**
     * Pushes your changes to the [RigidBody] back to the physics engine.
     */
    fun setRigidbody() {
        val native = native ?: if (exceptionOnError) {
            throw IllegalStateException("Native engine is not initialised")
        } else {
            return
        }

        native.setRigidbody(this)
    }

    /**
     * Fetches all child [Collider] under this [RigidBody].
     *
     * Returns `null` if there is no component such as `ColliderGroup` attached to the
     * entity, or an [emptyList] if there are no child colliders.
     */
    fun getChildColliders(): List<Collider>? {
        val native = native ?: if (exceptionOnError) {
            throw IllegalStateException("Native engine is not initialised")
        } else {
            return null
        }
        return native.getChildColliders(index)
    }
}