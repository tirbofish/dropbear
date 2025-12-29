package com.dropbear

import com.dropbear.asset.ModelHandle
import com.dropbear.asset.TextureHandle

actual fun MeshRenderer.getModel(id: EntityId): ModelHandle? {
    TODO("Not yet implemented")
}

actual fun MeshRenderer.setModel(id: EntityId, model: ModelHandle?) {
}

actual fun MeshRenderer.getAllTextureIds(id: EntityId): List<TextureHandle>? {
    TODO("Not yet implemented")
}

actual fun MeshRenderer.getMaterialNames(id: EntityId): List<String> {
    TODO("Not yet implemented")
}

actual fun MeshRenderer.setMaterialNames(id: EntityId, names: Array<String>) {
}

actual fun MeshRenderer.getTexture(id: EntityId, materialName: String): Long {
    TODO("Not yet implemented")
}

actual fun MeshRenderer.setTextureOverride(
    id: EntityId,
    materialName: String,
    textureHandle: Long
) {
}

actual fun meshRendererExistsForEntity(entityId: EntityId): Boolean {
    TODO("Not yet implemented")
}