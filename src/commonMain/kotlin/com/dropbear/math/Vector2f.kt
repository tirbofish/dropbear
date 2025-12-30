package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

/**
 * A class for holding a vector of `2` float values.
 */
class Vector2f(
    @JvmField var x: Float,
    @JvmField var y: Float
) {
    companion object {
        @JvmStatic
        fun zero() = Vector2f(0f, 0f)
    }

    operator fun plus(other: Vector2f) = Vector2f(x + other.x, y + other.y)
    operator fun plus(scalar: Float) = Vector2f(x + scalar, y + scalar)

    operator fun minus(other: Vector2f) = Vector2f(x - other.x, y - other.y)
    operator fun minus(scalar: Float) = Vector2f(x - scalar, y - scalar)

    operator fun times(other: Vector2f) = Vector2f(x * other.x, y * other.y)
    operator fun times(scalar: Float) = Vector2f(x * scalar, y * scalar)

    operator fun div(other: Vector2f) = Vector2f(x / other.x, y / other.y)
    operator fun div(scalar: Float) = Vector2f(x / scalar, y / scalar)

    operator fun unaryMinus() = Vector2f(-x, -y)

    fun length(): Float = sqrt(x * x + y * y)
    fun lengthSquared(): Float = x * x + y * y

    fun normalize(): Vector2f {
        val l = length()
        return if (l != 0f) Vector2f(x / l, y / l) else zero()
    }

    // --- Conversions ---
    fun toDouble() = Vector2d(x.toDouble(), y.toDouble())
    fun toInt() = Vector2i(x.toInt(), y.toInt())

    override fun toString() = "Vector2f($x, $y)"
}