@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.asset.ModelHandle
import com.dropbear.asset.TextureHandle
import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

internal actual fun MeshRenderer.getModel(id: EntityId): ModelHandle? = null // no C API for mesh renderer model getter

internal actual fun MeshRenderer.setModel(id: EntityId, model: ModelHandle?) {
    // no C API for mesh renderer model setter
}

internal actual fun MeshRenderer.getAllTextureIds(id: EntityId): List<TextureHandle>? = null // no C API for bulk texture listing

internal actual fun MeshRenderer.getTexture(id: EntityId, materialName: String): Long? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val assets = DropbearEngine.native.assetHandle ?: return@memScoped null
    val out = alloc<ULongVar>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_mesh_get_texture(world, assets, id.raw.toULong(), materialName, out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value.toLong()
}

internal actual fun MeshRenderer.setTextureOverride(id: EntityId, materialName: String, textureHandle: Long) = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped
    val assets = DropbearEngine.native.assetHandle ?: return@memScoped
    dropbear_mesh_set_texture_override(world, assets, id.raw.toULong(), materialName, textureHandle.toULong())
}

internal actual fun meshRendererExistsForEntity(entityId: EntityId): Boolean = false // no dedicated exists check in C API