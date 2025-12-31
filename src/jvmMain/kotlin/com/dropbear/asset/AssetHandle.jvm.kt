package com.dropbear.asset

import com.dropbear.DropbearEngine

actual fun isModelHandle(id: Long): Boolean {
    return AssetHandleNative.isModelHandle(DropbearEngine.native.assetHandle, id)
}

actual fun isTextureHandle(id: Long): Boolean {
    return AssetHandleNative.isTextureHandle(DropbearEngine.native.assetHandle, id)
}