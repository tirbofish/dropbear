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
expect class InputState() {
    fun printInputState()
    fun isKeyPressed(key: KeyCode): Boolean
    fun getMousePosition(): Vector2D
    fun isMouseButtonPressed(button: MouseButton): Boolean
    fun getMouseDelta(): Vector2D
    fun isCursorLocked(): Boolean
    fun setCursorLocked(locked: Boolean)
    fun getLastMousePos(): Vector2D
    fun isCursorHidden(): Boolean
    fun setCursorHidden(hidden: Boolean)
    fun getConnectedGamepads(): List<Gamepad>
}