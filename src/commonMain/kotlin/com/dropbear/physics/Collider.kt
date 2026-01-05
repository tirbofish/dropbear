package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.math.Vector3d

/**
 * A collider that is part of a [ColliderGroup].
 *
 * There can be multiple colliders for one rigidbody or for one collider group.
 */
class Collider(
    internal val index: Index,
    internal val entity: EntityId,
    internal val id: UInt,
) {
    /**
     * The shape of the collider in the form of a [ColliderShape]
     */
    var colliderShape: ColliderShape
        get() = getColliderShape(this)
        set(value) = setColliderShape(this, value)

    /**
     * The density of the object.
     */
    var density: Double
        get() = getColliderDensity(this)
        set(value) = setColliderDensity(this, value)

    /**
     * The friction value.
     *
     * A higher friction value causes higher drag, which can slow the object
     * down significantly.
     */
    var friction: Double
        get() = getColliderFriction(this)
        set(value) = setColliderFriction(this, value)

    /**
     * The "bounciness" of an object.
     *
     * - `0.0` = no bounce (clay, soft material)
     * - `1.0` = perfect bounce (ideal elastic collision)
     * - `> 1.0` = super bouncy (gains energy, unrealistic but fun)
     *
     * Typical values: `0.0` - `0.8`
     */
    var restitution: Double
        get() = getColliderRestitution(this)
        set(value) = setColliderRestitution(this, value)

    /**
     * Checks if this collider is a sensor.
     *
     * Sensors are like “trigger zones” - they detect when other colliders enter/exit them but
     * don’t create physical contact forces. Often used for:
     * - Trigger zones
     * - Proximity detection
     * - Collectible items
     * - Area-of-effect detection
     */
    var isSensor: Boolean
        get() = getColliderIsSensor(this)
        set(value) = setColliderIsSensor(this, value)

    /**
     * The translation offset of the collider from the entity's position.
     */
    var translation: Vector3d
        get() = getColliderTranslation(this)
        set(value) = setColliderTranslation(this, value)

    /**
     * The rotational offset of the collider from the entity's rotation.
     */
    var rotation: Vector3d
        get() = getColliderRotation(this)
        set(value) = setColliderRotation(this, value)

    /**
     * The mass of the collider.
     *
     * This is calculated through the collider shape provided, however
     * can be overridden by setting a custom one.
     */
    var mass: Double
        get() = getColliderMass(this)
        set(value) = setColliderMass(this, value)

    override fun toString(): String {
        return "Collider(index=$index, entity=$entity, id=$id)"
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Collider) return false
        return index == other.index
    }

    // had to be implemented
    override fun hashCode(): Int {
        var result = index.hashCode()
        result = 31 * result + entity.hashCode()
        result = 31 * result + id.hashCode()
        result = 31 * result + density.hashCode()
        result = 31 * result + friction.hashCode()
        result = 31 * result + restitution.hashCode()
        result = 31 * result + isSensor.hashCode()
        result = 31 * result + mass.hashCode()
        result = 31 * result + colliderShape.hashCode()
        result = 31 * result + translation.hashCode()
        result = 31 * result + rotation.hashCode()
        return result
    }
}

internal expect fun Collider.getColliderShape(collider: Collider): ColliderShape
internal expect fun Collider.setColliderShape(collider: Collider, shape: ColliderShape)
internal expect fun Collider.getColliderDensity(collider: Collider): Double
internal expect fun Collider.setColliderDensity(collider: Collider, density: Double)
internal expect fun Collider.getColliderFriction(collider: Collider): Double
internal expect fun Collider.setColliderFriction(collider: Collider, friction: Double)
internal expect fun Collider.getColliderRestitution(collider: Collider): Double
internal expect fun Collider.setColliderRestitution(collider: Collider, restitution: Double)
internal expect fun Collider.getColliderIsSensor(collider: Collider): Boolean
internal expect fun Collider.setColliderIsSensor(collider: Collider, isSensor: Boolean)
internal expect fun Collider.getColliderTranslation(collider: Collider): Vector3d
internal expect fun Collider.setColliderTranslation(collider: Collider, translation: Vector3d)
internal expect fun Collider.getColliderRotation(collider: Collider): Vector3d
internal expect fun Collider.setColliderRotation(collider: Collider, rotation: Vector3d)
internal expect fun Collider.getColliderMass(collider: Collider): Double
internal expect fun Collider.setColliderMass(collider: Collider, mass: Double)