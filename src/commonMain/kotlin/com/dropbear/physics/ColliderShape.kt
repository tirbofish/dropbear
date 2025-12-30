package com.dropbear.physics

import com.dropbear.math.Vector3d

sealed class ColliderShape {

    /**
     * Box shape with half-extents (half-width, half-height, half-depth).
     */
    data class Box(val halfExtents: Vector3d) : ColliderShape()

    /**
     * Sphere shape with radius.
     */
    data class Sphere(val radius: Float) : ColliderShape()

    /**
     * Capsule shape along Y-axis.
     */
    data class Capsule(val halfHeight: Float, val radius: Float) : ColliderShape()

    /**
     * Cylinder shape along Y-axis.
     */
    data class Cylinder(val halfHeight: Float, val radius: Float) : ColliderShape()
    /**
     * Cone shape along Y-axis.
     */
    data class Cone(val halfHeight: Float, val radius: Float) : ColliderShape()
}