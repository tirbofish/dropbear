use eucalyptus_core::input::InputState;
use eucalyptus_core::ptr::{CommandBufferPtr, CommandBufferUnwrapped, InputStatePtr};
use eucalyptus_core::scripting::result::DropbearNativeResult;
use crate::math::NVector2;

pub mod shared {
    use crossbeam_channel::Sender;
    use dropbear_engine::winit::event::MouseButton;
    use eucalyptus_core::command::{CommandBuffer, WindowCommand};
    use eucalyptus_core::input::InputState;
    use eucalyptus_core::scripting::native::DropbearNativeError;
    use eucalyptus_core::scripting::result::DropbearNativeResult;
    use eucalyptus_core::utils::keycode_from_ordinal;
    use crate::math::NVector2;

    fn map_int_to_gamepad_button(ordinal: i32) -> Option<dropbear_engine::gilrs::Button> {
        match ordinal {
            0 => Some(dropbear_engine::gilrs::Button::Unknown),
            1 => Some(dropbear_engine::gilrs::Button::South),
            2 => Some(dropbear_engine::gilrs::Button::East),
            3 => Some(dropbear_engine::gilrs::Button::North),
            4 => Some(dropbear_engine::gilrs::Button::West),
            5 => Some(dropbear_engine::gilrs::Button::C),
            6 => Some(dropbear_engine::gilrs::Button::Z),
            7 => Some(dropbear_engine::gilrs::Button::LeftTrigger),
            8 => Some(dropbear_engine::gilrs::Button::RightTrigger),
            9 => Some(dropbear_engine::gilrs::Button::LeftTrigger2),
            10 => Some(dropbear_engine::gilrs::Button::RightTrigger2),
            11 => Some(dropbear_engine::gilrs::Button::Select),
            12 => Some(dropbear_engine::gilrs::Button::Start),
            13 => Some(dropbear_engine::gilrs::Button::Mode),
            14 => Some(dropbear_engine::gilrs::Button::LeftThumb),
            15 => Some(dropbear_engine::gilrs::Button::RightThumb),
            16 => Some(dropbear_engine::gilrs::Button::DPadUp),
            17 => Some(dropbear_engine::gilrs::Button::DPadDown),
            18 => Some(dropbear_engine::gilrs::Button::DPadLeft),
            19 => Some(dropbear_engine::gilrs::Button::DPadRight),
            _ => None,
        }
    }

    pub fn get_gamepad_id(
        input: &InputState,
        target: usize,
    ) -> Option<dropbear_engine::gilrs::GamepadId> {
        input
            .connected_gamepads
            .iter()
            .find(|g| usize::from(**g) == target)
            .copied()
    }

    pub fn is_gamepad_button_pressed(
        input: &InputState,
        gamepad_id: u64,
        button_ordinal: i32,
    ) -> bool {
        let Some(id) = get_gamepad_id(input, gamepad_id as usize) else {
            return false;
        };

        if let Some(btn) = map_int_to_gamepad_button(button_ordinal) {
            input.is_button_pressed(id, btn)
        } else {
            false
        }
    }

    pub fn get_left_stick(input: &InputState, gamepad_id: u64) -> NVector2 {
        let Some(id) = get_gamepad_id(input, gamepad_id as usize) else {
            return NVector2 { x: 0.0, y: 0.0 };
        };
        let (x, y) = input.get_left_stick(id);
        NVector2 {
            x: x as f64,
            y: y as f64,
        }
    }

    pub fn get_right_stick(input: &InputState, gamepad_id: u64) -> NVector2 {
        let Some(id) = get_gamepad_id(input, gamepad_id as usize) else {
            return NVector2 { x: 0.0, y: 0.0 };
        };
        let (x, y) = input.get_right_stick(id);
        NVector2 {
            x: x as f64,
            y: y as f64,
        }
    }

