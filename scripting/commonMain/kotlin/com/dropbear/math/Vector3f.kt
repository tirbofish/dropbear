package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

class Vector3f(
    @JvmField var x: Float,
    @JvmField var y: Float,
    @JvmField var z: Float
) {
    companion object {
        @JvmStatic
        fun zero() = Vector3f(0f, 0f, 0f)
    }

    operator fun plus(other: Vector3f) = Vector3f(x + other.x, y + other.y, z + other.z)
    operator fun plus(scalar: Float) = Vector3f(x + scalar, y + scalar, z + scalar)

    operator fun minus(other: Vector3f) = Vector3f(x - other.x, y - other.y, z - other.z)
    operator fun minus(scalar: Float) = Vector3f(x - scalar, y - scalar, z - scalar)

    operator fun times(other: Vector3f) = Vector3f(x * other.x, y * other.y, z * other.z)
    operator fun times(scalar: Float) = Vector3f(x * scalar, y * scalar, z * scalar)

    operator fun div(other: Vector3f) = Vector3f(x / other.x, y / other.y, z / other.z)
    operator fun div(scalar: Float) = Vector3f(x / scalar, y / scalar, z / scalar)

    operator fun unaryMinus() = Vector3f(-x, -y, -z)

    fun length(): Float = sqrt(x * x + y * y + z * z)
    fun lengthSquared(): Float = x * x + y * y + z * z

    fun dot(other: Vector3f) = x * other.x + y * other.y + z * other.z

    fun cross(other: Vector3f): Vector3f {
        return Vector3f(
            y * other.z - z * other.y,
            z * other.x - x * other.z,
            x * other.y - y * other.x
        )
    }

    fun normalize(): Vector3f {
        val l = length()
        return if (l != 0f) Vector3f(x / l, y / l, z / l) else zero()
    }

    fun toDouble() = Vector3d(x.toDouble(), y.toDouble(), z.toDouble())
    fun toInt() = Vector3i(x.toInt(), y.toInt(), z.toInt())
    fun toVector2f() = Vector2f(x, y)

    override fun toString() = "Vector3f($x, $y, $z)"
}