package com.dropbear.physics

import com.dropbear.math.Vector3d

var gravity: Vector3d
    get() = getGravity()
    set(value) = setGravity(value)

internal expect fun getGravity(): Vector3d
internal expect fun setGravity(gravity: Vector3d)