package com.dropbear.asset.model

data class Mesh(
    val name: String,
    val numElements: Int,
    val materialIndex: Int,
    val vertices: List<ModelVertex>
)