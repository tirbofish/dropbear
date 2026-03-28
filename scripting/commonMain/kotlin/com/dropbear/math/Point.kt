package com.dropbear.math

/**
 * Defined a position and a rotation in world-space.
 *
 * @param position World-space position of the waypoint.
 * @param rotation Explicit orientation to hold at this point. When `null`, the runtime derives
 *   orientation from the path tangent (i.e., the camera looks in the direction of travel).
 */
data class Point(
    val position: Vector3d,
    val rotation: Quaterniond? = null,
)