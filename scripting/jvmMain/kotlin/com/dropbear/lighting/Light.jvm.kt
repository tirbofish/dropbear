package com.dropbear.lighting

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.math.Vector3d
import com.dropbear.utils.Colour
import com.dropbear.utils.Range

internal actual fun lightExistsForEntity(entityId: EntityId): Boolean {
    return LightNative.lightExistsForEntity(DropbearEngine.native.worldHandle, entityId.raw)
}

internal actual fun Light.getPosition(): Vector3d {
    return LightNative.getPosition(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setPosition(position: Vector3d) {
    return LightNative.setPosition(DropbearEngine.native.worldHandle, entity.raw, position)
}

internal actual fun Light.getDirection(): Vector3d {
    return LightNative.getDirection(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setDirection(direction: Vector3d) {
    return LightNative.setDirection(DropbearEngine.native.worldHandle, entity.raw, direction)
}

internal actual fun Light.getColour(): Colour {
    return LightNative.getColour(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setColour(colour: Colour) {
    return LightNative.setColour(DropbearEngine.native.worldHandle, entity.raw, colour)
}

internal actual fun Light.getLightType(): LightType {
    val result = LightNative.getLightType(DropbearEngine.native.worldHandle, entity.raw)
    return LightType.entries[result]
}

internal actual fun Light.setLightType(lightType: LightType) {
    return LightNative.setLightType(DropbearEngine.native.worldHandle, entity.raw, lightType.ordinal)
}

internal actual fun Light.getIntensity(): Double {
    return LightNative.getIntensity(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setIntensity(intensity: Double) {
    return LightNative.setIntensity(DropbearEngine.native.worldHandle, entity.raw, intensity)
}

internal actual fun Light.getAttenuation(): Attenuation {
    return LightNative.getAttenuation(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setAttenuation(attenuation: Attenuation) {
    return LightNative.setAttenuation(DropbearEngine.native.worldHandle, entity.raw, attenuation)
}

internal actual fun Light.getEnabled(): Boolean {
    return LightNative.getEnabled(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setEnabled(enabled: Boolean) {
    return LightNative.setEnabled(DropbearEngine.native.worldHandle, entity.raw, enabled)
}

internal actual fun Light.getCutoffAngle(): Double {
    return LightNative.getCutoffAngle(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setCutoffAngle(cutoffAngle: Double) {
    return LightNative.setCutoffAngle(DropbearEngine.native.worldHandle, entity.raw, cutoffAngle)
}

internal actual fun Light.getOuterCutoffAngle(): Double {
    return LightNative.getOuterCutoffAngle(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setOuterCutoffAngle(outerCutoffAngle: Double) {
    return LightNative.setOuterCutoffAngle(DropbearEngine.native.worldHandle, entity.raw, outerCutoffAngle)
}

internal actual fun Light.getCastShadows(): Boolean {
    return LightNative.getCastsShadows(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setCastShadows(castShadows: Boolean) {
    return LightNative.setCastsShadows(DropbearEngine.native.worldHandle, entity.raw, castShadows)
}

internal actual fun Light.getDepth(): Range {
    return LightNative.getDepth(DropbearEngine.native.worldHandle, entity.raw)
}

internal actual fun Light.setDepth(depth: Range) {
    return LightNative.setDepth(DropbearEngine.native.worldHandle, entity.raw, depth)
}