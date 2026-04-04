@file:OptIn(ExperimentalForeignApi::class)

package io.github.tirbofish.surfaceNets

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import io.github.tirbofish.surfaceNets.ffi.generated.surface_nets_plugin_exists_for_entity
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.alloc
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.value
import kotlinx.cinterop.BooleanVar
import kotlinx.cinterop.ptr

internal actual fun surfaceNetsExistsForEntity(entityId: EntityId): Boolean = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    surface_nets_plugin_exists_for_entity(world, entityId.raw.toULong(), out.ptr)
    out.value
}
