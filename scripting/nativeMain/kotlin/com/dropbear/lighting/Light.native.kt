@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.lighting

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.math.Vector3d
import com.dropbear.utils.Colour
import com.dropbear.utils.Range
import kotlinx.cinterop.*

internal actual fun lightExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_lighting_light_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Light.getPosition(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_lighting_get_position(world, entity.raw.toULong(), out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun Light.setPosition(position: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val nv = allocVec3(position)
    dropbear_lighting_set_position(world, entity.raw.toULong(), nv.ptr)
}

internal actual fun Light.getDirection(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val out = alloc<NVector3>()
    dropbear_lighting_get_direction(world, entity.raw.toULong(), out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun Light.setDirection(direction: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val nv = allocVec3(direction)
    dropbear_lighting_set_direction(world, entity.raw.toULong(), nv.ptr)
}

internal actual fun Light.getColour(): Colour = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Colour(255u, 255u, 255u, 255u)
    val out = alloc<NColour>()
    dropbear_lighting_get_colour(world, entity.raw.toULong(), out.ptr)
    readColour(out)
}

internal actual fun Light.setColour(colour: Colour) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val nc = allocColour(colour)
    dropbear_lighting_set_colour(world, entity.raw.toULong(), nc.ptr)
}

internal actual fun Light.getLightType(): LightType = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped LightType.Directional
    val out = alloc<IntVar>()
    dropbear_lighting_get_light_type(world, entity.raw.toULong(), out.ptr)
    LightType.entries[out.value.coerceIn(0, LightType.entries.lastIndex)]
}

internal actual fun Light.setLightType(lightType: LightType) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_lighting_set_light_type(world, entity.raw.toULong(), lightType.ordinal)
}

internal actual fun Light.getIntensity(): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 1.0
    val out = alloc<DoubleVar>()
    dropbear_lighting_get_intensity(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Light.setIntensity(intensity: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_lighting_set_intensity(world, entity.raw.toULong(), intensity)
}

internal actual fun Light.getAttenuation(): Attenuation = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Attenuation()
    val out = alloc<NAttenuation>()
    dropbear_lighting_get_attenuation(world, entity.raw.toULong(), out.ptr)
    Attenuation(out.constant, out.linear, out.quadratic)
}

internal actual fun Light.setAttenuation(attenuation: Attenuation) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val na = alloc<NAttenuation>()
    na.constant = attenuation.constant
    na.linear = attenuation.linear
    na.quadratic = attenuation.quadratic
    dropbear_lighting_set_attenuation(world, entity.raw.toULong(), na.ptr)
}

internal actual fun Light.getEnabled(): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_lighting_get_enabled(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Light.setEnabled(enabled: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_lighting_set_enabled(world, entity.raw.toULong(), enabled)
}

internal actual fun Light.getCutoffAngle(): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val out = alloc<DoubleVar>()
    dropbear_lighting_get_cutoff_angle(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Light.setCutoffAngle(cutoffAngle: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_lighting_set_cutoff_angle(world, entity.raw.toULong(), cutoffAngle)
}

internal actual fun Light.getOuterCutoffAngle(): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val out = alloc<DoubleVar>()
    dropbear_lighting_get_outer_cutoff_angle(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Light.setOuterCutoffAngle(outerCutoffAngle: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_lighting_set_outer_cutoff_angle(world, entity.raw.toULong(), outerCutoffAngle)
}

internal actual fun Light.getCastShadows(): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_lighting_get_casts_shadows(world, entity.raw.toULong(), out.ptr)
    out.value
}

internal actual fun Light.setCastShadows(castShadows: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_lighting_set_casts_shadows(world, entity.raw.toULong(), castShadows)
}

internal actual fun Light.getDepth(): Range = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Range(0.0, 100.0)
    val out = alloc<NRange>()
    dropbear_lighting_get_depth(world, entity.raw.toULong(), out.ptr)
    Range(out.start.toDouble(), out.end.toDouble())
}

internal actual fun Light.setDepth(depth: Range) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val nr = alloc<NRange>().also { it.start = depth.start.toFloat(); it.end = depth.end.toFloat() }
    dropbear_lighting_set_depth(world, entity.raw.toULong(), nr.ptr)
}