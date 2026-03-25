@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.physics

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

internal actual fun ColliderGroup.getColliderGroupColliders(colliderGroup: ColliderGroup): List<Collider> = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped emptyList()
    val physics = DropbearEngine.native.physicsEngineHandle ?: return@memScoped emptyList()
    val out = alloc<NColliderArray>()
    val rc = dropbear_collider_group_get_colliders(world, physics, colliderGroup.entity.raw.toULong(), out.ptr)
    if (rc != 0) return@memScoped emptyList()
    val ptr = out.values ?: return@memScoped emptyList()
    val len = out.length.toInt()
    (0 until len).map { i -> readCollider(ptr[i]) }
}

internal actual fun colliderGroupExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_collider_group_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}