@file:OptIn(ExperimentalForeignApi::class)

package com.dropbear.input

import com.dropbear.DropbearEngine
import com.dropbear.ffi.generated.*
import kotlin.String
import com.dropbear.math.Vector2d
import kotlinx.cinterop.*

actual class InputState actual constructor() {
    actual fun printInputState() {
        val input = DropbearEngine.native.inputHandle ?: return
        memScoped { dropbear_input_print_input_state(input) }
    }

    actual fun isKeyPressed(key: KeyCode): Boolean = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped false
        val out = alloc<BooleanVar>()
        dropbear_input_is_key_pressed(input, key.ordinal, out.ptr)
        out.value
    }

    actual fun getMousePosition(): Vector2d = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped Vector2d(0.0, 0.0)
        val out = alloc<NVector2>()
        dropbear_input_get_mouse_position(input, out.ptr)
        Vector2d(out.x, out.y)
    }

    actual fun isMouseButtonPressed(button: MouseButton): Boolean = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped false
        val out = alloc<BooleanVar>()
        dropbear_input_is_mouse_button_pressed(input, button.ordinal, out.ptr)
        out.value
    }

    actual fun getMouseDelta(): Vector2d = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped Vector2d(0.0, 0.0)
        val out = alloc<NVector2>()
        dropbear_input_get_mouse_delta(input, out.ptr)
        Vector2d(out.x, out.y)
    }

    actual fun isCursorLocked(): Boolean = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped false
        val out = alloc<BooleanVar>()
        dropbear_input_is_cursor_locked(input, out.ptr)
        out.value
    }

    actual fun setCursorLocked(locked: Boolean) {
        val cmd = DropbearEngine.native.commandBufferHandle ?: return
        val input = DropbearEngine.native.inputHandle ?: return
        memScoped { dropbear_input_set_cursor_locked(cmd, input, locked) }
    }

    actual fun getLastMousePos(): Vector2d = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped Vector2d(0.0, 0.0)
        val out = alloc<NVector2>()
        dropbear_input_get_last_mouse_pos(input, out.ptr)
        Vector2d(out.x, out.y)
    }

    actual fun isCursorHidden(): Boolean = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped false
        val out = alloc<BooleanVar>()
        dropbear_input_is_cursor_hidden(input, out.ptr)
        out.value
    }

    actual fun setCursorHidden(hidden: Boolean) {
        val cmd = DropbearEngine.native.commandBufferHandle ?: return
        val input = DropbearEngine.native.inputHandle ?: return
        memScoped { dropbear_input_set_cursor_hidden(cmd, input, hidden) }
    }

    actual fun getConnectedGamepads(): List<Gamepad> = memScoped {
        val input = DropbearEngine.native.inputHandle ?: return@memScoped emptyList()
        val out = alloc<ConnectedGamepadIds>()
        val rc = dropbear_input_get_connected_gamepads(input, out.ptr)
        if (rc != 0) return@memScoped emptyList()
        val ptr = out.ids.values ?: return@memScoped emptyList()
        val len = out.ids.length.toInt()
        (0 until len).map { i -> Gamepad(ptr[i].toLong()) }
    }
}