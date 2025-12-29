package com.dropbear.input

import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector2D

/**
 * Information about a specific gamepad for that time in frame.
 */
class Gamepad(
    val id: Long,
    val leftStickPosition: Vector2D,
    val rightStickPosition: Vector2D,
) {
    fun isButtonPressed(button: GamepadButton): Boolean {
        return isGamepadButtonPressed(id, button)
    }

    override fun toString(): String {
        return "Gamepad $id @ ($leftStickPosition ; $rightStickPosition)"
    }
}

expect fun Gamepad.isGamepadButtonPressed(id: Long, button: GamepadButton): Boolean