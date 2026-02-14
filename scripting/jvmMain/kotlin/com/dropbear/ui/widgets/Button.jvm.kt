package com.dropbear.ui.widgets

import com.dropbear.DropbearEngine

actual fun Button.getClicked(): Boolean {
    return ButtonNative.getClicked(DropbearEngine.native.uiBufferHandle, this.id.id)
}

actual fun Button.getHovering(): Boolean {
    return ButtonNative.getHovering(DropbearEngine.native.uiBufferHandle, this.id.id)
}