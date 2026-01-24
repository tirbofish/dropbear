package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

class Vector3d(
    @JvmField var x: Double,
    @JvmField var y: Double,
    @JvmField var z: Double
) {
    companion object {
        @JvmStatic fun zero() = Vector3d(0.0, 0.0, 0.0)
        @JvmStatic fun one() = Vector3d(1.0, 1.0, 1.0)

        @JvmStatic fun right() = Vector3d(1.0, 0.0, 0.0) // X
        @JvmStatic fun up()    = Vector3d(0.0, 1.0, 0.0) // Y
        @JvmStatic fun forward() = Vector3d(0.0, 0.0, 1.0) // Z
    }

    operator fun plus(other: Vector3d) = Vector3d(x + other.x, y + other.y, z + other.z)
    operator fun plus(scalar: Double) = Vector3d(x + scalar, y + scalar, z + scalar)

    operator fun minus(other: Vector3d) = Vector3d(x - other.x, y - other.y, z - other.z)
    operator fun minus(scalar: Double) = Vector3d(x - scalar, y - scalar, z - scalar)

    operator fun times(other: Vector3d) = Vector3d(x * other.x, y * other.y, z * other.z)
    operator fun times(scalar: Double) = Vector3d(x * scalar, y * scalar, z * scalar)

    operator fun div(other: Vector3d) = Vector3d(x / other.x, y / other.y, z / other.z)
    operator fun div(scalar: Double) = Vector3d(x / scalar, y / scalar, z / scalar)

    operator fun unaryMinus() = Vector3d(-x, -y, -z)

    fun length() = sqrt(x * x + y * y + z * z)
    fun lengthSquared() = x * x + y * y + z * z

    fun dot(other: Vector3d) = x * other.x + y * other.y + z * other.z

    /**
     * Calculates the Cross Product.
     * Result is perpendicular to both this vector and the other.
     */
    fun cross(other: Vector3d): Vector3d {
        return Vector3d(
            y * other.z - z * other.y,
            z * other.x - x * other.z,
            x * other.y - y * other.x
        )
    }

    fun distanceTo(other: Vector3d): Double {
        val dx = x - other.x
        val dy = y - other.y
        val dz = z - other.z
        return sqrt(dx * dx + dy * dy + dz * dz)
    }

    fun normalize(): Vector3d {
        val l = length()
        return if (l != 0.0) this / l else zero()
    }

    fun lerp(target: Vector3d, alpha: Double): Vector3d {
        val inv = 1.0 - alpha
        return Vector3d(
            x * inv + target.x * alpha,
            y * inv + target.y * alpha,
            z * inv + target.z * alpha
        )
    }

    fun toFloat() = Vector3f(x.toFloat(), y.toFloat(), z.toFloat())
    fun toInt() = Vector3i(x.toInt(), y.toInt(), z.toInt())
    fun toVector2d() = Vector2d(x, y)
    fun toVector4d(w: Double) = Vector4d(x, y, z, w)

    fun toGeneric() = Vector3(x, y, z)

    override fun toString() = "Vector3d($x, $y, $z)"
    override fun equals(other: Any?) = other is Vector3d && x == other.x && y == other.y && z == other.z
    override fun hashCode(): Int {
        var result = x.hashCode()
        result = 31 * result + y.hashCode()
        result = 31 * result + z.hashCode()
        return result
    }
}