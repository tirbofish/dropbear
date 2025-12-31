package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

class Vector4f(
    @JvmField var x: Float,
    @JvmField var y: Float,
    @JvmField var z: Float,
    @JvmField var w: Float
) {
    companion object {
        @JvmStatic
        fun zero() = Vector4f(0f, 0f, 0f, 0f)
    }

    operator fun plus(other: Vector4f) = Vector4f(x + other.x, y + other.y, z + other.z, w + other.w)
    operator fun plus(scalar: Float) = Vector4f(x + scalar, y + scalar, z + scalar, w + scalar)

    operator fun minus(other: Vector4f) = Vector4f(x - other.x, y - other.y, z - other.z, w - other.w)
    operator fun minus(scalar: Float) = Vector4f(x - scalar, y - scalar, z - scalar, w - scalar)

    operator fun times(other: Vector4f) = Vector4f(x * other.x, y * other.y, z * other.z, w * other.w)
    operator fun times(scalar: Float) = Vector4f(x * scalar, y * scalar, z * scalar, w * scalar)

    operator fun div(other: Vector4f) = Vector4f(x / other.x, y / other.y, z / other.z, w / other.w)
    operator fun div(scalar: Float) = Vector4f(x / scalar, y / scalar, z / scalar, w / scalar)

    operator fun unaryMinus() = Vector4f(-x, -y, -z, -w)

    fun length(): Float = sqrt(x * x + y * y + z * z + w * w)
    fun lengthSquared(): Float = x * x + y * y + z * z + w * w

    fun dot(other: Vector4f) = x * other.x + y * other.y + z * other.z + w * other.w

    fun normalize(): Vector4f {
        val l = length()
        return if (l != 0f) Vector4f(x / l, y / l, z / l, w / l) else zero()
    }

    fun toDouble() = Vector4d(x.toDouble(), y.toDouble(), z.toDouble(), w.toDouble())
    fun toInt() = Vector4i(x.toInt(), y.toInt(), z.toInt(), w.toInt())
    fun toVector3f() = Vector3f(x, y, z)

    operator fun component1() = x
    operator fun component2() = y
    operator fun component3() = z
    operator fun component4() = w

    override fun toString() = "Vector4f($x, $y, $z, $w)"
}