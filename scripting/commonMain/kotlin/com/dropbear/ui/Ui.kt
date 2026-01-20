package com.dropbear.ui

import com.dropbear.math.Vector2d
import com.dropbear.ui.primitive.Circle
import com.dropbear.ui.primitive.Rectangle
import com.dropbear.utils.ID

/**
 * A command buffer to the 2D part of the game. Can be used for HUD and stuff.
 *
 * All content will be rendered the next frame.
 */
class Ui internal constructor() {
    /**
     * Draws/adds a [Widget] to the viewport. Typically used for HUD's and menus.
     *
     * The drawn widget will return a [Response], in which it is alive for 3 frames if
     * not altered or rerendered.
     */
    fun add(widget: Widget): Response {
        return widget.draw(this)
    }
}

internal expect fun Ui.pushRect(rect: Rectangle)
internal expect fun Ui.pushCircle(circle: Circle)

internal expect fun Ui.wasClicked(id: ID): Boolean