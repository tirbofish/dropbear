@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.NCollider
import com.dropbear.ffi.generated.NShapeCastStatus
import com.dropbear.ffi.generated.NTransform
import com.dropbear.ffi.generated.NVector3
import com.dropbear.ffi.generated.allocIndexNative
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_collider
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_normal1
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_normal2
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_position
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_status
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_time_of_impact
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_translation_applied
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_translation_remaining
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_witness1
import com.dropbear.ffi.generated.dropbear_character_collision_get_character_collision_witness2
import com.dropbear.ffi.generated.readCollider
import com.dropbear.ffi.generated.readShapeCastStatus
import com.dropbear.ffi.generated.readTransform
import com.dropbear.math.Transform
import com.dropbear.math.Vector3d
import kotlinx.cinterop.*

internal actual fun CharacterCollision.getCollider(): Collider = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Collider(Index(0u, 0u), entity, 0u)
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NCollider>()
    dropbear_character_collision_get_character_collision_collider(world, entity.raw.toULong(), ni.ptr, out.ptr)
    readCollider(out)
}

internal actual fun CharacterCollision.getCharacterPosition(): Transform = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Transform.identity()
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NTransform>()
    dropbear_character_collision_get_character_collision_position(world, entity.raw.toULong(), ni.ptr, out.ptr)
    readTransform(out)
}

internal actual fun CharacterCollision.getTranslationApplied(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NVector3>()
    dropbear_character_collision_get_character_collision_translation_applied(world, entity.raw.toULong(), ni.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun CharacterCollision.getTranslationRemaining(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NVector3>()
    dropbear_character_collision_get_character_collision_translation_remaining(world, entity.raw.toULong(), ni.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun CharacterCollision.getTimeOfImpact(): Double = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped 0.0
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<DoubleVar>()
    dropbear_character_collision_get_character_collision_time_of_impact(world, entity.raw.toULong(), ni.ptr, out.ptr)
    out.value
}

internal actual fun CharacterCollision.getWitness1(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NVector3>()
    dropbear_character_collision_get_character_collision_witness1(world, entity.raw.toULong(), ni.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun CharacterCollision.getWitness2(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NVector3>()
    dropbear_character_collision_get_character_collision_witness2(world, entity.raw.toULong(), ni.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun CharacterCollision.getNormal1(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NVector3>()
    dropbear_character_collision_get_character_collision_normal1(world, entity.raw.toULong(), ni.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun CharacterCollision.getNormal2(): Vector3d = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Vector3d.zero()
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NVector3>()
    dropbear_character_collision_get_character_collision_normal2(world, entity.raw.toULong(), ni.ptr, out.ptr)
    Vector3d(out.x, out.y, out.z)
}

internal actual fun CharacterCollision.getStatus(): ShapeCastStatus = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped ShapeCastStatus.Failed
    val ni = allocIndexNative(collisionHandle)
    val out = alloc<NShapeCastStatus>()
    dropbear_character_collision_get_character_collision_status(world, entity.raw.toULong(), ni.ptr, out.ptr)
    readShapeCastStatus(out)
}