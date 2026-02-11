package com.dropbear.animation

import com.dropbear.EntityId
import com.dropbear.ecs.Component

class AnimationComponent(parentEntity: EntityId) : Component(parentEntity, "AnimationComponent") {
    var activeAnimationIndex: Int?
        get() = getActiveAnimationIndex()
        set(value) = setActiveAnimationIndex(value)

    var time: Float
        get() = getTime()
        set(value) = setTime(value)

    var speed: Float
        get() = getSpeed()
        set(value) = setSpeed(value)

    var looping: Boolean
        get() = getLooping()
        set(value) = setLooping(value)

    var isPlaying: Boolean
        get() = getIsPlaying()
        set(value) = setIsPlaying(value)

    fun pause() {
        isPlaying = false
    }

    fun play() {
        isPlaying = true
    }

    fun stop() {
        isPlaying = false
        time = 0f
        activeAnimationIndex = null
    }

    fun reset() {
        time = 0f
    }

    fun setAnimation(index: Int, speed: Float = 1f) = setActiveAnimationIndex(index).let { setSpeed(speed) }
}

expect fun AnimationComponent.getActiveAnimationIndex(): Int?
expect fun AnimationComponent.setActiveAnimationIndex(index: Int?)
expect fun AnimationComponent.getTime(): Float
expect fun AnimationComponent.setTime(value: Float)
expect fun AnimationComponent.getSpeed(): Float
expect fun AnimationComponent.setSpeed(value: Float)
expect fun AnimationComponent.getLooping(): Boolean
expect fun AnimationComponent.setLooping(value: Boolean)
expect fun AnimationComponent.getIsPlaying(): Boolean
expect fun AnimationComponent.setIsPlaying(value: Boolean)