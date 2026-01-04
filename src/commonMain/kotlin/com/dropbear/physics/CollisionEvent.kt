package com.dropbear.physics

/**
 * Defined a type of collision that can occur between two different colliders.
 *
 * @property eventType The type of event that occured. 
 * @property collider1 One of the colliders that was in the collision
 * @property collider2 One of the colliders that was in the collision
 */
class CollisionEvent(
    val eventType: CollisionEventType,
    val collider1: Collider,
    val collider2: Collider,
    internal val flags: Int,
) {
    /**
     * Returns `true` if this is a [CollisionEventType.Started]
     */
    fun started(): Boolean = eventType == CollisionEventType.Started

    /**
     * Returns `true` if this is a [CollisionEventType.Stopped]
     */
    fun stopped(): Boolean = eventType == CollisionEventType.Stopped

    /**
     * Was at least one of the colliders involved in the collision a sensor?
     */
    fun sensor(): Boolean = flags and CollisionEventFlags.SENSOR != 0

    /**
     * Was at least one of the colliders involved in the collision removed?
     */
    fun removed(): Boolean = flags and CollisionEventFlags.REMOVED != 0

    override fun toString(): String {
        return "CollisionEvent(eventType=${eventType.name}, collider1=$collider1, collider2=$collider2, flags=[$flags]<sensor=${sensor()}, removed=${removed()}>)"
    }
}

/**
 * The type of collision that occurred. 
 */
enum class CollisionEventType {
    /**
     * The collision started at this frame
     */
    Started,

    /**
     * The collision stopped at this frame. 
     */
    Stopped,
}

class CollisionEventFlags {
    companion object {
        /**
         * At least one of the colliders was a sensor
         */
        const val SENSOR = 0b0001

        /**
         * At least one of the colliders was removed. 
         */
        const val REMOVED = 0b0010
    }
}