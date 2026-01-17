package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

class Vector3i(
    @JvmField var x: Int,
    @JvmField var y: Int,
    @JvmField var z: Int
) {
    companion object {
        @JvmStatic
        fun zero() = Vector3i(0, 0, 0)
    }

    operator fun plus(other: Vector3i) = Vector3i(x + other.x, y + other.y, z + other.z)
    operator fun minus(other: Vector3i) = Vector3i(x - other.x, y - other.y, z - other.z)
    operator fun times(scalar: Int) = Vector3i(x * scalar, y * scalar, z * scalar)
    operator fun div(scalar: Int) = Vector3i(x / scalar, y / scalar, z / scalar)

    fun length(): Double = sqrt((x * x + y * y + z * z).toDouble())

    fun toFloat() = Vector3f(x.toFloat(), y.toFloat(), z.toFloat())
    fun toDouble() = Vector3d(x.toDouble(), y.toDouble(), z.toDouble())

    override fun toString() = "Vector3i($x, $y, $z)"
}