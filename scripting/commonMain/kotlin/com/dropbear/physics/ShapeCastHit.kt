package com.dropbear.physics

import com.dropbear.math.Vector3d

/**
 * Defines a hit from a shape-cast (sweeping a volume through space).
 *
 * @param collider The first collider that is hit.
 * @param distance The distance travelled along the cast direction at the time of impact.
 * @param witness1 Contact point on the world collider.
 * @param witness2 Contact point on the casted shape.
 * @param normal1 Normal pointing from the world collider toward the casted shape.
 * @param normal2 Normal pointing from the casted shape toward the world collider.
 * @param status Status of the cast result.
 */
class ShapeCastHit(
    val collider: Collider,
    val distance: Double,
    val witness1: Vector3d,
    val witness2: Vector3d,
    val normal1: Vector3d,
    val normal2: Vector3d,
    val status: ShapeCastStatus,
) {
    override fun toString(): String {
        return "ShapeCastHit(collider=$collider, distance=$distance, status=$status)"
    }
}
