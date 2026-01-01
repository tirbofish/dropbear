package com.dropbear.components

import com.dropbear.EntityId
import com.dropbear.asset.ModelHandle
import com.dropbear.asset.TextureHandle
import com.dropbear.ecs.Component
import com.dropbear.ecs.ComponentType

/**
 * A component that allows for a 3D model to be rendered.
 *
 * It must require the `MeshRenderer` component in the editor to be queryable.
 */
class MeshRenderer(val id: EntityId) : Component(id, "MeshRenderer") {

    /**
     * The active model currently assigned to this entity.
     * Setting this to null removes the model.
     *
     * Usage:
     * ```
     * val handle = renderer.model
     * renderer.model = newHandle
     * ```
     */
    var model: ModelHandle?
        get() {
            return getModel(id)
        }
        set(value) {
            setModel(id, value)
        }

    /**
     * A list of all active Texture handles currently applied to the model.
     */
    val textures: List<TextureHandle>?
        get() = getAllTextureIds(id)

    /**
     * Fetches the texture assigned to a specific material slot.
     * Returns null if the material doesn't exist or has no texture.
     */
    fun getTexture(materialName: String): TextureHandle? {
        val rawId = getTexture(id, materialName)
        return if (rawId == 0L || rawId == null) null else TextureHandle(rawId)
    }

    /**
     * Overrides the texture for a specific material on the active model.
     */
    fun setTextureOverride(materialName: String, textureHandle: TextureHandle) {
        setTextureOverride(id, materialName, textureHandle.raw())
    }

    companion object : ComponentType<MeshRenderer> {
        override fun get(entityId: EntityId): MeshRenderer? {
            return if (meshRendererExistsForEntity(entityId)) MeshRenderer(entityId) else null
        }
    }
}

internal expect fun MeshRenderer.getModel(id: EntityId): ModelHandle?
internal expect fun MeshRenderer.setModel(id: EntityId, model: ModelHandle?)
internal expect fun MeshRenderer.getAllTextureIds(id: EntityId): List<TextureHandle>?
internal expect fun MeshRenderer.getTexture(id: EntityId, materialName: String): Long?
internal expect fun MeshRenderer.setTextureOverride(id: EntityId, materialName: String, textureHandle: Long)

internal expect fun meshRendererExistsForEntity(entityId: EntityId): Boolean
