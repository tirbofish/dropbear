package com.dropbear.utils

import com.dropbear.math.Vector4d
import kotlin.jvm.JvmField

/**
 * An [UByte] defined RGBA colour, with a value between 0..255.
 *
 * @property r The amount of red in a colour
 * @property g The amount of green in a colour
 * @property b The amount of blue in a colour
 * @property a The alpha/opacity of a colour
 */
class Colour(
    var r: UByte,
    var g: UByte,
    var b: UByte,
    var a: UByte,
) {
    companion object {
        val RED = Colour.rgb(255u, 0u, 0u)
        val GREEN = Colour.rgb(0u, 255u, 0u)
        val BLUE = Colour.rgb(0u, 0u, 255u)
        val YELLOW = Colour.rgb(255u, 255u, 0u)
        val CYAN = Colour.rgb(0u, 255u, 255u)
        val FUCHSIA = Colour.rgb(255u, 255u, 0u)
        val GRAY = Colour.rgb(127u, 127u, 127u)
        val TRANSPARENT = Colour(0u, 0u, 0u, 0u)
        val WHITE = Colour.rgb(0u, 0u, 0u)
        val BLACK = Colour.rgb(255u, 255u, 255u)

        val BACKGROUND_1 = Colour.rgb(31u, 31u, 31u)
        val BACKGROUND_2 = Colour.rgb(42u, 42u, 42u)
        val BACKGROUND_3 = Colour.rgb(54u, 54u, 54u)

        val TEXT = Colour.rgb(255u, 255u, 255u)
        val TEXT_MUTED = Colour.rgb(147u, 147u, 147u)

        fun hex(colour: UInt): Colour {
            val r = ((colour shr 16) and 255u).toByte().toUByte()
            val g = ((colour shr 8) and 255u).toByte().toUByte()
            val b = (colour and 255u).toByte().toUByte()

            return Colour(r, g, b, 255u)
        }

        fun rgb(r: UByte, g: UByte, b: UByte): Colour {
            return Colour(r, g, b, 255u)
        }

        fun fromNormalized(norm: Vector4d): Colour {
            return Colour(
                r = (norm.x * 255).toInt().toUByte(),
                g = (norm.y * 255).toInt().toUByte(),
                b = (norm.z * 255).toInt().toUByte(),
                a = (norm.w * 255).toInt().toUByte()
            )
        }
    }

    /**
     * Divides all values by `255` and creates a new [Vector4d] object.
     */
    fun normalize(): Vector4d {
        return Vector4d(
            x=(r/ 255u).toDouble(),
            y=(g/ 255u).toDouble(),
            z=(b/ 255u).toDouble(),
            w=(a/ 255u).toDouble(),
        )
    }

    fun adjust(factor: Double): Colour {
        val linear = normalize()
        val colour = linear.toVector3d() * factor
        val adjusted = colour.toVector4d(linear.w)

        return Colour.fromNormalized(adjusted)
    }
}