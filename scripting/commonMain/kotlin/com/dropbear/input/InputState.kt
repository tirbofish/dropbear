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
    /**
     * Prints out debug information to the console. Useful for debugging.
     */
    fun printInputState()

    /**
     * Checks if a keyboard key (in the form of [KeyCode]) is pressed
     */
    fun isKeyPressed(key: KeyCode): Boolean

    /**
     * Fetches the mouse position.
     *
     * @return The [Vector2d] if pressed or [Vector2d.zero] if there was an error.
     */
    fun getMousePosition(): Vector2d

    /**
     * Queries if a mouse button (represented by [MouseButton]) is pressed.
     *
     * @return `true` if pressed or `false` if not pressed/error.
     */
    fun isMouseButtonPressed(button: MouseButton): Boolean

    /**
     * Fetches the mouse delta/the difference between the current mouse position and the last mouse position.
     *
     * @return [Vector2d] if pressed or [Vector2d.zero] if error.
     */
    fun getMouseDelta(): Vector2d

    /**
     * Checks if the cursor is locked by the window. If it is locked by the window, it means
     * the cursor is locked to the centre. Different operating systems have their own methods
     * on locking.
     *
     * @return `true` if pressed or `false` if not pressed.
     */
    fun isCursorLocked(): Boolean

    /**
     * Sets the cursor to be locked.
     *
     * @param locked The new lock state of the cursor.
     */
    fun setCursorLocked(locked: Boolean)

    /**
     * Fetches the last mouse position.
     */
    fun getLastMousePos(): Vector2d

    /**
     * Checks if the cursor is hidden or not.
     */
    fun isCursorHidden(): Boolean

    /**
     * Sets the cursor hidden to [hidden]
     *
     * @param hidden The new hidden value
     */
    fun setCursorHidden(hidden: Boolean)

    /**
     * Fetches all connected gamepads at this frame.
     *
     * @return A list of gamepads. Can be empty if no gamepads are connected to the system.
     */
    fun getConnectedGamepads(): List<Gamepad>
}