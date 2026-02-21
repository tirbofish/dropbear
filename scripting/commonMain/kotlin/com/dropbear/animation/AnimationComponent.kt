package com.dropbear.animation

import com.dropbear.EntityId
import com.dropbear.components.Camera
import com.dropbear.components.cameraExistsForEntity
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType

class AnimationComponent(val parentEntity: EntityId) : Component(parentEntity, "AnimationComponent") {
    val availableAnimations: List<String>
        get() = getAvailableAnimations()
    
    var activeAnimationIndex: Int?
        get() = getActiveAnimationIndex()
        set(value) = setActiveAnimationIndex(value)

    var time: Double
        get() = getTime()
        set(value) = setTime(value)

    var speed: Double
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
        if (activeAnimationIndex == null) {
            val first = availableAnimations.firstOrNull()
            if (first != null) {
                activeAnimationIndex = 0
            }
        }
        isPlaying = true
    }

    fun stop() {
        isPlaying = false
        time = 0.0
        activeAnimationIndex = null
    }

    fun reset() {
        time = 0.0
    }

    fun setAnimation(index: Int) = setActiveAnimationIndex(index)
    fun setAnimation(animationName: String) {
        val index = getIndexFromString(animationName) ?: return
        setActiveAnimationIndex(index)
    }

    companion object : ComponentType<AnimationComponent> {
        override fun get(entityId: EntityId): AnimationComponent? {
            return if (animationComponentExistsForEntity(entityId)) AnimationComponent(entityId) else null
        }
    }
}

expect fun animationComponentExistsForEntity(entityId: EntityId): Boolean

expect fun AnimationComponent.getActiveAnimationIndex(): Int?
expect fun AnimationComponent.setActiveAnimationIndex(index: Int?)
expect fun AnimationComponent.getTime(): Double
expect fun AnimationComponent.setTime(value: Double)
expect fun AnimationComponent.getSpeed(): Double
expect fun AnimationComponent.setSpeed(value: Double)
expect fun AnimationComponent.getLooping(): Boolean
expect fun AnimationComponent.setLooping(value: Boolean)
expect fun AnimationComponent.getIsPlaying(): Boolean
expect fun AnimationComponent.setIsPlaying(value: Boolean)
expect fun AnimationComponent.getIndexFromString(name: String): Int?
expect fun AnimationComponent.getAvailableAnimations(): List<String>