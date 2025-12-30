package com.dropbear.math

import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

/**
 * A Generic Vector2.
 * WARNING: Uses boxing. Prefer [Vector2d]/[Vector2f]/[Vector2i] for performance.
 */
class Vector2<T : Number>(
    @JvmField var x: T,
    @JvmField var y: T
) {
    companion object {
        @JvmStatic
        fun fromDoubles(x: Double, y: Double): Vector2<Double> = Vector2(x, y)
    }

    fun asVector2d(): Vector2d = Vector2d(x.toDouble(), y.toDouble())
    fun asVector2f(): Vector2f = Vector2f(x.toFloat(), y.toFloat())
    fun asVector2i(): Vector2i = Vector2i(x.toInt(), y.toInt())

    override fun toString() = "Vector2($x, $y)"

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Vector2<*>) return false
        return x == other.x && y == other.y
    }

    override fun hashCode() = 31 * x.hashCode() + y.hashCode()
}