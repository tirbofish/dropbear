package com.dropbear

import com.dropbear.components.Camera

actual fun getEntity(label: String): Long? {
    return DropbearEngineNative.getEntity(DropbearEngine.native.worldHandle, label)
}

actual fun getAsset(eucaURI: String): Long? {
    return DropbearEngineNative.getAsset(DropbearEngine.native.worldHandle, eucaURI)
}

actual fun quit() {
    DropbearEngineNative.quit(DropbearEngine.native.commandBufferHandle)
}