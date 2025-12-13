package com.dropbear.input

import com.dropbear.math.Vector2D

class Gamepad(
    val id: Int,
    val leftStickPosition: Vector2D,
    val rightStickPosition: Vector2D,
) {
    fun isButtonPressed(button: GamepadButton): Boolean {
        TODO("Not yet implemented")
    }

    override fun toString(): String {
        return "Gamepad $id @ ($leftStickPosition ; $rightStickPosition)"
    }
}