@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear

import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.ui.UIInstruction
import kotlinx.cinterop.*

internal actual fun getEntity(label: String): Long? = memScoped {
    val world = DropbearEngine.native.worldHandle ?: return@memScoped null
    val out = alloc<ULongVar>()
    val rc = dropbear_engine_get_entity(world, label, out.ptr)
    if (rc != 0) null else out.value.toLong()
}

internal actual fun getAsset(eucaURI: String): Long? = memScoped {
    val assets = DropbearEngine.native.assetHandle ?: return@memScoped null
    val outId = alloc<ULongVar>()
    val outPresent = alloc<BooleanVar>()

    val kindVar = alloc<UIntVar>()
    // Try Texture
    kindVar.value = AssetKind_Texture
    var rc = dropbear_engine_get_asset(assets, eucaURI, kindVar.ptr, outId.ptr, outPresent.ptr)
    if (rc == 0 && outPresent.value) return@memScoped outId.value.toLong()

    // Try Model
    kindVar.value = AssetKind_Model
    rc = dropbear_engine_get_asset(assets, eucaURI, kindVar.ptr, outId.ptr, outPresent.ptr)
    if (rc == 0 && outPresent.value) outId.value.toLong() else null
}

internal actual fun quit() {
    val cmd = DropbearEngine.native.commandBufferHandle ?: return
    memScoped { dropbear_engine_quit(cmd) }
}

internal actual fun renderUI(instructions: List<UIInstruction>) {
    // UI rendering via native scripting is not yet wired
}