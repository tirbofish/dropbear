#ifndef DROPBEAR_INPUT_H
#define DROPBEAR_INPUT_H

#include "dropbear_common.h"
#include "dropbear_math.h"

/**
 * @brief A struct that represents an external input device in the shape of a controller.
 */
typedef struct {
    int id;
    Vector2D left_stick_pos;
    Vector2D right_stick_pos;
} Gamepad;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * @brief Prints the input state to the console. Does not return anything, failure does not do anything.
 *
 * Can be useful for debugging.
 */
DROPBEAR_NATIVE dropbear_print_input_state(const InputState* input_ptr);

/**
 * @brief Checks if a key is currently pressed. If pressed, returns 1, otherwise 0.
 */
DROPBEAR_NATIVE dropbear_is_key_pressed(const InputState* input_ptr, int32_t key_ordinal, BOOL* out_pressed);

/**
 * @brief Fetches the current mouse position for that frame.
 */
DROPBEAR_NATIVE dropbear_get_mouse_position(const InputState* input_ptr, float* out_x, float* out_y);

/**
 * @brief Checks if a mouse button is currently pressed. If pressed, returns 1, otherwise 0.
 */
DROPBEAR_NATIVE dropbear_is_mouse_button_pressed(const InputState* input_ptr, int32_t button_ordinal, BOOL* out_pressed);

/**
 * @brief Fetches the delta of the mouse position since the last frame.
 */
DROPBEAR_NATIVE dropbear_get_mouse_delta(const InputState* input_ptr, float* out_dx, float* out_dy);

/**
 * @brief Checks if the cursor is currently locked. If locked, returns 1, otherwise 0.
 */
DROPBEAR_NATIVE dropbear_is_cursor_locked(const InputState* input_ptr, BOOL* out_locked);

/**
 * @brief Sets the mouse cursor to be locked or unlocked.
 */
DROPBEAR_NATIVE dropbear_set_cursor_locked(InputState* input_ptr, CommandBuffer* graphics_ptr, BOOL locked);

/**
 * @brief Fetches the mouse position of the previous frame.
 *
 * Can be used to calculate the delta of the mouse position.
 */
DROPBEAR_NATIVE dropbear_get_last_mouse_pos(const InputState* input_ptr, float* out_x, float* out_y);

/**
 * @brief Checks if the cursor is currently hidden. If hidden, returns 1, otherwise 0.
 */
DROPBEAR_NATIVE dropbear_is_cursor_hidden(const InputState* input_ptr, BOOL* out_hidden);

/**
 * @brief Sets the cursor to either hidden (invisible) or visible
 */
DROPBEAR_NATIVE dropbear_set_cursor_hidden(InputState* input_ptr, CommandBuffer* graphics_ptr, BOOL hidden);

/**
 * @brief Fetches all available connected gamepads in the input state.
 */
DROPBEAR_NATIVE dropbear_get_connected_gamepads(InputState* input_ptr, const Gamepad** out_gamepads, int32_t* out_count);

/**
 * @brief Checks if a button has been pressed on a specific gamepad.
 */
DROPBEAR_NATIVE dropbear_is_gamepad_button_pressed(const InputState* input_ptr, HANDLE gamepad_id, int ordinal, BOOL* out_pressed);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
#endif // DROPBEAR_INPUT_H