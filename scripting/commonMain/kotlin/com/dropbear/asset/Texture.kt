package com.dropbear.asset

class Texture(val id: Long): AssetType {
    var label: String?
        get() = getLabel()
        set(value) = setLabel(value)

    val width: Int
        get() = getWidth()

    val height: Int
        get() = getHeight()

    val depth: Int
        get() = getDepth()
}

expect fun Texture.getLabel(): String?
expect fun Texture.setLabel(value: String?)
expect fun Texture.getWidth(): Int
expect fun Texture.getHeight(): Int
expect fun Texture.getDepth(): Int