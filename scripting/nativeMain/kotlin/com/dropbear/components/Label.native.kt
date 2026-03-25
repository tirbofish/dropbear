@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

internal actual fun labelExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_entity_label_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}