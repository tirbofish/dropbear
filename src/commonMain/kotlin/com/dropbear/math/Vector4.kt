package com.dropbear.math

import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

/**
 * A Generic Vector4.
 * WARNING: Uses boxing. Prefer [Vector4d]/[Vector4f]/[Vector4i] for performance.
 */
class Vector4<T : Number>(
    @JvmField var x: T,
    @JvmField var y: T,
    @JvmField var z: T,
    @JvmField var w: T
) {
    companion object {
        @JvmStatic
        fun fromDoubles(x: Double, y: Double, z: Double, w: Double): Vector4<Double> = Vector4(x, y, z, w)

        @JvmStatic
        fun zero(): Vector4<Double> = Vector4(0.0, 0.0, 0.0, 0.0)
    }

    fun asVector4d(): Vector4d = Vector4d(x.toDouble(), y.toDouble(), z.toDouble(), w.toDouble())
    fun asVector4f(): Vector4f = Vector4f(x.toFloat(), y.toFloat(), z.toFloat(), w.toFloat())
    fun asVector4i(): Vector4i = Vector4i(x.toInt(), y.toInt(), z.toInt(), w.toInt())

    operator fun component1() = x
    operator fun component2() = y
    operator fun component3() = z
    operator fun component4() = w

    override fun toString() = "Vector4($x, $y, $z, $w)"

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Vector4<*>) return false
        return x == other.x && y == other.y && z == other.z && w == other.w
    }

    override fun hashCode(): Int {
        var result = x.hashCode()
        result = 31 * result + y.hashCode()
        result = 31 * result + z.hashCode()
        result = 31 * result + w.hashCode()
        return result
    }
}