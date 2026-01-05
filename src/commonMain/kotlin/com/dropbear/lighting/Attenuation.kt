package com.dropbear.lighting

/**
 * Attenuation parameters for a light source.
 *
 * @param constant The constant attenuation factor.
 * @param linear The linear attenuation factor.
 * @param quadratic The quadratic attenuation factor.
 */
data class Attenuation(
    val constant: Float = 1f,
    val linear: Float = 0f,
    val quadratic: Float = 0f
)