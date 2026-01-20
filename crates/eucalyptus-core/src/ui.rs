pub mod rect;

use egui::{Rect, Response, Sense};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use crate::utils::hashmap::StaleTracker;

pub static UI_COMMAND_BUFFER: Lazy<UiContext> = Lazy::new(|| UiContext::new());

pub enum UiCommand {
    Rect {
        id: u64,
        initial: (f32, f32),
        size: (f32, f32),
        corner_radius: f32,
        stroke: egui::Stroke,
        fill: egui::Color32,
        stroke_kind: egui::StrokeKind,
    },
    Circle {
        id: u64,
        center_x: f64,
        center_y: f64,
        radius: f64,
    },
}

pub struct UiContext {
    command_buffer: Mutex<Vec<UiCommand>>,
    currently_rendering: Mutex<StaleTracker<u64, Response>>,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            command_buffer: Mutex::new(Vec::new()),
            currently_rendering: Mutex::new(StaleTracker::new()),
        }
    }

    pub fn push(&self, command: UiCommand) {
        self.command_buffer.lock().push(command);
    }
}

pub fn poll(ui: &mut egui::Ui) -> anyhow::Result<()> {
    let mut buffer = UI_COMMAND_BUFFER.command_buffer.lock();
    let mut rendering = UI_COMMAND_BUFFER.currently_rendering.lock();
    rendering.tick();
    for cmd in buffer.drain(..) {
        match cmd {
            UiCommand::Rect {
                id,
                initial,
                size,
                corner_radius,
                stroke,
                fill,
                stroke_kind
            } => {
                let (resp, painter) = ui.allocate_painter(size.into(), Sense::hover());

                painter.rect(
                    Rect {
                        min: initial.into(),
                        max: [initial.0 + size.0, initial.1 + size.1].into(),
                    },
                    corner_radius,
                    fill,
                    stroke,
                    stroke_kind
                );

                rendering.insert(id, resp);
            }
            UiCommand::Circle { .. } => {

            }
        }
    }

    // remove anything past 3 gen
    rendering.remove_stale(3);

    Ok(())
}

pub mod jni {
    #![allow(non_snake_case)]

    use jni::JNIEnv;
    use jni::sys::{jboolean, jlong};
    use jni::objects::JObject;
    use crate::{convert_ptr};
    use crate::scripting::jni::utils::FromJObject;
    use crate::ui::{UiCommand, UiContext};
    use crate::ui::rect::Rect;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_UINative_pushRect(
        mut env: JNIEnv,
        _class: jni::objects::JClass,
        ui_buffer_handle: jlong,
        rect: JObject,
    ) {
        let ui = convert_ptr!(ui_buffer_handle => UiContext);

        let rect: Rect = match Rect::from_jobject(&mut env, &rect) {
            Ok(v) => v,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Unable to convert Rectangle->Rect: {:?}", e));
                return;
            }
        };

        ui.push(UiCommand::Rect {
            id: rect.id,
            initial: rect.initial_pos,
            size: rect.size,
            corner_radius: rect.corner_radius,
            stroke: rect.stroke,
            fill: rect.fill,
            stroke_kind: rect.stroke_kind,
        });
    }
    
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_UINative_pushCircle(
        mut env: JNIEnv,
        _class: jni::objects::JClass,
        ui_buffer_handle: jlong,
        circle: JObject,
    ) {
        let ui = convert_ptr!(ui_buffer_handle => UiContext);

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

        let id = match env.call_method(&id_obj, "getId", "()J", &[]).and_then(|v| v.j()) {
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

        let center_x = match env.get_field(&center_obj, "x", "D").and_then(|v| v.d()) {
            Ok(val) => val,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get center.x: {}", e));
                return;
            }
        };

        let center_y = match env.get_field(&center_obj, "y", "D").and_then(|v| v.d()) {
            Ok(val) => val,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get center.y: {}", e));
                return;
            }
        };

        let radius = match env.get_field(&circle, "radius", "D").and_then(|v| v.d()) {
            Ok(val) => val,
            Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to get radius: {}", e));
                return;
            }
        };

        ui.push(UiCommand::Circle {
            id,
            center_x,
            center_y,
            radius,
        });
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