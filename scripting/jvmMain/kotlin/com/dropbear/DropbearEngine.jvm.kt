package com.dropbear

import com.dropbear.ui.UIInstruction
import com.dropbear.ui.UINative

internal actual fun getEntity(label: String): Long? {
    return DropbearEngineNative.getEntity(DropbearEngine.native.worldHandle, label)
}

internal actual fun getAsset(eucaURI: String): Long? {
    return DropbearEngineNative.getAsset(DropbearEngine.native.assetHandle, eucaURI)
}

internal actual fun quit() {
    DropbearEngineNative.quit(DropbearEngine.native.commandBufferHandle)
}

internal actual fun renderUI(uiInstructionSet: List<UIInstruction>) {
    UINative.renderUI(DropbearEngine.native.uiBufferHandle, uiInstructionSet.toTypedArray())
}