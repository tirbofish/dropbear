package com.dropbear.ffi

import com.dropbear.DropbearEngine

actual fun getEntity(label: String): Long? {
    TODO("Not yet implemented")
}
actual fun getAsset(eucaURI: String): Long? {
    DropbearEngine.native.assetHandle
    TODO("Not yet implemented")
}