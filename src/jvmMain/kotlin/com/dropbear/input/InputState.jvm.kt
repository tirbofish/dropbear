package com.dropbear.input

import com.dropbear.DropbearEngine
import com.dropbear.math.Vector2d

actual class InputState actual constructor() {
    internal actual fun printInputState() {
        return InputStateNative.printInputState(DropbearEngine.native.inputHandle)
    }

    internal actual fun isKeyPressed(key: KeyCode): Boolean {
        return InputStateNative.isKeyPressed(DropbearEngine.native.inputHandle, key.ordinal)
    }

    internal actual fun getMousePosition(): Vector2d {
        return InputStateNative.getMousePosition(DropbearEngine.native.inputHandle)
    }

    internal actual fun isMouseButtonPressed(button: MouseButton): Boolean {
        return InputStateNative.isMouseButtonPressed(DropbearEngine.native.inputHandle, button)
    }

    internal actual fun getMouseDelta(): Vector2d {
        return InputStateNative.getMouseDelta(DropbearEngine.native.inputHandle)
    }

    internal actual fun isCursorLocked(): Boolean {
        return InputStateNative.isCursorLocked(DropbearEngine.native.inputHandle)
    }

    internal actual fun setCursorLocked(locked: Boolean) {
        return InputStateNative.setCursorLocked(DropbearEngine.native.commandBufferHandle, DropbearEngine.native.inputHandle, locked)
    }

    internal actual fun getLastMousePos(): Vector2d {
        return InputStateNative.getLastMousePos(DropbearEngine.native.inputHandle)
    }

    internal actual fun isCursorHidden(): Boolean {
        return InputStateNative.isCursorHidden(DropbearEngine.native.inputHandle)
    }

    internal actual fun setCursorHidden(hidden: Boolean) {
        return InputStateNative.setCursorHidden(DropbearEngine.native.commandBufferHandle, DropbearEngine.native.inputHandle, hidden)
    }

    internal actual fun getConnectedGamepads(): List<Gamepad> {
        val result = InputStateNative.getConnectedGamepads(DropbearEngine.native.inputHandle)
        return result.map { Gamepad(it) }.toList()
    }
}