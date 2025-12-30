package com.dropbear.input

import com.dropbear.math.Vector2d

actual class InputState actual constructor() {
    actual fun printInputState() {
    }

    actual fun isKeyPressed(key: KeyCode): Boolean {
        TODO("Not yet implemented")
    }

    actual fun getMousePosition(): Vector2d {
        TODO("Not yet implemented")
    }

    actual fun isMouseButtonPressed(button: MouseButton): Boolean {
        TODO("Not yet implemented")
    }

    actual fun getMouseDelta(): Vector2d {
        TODO("Not yet implemented")
    }

    actual fun isCursorLocked(): Boolean {
        TODO("Not yet implemented")
    }

    actual fun setCursorLocked(locked: Boolean) {
    }

    actual fun getLastMousePos(): Vector2d {
        TODO("Not yet implemented")
    }

    actual fun isCursorHidden(): Boolean {
        TODO("Not yet implemented")
    }

    actual fun setCursorHidden(hidden: Boolean) {
    }

    actual fun getConnectedGamepads(): List<Gamepad> {
        TODO("Not yet implemented")
    }
}