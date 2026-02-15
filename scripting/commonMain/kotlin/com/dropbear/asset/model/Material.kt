package com.dropbear.asset.model

import com.dropbear.asset.Texture
import com.dropbear.math.Vector2d
import com.dropbear.math.Vector3d
import com.dropbear.math.Vector4d

data class Material(
    val name: String,
    val diffuseTexture: Texture,
    val normalTexture: Texture,
    val tint: Vector4d,
    val emissiveFactor: Vector3d,
    val metallicFactor: Double,
    val roughnessFactor: Double,
    val alphaCutoff: Double?,
    val doubleSided: Boolean,
    val occlusionStrength: Double,
    val normalScale: Double,
    val uvTiling: Vector2d,
    val emissiveTexture: Texture?,
    val metallicRoughnessTexture: Texture?,
    val occlusionTexture: Texture?
)
