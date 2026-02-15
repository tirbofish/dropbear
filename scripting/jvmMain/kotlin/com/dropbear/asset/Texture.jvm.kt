package com.dropbear.asset

import com.dropbear.DropbearEngine

actual fun Texture.getLabel(): String? {
    return TextureNative.getLabel(DropbearEngine.native.assetHandle, id)
}

actual fun Texture.getWidth(): Int {
    return TextureNative.getWidth(DropbearEngine.native.assetHandle, id)
}

actual fun Texture.getHeight(): Int {
    return TextureNative.getHeight(DropbearEngine.native.assetHandle, id)
}
