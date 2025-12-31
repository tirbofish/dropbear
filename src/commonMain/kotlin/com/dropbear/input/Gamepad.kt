package com.dropbear.input

import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector2d

/**
 * Information about a specific gamepad for that time in frame.
 */
class Gamepad(
    val id: Long,
) {
    val leftStickPosition: Vector2d
        get() = getLeftStickPosition(id)
    val rightStickPosition: Vector2d
        get() = getRightStickPosition(id)

    fun isButtonPressed(button: GamepadButton): Boolean {
        return isGamepadButtonPressed(id, button)
    }

    override fun toString(): String {
        return "Gamepad(id=$id)"
    }
}

expect fun Gamepad.isGamepadButtonPressed(id: Long, button: GamepadButton): Boolean
expect fun Gamepad.getLeftStickPosition(id: Long): Vector2d
expect fun Gamepad.getRightStickPosition(id: Long): Vector2d