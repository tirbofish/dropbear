package com.dropbear.asset

import com.dropbear.DropbearEngine

internal actual fun TextureHandle.getTextureName(id: Long): String? {
    return TextureHandleNative.getTextureName(DropbearEngine.native.assetHandle, id)
}