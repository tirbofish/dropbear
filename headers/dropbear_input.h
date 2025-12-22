#ifndef DROPBEAR_INPUT_H
#define DROPBEAR_INPUT_H

#include "dropbear_common.h"
#include "dropbear_math.h"

typedef struct {
    int id;
    Vector2D left_stick_pos;
    Vector2D right_stick_pos;
} Gamepad;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

DROPBEAR_NATIVE dropbear_print_input_state(const InputState* input_ptr);
DROPBEAR_NATIVE dropbear_is_key_pressed(const InputState* input_ptr, int32_t key_ordinal, BOOL* out_pressed);
DROPBEAR_NATIVE dropbear_get_mouse_position(const InputState* input_ptr, float* out_x, float* out_y);
DROPBEAR_NATIVE dropbear_is_mouse_button_pressed(const InputState* input_ptr, int32_t button_ordinal, BOOL* out_pressed);
DROPBEAR_NATIVE dropbear_get_mouse_delta(const InputState* input_ptr, float* out_dx, float* out_dy);
DROPBEAR_NATIVE dropbear_is_cursor_locked(const InputState* input_ptr, BOOL* out_locked);
DROPBEAR_NATIVE dropbear_set_cursor_locked(InputState* input_ptr, CommandBuffer* graphics_ptr, BOOL locked);
DROPBEAR_NATIVE dropbear_get_last_mouse_pos(const InputState* input_ptr, float* out_x, float* out_y);
DROPBEAR_NATIVE dropbear_is_cursor_hidden(const InputState* input_ptr, BOOL* out_hidden);
DROPBEAR_NATIVE dropbear_set_cursor_hidden(InputState* input_ptr, CommandBuffer* graphics_ptr, BOOL hidden);

// gamepad
DROPBEAR_NATIVE dropbear_get_connected_gamepads(InputState* input_ptr, const Gamepad** out_gamepads, int32_t* out_count);
DROPBEAR_NATIVE dropbear_is_gamepad_button_pressed(const InputState* input_ptr, HANDLE gamepad_id, int ordinal, BOOL* out_pressed);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_INPUT_H