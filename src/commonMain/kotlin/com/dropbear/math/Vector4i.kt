package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

class Vector4i(
    @JvmField var x: Int,
    @JvmField var y: Int,
    @JvmField var z: Int,
    @JvmField var w: Int
) {
    companion object {
        @JvmStatic
        fun zero() = Vector4i(0, 0, 0, 0)
    }

    operator fun plus(other: Vector4i) = Vector4i(x + other.x, y + other.y, z + other.z, w + other.w)
    operator fun minus(other: Vector4i) = Vector4i(x - other.x, y - other.y, z - other.z, w - other.w)

    operator fun times(scalar: Int) = Vector4i(x * scalar, y * scalar, z * scalar, w * scalar)
    operator fun div(scalar: Int) = Vector4i(x / scalar, y / scalar, z / scalar, w / scalar)

    fun length(): Double = sqrt((x * x + y * y + z * z + w * w).toDouble())

    fun toFloat() = Vector4f(x.toFloat(), y.toFloat(), z.toFloat(), w.toFloat())
    fun toDouble() = Vector4d(x.toDouble(), y.toDouble(), z.toDouble(), w.toDouble())

    operator fun component1() = x
    operator fun component2() = y
    operator fun component3() = z
    operator fun component4() = w

    override fun toString() = "Vector4i($x, $y, $z, $w)"
}