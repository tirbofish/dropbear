package com.dropbear.asset

import com.dropbear.DropbearEngine

/**
 * Another type of asset handle, this wraps the id into 
 * another form that only texture related functions can use. 
 */
class TextureHandle(private val id: Long): Handle(id) {
    override fun asAssetHandle(): AssetHandle = AssetHandle(id)

    /**
     * Fetches the name of that specific texture
     */
    fun getName(): String? {
        return getTextureName(id)
    }

    override fun toString(): String {
        return "TextureHandle(id=$id)"
    }
}

expect fun TextureHandle.getTextureName(id: Long): String?