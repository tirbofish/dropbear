@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.asset

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.*
import kotlin.String
import kotlinx.cinterop.*

actual fun Texture.getLabel(): String? = memScoped {
    val assets = DropbearEngine.native.assetHandle ?: return@memScoped null
    val out = alloc<CPointerVar<ByteVar>>()
    val present = alloc<BooleanVar>()
    val rc = dropbear_asset_texture_get_label(assets, id.toULong(), out.ptr, present.ptr)
    if (rc != 0 || !present.value) null else out.value?.toKString()
}

actual fun Texture.getWidth(): Int = memScoped {
    val assets = DropbearEngine.native.assetHandle ?: return@memScoped 0
    val out = alloc<UIntVar>()
    dropbear_asset_texture_get_width(assets, id.toULong(), out.ptr)
    out.value.toInt()
}

actual fun Texture.getHeight(): Int = memScoped {
    val assets = DropbearEngine.native.assetHandle ?: return@memScoped 0
    val out = alloc<UIntVar>()
    dropbear_asset_texture_get_height(assets, id.toULong(), out.ptr)
    out.value.toInt()
}