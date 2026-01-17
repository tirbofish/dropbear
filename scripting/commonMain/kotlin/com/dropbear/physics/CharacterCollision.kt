package com.dropbear.physics

import com.dropbear.EntityId
import com.dropbear.math.Transform
import com.dropbear.math.Vector3d

class CharacterCollision(
    val entity: EntityId,
    internal val collisionHandle: Index = Index(0u, 0u),
) {
    /**
     * The collider hit by the character
     */
    val handle: Collider
        get() = getCollider()

    /**
     * The position of the character when the collider was hit
     */
    val characterPosition: Transform
        get() = getCharacterPosition()

    /**
     * The translation that was already applied to the character when the hit happens
     */
    val translationApplied: Vector3d
        get() = getTranslationApplied()

    /**
     * The translation that was still waiting to be applied to the character when the hit happens.
     */
    val translationRemaining: Vector3d
        get() = getTranslationRemaining()

    /**
     * The time of impact represents how far the collision occurred, shown by a normalized value between 0.0 and 1.0.
     *
     * 0.0 represents the ray at the starting position, while 1.0 represents the collision at the end of the full movement.
     */
    val timeOfImpact: Double
        get() = getTimeOfImpact()

    /**
     * Contact point on the world collider (floor)
     */
    val witness1: Vector3d
        get() = getWitness1()

    /**
     * Contact point on your character
     */
    val witness2: Vector3d
        get() = getWitness2()

    /**
     * Normal pointing from the world collider (floor) toward your character
     */
    val normal1: Vector3d
        get() = getNormal1()

    /**
     * Normal pointing from your character toward the world collider
     */
    val normal2: Vector3d
        get() = getNormal2()

    /**
     * Status of the shape cast. The most common shape cast status is [ShapeCastStatus.Converged].
     */
    val status: ShapeCastStatus
        get() = getStatus()

    override fun toString(): String {
        return "CharacterCollision(entity=$entity)"
    }

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is CharacterCollision) return false
        if (entity != other.entity) return false
        if (collisionHandle != other.collisionHandle) return false
        return true
    }

    override fun hashCode(): Int {
        var result = entity.hashCode()
        result = 31 * result + collisionHandle.hashCode()
        result = 31 * result + timeOfImpact.hashCode()
        result = 31 * result + handle.hashCode()
        result = 31 * result + characterPosition.hashCode()
        result = 31 * result + translationApplied.hashCode()
        result = 31 * result + translationRemaining.hashCode()
        result = 31 * result + witness1.hashCode()
        result = 31 * result + witness2.hashCode()
        result = 31 * result + normal1.hashCode()
        result = 31 * result + normal2.hashCode()
        result = 31 * result + status.hashCode()
        return result
    }
}

internal expect fun CharacterCollision.getCollider(): Collider
internal expect fun CharacterCollision.getCharacterPosition(): Transform
internal expect fun CharacterCollision.getTranslationApplied(): Vector3d
internal expect fun CharacterCollision.getTranslationRemaining(): Vector3d
internal expect fun CharacterCollision.getTimeOfImpact(): Double
internal expect fun CharacterCollision.getWitness1(): Vector3d
internal expect fun CharacterCollision.getWitness2(): Vector3d
internal expect fun CharacterCollision.getNormal1(): Vector3d
internal expect fun CharacterCollision.getNormal2(): Vector3d
internal expect fun CharacterCollision.getStatus(): ShapeCastStatus