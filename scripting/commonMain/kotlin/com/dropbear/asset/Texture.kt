package com.dropbear.asset

class Texture(override val id: Long): AssetType(id) {
    val label: String?
        get() = getLabel()

    val width: Int
        get() = getWidth()

    val height: Int
        get() = getHeight()
}

expect fun Texture.getLabel(): String?
expect fun Texture.getWidth(): Int
expect fun Texture.getHeight(): Int