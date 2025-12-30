package com.dropbear.input

import com.dropbear.math.Vector2d

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
    fun getMousePosition(): Vector2d
    fun isMouseButtonPressed(button: MouseButton): Boolean
    fun getMouseDelta(): Vector2d
    fun isCursorLocked(): Boolean
    fun setCursorLocked(locked: Boolean)
    fun getLastMousePos(): Vector2d
    fun isCursorHidden(): Boolean
    fun setCursorHidden(hidden: Boolean)
    fun getConnectedGamepads(): List<Gamepad>
}