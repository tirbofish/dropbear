package com.dropbear.input

import com.dropbear.DropbearEngine
import com.dropbear.ffi.NativeEngine
import com.dropbear.math.Vector2D

/**
 * The current state of the input system.
 * 
 * It mirrors `eucalyptus_core::input::InputState` and has 
 * functions that are basically getters. 
 * 
 * The InputState does not have any values that can be mutated, only
 * queried. 
 */
class InputState(private val native: NativeEngine) {

    fun printInputState() {
        native.printInputState()
    }

    fun isKeyPressed(key: KeyCode): Boolean {
        return native.isKeyPressed(key)
    }

    fun getMousePosition(): Vector2D {
        return native.getMousePosition() ?: Vector2D(0.0, 0.0)
    }

    fun isMouseButtonPressed(button: MouseButton): Boolean {
        return native.isMouseButtonPressed(button)
    }

    fun getMouseDelta(): Vector2D {
        return native.getMouseDelta() ?: Vector2D(0.0, 0.0)
    }

    fun isCursorLocked(): Boolean {
        return native.isCursorLocked()
    }

    fun setCursorLocked(locked: Boolean) {
        return native.setCursorLocked(locked)
    }

    fun getLastMousePos(): Vector2D {
        return native.getLastMousePos() ?: Vector2D(0.0, 0.0)
    }

    fun isCursorHidden(): Boolean {
        return native.isCursorHidden()
    }

    fun setCursorHidden(hidden: Boolean) {
        return native.setCursorHidden(hidden)
    }

    fun getConnectedGamepads(): List<Gamepad> {
        return native.getConnectedGamepads()
    }
}