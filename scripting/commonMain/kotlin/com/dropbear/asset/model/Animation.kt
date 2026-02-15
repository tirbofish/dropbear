package com.dropbear.asset.model

import com.dropbear.math.Vector3f
import com.dropbear.math.Quaternionf

data class Animation(
    val name: String,
    val channels: List<AnimationChannel>,
    val duration: Double
)

data class AnimationChannel(
    val targetNode: Int,
    val times: DoubleArray,
    val values: ChannelValues,
    val interpolation: AnimationInterpolation
) {
    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other == null || this::class != other::class) return false

        other as AnimationChannel

        if (targetNode != other.targetNode) return false
        if (!times.contentEquals(other.times)) return false
        if (values != other.values) return false
        if (interpolation != other.interpolation) return false

        return true
    }

    override fun hashCode(): Int {
        var result = targetNode
        result = 31 * result + times.contentHashCode()
        result = 31 * result + values.hashCode()
        result = 31 * result + interpolation.hashCode()
        return result
    }
}

enum class AnimationInterpolation {
    LINEAR,
    STEP,
    CUBIC_SPLINE
}

sealed class ChannelValues {
    data class Translations(val values: List<Vector3f>) : ChannelValues()
    data class Rotations(val values: List<Quaternionf>) : ChannelValues()
    data class Scales(val values: List<Vector3f>) : ChannelValues()
}
