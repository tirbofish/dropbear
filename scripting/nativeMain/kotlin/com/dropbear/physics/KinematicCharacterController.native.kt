@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.CharacterCollisionArray
import com.dropbear.ffi.generated.CharacterMovementResult as FfiCharacterMovementResult
import com.dropbear.ffi.generated.NQuaternion
import com.dropbear.ffi.generated.NVector3
import com.dropbear.ffi.generated.allocIndexNative
import com.dropbear.ffi.generated.dropbear_kcc_get_hit
import com.dropbear.ffi.generated.dropbear_kcc_get_movement_result
import com.dropbear.ffi.generated.dropbear_kcc_kcc_exists_for_entity
import com.dropbear.ffi.generated.dropbear_kcc_move_character
import com.dropbear.ffi.generated.dropbear_kcc_set_rotation
import com.dropbear.math.Quaterniond
import com.dropbear.math.Vector3d
import kotlinx.cinterop.*

internal actual fun KinematicCharacterController.moveCharacter(dt: Double, translation: Vector3d) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nv = alloc<NVector3>().also { it.x = translation.x; it.y = translation.y; it.z = translation.z }
    dropbear_kcc_move_character(world, physics, entity.raw.toULong(), nv.ptr, dt)
}

internal actual fun KinematicCharacterController.setRotationNative(rotation: Quaterniond) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped
    val nq = alloc<NQuaternion>().also { it.x = rotation.x; it.y = rotation.y; it.z = rotation.z; it.w = rotation.w }
    dropbear_kcc_set_rotation(world, physics, entity.raw.toULong(), nq.ptr)
}

internal actual fun kccExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_kcc_kcc_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}

internal actual fun KinematicCharacterController.getHitsNative(): List<CharacterCollision> = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped emptyList()
    val out = alloc<CharacterCollisionArray>()
    val rc = dropbear_kcc_get_hit(world, entity.raw.toULong(), out.ptr)
    if (rc != 0) return@memScoped emptyList()
    val collisionEntityId = EntityId(out.entity_id.toLong())
    val ptr = out.collisions.values ?: return@memScoped emptyList()
    val len = out.collisions.length.toInt()
    (0 until len).map { i ->
        val idx = ptr[i]
        CharacterCollision(collisionEntityId, Index(idx.index, idx.generation))
    }
}

internal actual fun KinematicCharacterController.getMovementResult(): CharacterMovementResult? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<FfiCharacterMovementResult>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_kcc_get_movement_result(world, entity.raw.toULong(), out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else CharacterMovementResult(
        Vector3d(out.translation.x, out.translation.y, out.translation.z),
        out.grounded,
        out.is_sliding_down_slope,
    )
}