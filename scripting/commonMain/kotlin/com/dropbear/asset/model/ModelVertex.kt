package com.dropbear.asset.model

import com.dropbear.math.Vector2f
import com.dropbear.math.Vector3f
import com.dropbear.math.Vector4f

data class ModelVertex(
    val position: Vector3f,
    val normal: Vector3f,
    val tangent: Vector4f,
    val texCoords0: Vector2f,
    val texCoords1: Vector2f,
    val colour0: Vector4f,
    val joints0: IntArray,
    val weights0: Vector4f
)
