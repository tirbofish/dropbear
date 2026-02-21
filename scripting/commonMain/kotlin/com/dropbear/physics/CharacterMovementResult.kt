package com.dropbear.physics

import com.dropbear.math.Vector3d

data class CharacterMovementResult(
    val translation: Vector3d,
    val grounded: Boolean,
    val isSlidingDownSlope: Boolean,
)