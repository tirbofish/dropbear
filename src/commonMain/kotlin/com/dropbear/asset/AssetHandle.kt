package com.dropbear.asset

import com.dropbear.DropbearEngine

/**
 * A handle that describes the type of asset in the ASSET_REGISTRY
 */
class AssetHandle(private val id: Long): Handle(id) {
    /**
     * Converts an [AssetHandle] to a [ModelHandle].
     *
     * It can return null if the asset is not a model.
     */
    fun asModelHandle(): ModelHandle? {
        val result = isModelHandle(id)
        return if (result) {
            ModelHandle(id)
        } else {
            null
        }
    }

    override fun asAssetHandle(): AssetHandle {
        return this
    }

    /**
     * Converts an [AssetHandle] to a [TextureHandle].
     *
     * It can return null if the asset is not a texture.
     */
    fun asTextureHandle(): TextureHandle? {
        return if (isTextureHandle(id)) TextureHandle(id) else null
    }

    override fun toString(): String {
        return "AssetHandle(id=$id)"
    }
}


internal expect fun isTextureHandle(id: Long): Boolean
internal expect fun isModelHandle(id: Long): Boolean