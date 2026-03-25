@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.input

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.math.Vector2d
import kotlinx.cinterop.*

internal actual fun Gamepad.isGamepadButtonPressed(button: GamepadButton): Boolean = memScoped {
    val input = DropbearEngine.native.inputHandle ?: return@memScoped false
    val out = alloc<BooleanVar>()
    dropbear_gamepad_is_button_pressed(input, id.toULong(), button.ordinal, out.ptr)
    out.value
}

internal actual fun Gamepad.getLeftStickPosition(): Vector2d = memScoped {
    val input = DropbearEngine.native.inputHandle ?: return@memScoped Vector2d(0.0, 0.0)
    val out = alloc<NVector2>()
    dropbear_gamepad_get_left_stick_position(input, id.toULong(), out.ptr)
    Vector2d(out.x, out.y)
}

internal actual fun Gamepad.getRightStickPosition(): Vector2d = memScoped {
    val input = DropbearEngine.native.inputHandle ?: return@memScoped Vector2d(0.0, 0.0)
    val out = alloc<NVector2>()
    dropbear_gamepad_get_right_stick_position(input, id.toULong(), out.ptr)
    Vector2d(out.x, out.y)
}
