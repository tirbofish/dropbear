use std::cell::RefCell;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use yakui::Yakui;
use dropbear_engine::utils::ResourceReference;
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;

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

#[derive(Default, Debug)]
pub enum UIInstruction {
    #[default]
    Nothing,

    StartColumn,
    EndColumn,
}

pub struct UiContext {
    pub yakui_state: Mutex<Yakui>,
    pub instruction_set: Mutex<Vec<UIInstruction>>,
}

pub fn poll() {
    UI_CONTEXT.with(|v| {
        let ctx = v.borrow();
        let _yakui = ctx.yakui_state.lock();
        let instructions = ctx.instruction_set.lock().drain(..).collect::<Vec<UIInstruction>>();
        for i in instructions {
            match i {
                UIInstruction::StartColumn => {
                    
                },
                UIInstruction::EndColumn => {

                },
                UIInstruction::Nothing => {}
            }
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

pub mod jni {
    use jni::sys::jlong;
    use crate::convert_ptr;
    use crate::ui::{UiContext};

    #[unsafe(no_mangle)]
    pub extern "C" fn Java_foobar_addOverlay(
        _env: jni::JNIEnv,
        _class: jni::objects::JClass,
        ui_buf_ptr: jlong,
    ) {
        let ui = convert_ptr!(ui_buf_ptr => UiContext);

        ui.instruction_set.lock().push(crate::ui::UIInstruction::Nothing);
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn Java_foobar_aisClicked(
        _env: jni::JNIEnv,
        _class: jni::objects::JClass,
        ui_buf_ptr: jlong,
    ) {
        let _ui = convert_ptr!(ui_buf_ptr => UiContext);

        // ui.yakui_state.lock()
    }
}