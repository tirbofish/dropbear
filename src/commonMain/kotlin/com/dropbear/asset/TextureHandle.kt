package com.dropbear.asset

/**
 * Another type of asset handle, this wraps the id into 
 * another form that only texture related functions can use. 
 */
class TextureHandle(private val id: Long): Handle(id) {
    /**
     * The name of the texture/material.
     */
    val name: String?
        get() = getTextureName(id)

    override fun asAssetHandle(): AssetHandle = AssetHandle(id)

    override fun toString(): String {
        return "TextureHandle(id=$id)"
    }
}

internal expect fun TextureHandle.getTextureName(id: Long): String?