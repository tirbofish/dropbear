use crate::input::InputState;
use crate::ptr::InputStatePtr;
use crate::scripting::result::DropbearNativeResult;
use crate::types::NVector2;

pub mod shared {
    use jni::JNIEnv;
    use jni::objects::{JObject, JValue};
    use crate::input::InputState;
    use crate::scripting::jni::utils::{FromJObject, ToJObject};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::NVector2;

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

    pub fn get_gamepad_id(input: &InputState, target: usize) -> Option<dropbear_engine::gilrs::GamepadId> {
        input.connected_gamepads.iter().find(|g| usize::from(**g) == target).copied()
    }

    pub fn is_gamepad_button_pressed(input: &InputState, gamepad_id: u64, button_ordinal: i32) -> bool {
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
            return NVector2 {
                x: 0.0,
                y: 0.0
            }
        };
        let (x, y) = input.get_left_stick(id);
        NVector2 { x: x as f64, y: y as f64 }
    }

    pub fn get_right_stick(input: &InputState, gamepad_id: u64) -> NVector2 {
        let Some(id) = get_gamepad_id(input, gamepad_id as usize) else {
            return NVector2 {
                x: 0.0,
                y: 0.0
            }
        };
        let (x, y) = input.get_right_stick(id);
        NVector2 { x: x as f64, y: y as f64 }
    }

    impl ToJObject for NVector2 {
        fn to_jobject<'a>(&self, env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
            let cls = env.find_class("com/dropbear/math/Vector2d")
                .map_err(|e| {
                    eprintln!("Could not find Vector2d class: {:?}", e);
                    DropbearNativeError::GenericError
                })?;

            let obj = env.new_object(
                cls,
                "(DD)V",
                &[
                    JValue::Double(self.x),
                    JValue::Double(self.y)
                ]
            ).map_err(|e| {
                eprintln!("Failed to create Vector2d object: {:?}", e);
                DropbearNativeError::GenericError
            })?;

            Ok(obj)
        }
    }

    impl FromJObject for NVector2 {
        fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
        where
            Self: Sized
        {
            let x_val = env
                .get_field(obj, "x", "D")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

            let y_val = env
                .get_field(obj, "y", "D")
                .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
                .d()
                .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

            Ok(NVector2 {
                x: x_val,
                y: y_val,
            })
        }
    }
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.GamepadNative", func = "isGamepadButtonPressed"),
    c
)]
fn is_button_pressed(
    #[dropbear_macro::define(InputStatePtr)]
    input: &InputState,
    gamepad_id: u64,
    button_ordinal: i32,
) -> DropbearNativeResult<bool> {
    Ok(shared::is_gamepad_button_pressed(&input, gamepad_id, button_ordinal))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.GamepadNative", func = "getLeftStickPosition"),
    c
)]
fn get_left_stick_position(
    #[dropbear_macro::define(InputStatePtr)]
    input: &InputState,
    gamepad_id: u64,
) -> DropbearNativeResult<NVector2> {
    Ok(shared::get_left_stick(&input, gamepad_id))
}

#[dropbear_macro::export(
    kotlin(class = "com.dropbear.input.GamepadNative", func = "getRightStickPosition"),
    c
)]
fn get_right_stick_position(
    #[dropbear_macro::define(InputStatePtr)]
    input: &InputState,
    gamepad_id: u64,
) -> DropbearNativeResult<NVector2> {
    Ok(shared::get_right_stick(&input, gamepad_id))
}