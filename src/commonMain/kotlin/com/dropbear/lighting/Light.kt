package com.dropbear.lighting

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3d
import com.dropbear.utils.Colour
import com.dropbear.utils.Range

class Light(
    entity: EntityId
): Component(entity, "Light") {
    var position: Vector3d
        get() = getPosition()
        set(value) = setPosition(value)

    var direction: Vector3d
        get() = getDirection()
        set(value) = setDirection(value)

    var colour: Colour
        get() = getColour()
        set(value) = setColour(value)

    var lightType: LightType
        get() = getLightType()
        set(value) = setLightType(value)

    var intensity: Double
        get() = getIntensity()
        set(value) = setIntensity(value)

    var attenuation: Attenuation
        get() = getAttenuation()
        set(value) = setAttenuation(value)

    var enabled: Boolean
        get() = getEnabled()
        set(value) = setEnabled(value)

    var visible: Boolean
        get() = getVisible()
        set(value) = setVisible(value)

    var cutoffAngle: Double
        get() = getCutoffAngle()
        set(value) = setCutoffAngle(value)

    var outerCutoffAngle: Double
        get() = getOuterCutoffAngle()
        set(value) = setOuterCutoffAngle(value)

    var castShadows: Boolean
        get() = getCastShadows()
        set(value) = setCastShadows(value)

    var shadowDepth: Range
        get() = getDepth()
        set(value) = setDepth(value)

    companion object : ComponentType<Light> {
        override fun get(entityId: EntityId): Light? {
            return if (lightExistsForEntity(entityId)) Light(entityId) else null
        }
    }
}



internal expect fun lightExistsForEntity(entityId: EntityId): Boolean

internal expect fun Light.getPosition(): Vector3d
internal expect fun Light.setPosition(position: Vector3d)

internal expect fun Light.getDirection(): Vector3d
internal expect fun Light.setDirection(direction: Vector3d)

internal expect fun Light.getColour(): Colour
internal expect fun Light.setColour(colour: Colour)

internal expect fun Light.getLightType(): LightType
internal expect fun Light.setLightType(lightType: LightType)

internal expect fun Light.getIntensity(): Double
internal expect fun Light.setIntensity(intensity: Double)

internal expect fun Light.getAttenuation(): Attenuation
internal expect fun Light.setAttenuation(attenuation: Attenuation)

internal expect fun Light.getEnabled(): Boolean
internal expect fun Light.setEnabled(enabled: Boolean)

internal expect fun Light.getVisible(): Boolean
internal expect fun Light.setVisible(visible: Boolean)

internal expect fun Light.getCutoffAngle(): Double
internal expect fun Light.setCutoffAngle(cutoffAngle: Double)

internal expect fun Light.getOuterCutoffAngle(): Double
internal expect fun Light.setOuterCutoffAngle(outerCutoffAngle: Double)

internal expect fun Light.getCastShadows(): Boolean
internal expect fun Light.setCastShadows(castShadows: Boolean)

internal expect fun Light.getDepth(): Range
internal expect fun Light.setDepth(depth: Range)