package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.asset.ModelHandle
import com.dropbear.asset.TextureHandle

actual fun MeshRenderer.getModel(id: EntityId): ModelHandle? {
    return ModelHandle(MeshRendererNative.getModel(DropbearEngine.native.worldHandle, id.raw))
}

actual fun MeshRenderer.setModel(id: EntityId, model: ModelHandle?) {
    if (model == null) {
        MeshRendererNative.setModel(DropbearEngine.native.worldHandle, id.raw, 0L)
        return
    }

    return MeshRendererNative.setModel(
        DropbearEngine.native.worldHandle,
        id.raw,
        model.raw()
    )
}

actual fun MeshRenderer.getAllTextureIds(id: EntityId): List<TextureHandle>? {
    val textureHandles = MeshRendererNative.getAllTextureIds(
        DropbearEngine.native.worldHandle,
        id.raw
    ) ?: return null

    return textureHandles.map { TextureHandle(it) }
}

actual fun MeshRenderer.getTexture(id: EntityId, materialName: String): Long {
    return MeshRendererNative.getTexture(
        DropbearEngine.native.worldHandle,
        id.raw,
        materialName
    )
}

actual fun MeshRenderer.setTextureOverride(
    id: EntityId,
    materialName: String,
    textureHandle: Long
) {
    return MeshRendererNative.setTextureOverride(
        DropbearEngine.native.worldHandle,
        id.raw,
        materialName,
        textureHandle
    )
}

actual fun meshRendererExistsForEntity(entityId: EntityId): Boolean {
    return MeshRendererNative.meshRendererExistsForEntity(
        DropbearEngine.native.worldHandle,
        entityId.raw
    )
}