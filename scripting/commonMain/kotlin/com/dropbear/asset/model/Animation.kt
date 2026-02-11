package com.dropbear.asset.model

import com.dropbear.math.Vector3f
import com.dropbear.math.Quaternionf

data class Animation(
    val name: String,
    val channels: List<AnimationChannel>,
    val duration: Float
)

data class AnimationChannel(
    val targetNode: Int,
    val times: DoubleArray,
    val values: ChannelValues,
    val interpolation: AnimationInterpolation
)

enum class AnimationInterpolation {
    LINEAR,
    STEP,
    CUBICSPLINE
}

sealed class ChannelValues {
    data class Translations(val values: List<Vector3f>) : ChannelValues()
    data class Rotations(val values: List<Quaternionf>) : ChannelValues()
    data class Scales(val values: List<Vector3f>) : ChannelValues()
}
