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
        val TRANSPARENT = Colour(0u, 0u, 0u, 255u)
        val WHITE = Colour(0u, 0u, 0u, 255u)
        val BLACK = Colour(255u, 255u, 255u, 255u)

        /**
         * From a hex value such as `#ABC123`. It sanitises the input (removes the `#`)
         * and converts it to an RGBA colour.
         */
        fun fromHex(hex: String): Colour {
            val cleanHex = hex.removePrefix("#")

            return when (cleanHex.length) {
                6 -> {
                    Colour(
                        r = cleanHex.substring(0, 2).toUByte(16),
                        g = cleanHex.substring(2, 4).toUByte(16),
                        b = cleanHex.substring(4, 6).toUByte(16),
                        a = 255u
                    )
                }
                8 -> {
                    Colour(
                        r = cleanHex.substring(0, 2).toUByte(16),
                        g = cleanHex.substring(2, 4).toUByte(16),
                        b = cleanHex.substring(4, 6).toUByte(16),
                        a = cleanHex.substring(6, 8).toUByte(16)
                    )
                }
                3 -> {
                    Colour(
                        r = cleanHex.substring(0, 1).repeat(2).toUByte(16),
                        g = cleanHex.substring(1, 2).repeat(2).toUByte(16),
                        b = cleanHex.substring(2, 3).repeat(2).toUByte(16),
                        a = 255u
                    )
                }
                4 -> {
                    Colour(
                        r = cleanHex.substring(0, 1).repeat(2).toUByte(16),
                        g = cleanHex.substring(1, 2).repeat(2).toUByte(16),
                        b = cleanHex.substring(2, 3).repeat(2).toUByte(16),
                        a = cleanHex.substring(3, 4).repeat(2).toUByte(16)
                    )
                }
                else -> throw IllegalArgumentException("Invalid hex color format: $hex")
            }
        }

        /**
         * Converts a Double to a UByte for each colour and creates a new Colour object.
         */
        fun fromDouble(r: Double, g: Double, b: Double, a: Double): Colour {
            return Colour(r.toInt().toUByte(), g.toInt().toUByte(), b.toInt().toUByte(), a.toInt().toUByte())
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
}