mod button;
mod utils;

use std::cell::RefCell;
use ::jni::JNIEnv;
use ::jni::objects::JObject;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use yakui::{Yakui};
use dropbear_engine::utils::ResourceReference;
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use crate::scripting::jni::utils::{FromJObject};
use crate::scripting::result::DropbearNativeResult;

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

// note for tomorrow: use UIInstruction like that of asm

pub trait NativeWidget: Send + std::fmt::Debug {
    fn build(self: Box<Self>);
}

impl NativeWidget for yakui::widgets::Button {
    fn build(mut self: Box<Self>) {
        let _ = self.show();
    }
}

pub trait WidgetParser: Send + Sync {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<Box<dyn NativeWidget>>>;
}

static PARSERS: std::sync::OnceLock<Mutex<Vec<Box<dyn WidgetParser>>>> = std::sync::OnceLock::new();

pub fn register_widget_parser<P: WidgetParser + 'static>(parser: P) {
    let parsers = PARSERS.get_or_init(|| Mutex::new(Vec::new()));
    parsers.lock().push(Box::new(parser));
}

fn get_parsers() -> &'static Mutex<Vec<Box<dyn WidgetParser>>> {
    PARSERS.get_or_init(|| {
        let mut vec: Vec<Box<dyn WidgetParser>> = Vec::new();
        vec.push(Box::new(ButtonParser));
        Mutex::new(vec)
    })
}

struct ButtonParser;

impl WidgetParser for ButtonParser {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<Box<dyn NativeWidget>>> {
        let class = env.get_object_class(obj)?;
        let name_str_obj = env.call_method(class, "getName", "()Ljava/lang/String;", &[])?.l()?;
        let name_string: String = env.get_string(&name_str_obj.into())?.into();

        if name_string.contains("ButtonInstruction$Button") {
            let button_obj = env.get_field(obj, "button", "Lcom/dropbear/ui/widgets/Button;")?.l()?;
            let btn = yakui::widgets::Button::from_jobject(env, &button_obj)?;
            return Ok(Some(Box::new(btn)));
        }

        Ok(None)
    }
}


pub struct UiContext {
    pub yakui_state: Mutex<Yakui>,
    pub instruction_set: Mutex<Vec<Box<dyn NativeWidget>>>,
}

pub fn poll() {
    UI_CONTEXT.with(|v| {
        let ctx = v.borrow();
        let mut instructions = ctx.instruction_set.lock();
        let current_instructions = instructions.drain(..).collect::<Vec<Box<dyn NativeWidget>>>();
        for i in current_instructions {
            i.build();
        }
    });
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            yakui_state: Mutex::new(Yakui::new()),
            instruction_set: Default::default(),
        }
    }
}

pub trait UiWidgetType: FromJObject {
    type UIWidgetType;

    fn as_id(&self) -> u32;
    fn from_id(id: u32) -> Self::UIWidgetType;
}

pub mod jni {
    use jni::sys::jlong;
    use jni::objects::{JClass, JObjectArray};
    use jni::JNIEnv;
    use crate::convert_ptr;
    use crate::ui::{UiContext, get_parsers};
    use crate::scripting::jni::utils::FromJObject;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_UINative_renderUI(
        mut env: JNIEnv,
        _class: JClass,
        ui_buf_ptr: jlong,
        instructions: JObjectArray,
    ) {
        let ui = convert_ptr!(ui_buf_ptr => UiContext);
        let mut rust_instructions = Vec::new();

        let count = env.get_array_length(&instructions).unwrap_or(0);
        let parsers_guard = get_parsers().lock();

        for i in 0..count {
            let obj = match env.get_object_array_element(&instructions, i) {
                Ok(o) => o,
                Err(_) => continue,
            };
            if obj.is_null() { continue; }

            for parser in parsers_guard.iter() {
                match parser.parse(&mut env, &obj) {
                    Ok(Some(widget)) => {
                        rust_instructions.push(widget);
                        break;
                    },
                    Ok(None) => continue,
                    Err(e) => {
                        eprintln!("Error converting UI instruction: {:?}", e);
                    }
                }
            }
        }

        ui.instruction_set.lock().extend(rust_instructions);
    }
}