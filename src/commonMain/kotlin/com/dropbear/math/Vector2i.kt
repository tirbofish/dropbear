package com.dropbear.math

import kotlin.math.sqrt
import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

/**
 * A class for holding a vector of `2` integer values.
 */
class Vector2i(
    @JvmField var x: Int,
    @JvmField var y: Int
) {
    companion object {
        @JvmStatic
        fun zero() = Vector2i(0, 0)
    }

    operator fun plus(other: Vector2i) = Vector2i(x + other.x, y + other.y)
    operator fun minus(other: Vector2i) = Vector2i(x - other.x, y - other.y)

    operator fun times(scalar: Int) = Vector2i(x * scalar, y * scalar)

    operator fun div(scalar: Int) = Vector2i(x / scalar, y / scalar)

    fun length(): Double = sqrt((x * x + y * y).toDouble())

    fun toFloat() = Vector2f(x.toFloat(), y.toFloat())
    fun toDouble() = Vector2d(x.toDouble(), y.toDouble())

    override fun toString() = "Vector2i($x, $y)"
}