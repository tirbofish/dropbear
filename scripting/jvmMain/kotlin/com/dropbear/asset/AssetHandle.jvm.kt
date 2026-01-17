package com.dropbear.asset

import com.dropbear.DropbearEngine

internal actual fun isModelHandle(id: Long): Boolean {
    return AssetHandleNative.isModelHandle(DropbearEngine.native.assetHandle, id)
}

internal actual fun isTextureHandle(id: Long): Boolean {
    return AssetHandleNative.isTextureHandle(DropbearEngine.native.assetHandle, id)
}