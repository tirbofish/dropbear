package com.dropbear.ui.widgets

import com.dropbear.DropbearEngine

actual fun Checkbox.getChecked(): Boolean {
    return CheckboxNative.getChecked(DropbearEngine.native.uiBufferHandle, id.id)
}

actual fun Checkbox.hasCheckedState(): Boolean {
    return CheckboxNative.hasCheckedState(DropbearEngine.native.uiBufferHandle, id.id)
}