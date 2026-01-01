package com.dropbear.physics

/**
 * Defines a hit from a ray-cast.
 *
 * @param collider The first collider that is hit.
 * @param distance The distance from the origin of the ray to the collider that is hit.
 */
class RayHit(
    val collider: Collider,
    val distance: Double,
)