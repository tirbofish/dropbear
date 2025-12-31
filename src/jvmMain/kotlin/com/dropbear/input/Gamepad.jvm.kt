package com.dropbear.input

import com.dropbear.DropbearEngine
import com.dropbear.math.Vector2d

actual fun Gamepad.isGamepadButtonPressed(
    id: Long,
    button: GamepadButton
): Boolean {
    return GamepadNative.isGamepadButtonPressed(
        DropbearEngine.native.inputHandle,
        id,
        button.ordinal
    )
}

@Suppress("UNCHECKED_CAST")
actual fun Gamepad.getLeftStickPosition(id: Long): Vector2d {
    val result = GamepadNative.getLeftStickPosition(
        DropbearEngine.native.inputHandle,
        id
    )

    return result ?: Vector2d.zero()
}

@Suppress("UNCHECKED_CAST")
actual fun Gamepad.getRightStickPosition(id: Long): Vector2d {
    val result = GamepadNative.getRightStickPosition(
        DropbearEngine.native.inputHandle,
        id
    )

    return result ?: Vector2d.zero()
}