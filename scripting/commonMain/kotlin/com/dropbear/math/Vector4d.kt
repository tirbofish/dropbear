package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

class Vector4d(
    @JvmField var x: Double,
    @JvmField var y: Double,
    @JvmField var z: Double,
    @JvmField var w: Double
) {
    companion object {
        @JvmStatic fun zero() = Vector4d(0.0, 0.0, 0.0, 0.0)
        @JvmStatic fun one() = Vector4d(1.0, 1.0, 1.0, 1.0)
    }

    operator fun plus(other: Vector4d) = Vector4d(x + other.x, y + other.y, z + other.z, w + other.w)
    operator fun plus(scalar: Double) = Vector4d(x + scalar, y + scalar, z + scalar, w + scalar)

    operator fun minus(other: Vector4d) = Vector4d(x - other.x, y - other.y, z - other.z, w - other.w)
    operator fun minus(scalar: Double) = Vector4d(x - scalar, y - scalar, z - scalar, w - scalar)

    operator fun times(other: Vector4d) = Vector4d(x * other.x, y * other.y, z * other.z, w * other.w)
    operator fun times(scalar: Double) = Vector4d(x * scalar, y * scalar, z * scalar, w * scalar)

    operator fun div(other: Vector4d) = Vector4d(x / other.x, y / other.y, z / other.z, w / other.w)
    operator fun div(scalar: Double) = Vector4d(x / scalar, y / scalar, z / scalar, w / scalar)

    operator fun unaryMinus() = Vector4d(-x, -y, -z, -w)

    fun length() = sqrt(x * x + y * y + z * z + w * w)
    fun lengthSquared() = x * x + y * y + z * z + w * w

    fun dot(other: Vector4d) = x * other.x + y * other.y + z * other.z + w * other.w

    fun distanceTo(other: Vector4d): Double {
        val dx = x - other.x
        val dy = y - other.y
        val dz = z - other.z
        val dw = w - other.w
        return sqrt(dx * dx + dy * dy + dz * dz + dw * dw)
    }

    fun normalize(): Vector4d {
        val l = length()
        return if (l != 0.0) this / l else zero()
    }

    fun lerp(target: Vector4d, alpha: Double): Vector4d {
        val inv = 1.0 - alpha
        return Vector4d(
            x * inv + target.x * alpha,
            y * inv + target.y * alpha,
            z * inv + target.z * alpha,
            w * inv + target.w * alpha
        )
    }

    fun toFloat() = Vector4f(x.toFloat(), y.toFloat(), z.toFloat(), w.toFloat())
    fun toInt() = Vector4i(x.toInt(), y.toInt(), z.toInt(), w.toInt())
    fun toVector3d() = Vector3d(x, y, z)
    fun toVector2d() = Vector2d(x, y)

    operator fun component1() = x
    operator fun component2() = y
    operator fun component3() = z
    operator fun component4() = w

    fun toGeneric() = Vector4(x, y, z, w)

    override fun toString() = "Vector4d($x, $y, $z, $w)"
    override fun equals(other: Any?) = other is Vector4d && x == other.x && y == other.y && z == other.z && w == other.w
    override fun hashCode(): Int {
        var result = x.hashCode()
        result = 31 * result + y.hashCode()
        result = 31 * result + z.hashCode()
        result = 31 * result + w.hashCode()
        return result
    }
}