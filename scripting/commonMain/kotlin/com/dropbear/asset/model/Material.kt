package com.dropbear.asset.model

import com.dropbear.asset.Texture
import com.dropbear.math.Vector2f
import com.dropbear.math.Vector3f
import com.dropbear.math.Vector4f

data class Material(
    val name: String,
    val diffuseTexture: Texture,
    val normalTexture: Texture,
    val tint: Vector4f,
    val emissiveFactor: Vector3f,
    val metallicFactor: Float,
    val roughnessFactor: Float,
    val alphaCutoff: Float?,
    val doubleSided: Boolean,
    val occlusionStrength: Float,
    val normalScale: Float,
    val uvTiling: Vector2f,
    val emissiveTexture: Texture?,
    val metallicRoughnessTexture: Texture?,
    val occlusionTexture: Texture?
)
