pub mod shared {
    use dropbear_engine::gilrs::{Button, GamepadId};
    use jni::JNIEnv;
    use jni::objects::{JObject, JValue};
    use dropbear_engine::gilrs;
    use crate::input::InputState;
    use crate::scripting::jni::utils::{FromJObject, ToJObject};
    use crate::scripting::native::DropbearNativeError;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::Vector2;

    fn map_int_to_gamepad_button(ordinal: i32) -> Option<Button> {
        match ordinal {
            0 => Some(gilrs::Button::Unknown),
            1 => Some(gilrs::Button::South),
            2 => Some(gilrs::Button::East),
            3 => Some(gilrs::Button::North),
            4 => Some(gilrs::Button::West),
            5 => Some(gilrs::Button::C),
            6 => Some(gilrs::Button::Z),
            7 => Some(gilrs::Button::LeftTrigger),
            8 => Some(gilrs::Button::RightTrigger),
            9 => Some(gilrs::Button::LeftTrigger2),
            10 => Some(gilrs::Button::RightTrigger2),
            11 => Some(gilrs::Button::Select),
            12 => Some(gilrs::Button::Start),
            13 => Some(gilrs::Button::Mode),
            14 => Some(gilrs::Button::LeftThumb),
            15 => Some(gilrs::Button::RightThumb),
            16 => Some(gilrs::Button::DPadUp),
            17 => Some(gilrs::Button::DPadDown),
            18 => Some(gilrs::Button::DPadLeft),
            19 => Some(gilrs::Button::DPadRight),
            _ => None,
        }
    }

    pub fn get_gamepad_id(input: &InputState, target: usize) -> Option<GamepadId> {
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

    pub fn get_left_stick(input: &InputState, gamepad_id: u64) -> Vector2 {
        let Some(id) = get_gamepad_id(input, gamepad_id as usize) else {
            return Vector2 {
                x: 0.0,
                y: 0.0
            }
        };
        let (x, y) = input.get_left_stick(id);
        Vector2 { x: x as f64, y: y as f64 }
    }

    pub fn get_right_stick(input: &InputState, gamepad_id: u64) -> Vector2 {
        let Some(id) = get_gamepad_id(input, gamepad_id as usize) else {
            return Vector2 {
                x: 0.0,
                y: 0.0
            }
        };
        let (x, y) = input.get_right_stick(id);
        Vector2 { x: x as f64, y: y as f64 }
    }

    impl ToJObject for Vector2 {
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

    impl FromJObject for Vector2 {
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

            Ok(Vector2 {
                x: x_val,
                y: y_val,
            })
        }
    }
}

pub mod jni {
    #![allow(non_snake_case)]

    use jni::JNIEnv;
    use jni::objects::JClass;
    use jni::sys::{jboolean, jint, jlong, jobject};
    use crate::input::InputState;
    use crate::scripting::jni::utils::ToJObject;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_input_GamepadNative_isGamepadButtonPressed(
        _env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
        gamepad_id: jlong,
        button_ordinal: jint,
    ) -> jboolean {
        let input = crate::convert_ptr!(input_ptr => InputState);
        if super::shared::is_gamepad_button_pressed(&input, gamepad_id as u64, button_ordinal) {
            1
        } else {
            0
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_input_GamepadNative_getLeftStickPosition(
        mut env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
        gamepad_id: jlong,
    ) -> jobject {
        let input = crate::convert_ptr!(input_ptr => InputState);
        let vec = super::shared::get_left_stick(&input, gamepad_id as u64);

        match vec.to_jobject(&mut env) {
            Ok(obj) => obj.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_input_GamepadNative_getRightStickPosition(
        mut env: JNIEnv,
        _class: JClass,
        input_ptr: jlong,
        gamepad_id: jlong,
    ) -> jobject {
        let input = crate::convert_ptr!(input_ptr => InputState);
        let vec = super::shared::get_right_stick(&input, gamepad_id as u64);

        match vec.to_jobject(&mut env) {
            Ok(obj) => obj.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[dropbear_macro::impl_c_api]
pub mod native {
    use crate::ptr::InputStatePtr;
    use crate::input::{InputState};
    use crate::convert_ptr;
    use crate::scripting::result::DropbearNativeResult;
    use crate::types::Vector2;

    pub fn dropbear_is_gamepad_button_pressed(
        input_ptr: InputStatePtr,
        gamepad_id: u64,
        button_ordinal: i32
    ) -> DropbearNativeResult<bool> {
        let input = convert_ptr!(input_ptr => InputState);
        let result = super::shared::is_gamepad_button_pressed(input, gamepad_id, button_ordinal);
        DropbearNativeResult::Ok(result)
    }

    pub fn dropbear_get_left_stick_position(
        input_ptr: InputStatePtr,
        gamepad_id: u64
    ) -> DropbearNativeResult<Vector2> {
        let input = convert_ptr!(input_ptr => InputState);
        let vec = super::shared::get_left_stick(input, gamepad_id);
        DropbearNativeResult::Ok(vec)
    }

    pub fn dropbear_get_right_stick_position(
        input_ptr: InputStatePtr,
        gamepad_id: u64
    ) -> DropbearNativeResult<Vector2> {
        let input = convert_ptr!(input_ptr => InputState);
        let vec = super::shared::get_right_stick(input, gamepad_id);
        DropbearNativeResult::Ok(vec)
    }

    pub fn dropbear_free_gamepads_array(
        ptr: *mut u64,
        count: usize,
    ) -> DropbearNativeResult<()> {
        if ptr.is_null() {
            return DropbearNativeResult::Ok(());
        }

        unsafe {
            let _ = Vec::from_raw_parts(ptr, count, count);
        }

        DropbearNativeResult::Ok(())
    }
}