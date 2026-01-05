package com.dropbear.lighting

import com.dropbear.EntityId
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType
import com.dropbear.math.Vector3d
import com.dropbear.utils.Colour
import com.dropbear.utils.Range

/**
 * Describes a light source, as defined in `dropbear_engine::lighting::Light` and
 * `dropbear_engine::lighting::LightComponent`.
 *
 * This class is a component under the name `Light` and must be attached to an entity as a component.
 *
 * @property entity The entity this component is attached to.
 */
class Light(
    val entity: EntityId
): Component(entity, "Light") {
    /**
     * The position of the light in 3D space.
     *
     * This is relevant for point lights and spotlights.
     */
    var position: Vector3d
        get() = getPosition()
        set(value) = setPosition(value)

    /**
     * The direction the light is pointing.
     *
     * This is relevant for directional lights and spotlights.
     */
    var direction: Vector3d
        get() = getDirection()
        set(value) = setDirection(value)

    /**
     * The colour of the light.
     *
     * Typical: White (1.0, 1.0, 1.0, 1.0)
     */
    var colour: Colour
        get() = getColour()
        set(value) = setColour(value)

    /**
     * The type of light.
     */
    var lightType: LightType
        get() = getLightType()
        set(value) = setLightType(value)

    /**
     * The intensity/brightness of the light.
     */
    var intensity: Double
        get() = getIntensity()
        set(value) = setIntensity(value)

    /**
     * The attenuation parameters for the light.
     */
    var attenuation: Attenuation
        get() = getAttenuation()
        set(value) = setAttenuation(value)

    /**
     * Is the light enabled/emitting light?
     */
    var enabled: Boolean
        get() = getEnabled()
        set(value) = setEnabled(value)

    /**
     * The cutoff angle for spotlights, in degrees.
     */
    var cutoffAngle: Double
        get() = getCutoffAngle()
        set(value) = setCutoffAngle(value)

    /**
     * The outer cutoff angle for spotlights, in degrees.
     */
    var outerCutoffAngle: Double
        get() = getOuterCutoffAngle()
        set(value) = setOuterCutoffAngle(value)

    /**
     * Does the light cast shadows?
     */
    var castShadows: Boolean
        get() = getCastShadows()
        set(value) = setCastShadows(value)

    /**
     * The shadow depth range for the light.
     */
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

internal expect fun Light.getCutoffAngle(): Double
internal expect fun Light.setCutoffAngle(cutoffAngle: Double)

internal expect fun Light.getOuterCutoffAngle(): Double
internal expect fun Light.setOuterCutoffAngle(outerCutoffAngle: Double)

internal expect fun Light.getCastShadows(): Boolean
internal expect fun Light.setCastShadows(castShadows: Boolean)

internal expect fun Light.getDepth(): Range
internal expect fun Light.setDepth(depth: Range)