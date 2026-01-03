package com.dropbear.input

import com.dropbear.math.Vector2d

actual class InputState actual constructor() {
    internal actual fun printInputState() {
    }

    internal actual fun isKeyPressed(key: KeyCode): Boolean {
        TODO("Not yet implemented")
    }

    internal actual fun getMousePosition(): Vector2d {
        TODO("Not yet implemented")
    }

    internal actual fun isMouseButtonPressed(button: MouseButton): Boolean {
        TODO("Not yet implemented")
    }

    internal actual fun getMouseDelta(): Vector2d {
        TODO("Not yet implemented")
    }

    internal actual fun isCursorLocked(): Boolean {
        TODO("Not yet implemented")
    }

    internal actual fun setCursorLocked(locked: Boolean) {
    }

    internal actual fun getLastMousePos(): Vector2d {
        TODO("Not yet implemented")
    }

    internal actual fun isCursorHidden(): Boolean {
        TODO("Not yet implemented")
    }

    internal actual fun setCursorHidden(hidden: Boolean) {
    }

    internal actual fun getConnectedGamepads(): List<Gamepad> {
        TODO("Not yet implemented")
    }
}