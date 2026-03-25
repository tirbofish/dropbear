@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear

import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

internal actual fun EntityRef.Companion.getEntityLabel(entity: EntityId): String = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped ""
    val out = alloc<CPointerVar<ByteVar>>()
    val rc = dropbear_entity_get_label(world, entity.raw.toULong(), out.ptr)
    if (rc != 0) "" else out.value?.toKString() ?: ""
}

internal actual fun EntityRef.getChildren(entityId: EntityId): Array<EntityRef>? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<u64Array>()
    val rc = dropbear_entity_get_children(world, entityId.raw.toULong(), out.ptr)
    if (rc != 0) return@memScoped null
    val ptr = out.values ?: return@memScoped emptyArray()
    val len = out.length.toInt()
    Array(len) { i -> EntityRef(EntityId(ptr[i].toLong())) }
}

internal actual fun EntityRef.getChildByLabel(entityId: EntityId, label: String): EntityRef? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<ULongVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_entity_get_child_by_label(world, entityId.raw.toULong(), label, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else EntityRef(EntityId(out.value.toLong()))
}

internal actual fun EntityRef.getParent(entityId: EntityId): EntityRef? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<ULongVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_entity_get_parent(world, entityId.raw.toULong(), out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else EntityRef(EntityId(out.value.toLong()))
}