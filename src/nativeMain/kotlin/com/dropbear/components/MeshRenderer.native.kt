package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.asset.ModelHandle
import com.dropbear.asset.TextureHandle

internal actual fun MeshRenderer.getModel(id: EntityId): ModelHandle? {
    TODO("Not yet implemented")
}

internal actual fun MeshRenderer.setModel(id: EntityId, model: ModelHandle?) {
}

internal actual fun MeshRenderer.getAllTextureIds(id: EntityId): List<TextureHandle>? {
    TODO("Not yet implemented")
}

internal actual fun MeshRenderer.getTexture(id: EntityId, materialName: String): Long? {
    TODO("Not yet implemented")
}

internal actual fun MeshRenderer.setTextureOverride(
    id: EntityId,
    materialName: String,
    textureHandle: Long
) {
}

internal actual fun meshRendererExistsForEntity(entityId: EntityId): Boolean {
    TODO("Not yet implemented")
}