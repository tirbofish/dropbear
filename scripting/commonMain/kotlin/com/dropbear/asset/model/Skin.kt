package com.dropbear.asset.model

data class Skin(
    val name: String,
    val joints: List<Int>,
    val inverseBindMatrices: List<DoubleArray>,
    val skeletonRoot: Int?
)
