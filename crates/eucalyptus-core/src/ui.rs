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

pub struct UiContext {
    pub yakui_state: Mutex<Yakui>,
    pub to_render: Mutex<Vec<Box<dyn FnOnce()>>>,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            yakui_state: Mutex::new(Yakui::new()),
            to_render: Default::default(),
        }
    }
}

pub mod jni {
    use jni::sys::jlong;
    use crate::convert_ptr;
    use crate::ui::{UiContext};

    #[unsafe(no_mangle)]
    pub extern "C" fn Java_YourClass_addOverlay(
        _env: jni::JNIEnv,
        _class: jni::objects::JClass,
        ui_buf_ptr: jlong,
    ) {
        let ui = convert_ptr!(ui_buf_ptr => UiContext);

        let mut state = ui.to_render.lock();

        state.push(Box::new(move || {
            // yakui::colored_box(
                // Color::rgba(255, 0, 0, 128),
                // yakui::geometry::Vec2::new(width, height)
            // );
        }));
    }
}