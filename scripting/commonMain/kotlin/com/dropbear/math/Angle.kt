package com.dropbear.math

import kotlin.math.*

/**
 * Represents an angle with support for both degrees and radians.
 * Provides normalization and common trigonometric operations.
 */
class Angle private constructor(private val radians: Double) {

    companion object {
        /** Creates an Angle from degrees */
        fun fromDegrees(degrees: Double): Angle = Angle(degreesToRadians(degrees))

        /** Creates an Angle from radians */
        fun fromRadians(radians: Double): Angle = Angle(radians)

        /** Zero angle */
        val ZERO = Angle(0.0)

        /** Right angle (90 degrees) */
        val RIGHT = fromDegrees(90.0)

        /** Straight angle (180 degrees) */
        val STRAIGHT = fromDegrees(180.0)

        /** Full rotation (360 degrees) */
        val FULL = fromDegrees(360.0)
    }

    /** Get the angle value in degrees */
    val degrees: Double
        get() = radiansToDegrees(radians)

    /** Get the angle value in radians */
    fun toRadians(): Double = radians

    /** Get the angle value in degrees */
    fun toDegrees(): Double = degrees

    /** Normalize the angle to range [0, 360) degrees or [0, 2π) radians */
    fun normalized(): Angle {
        val normalized = radians % (2 * PI)
        return Angle(if (normalized < 0) normalized + 2 * PI else normalized)
    }

    /** Normalize the angle to range [-180, 180) degrees or [-π, π) radians */
    fun normalizedSigned(): Angle {
        val norm = normalized().radians
        return if (norm > PI) Angle(norm - 2 * PI) else Angle(norm)
    }

    operator fun plus(other: Angle): Angle = Angle(radians + other.radians)
    operator fun minus(other: Angle): Angle = Angle(radians - other.radians)
    operator fun times(scalar: Double): Angle = Angle(radians * scalar)
    operator fun div(scalar: Double): Angle = Angle(radians / scalar)
    operator fun unaryMinus(): Angle = Angle(-radians)

    operator fun compareTo(other: Angle): Int = radians.compareTo(other.radians)

    fun sin(): Double = sin(radians)
    fun cos(): Double = cos(radians)
    fun tan(): Double = tan(radians)

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is Angle) return false
        return radians == other.radians
    }

    override fun hashCode(): Int = radians.hashCode()

    override fun toString(): String = "${degrees}°"
}

fun Double.degrees(): Angle = Angle.fromDegrees(this)
fun Double.radians(): Angle = Angle.fromRadians(this)
fun Int.degrees(): Angle = Angle.fromDegrees(this.toDouble())
fun Int.radians(): Angle = Angle.fromRadians(this.toDouble())