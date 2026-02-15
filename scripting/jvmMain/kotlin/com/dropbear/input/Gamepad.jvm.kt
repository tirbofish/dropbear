package com.dropbear.input

import com.dropbear.DropbearEngine
import com.dropbear.math.Vector2d

internal actual fun Gamepad.isGamepadButtonPressed(
    button: GamepadButton
): Boolean {
    return GamepadNative.isGamepadButtonPressed(
        DropbearEngine.native.inputHandle,
        id,
        button.ordinal
    )
}

@Suppress("UNCHECKED_CAST")
internal actual fun Gamepad.getLeftStickPosition(): Vector2d {
    val result = GamepadNative.getLeftStickPosition(
        DropbearEngine.native.inputHandle,
        id
    )

    return result ?: Vector2d.zero()
}

@Suppress("UNCHECKED_CAST")
internal actual fun Gamepad.getRightStickPosition(): Vector2d {
    val result = GamepadNative.getRightStickPosition(
        DropbearEngine.native.inputHandle,
        id
    )

    return result ?: Vector2d.zero()
}