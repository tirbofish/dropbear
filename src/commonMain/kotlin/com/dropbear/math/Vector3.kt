package com.dropbear.math

import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

/**
 * A Generic Vector3.
 * WARNING: Uses boxing. Prefer [Vector3d]/[Vector3f]/[Vector3i] for performance.
 */
class Vector3<T : Number>(
    @JvmField var x: T,
    @JvmField var y: T,
    @JvmField var z: T
) {
    companion object {
        @JvmStatic
        fun fromDoubles(x: Double, y: Double, z: Double): Vector3<Double> = Vector3(x, y, z)

        @JvmStatic
        fun zero(): Vector3<Double> = Vector3(0.0, 0.0, 0.0)
    }

    fun asVector3d(): Vector3d = Vector3d(x.toDouble(), y.toDouble(), z.toDouble())
    fun asVector3f(): Vector3f = Vector3f(x.toFloat(), y.toFloat(), z.toFloat())
    fun asVector3i(): Vector3i = Vector3i(x.toInt(), y.toInt(), z.toInt())

    override fun toString() = "Vector3($x, $y, $z)"

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Vector3<*>) return false
        return x == other.x && y == other.y && z == other.z
    }

    override fun hashCode(): Int {
        var result = x.hashCode()
        result = 31 * result + y.hashCode()
        result = 31 * result + z.hashCode()
        return result
    }
}