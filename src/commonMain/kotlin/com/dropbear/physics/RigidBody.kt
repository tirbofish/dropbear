package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.exceptionOnError
import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector3D

class RigidBody(
    internal val index: Index,
    internal val entity: EntityId,
    val rigidBodyMode: RigidBodyMode,
    var gravityScale: Double,
    var canSleep: Boolean,
    var ccdEnabled: Boolean,
    var linearVelocity: Vector3D,
    var angularVelocity: Vector3D,
    var linearDamping: Double,
    var angularDamping: Double,
    var lockTranslation: AxisLock,
    var lockRotation: AxisLock,
) {
    internal constructor(
        index: Index,
        entity: EntityId,
        rigidBodyMode: RigidBodyMode,
        gravityScale: Double,
        canSleep: Boolean,
        ccdEnabled: Boolean,
        linearVelocity: Vector3D,
        angularVelocity: Vector3D,
        linearDamping: Double,
        angularDamping: Double,
        lockTranslation: AxisLock,
        lockRotation: AxisLock,
        native: NativeEngine
    ): this(
        index,
        entity,
        rigidBodyMode,
        gravityScale,
        canSleep,
        ccdEnabled,
        linearVelocity,
        angularVelocity,
        linearDamping,
        angularDamping,
        lockTranslation,
        lockRotation
    ) {
        this.native = native
    }

    // will be manually set
    private var native: NativeEngine? = null

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