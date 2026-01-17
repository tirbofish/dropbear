package com.dropbear

import com.dropbear.components.Camera

internal actual fun getEntity(label: String): Long? {
    return DropbearEngineNative.getEntity(DropbearEngine.native.worldHandle, label)
}

internal actual fun getAsset(eucaURI: String): Long? {
    return DropbearEngineNative.getAsset(DropbearEngine.native.assetHandle, eucaURI)
}

internal actual fun quit() {
    DropbearEngineNative.quit(DropbearEngine.native.commandBufferHandle)
}