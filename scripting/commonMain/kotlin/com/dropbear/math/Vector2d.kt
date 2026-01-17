package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

/**
 * A class for holding a vector of `2` double values.
 */
class Vector2d(
    @JvmField var x: Double,
    @JvmField var y: Double
) {
    companion object {
        @JvmStatic fun zero() = Vector2d(0.0, 0.0)
        @JvmStatic fun one() = Vector2d(1.0, 1.0)
    }

    operator fun plus(other: Vector2d) = Vector2d(x + other.x, y + other.y)
    operator fun plus(scalar: Double) = Vector2d(x + scalar, y + scalar)

    operator fun minus(other: Vector2d) = Vector2d(x - other.x, y - other.y)
    operator fun minus(scalar: Double) = Vector2d(x - scalar, y - scalar)

    operator fun times(other: Vector2d) = Vector2d(x * other.x, y * other.y)
    operator fun times(scalar: Double) = Vector2d(x * scalar, y * scalar)

    operator fun div(other: Vector2d) = Vector2d(x / other.x, y / other.y)
    operator fun div(scalar: Double) = Vector2d(x / scalar, y / scalar)

    operator fun unaryMinus() = Vector2d(-x, -y)

    fun length() = sqrt(x * x + y * y)
    fun lengthSquared() = x * x + y * y

    fun dot(other: Vector2d) = x * other.x + y * other.y

    fun distanceTo(other: Vector2d): Double {
        val dx = x - other.x
        val dy = y - other.y
        return sqrt(dx * dx + dy * dy)
    }

    fun normalize(): Vector2d {
        val l = length()
        return if (l != 0.0) this / l else zero()
    }

    fun lerp(target: Vector2d, alpha: Double): Vector2d {
        val inv = 1.0 - alpha
        return Vector2d(x * inv + target.x * alpha, y * inv + target.y * alpha)
    }

    fun toFloat() = Vector2f(x.toFloat(), y.toFloat())
    fun toInt() = Vector2i(x.toInt(), y.toInt())

    fun toGeneric() = Vector2(x, y)

    override fun toString() = "Vector2d($x, $y)"
    override fun equals(other: Any?) = other is Vector2d && x == other.x && y == other.y
    override fun hashCode() = 31 * x.hashCode() + y.hashCode()
}