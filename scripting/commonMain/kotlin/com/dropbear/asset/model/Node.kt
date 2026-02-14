package com.dropbear.asset.model

import com.dropbear.math.Vector3f
import com.dropbear.math.Quaternionf

data class NodeTransform(
    val translation: Vector3f,
    val rotation: Quaternionf,
    val scale: Vector3f
)

data class Node(
    val name: String,
    val parent: Int?,
    val children: List<Int>,
    val transform: NodeTransform
)
