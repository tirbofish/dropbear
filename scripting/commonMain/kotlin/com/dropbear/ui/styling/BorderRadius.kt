package com.dropbear.ui.styling

data class BorderRadius(
    var topLeft: Double = 0.0,
    var topRight: Double = 0.0,
    var bottomLeft: Double = 0.0,
    var bottomRight: Double = 0.0
) {
    companion object {
        fun uniform(value: Double): BorderRadius {
            return BorderRadius(
                value,
                value,
                value,
                value,
            )
        }

        fun top(value: Double): BorderRadius {
            return BorderRadius(
                value,
                value,
                0.0,
                0.0,
            )
        }

        fun bottom(value: Double): BorderRadius {
            return BorderRadius(
                0.0,
                0.0,
                value,
                value,
            )
        }

        fun left(value: Double): BorderRadius {
            return BorderRadius(
                value,
                0.0,
                value,
                0.0,
            )
        }

        fun right(value: Double): BorderRadius {
            return BorderRadius(
                0.0,
                value,
                0.0,
                value,
            )
        }
    }
}