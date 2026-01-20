package com.dropbear.ui

import com.dropbear.DropbearEngine
import com.dropbear.ui.primitive.Circle
import com.dropbear.ui.primitive.Rectangle
import com.dropbear.utils.ID

internal actual fun Ui.wasClicked(id: ID): Boolean {
    return UINative.wasClicked(DropbearEngine.native.uiBufferHandle, id.getId())
}

internal actual fun Ui.pushRect(rect: Rectangle) {
    UINative.pushRect(
        DropbearEngine.native.uiBufferHandle,
        rect
    )
}

internal actual fun Ui.pushCircle(circle: Circle) {
    UINative.pushCircle(
        DropbearEngine.native.uiBufferHandle,
        circle
    )
}