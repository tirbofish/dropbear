package com.dropbear.physics

import com.dropbear.math.Vector3D
import kotlin.math.PI
import kotlin.math.pow

sealed class ColliderShape {

    /**
     * Box shape with half-extents (half-width, half-height, half-depth).
     */
    data class Box(val halfExtents: Vector3D) : ColliderShape() {
        /**
         * Returns the volume of the shape/box.
         *
         * In this case, volume is calculated as: [halfExtents].x * [halfExtents].y * [halfExtents].z
         */
        fun volume(): Double {
            return halfExtents.x * halfExtents.y * halfExtents.z
        }
    }

    /**
     * Sphere shape with radius.
     */
    data class Sphere(val radius: Float) : ColliderShape() {
        /**
         * Returns the volume of the shape/sphere.
         *
         * In this case, volume is calculated as: (4/3) * π * [radius]^3
         */
        fun volume(): Double {
            return (4.0 / 3.0) * PI * radius.toDouble().pow(3.0)
        }
    }

    /**
     * Capsule shape along Y-axis.
     */
    data class Capsule(val halfHeight: Float, val radius: Float) : ColliderShape() {
        /**
         * Returns the volume of the shape/capsule.
         *
         * In this case, volume is calculated as: (4/3) * π * [radius]^3 + π * [radius]^2 * (2 * [halfHeight])
         */
        fun volume(): Double {
            val sphereVolume = (4.0 / 3.0) * PI * radius.toDouble().pow(3.0)
            val cylinderVolume = PI * radius.toDouble().pow(2.0) * (2.0 * halfHeight.toDouble())
            return sphereVolume + cylinderVolume
        }
    }

    /**
     * Cylinder shape along Y-axis.
     */
    data class Cylinder(val halfHeight: Float, val radius: Float) : ColliderShape() {
        /**
         * Returns the volume of the shape/cylinder.
         *
         * In this case, volume is calculated as: π * [radius]^2 * (2 * [halfHeight])
         */
        fun volume(): Double {
            return PI * radius.toDouble().pow(2.0) * (2.0 * halfHeight.toDouble())
        }
    }

    /**
     * Cone shape along Y-axis.
     */
    data class Cone(val halfHeight: Float, val radius: Float) : ColliderShape() {
        /**
         * Returns the volume of the shape/cone.
         *
         * In this case, volume is calculated as: (1/3) * π * [radius]^2 * (2 * [halfHeight])
         */
        fun volume(): Double {
            return (1.0 / 3.0) * PI * radius.toDouble().pow(2.0) * (2.0 * halfHeight.toDouble())
        }
    }
}