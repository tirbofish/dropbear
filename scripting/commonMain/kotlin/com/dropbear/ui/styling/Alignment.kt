package com.dropbear.ui.styling

import com.dropbear.math.Vector2d

class Alignment(
    var x: Double,
    var y: Double,
) {
    companion object {
        val TOP_LEFT = Alignment(0.0, 0.0)
        val TOP_CENTER = Alignment(0.5, 0.0)
        val TOP_RIGHT = Alignment(1.0, 0.0)

        val CENTER_LEFT = Alignment(0.0, 0.5)
        val CENTER = Alignment(0.5, 0.5)
        val CENTER_RIGHT = Alignment(1.0, 0.5)

        val BOTTOM_LEFT = Alignment(0.0, 1.0)
        val BOTTOM_CENTER = Alignment(0.5, 1.0)
        val BOTTOM_RIGHT = Alignment(1.0, 1.0)
    }

    fun asVector2d(): Vector2d {
        return Vector2d(x, y)
    }
}