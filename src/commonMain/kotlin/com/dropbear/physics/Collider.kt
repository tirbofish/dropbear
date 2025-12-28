package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.exceptionOnError
import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector3D

class Collider(
    internal val index: Index,
    internal val entity: EntityId,
    internal val id: UInt,
    var colliderShape: ColliderShape,
    var density: Double,
    var friction: Double,
    var restitution: Double,
    var isSensor: Boolean,
    var translation: Vector3D,
    var rotation: Vector3D,
) {
    internal constructor(
        index: Index,
        entity: EntityId,
        id: UInt,
        colliderShape: ColliderShape,
        density: Double,
        friction: Double,
        restitution: Double,
        isSensor: Boolean,
        translation: Vector3D,
        rotation: Vector3D,
        native: NativeEngine,
    ): this(
        index,
        entity,
        id,
        colliderShape,
        density,
        friction,
        restitution,
        isSensor,
        translation,
        rotation
    ) {
        this.native = native
    }

    internal var native: NativeEngine? = null

    /**
     * Calculates the mass of the [Collider] based on its shape (determined based on colliderShape) and density.
     *
     * @return Mass of the collider in kilograms.
     */
    fun mass(): Double {
        return when (colliderShape) {
            is ColliderShape.Box -> {
                (colliderShape as ColliderShape.Box).volume() * density
            }
            is ColliderShape.Sphere -> {
                (colliderShape as ColliderShape.Sphere).volume() * density
            }
            is ColliderShape.Capsule -> {
                (colliderShape as ColliderShape.Capsule).volume() * density
            }

            is ColliderShape.Cone -> (colliderShape as ColliderShape.Cone).volume() * density
            is ColliderShape.Cylinder -> (colliderShape as ColliderShape.Cylinder).volume() * density
        }
    }

    /**
     * Pushes your changes to the [Collider] back to the physics engine.
     */
    fun setCollider() {
        val native = native ?: if (exceptionOnError) {
            throw IllegalStateException("Native engine is not initialised")
        } else {
            return
        }
        native.setCollider(this)
    }
}