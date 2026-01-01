package com.dropbear.input

import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector2d

/**
 * Information about a specific gamepad for that time in frame.
 */
class Gamepad(
    val id: Long,
) {
    /**
     * The position of the left stick.
     */
    val leftStickPosition: Vector2d
        get() = getLeftStickPosition(id)

    /**
     * The position of the right stick.
     */
    val rightStickPosition: Vector2d
        get() = getRightStickPosition(id)

    /**
     * Queries if a button is pressed, and returns either `true` if pressed
     * or `false` if not.
     */
    fun isButtonPressed(button: GamepadButton): Boolean {
        return isGamepadButtonPressed(id, button)
    }

    override fun toString(): String {
        return "Gamepad(id=$id)"
    }
}

internal expect fun Gamepad.isGamepadButtonPressed(id: Long, button: GamepadButton): Boolean
internal expect fun Gamepad.getLeftStickPosition(id: Long): Vector2d
internal expect fun Gamepad.getRightStickPosition(id: Long): Vector2d