package com.dropbear.input

import com.dropbear.DropbearEngine
import com.dropbear.math.Vector2d

actual class InputState actual constructor() {
    actual fun printInputState() {
        return InputStateNative.printInputState(DropbearEngine.native.inputHandle)
    }

    actual fun isKeyPressed(key: KeyCode): Boolean {
        return InputStateNative.isKeyPressed(DropbearEngine.native.inputHandle, key.ordinal)
    }

    actual fun getMousePosition(): Vector2d {
        return InputStateNative.getMousePosition(DropbearEngine.native.inputHandle)
    }

    actual fun isMouseButtonPressed(button: MouseButton): Boolean {
        return InputStateNative.isMouseButtonPressed(DropbearEngine.native.inputHandle, button.ordinal)
    }

    actual fun getMouseDelta(): Vector2d {
        return InputStateNative.getMouseDelta(DropbearEngine.native.inputHandle)
    }

    actual fun isCursorLocked(): Boolean {
        return InputStateNative.isCursorLocked(DropbearEngine.native.inputHandle)
    }

    actual fun setCursorLocked(locked: Boolean) {
        return InputStateNative.setCursorLocked(DropbearEngine.native.commandBufferHandle, DropbearEngine.native.inputHandle, locked)
    }

    actual fun getLastMousePos(): Vector2d {
        return InputStateNative.getLastMousePos(DropbearEngine.native.inputHandle)
    }

    actual fun isCursorHidden(): Boolean {
        return InputStateNative.isCursorHidden(DropbearEngine.native.inputHandle)
    }

    actual fun setCursorHidden(hidden: Boolean) {
        return InputStateNative.setCursorHidden(DropbearEngine.native.commandBufferHandle, DropbearEngine.native.inputHandle, hidden)
    }

    actual fun getConnectedGamepads(): List<Gamepad> {
        val result = InputStateNative.getConnectedGamepads(DropbearEngine.native.inputHandle)
        return result.map { Gamepad(it) }.toList()
    }
}