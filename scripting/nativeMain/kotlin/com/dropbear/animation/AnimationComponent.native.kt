@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.animation

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

actual fun AnimationComponent.getActiveAnimationIndex(): Int? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<IntVar>()
    val present = alloc<BooleanVar>()
    dropbear_animation_get_active_animation_index(world, parentEntity.raw.toULong(), out.ptr, present.ptr)
    if (!present.value) null else out.value
}

actual fun AnimationComponent.setActiveAnimationIndex(index: Int?) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    if (index == null) {
        dropbear_animation_set_active_animation_index(world, parentEntity.raw.toULong(), null)
    } else {
        val iv = alloc<IntVar>(); iv.value = index
        val pv = alloc<CPointerVar<IntVar>>(); pv.value = iv.ptr
        dropbear_animation_set_active_animation_index(world, parentEntity.raw.toULong(), pv.ptr)
    }
}

actual fun AnimationComponent.getTime(): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val out = alloc<DoubleVar>()
    dropbear_animation_get_time(world, parentEntity.raw.toULong(), out.ptr)
    out.value
}

actual fun AnimationComponent.setTime(value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_animation_set_time(world, parentEntity.raw.toULong(), value)
}

actual fun AnimationComponent.getSpeed(): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 1.0
    val out = alloc<DoubleVar>()
    dropbear_animation_get_speed(world, parentEntity.raw.toULong(), out.ptr)
    out.value
}

actual fun AnimationComponent.setSpeed(value: Double) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_animation_set_speed(world, parentEntity.raw.toULong(), value)
}

actual fun AnimationComponent.getLooping(): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_animation_get_looping(world, parentEntity.raw.toULong(), out.ptr)
    out.value
}

actual fun AnimationComponent.setLooping(value: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_animation_set_looping(world, parentEntity.raw.toULong(), value)
}

actual fun AnimationComponent.getIsPlaying(): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_animation_get_is_playing(world, parentEntity.raw.toULong(), out.ptr)
    out.value
}

actual fun AnimationComponent.setIsPlaying(value: Boolean) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    dropbear_animation_set_is_playing(world, parentEntity.raw.toULong(), value)
}

actual fun AnimationComponent.getIndexFromString(name: String): Int? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<IntVar>()
    val present = alloc<BooleanVar>()
    dropbear_animation_get_index_from_string(world, parentEntity.raw.toULong(), name, out.ptr, present.ptr)
    if (!present.value) null else out.value
}

actual fun AnimationComponent.getAvailableAnimations(): List<String> = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped emptyList()
    val out = alloc<StringArray>()
    val rc = dropbear_animation_get_available_animations(world, parentEntity.raw.toULong(), out.ptr)
    if (rc != 0) return@memScoped emptyList()
    val ptr = out.values ?: return@memScoped emptyList()
    val len = out.length.toInt()
    (0 until len).mapNotNull { i -> ptr[i]?.toKString() }
}

actual fun animationComponentExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_animation_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}