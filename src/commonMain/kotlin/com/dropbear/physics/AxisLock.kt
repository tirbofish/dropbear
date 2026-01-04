package com.dropbear.physics

/**
 * Defines a custom axis lock.
 *
 * @param x The x value lock status
 * @param y The y value lock status
 * @param z The z value lock status
 */
data class AxisLock(
    var x: Boolean = false,
    var y: Boolean = false,
    var z: Boolean = false,
) {
    override fun toString(): String {
        return "AxisLock(x=$x, y=$y, z=$z)"
    }
}
