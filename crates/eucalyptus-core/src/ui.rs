mod button;
mod utils;
mod text;

use std::cell::RefCell;
use std::collections::HashMap;
use ::jni::JNIEnv;
use ::jni::objects::JObject;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use yakui::{Alignment, MainAxisSize, Yakui};
use yakui::font::Fonts;
use dropbear_engine::utils::ResourceReference;
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use crate::scripting::jni::utils::{FromJObject};
use crate::scripting::result::DropbearNativeResult;
use crate::ui::button::ButtonParser;
use crate::ui::text::TextParser;

thread_local! {
    pub static UI_CONTEXT: RefCell<UiContext> = RefCell::new(UiContext::new());
}

/// A component that can be attached to an entity that renders UI for the entire scene.
///
/// This UI is used in tandem with a `.kts` (Kotlin Script file) with the dropbear-engine scripting
/// ui DSL.
#[derive(Debug, Serialize, Deserialize, Clone, SerializableComponent, Default)]
pub struct UIComponent {
    /// Does not render the UI file.
    pub disabled: bool,
    /// The reference to the script file.
    pub ui_file: ResourceReference,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WidgetState {
    pub clicked: bool,
    pub hovering: bool,
}

pub trait NativeWidget: Send + std::fmt::Debug {
    fn build(self: Box<Self>, states: &mut HashMap<i64, WidgetState>);
}

#[derive(Debug)]
pub struct WrapperWidget<T> {
    pub id: i64,
    pub widget: T,
}

pub trait WidgetParser: Send + Sync {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<Box<dyn NativeWidget>>>;
    fn name(&self) -> String;
}

pub fn register_widget_parser<P: WidgetParser + 'static>(parser: P) {
    UI_CONTEXT.with(|v| {
        v.borrow().parsers.lock().push(Box::new(parser));
    });
}

pub struct UiContext {
    pub yakui_state: Mutex<Yakui>,
    pub instruction_set: Mutex<Vec<Box<dyn NativeWidget>>>,
    pub widget_states: Mutex<HashMap<i64, WidgetState>>,
    pub parsers: Mutex<Vec<Box<dyn WidgetParser>>>,
}

pub fn poll() {
    UI_CONTEXT.with(|v| {
        let ctx = v.borrow();
        let mut instructions = ctx.instruction_set.lock();
        let mut widget_states = ctx.widget_states.lock();

        widget_states.clear();
        
        let current_instructions = instructions.drain(..).collect::<Vec<Box<dyn NativeWidget>>>();
        yakui::widgets::Align::new(Alignment::TOP_LEFT).show(|| {
            yakui::widgets::List::column()
                .main_axis_size(MainAxisSize::Min)
                .show(|| {
                    for i in current_instructions {
                        i.build(&mut widget_states);
                    }
                });
        });
    });
}

impl UiContext {
    pub fn new() -> Self {
        let mut parsers: Vec<Box<dyn WidgetParser>> = Vec::new();

        parsers.push(Box::new(ButtonParser));
        parsers.push(Box::new(TextParser));

        let yakui = Yakui::new();
        let fonts = yakui.dom().get_global_or_init(Fonts::default);
        fonts.set_sans_serif_family("Roboto");
        fonts.set_serif_family("Roboto");
        fonts.set_cursive_family("Roboto");
        fonts.set_fantasy_family("Roboto");
        fonts.set_monospace_family("Roboto");

        Self {
            yakui_state: Mutex::new(yakui),
            instruction_set: Default::default(),
            widget_states: Default::default(),
            parsers: Mutex::new(parsers),
        }
    }
}

pub trait UiWidgetType: FromJObject {
    type UIWidgetType;

    fn as_id(&self) -> u32;
    fn from_id(id: u32) -> Self::UIWidgetType;
}

pub mod jni {
    #![allow(non_snake_case)]

    use jni::sys::{jboolean, jlong};
    use jni::objects::{JClass, JObjectArray};
    use jni::JNIEnv;
    use crate::convert_ptr;
    use crate::ui::{UiContext};

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_UINative_renderUI(
        mut env: JNIEnv,
        _class: JClass,
        ui_buf_ptr: jlong,
        instructions: JObjectArray,
    ) {
        // println!("[Java_com_dropbear_ui_UINative_renderUI] received new renderUI request");
        let ui = convert_ptr!(ui_buf_ptr => UiContext);
        let mut rust_instructions = Vec::new();

        let count = env.get_array_length(&instructions).unwrap_or(0);
        let parsers_guard = ui.parsers.lock();

        for i in 0..count {
            let obj = match env.get_object_array_element(&instructions, i) {
                Ok(o) => o,
                Err(_) => continue,
            };
            if obj.is_null() { println!("[Java_com_dropbear_ui_UINative_renderUI] obj is null at index {}", i); continue; }

            for parser in parsers_guard.iter() {
                match parser.parse(&mut env, &obj) {
                    Ok(Some(widget)) => {
                        // println!("Received widget: {:?}", widget);
                        rust_instructions.push(widget);
                        break;
                    },
                    Ok(None) => continue,
                    Err(e) => {
                        eprintln!("[Java_com_dropbear_ui_UINative_renderUI] Error converting UI instruction: {:?}", e);
                    }
                }
            }
        }

        ui.instruction_set.lock().extend(rust_instructions);
    }
    
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_widgets_ButtonNative_getClicked(
        _env: JNIEnv,
        _class: JClass,
        ui_buf_ptr: jlong,
        id: jlong,
    ) -> jboolean {
        let ui = convert_ptr!(ui_buf_ptr => UiContext);
        let states = ui.widget_states.lock();
        states.get(&id).map(|s| s.clicked).unwrap_or(false).into()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_widgets_ButtonNative_getHovering(
        _env: JNIEnv,
        _class: JClass,
        ui_buf_ptr: jlong,
        id: jlong,
    ) -> jboolean {
        let ui = convert_ptr!(ui_buf_ptr => UiContext);
        let states = ui.widget_states.lock();
        states.get(&id).map(|s| s.hovering).unwrap_or(false).into()
    }
}