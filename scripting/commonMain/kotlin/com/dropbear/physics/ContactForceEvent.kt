package com.dropbear.physics

import com.dropbear.math.Vector3d

/**
 * Event occurring when the sum of the magnitudes of the contact forces
 * between two colliders exceed a threshold.
 * 
 * @property collider1 The first collider involved in the contact
 * @property collider2 The second collider involved in the contact
 * @property totalForce The sum of all the forces between the two colliders
 * @property totalForceMagnitude The sum of the magnitudes of each force between the two colliders.
 * 
 * Note that this is **not** the same as the magnitude of [totalForce]. Here we are summing the magnitude of all the 
 * forces, instead of taking the magnitude of their sum.  
 * @property maxForceDirection The world-space (unit) direction of the force with the strongest magnitude
 * @property maxForceMagnitude The magnitude of the largest force at a contact point of this contact pair.
 */
class ContactForceEvent(
    val collider1: Collider,
    val collider2: Collider,
    val totalForce: Vector3d,
    val totalForceMagnitude: Double,
    val maxForceDirection: Vector3d,
    val maxForceMagnitude: Double,
) {
    /**
     * Checks if either [collider1] or [collider2] were one of the colliders in
     * the event.
     */
    fun includes(colliders: List<Collider>): Boolean {
        if (colliders.isEmpty()) return false
        colliders.forEach { c ->
            if (c == collider1 || c == collider2) {
                return true
            }
        }
        return false
    }

    override fun toString(): String {
        return "ContactForceEvent(collider1=$collider1, collider2=$collider2, totalForce=$totalForce, totalForceMagnitude=$totalForceMagnitude, maxForceDirection=$maxForceDirection, maxForceMagnitude=$maxForceMagnitude)"
    }
}