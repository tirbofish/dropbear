package com.dropbear.math

import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic

/**
 * Generic Quaternion wrapper.
 * WARNING: Uses boxing. Prefer [Quaterniond]/[Quaternionf] for performance.
 */
class Quaternion<T: Number>(
    @JvmField var x: T,
    @JvmField var y: T,
    @JvmField var z: T,
    @JvmField var w: T
) {
    companion object {
        @JvmStatic
        fun identity(): Quaternion<Double> = Quaternion(0.0, 0.0, 0.0, 1.0)

        @JvmStatic
        fun fromEulerAngles(pitch: Double, yaw: Double, roll: Double): Quaternion<Double> {
            return Quaterniond.fromEulerAngles(pitch, yaw, roll).toGeneric()
        }
    }

    fun asQuaterniond() = Quaterniond(x.toDouble(), y.toDouble(), z.toDouble(), w.toDouble())
    fun asQuaternionf() = Quaternionf(x.toFloat(), y.toFloat(), z.toFloat(), w.toFloat())

    fun normalize(): Quaternion<Double> = asQuaterniond().normalize().toGeneric()
    fun inverse(): Quaternion<Double> = asQuaterniond().inverse().toGeneric()

    override fun toString() = "Quaternion($x, $y, $z, $w)"

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Quaternion<*>) return false
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