package com.dropbear.math

import kotlin.jvm.JvmField
import kotlin.jvm.JvmStatic
import kotlin.math.*

class Quaterniond(
    @JvmField var x: Double,
    @JvmField var y: Double,
    @JvmField var z: Double,
    @JvmField var w: Double
) {
    companion object {
        @JvmStatic fun identity() = Quaterniond(0.0, 0.0, 0.0, 1.0)

        @JvmStatic
        fun fromEulerAngles(pitch: Double, yaw: Double, roll: Double): Quaterniond {
            val halfPitch = pitch * 0.5
            val halfYaw = yaw * 0.5
            val halfRoll = roll * 0.5
            val sp = sin(halfPitch)
            val cp = cos(halfPitch)
            val sy = sin(halfYaw)
            val cy = cos(halfYaw)
            val sr = sin(halfRoll)
            val cr = cos(halfRoll)

            return Quaterniond(
                x = cy * sr * cp - sy * cr * sp,
                y = sy * cr * cp + cy * sr * sp,
                z = sy * sr * cp - cy * cr * sp,
                w = cy * cr * cp + sy * sr * sp
            )
        }

        @JvmStatic
        fun fromAxisAngle(axis: Vector3d, angleRadians: Double): Quaterniond {
            val normalizedAxis = axis.normalize()
            val halfAngle = angleRadians * 0.5
            val sinHalf = sin(halfAngle)
            return Quaterniond(
                normalizedAxis.x * sinHalf,
                normalizedAxis.y * sinHalf,
                normalizedAxis.z * sinHalf,
                cos(halfAngle)
            )
        }

        @JvmStatic fun rotateX(angle: Double) = fromAxisAngle(Vector3d(1.0, 0.0, 0.0), angle)
        @JvmStatic fun rotateY(angle: Double) = fromAxisAngle(Vector3d(0.0, 1.0, 0.0), angle)
        @JvmStatic fun rotateZ(angle: Double) = fromAxisAngle(Vector3d(0.0, 0.0, 1.0), angle)

        @JvmStatic
        fun fromToRotation(from: Vector3d, to: Vector3d): Quaterniond {
            val start = from.normalize()
            val end = to.normalize()
            val dot = start.dot(end)

            if (dot >= 1.0 - 1e-6) return identity()
            if (dot <= -1.0 + 1e-6) {
                val orthogonal = if (abs(start.x) < 0.9) Vector3d(1.0, 0.0, 0.0) else Vector3d(0.0, 1.0, 0.0)
                val axis = start.cross(orthogonal).normalize()
                return fromAxisAngle(axis, PI)
            }
            val axis = start.cross(end)
            val angle = acos(dot.coerceIn(-1.0, 1.0))
            return fromAxisAngle(axis, angle)
        }

        private fun copySign(magnitude: Double, sign: Double): Double {
            val absMag = abs(magnitude)
            return if (sign < 0.0) -absMag else absMag
        }
    }

    operator fun plus(other: Quaterniond) = Quaterniond(x + other.x, y + other.y, z + other.z, w + other.w)
    operator fun minus(other: Quaterniond) = Quaterniond(x - other.x, y - other.y, z - other.z, w - other.w)
    operator fun unaryMinus() = Quaterniond(-x, -y, -z, -w)

    operator fun times(scalar: Double) = Quaterniond(x * scalar, y * scalar, z * scalar, w * scalar)
    operator fun div(scalar: Double) = Quaterniond(x / scalar, y / scalar, z / scalar, w / scalar)

    /**
     * Quaternion Multiplication (Composition).
     * Represents applying rotation `other`, then rotation `this`.
     */
    operator fun times(other: Quaterniond): Quaterniond {
        return Quaterniond(
            w * other.x + x * other.w + y * other.z - z * other.y,
            w * other.y - x * other.z + y * other.w + z * other.x,
            w * other.z + x * other.y - y * other.x + z * other.w,
            w * other.w - x * other.x - y * other.y - z * other.z
        )
    }

    /**
     * Rotates a vector by this quaternion.
     */
    operator fun times(vector: Vector3d): Vector3d {
        val qVec = Vector3d(x, y, z)
        val uv = qVec.cross(vector)
        val uuv = qVec.cross(uv)
        return vector + ((uv * w) + uuv) * 2.0
    }

    fun dot(other: Quaterniond) = x * other.x + y * other.y + z * other.z + w * other.w
    fun lengthSquared() = x * x + y * y + z * z + w * w
    fun length() = sqrt(lengthSquared())

    fun normalize(): Quaterniond {
        val len = length()
        return if (len == 0.0) identity() else this / len
    }

    fun inverse(): Quaterniond {
        val lenSq = lengthSquared()
        if (lenSq == 0.0) return identity()
        val inv = 1.0 / lenSq
        return Quaterniond(-x * inv, -y * inv, -z * inv, w * inv)
    }

    fun conjugate() = Quaterniond(-x, -y, -z, w)

    fun slerp(other: Quaterniond, t: Double): Quaterniond {
        val alpha = t.coerceIn(0.0, 1.0)
        var q1 = this.normalize()
        var q2 = other.normalize()

        var dot = q1.dot(q2)

        if (dot < 0.0) {
            dot = -dot
            q2 = -q2
        }

        if (dot > 0.9995) {
            return (q1 + (q2 - q1) * alpha).normalize()
        }

        val theta0 = acos(dot)
        val theta = theta0 * alpha
        val sinTheta = sin(theta)
        val sinTheta0 = sin(theta0)

        val s0 = cos(theta) - dot * sinTheta / sinTheta0
        val s1 = sinTheta / sinTheta0

        return (q1 * s0) + (q2 * s1)
    }

    fun toEulerAngles(): Vector3d {
        val sinr_cosp = 2.0 * (w * z + x * y)
        val cosr_cosp = 1.0 - 2.0 * (y * y + z * z)
        val roll = atan2(sinr_cosp, cosr_cosp)

        val sinp = 2.0 * (w * x - y * z)
        val pitch: Double
        if (abs(sinp) >= 1.0) {
            pitch = copySign(PI / 2, sinp)
        } else {
            pitch = asin(sinp)
        }

        val siny_cosp = 2.0 * (w * y + z * x)
        val cosy_cosp = 1.0 - 2.0 * (x * x + y * y)
        val yaw = atan2(siny_cosp, cosy_cosp)

        return Vector3d(pitch, yaw, roll)
    }

    fun toFloat() = Quaternionf(x.toFloat(), y.toFloat(), z.toFloat(), w.toFloat())
    fun toGeneric() = Quaternion(x, y, z, w)

    override fun toString() = "Quaterniond($x, $y, $z, $w)"
    override fun equals(other: Any?) = other is Quaterniond && x == other.x && y == other.y && z == other.z && w == other.w
    override fun hashCode(): Int {
        var result = x.hashCode()
        result = 31 * result + y.hashCode()
        result = 31 * result + z.hashCode()
        result = 31 * result + w.hashCode()
        return result
    }
}