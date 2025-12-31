package com.dropbear.math

/**
 * Represents a position, rotation and scale, typically
 * attached to an entity. 
 */
class Transform(
    var position: Vector3d,
    var rotation: Quaterniond,
    var scale: Vector3d
) {
    companion object {
        fun identity(): Transform {
            return Transform(
                Vector3d.zero(),
                Quaterniond.identity(),
                Vector3d.one(),
            )
        }
    }

    /**
     * Specific constructor for the individual raw primitive values.
     *
     * Primarily used in the JNI.
     */
    constructor(px: Double, py: Double, pz: Double,
                rx: Double, ry: Double, rz: Double, rw: Double,
                sx: Double, sy: Double, sz: Double)
            : this(
        Vector3d(px, py, pz),
        Quaterniond(rx, ry, rz, rw),
        Vector3d(sx, sy, sz)
            )

    override fun toString(): String {
        return "Transform(position=$position, rotation=$rotation, scale=$scale)"
    }
}