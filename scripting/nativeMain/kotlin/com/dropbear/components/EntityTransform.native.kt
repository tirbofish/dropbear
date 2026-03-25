@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.math.Transform
import kotlinx.cinterop.*

internal actual fun entityTransformExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_transform_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}

internal actual fun EntityTransform.getLocalTransform(entityId: EntityId): Transform = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Transform.identity()
    val out = alloc<NTransform>()
    dropbear_transform_get_local_transform(world, entityId.raw.toULong(), out.ptr)
    readTransform(out)
}

internal actual fun EntityTransform.setLocalTransform(entityId: EntityId, transform: Transform) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val nt = allocTransform(transform)
    dropbear_transform_set_local_transform(world, entityId.raw.toULong(), nt.ptr)
}

internal actual fun EntityTransform.getWorldTransform(entityId: EntityId): Transform = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped Transform.identity()
    val out = alloc<NTransform>()
    dropbear_transform_get_world_transform(world, entityId.raw.toULong(), out.ptr)
    readTransform(out)
}

internal actual fun EntityTransform.setWorldTransform(entityId: EntityId, transform: Transform) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val nt = allocTransform(transform)
    dropbear_transform_set_world_transform(world, entityId.raw.toULong(), nt.ptr)
}

internal actual fun EntityTransform.propagateTransform(entityId: EntityId): Transform? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<NTransform>()
    val rc = dropbear_transform_propogate_transform(world, entityId.raw.toULong(), out.ptr)
    if (rc != 0) null else readTransform(out)
}