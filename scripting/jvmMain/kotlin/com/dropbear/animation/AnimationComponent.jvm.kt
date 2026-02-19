package com.dropbear.animation

import com.dropbear.DropbearEngine
import com.dropbear.EntityId

actual fun AnimationComponent.getActiveAnimationIndex(): Int? {
    return AnimationComponentNative.getActiveAnimationIndex(DropbearEngine.native.worldHandle, parentEntity.raw)
}

actual fun AnimationComponent.setActiveAnimationIndex(index: Int?) {
    return AnimationComponentNative.setActiveAnimationIndex(DropbearEngine.native.worldHandle, parentEntity.raw, index)
}

actual fun AnimationComponent.getTime(): Double {
    return AnimationComponentNative.getTime(DropbearEngine.native.worldHandle, parentEntity.raw)
}

actual fun AnimationComponent.setTime(value: Double) {
    return AnimationComponentNative.setTime(DropbearEngine.native.worldHandle, parentEntity.raw, value)
}

actual fun AnimationComponent.getSpeed(): Double {
    return AnimationComponentNative.getSpeed(DropbearEngine.native.worldHandle, parentEntity.raw)
}

actual fun AnimationComponent.setSpeed(value: Double) {
    return AnimationComponentNative.setSpeed(DropbearEngine.native.worldHandle, parentEntity.raw, value)
}

actual fun AnimationComponent.getLooping(): Boolean {
    return AnimationComponentNative.getLooping(DropbearEngine.native.worldHandle, parentEntity.raw)
}

actual fun AnimationComponent.setLooping(value: Boolean) {
    return AnimationComponentNative.setLooping(DropbearEngine.native.worldHandle, parentEntity.raw, value)
}

actual fun AnimationComponent.getIsPlaying(): Boolean {
    return AnimationComponentNative.getIsPlaying(DropbearEngine.native.worldHandle, parentEntity.raw)
}

actual fun AnimationComponent.setIsPlaying(value: Boolean) {
    return AnimationComponentNative.setIsPlaying(DropbearEngine.native.worldHandle, parentEntity.raw, value)
}

actual fun AnimationComponent.getIndexFromString(name: String): Int? {
    return AnimationComponentNative.getIndexFromString(DropbearEngine.native.worldHandle, parentEntity.raw, name)
}

actual fun AnimationComponent.getAvailableAnimations(): List<String> {
    return AnimationComponentNative.getAvailableAnimations(DropbearEngine.native.worldHandle, parentEntity.raw).asList()
}

actual fun animationComponentExistsForEntity(entityId: EntityId): Boolean {
    return AnimationComponentNative.animationComponentExistsForEntity(DropbearEngine.native.worldHandle, entityId.raw)
}