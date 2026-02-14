package com.dropbear.components

import com.dropbear.DropbearEngine
import com.dropbear.EntityId
import com.dropbear.asset.TextureHandle

internal actual fun MeshRenderer.getModel(id: EntityId): ModelHandle? {
    return ModelHandle(MeshRendererNative.getModel(DropbearEngine.native.worldHandle, id.raw))
}

internal actual fun MeshRenderer.setModel(id: EntityId, model: ModelHandle?) {
    if (model == null) {
        throw IllegalArgumentException("ModelHandle cannot be null")
    }

    return MeshRendererNative.setModel(
        DropbearEngine.native.worldHandle,
        DropbearEngine.native.assetHandle,
        id.raw,
        model.raw()
    )
}

internal actual fun MeshRenderer.getAllTextureIds(id: EntityId): List<TextureHandle>? {
    val textureHandles = MeshRendererNative.getAllTextureIds(
        DropbearEngine.native.worldHandle,
        DropbearEngine.native.assetHandle,
        id.raw
    ) ?: return null

    return textureHandles.map { TextureHandle(it) }
}

internal actual fun MeshRenderer.getTexture(id: EntityId, materialName: String): Long? {
    return MeshRendererNative.getTexture(
        DropbearEngine.native.worldHandle,
        DropbearEngine.native.assetHandle,
        id.raw,
        materialName
    )
}

internal actual fun MeshRenderer.setTextureOverride(
    id: EntityId,
    materialName: String,
    textureHandle: Long
) {
    return MeshRendererNative.setTextureOverride(
        DropbearEngine.native.worldHandle,
        DropbearEngine.native.assetHandle,
        id.raw,
        materialName,
        textureHandle
    )
}

internal actual fun meshRendererExistsForEntity(entityId: EntityId): Boolean {
    return MeshRendererNative.meshRendererExistsForEntity(
        DropbearEngine.native.worldHandle,
        entityId.raw
    )
}