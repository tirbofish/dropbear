pub mod rect;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use kino_gui::prelude::*;
use std::collections::HashMap;

pub static UI_COMMAND_BUFFER: Lazy<UiContext> = Lazy::new(|| UiContext::new());

#[derive(Clone, Copy, Debug)]
pub struct UiResponse {
    pub id: u64,
    pub was_clicked: bool,
}

impl UiResponse {
    pub fn clicked(&self) -> bool {
        self.was_clicked
    }
}

pub enum UiCommand {
    Rect {
        id: u64,
        initial: Vector2,
        size: Size,
        corner_radius: f32,
        stroke: f32,
        fill: Colour,
        stroke_kind: String,
    },
    Circle {
        id: u64,
        center: Vector2,
        radius: f32,
        fill: Colour,
        stroke: f32,
    },
}

pub struct UiContext {
    commands: Mutex<Vec<UiCommand>>,
    pub currently_rendering: Mutex<HashMap<u64, UiResponse>>,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            commands: Mutex::new(Vec::new()),
            currently_rendering: Mutex::new(HashMap::new()),
        }
    }

    pub fn push(&self, command: UiCommand) {
        self.commands.lock().push(command);
    }

    pub fn drain_commands(&self) -> Vec<UiCommand> {
        self.commands.lock().drain(..).collect()
    }

    pub fn update_responses(&self, responses: HashMap<u64, UiResponse>) {
        let mut current = self.currently_rendering.lock();
        *current = responses;
    }
}

pub mod jni {
    #![allow(non_snake_case)]

    use jni::JNIEnv;
    use jni::sys::{jboolean, jlong};
    use jni::objects::JObject;
    use kino_gui::prelude::shapes::Rectangle;
    use kino_gui::Widget;
    use crate::{convert_ptr};
    use crate::scripting::jni::utils::FromJObject;
    use crate::ui::{UiCommand, UiContext};

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_UINative_pushRect(
        mut env: JNIEnv,
        _class: jni::objects::JClass,
        ui_buffer_handle: jlong,
        rect: JObject,
    ) {
        let ui = convert_ptr!(ui_buffer_handle => UiContext);

        let rect = match Rectangle::from_jobject(&mut env, &rect) {
            Ok(v) => v,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to convert scripting::Rectangle->kino::Rectangle: {:?}", e));
                return;
            }
        };

        ui.push(UiCommand::Rect {
            id: rect.id().as_u64(),
            initial: rect.initial,
            size: rect.size,
            corner_radius: 0.0, // rect.corner_radius when available
            stroke: 0.0, // rect.stroke when available
            fill: rect.fill_colour,
            stroke_kind: "Middle".to_string(), // rect.stroke_kind when available
        });
    }
    
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_UINative_pushCircle(
        mut env: JNIEnv,
        _class: jni::objects::JClass,
        ui_buffer_handle: jlong,
        circle: JObject,
    ) {
        let _ui = convert_ptr!(ui_buffer_handle => UiContext);

        // Extract Circle fields
        let id_obj = match env
            .get_field(&circle, "id", "Lcom/dropbear/utils/ID;")
            .and_then(|v| v.l())
        {
            Ok(obj) => obj,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get id field: {}", e));
                return;
            }
        };

        let _id = match env.call_method(&id_obj, "getId", "()J", &[]).and_then(|v| v.j()) {
            Ok(val) => val as u64,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get id value: {}", e));
                return;
            }
        };

        let center_obj = match env
            .get_field(&circle, "center", "Lcom/dropbear/math/Vector2d;")
            .and_then(|v| v.l())
        {
            Ok(obj) => obj,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get center field: {}", e));
                return;
            }
        };

        let _center_x = match env.get_field(&center_obj, "x", "D").and_then(|v| v.d()) {
            Ok(val) => val,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get center.x: {}", e));
                return;
            }
        };

        let _center_y = match env.get_field(&center_obj, "y", "D").and_then(|v| v.d()) {
            Ok(val) => val,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get center.y: {}", e));
                return;
            }
        };

        let _radius = match env.get_field(&circle, "radius", "D").and_then(|v| v.d()) {
            Ok(val) => val,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get radius: {}", e));
                return;
            }
        };

        panic!("this is not implemented yet :(")
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_UINative_wasClicked(
        _env: JNIEnv,
        _class: jni::objects::JClass,
        ui_buffer_handle: jlong,
        id: jlong,
    ) -> jboolean {
        let ui = convert_ptr!(ui_buffer_handle => UiContext);

        let mut rendering = ui.currently_rendering.lock();
        if let Some(response) = rendering.get(&(id as u64)) {
            response.clicked().into()
        } else {
            false.into()
        }
    }
}