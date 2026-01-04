package com.dropbear.utils

import com.dropbear.math.Vector4d

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