    pub fn map_ordinal_to_mouse_button(ordinal: i32) -> Option<MouseButton> {
        match ordinal {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Right),
            2 => Some(MouseButton::Middle),
            3 => Some(MouseButton::Back),
            4 => Some(MouseButton::Forward),
            ordinal if ordinal >= 0 => Some(MouseButton::Other(ordinal as u16)),
            _ => None,
        }
    }

    pub fn is_key_pressed(input: &InputState, key_ordinal: i32) -> bool {
        if let Some(key) = keycode_from_ordinal(key_ordinal) {
            input.is_key_pressed(key)
        } else {
            false
        }
    }

    pub fn is_mouse_button_pressed(input: &InputState, btn_ordinal: i32) -> bool {
        if let Some(btn) = map_ordinal_to_mouse_button(btn_ordinal) {
            input.mouse_button.contains(&btn)
        } else {
            false
        }
    }

    pub fn get_mouse_position(input: &InputState) -> NVector2 {
        NVector2 {
            x: input.mouse_pos.0,
            y: input.mouse_pos.1,
        }
    }

    pub fn get_mouse_delta(input: &InputState) -> NVector2 {
        input
            .mouse_delta
            .map(NVector2::from)
            .unwrap_or(NVector2 { x: 0.0, y: 0.0 })
    }

    pub fn get_last_mouse_pos(input: &InputState) -> NVector2 {
        input
            .last_mouse_pos
            .map(NVector2::from)
            .unwrap_or(NVector2 { x: 0.0, y: 0.0 })
    }

    pub fn set_cursor_locked(
        input: &mut InputState,
        sender: &Sender<CommandBuffer>,
        locked: bool,
    ) -> DropbearNativeResult<()> {
        input.is_cursor_locked = locked;

        sender
            .send(CommandBuffer::WindowCommand(WindowCommand::WindowGrab(
                locked,
            )))
            .map_err(|_| DropbearNativeError::SendError)?;

        Ok(())
    }

    pub fn set_cursor_hidden(
        input: &mut InputState,
        sender: &Sender<CommandBuffer>,
        hidden: bool,
    ) -> DropbearNativeResult<()> {
        input.is_cursor_hidden = hidden;
        sender
            .send(CommandBuffer::WindowCommand(WindowCommand::HideCursor(
                hidden,
            )))
            .map_err(|_| DropbearNativeError::SendError)?;
        Ok(())
    }

    pub fn get_connected_gamepads(input: &InputState) -> Vec<u64> {
        input
            .connected_gamepads
            .iter()
            .map(|id| Into::<usize>::into(*id) as u64)
            .collect()
    }
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "printInputState"
    ),
    c
)]
fn print_input_state(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<()> {
    println!("Input State: {:?}", input);
    Ok(())
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "isKeyPressed"),
    c
)]
fn is_key_pressed(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
    key_code: i32,
) -> DropbearNativeResult<bool> {
    Ok(shared::is_key_pressed(input, key_code))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "getMousePosition"
    ),
    c
)]
fn get_mouse_position(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<NVector2> {
    Ok(shared::get_mouse_position(input))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "isMouseButtonPressed"
    ),
    c
)]
fn is_mouse_button_pressed(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
    button_ordinal: i32,
) -> DropbearNativeResult<bool> {
    Ok(shared::is_mouse_button_pressed(input, button_ordinal))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "getMouseDelta"),
    c
)]
fn get_mouse_delta(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<NVector2> {
    Ok(shared::get_mouse_delta(input))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "isCursorLocked"),
    c
)]
fn is_cursor_locked(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<bool> {
    Ok(input.is_cursor_locked)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "setCursorLocked"
    ),
    c
)]
fn set_cursor_locked(
    #[dropbear_macro::define(CommandBufferPtr)]
    command_buffer: &CommandBufferUnwrapped,
    #[dropbear_macro::define(InputStatePtr)] input: &mut InputState,
    locked: bool,
) -> DropbearNativeResult<()> {
    shared::set_cursor_locked(input, command_buffer, locked)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "getLastMousePos"
    ),
    c
)]
fn get_last_mouse_pos(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<NVector2> {
    Ok(shared::get_last_mouse_pos(input))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.InputStateNative", func = "isCursorHidden"),
    c
)]
fn is_cursor_hidden(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<bool> {
    Ok(input.is_cursor_hidden)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "setCursorHidden"
    ),
    c
)]
fn set_cursor_hidden(
    #[dropbear_macro::define(CommandBufferPtr)]
    command_buffer: &CommandBufferUnwrapped,
    #[dropbear_macro::define(InputStatePtr)] input: &mut InputState,
    hidden: bool,
) -> DropbearNativeResult<()> {
    shared::set_cursor_hidden(input, command_buffer, hidden)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.InputStateNative",
        func = "getConnectedGamepads"
    ),
    c
)]
fn get_connected_gamepads(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
) -> DropbearNativeResult<Vec<u64>> {
    Ok(shared::get_connected_gamepads(input))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.GamepadNative",
        func = "isGamepadButtonPressed"
    ),
    c
)]
fn is_button_pressed(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
    gamepad_id: u64,
    button_ordinal: i32,
) -> DropbearNativeResult<bool> {
    Ok(shared::is_gamepad_button_pressed(input, gamepad_id, button_ordinal))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.GamepadNative",
        func = "getLeftStickPosition"
    ),
    c
)]
fn get_left_stick_position(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
    gamepad_id: u64,
) -> DropbearNativeResult<NVector2> {
    Ok(shared::get_left_stick(input, gamepad_id))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.input.GamepadNative",
        func = "getRightStickPosition"
    ),
    c
)]
fn get_right_stick_position(
    #[dropbear_macro::define(InputStatePtr)] input: &InputState,
    gamepad_id: u64,
) -> DropbearNativeResult<NVector2> {
    Ok(shared::get_right_stick(input, gamepad_id))
}