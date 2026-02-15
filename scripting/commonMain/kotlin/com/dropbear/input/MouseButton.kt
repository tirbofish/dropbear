package com.dropbear.input

/**
 * Buttons that can be pressed on a mouse
 */
sealed class MouseButton(val ordinal: Int) {
    object Left : MouseButton(0)
    object Right : MouseButton(1)
    object Middle : MouseButton(2)
    object Back : MouseButton(3)
    object Forward : MouseButton(4)
    data class Other(val value: Int) : MouseButton(value)
}