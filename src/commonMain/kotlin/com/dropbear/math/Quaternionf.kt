package com.dropbear.math

import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic
import kotlin.math.*

class Quaternionf(
    @JvmField var x: Float,
    @JvmField var y: Float,
    @JvmField var z: Float,
    @JvmField var w: Float
) {
    companion object {
        @JvmStatic fun identity() = Quaternionf(0f, 0f, 0f, 1f)

        @JvmStatic
        fun fromEulerAngles(pitch: Float, yaw: Float, roll: Float): Quaternionf {
            val halfPitch = pitch * 0.5f
            val halfYaw = yaw * 0.5f
            val halfRoll = roll * 0.5f
            val sp = sin(halfPitch)
            val cp = cos(halfPitch)
            val sy = sin(halfYaw)
            val cy = cos(halfYaw)
            val sr = sin(halfRoll)
            val cr = cos(halfRoll)

            return Quaternionf(
                x = cy * sr * cp - sy * cr * sp,
                y = sy * cr * cp + cy * sr * sp,
                z = sy * sr * cp - cy * cr * sp,
                w = cy * cr * cp + sy * sr * sp
            )
        }

        @JvmStatic
        fun fromAxisAngle(axis: Vector3f, angleRadians: Float): Quaternionf {
            val normalizedAxis = axis.normalize()
            val halfAngle = angleRadians * 0.5f
            val sinHalf = sin(halfAngle)
            return Quaternionf(
                normalizedAxis.x * sinHalf,
                normalizedAxis.y * sinHalf,
                normalizedAxis.z * sinHalf,
                cos(halfAngle)
            )
        }

        @JvmStatic fun rotateX(angle: Float) = fromAxisAngle(Vector3f(1f, 0f, 0f), angle)
        @JvmStatic fun rotateY(angle: Float) = fromAxisAngle(Vector3f(0f, 1f, 0f), angle)
        @JvmStatic fun rotateZ(angle: Float) = fromAxisAngle(Vector3f(0f, 0f, 1f), angle)

        private fun copySign(magnitude: Float, sign: Float): Float {
            val absMag = abs(magnitude)
            return if (sign < 0f) -absMag else absMag
        }
    }

    operator fun plus(other: Quaternionf) = Quaternionf(x + other.x, y + other.y, z + other.z, w + other.w)
    operator fun minus(other: Quaternionf) = Quaternionf(x - other.x, y - other.y, z - other.z, w - other.w)
    operator fun unaryMinus() = Quaternionf(-x, -y, -z, -w)

    operator fun times(scalar: Float) = Quaternionf(x * scalar, y * scalar, z * scalar, w * scalar)
    operator fun div(scalar: Float) = Quaternionf(x / scalar, y / scalar, z / scalar, w / scalar)

    operator fun times(other: Quaternionf): Quaternionf {
        return Quaternionf(
            w * other.x + x * other.w + y * other.z - z * other.y,
            w * other.y - x * other.z + y * other.w + z * other.x,
            w * other.z + x * other.y - y * other.x + z * other.w,
            w * other.w - x * other.x - y * other.y - z * other.z
        )
    }

    operator fun times(vector: Vector3f): Vector3f {
        val qVec = Vector3f(x, y, z)
        val uv = qVec.cross(vector)
        val uuv = qVec.cross(uv)
        return vector + ((uv * w) + uuv) * 2f
    }

    fun dot(other: Quaternionf) = x * other.x + y * other.y + z * other.z + w * other.w
    fun lengthSquared() = x * x + y * y + z * z + w * w
    fun length() = sqrt(lengthSquared())

    fun normalize(): Quaternionf {
        val len = length()
        return if (len == 0f) identity() else this / len
    }

    fun inverse(): Quaternionf {
        val lenSq = lengthSquared()
        if (lenSq == 0f) return identity()
        val inv = 1.0f / lenSq
        return Quaternionf(-x * inv, -y * inv, -z * inv, w * inv)
    }

    fun conjugate() = Quaternionf(-x, -y, -z, w)

    fun toDouble() = Quaterniond(x.toDouble(), y.toDouble(), z.toDouble(), w.toDouble())

    fun toEulerAngles(): Vector3f {
        val sinr_cosp = 2f * (w * z + x * y)
        val cosr_cosp = 1f - 2f * (y * y + z * z)
        val roll = atan2(sinr_cosp, cosr_cosp)

        val sinp = 2f * (w * x - y * z)
        val pitch = if (abs(sinp) >= 1f) {
            copySign(PI.toFloat() / 2f, sinp)
        } else {
            asin(sinp)
        }

        val siny_cosp = 2f * (w * y + z * x)
        val cosy_cosp = 1f - 2f * (x * x + y * y)
        val yaw = atan2(siny_cosp, cosy_cosp)

        return Vector3f(pitch, yaw, roll)
    }

    override fun toString() = "Quaternionf($x, $y, $z, $w)"
}