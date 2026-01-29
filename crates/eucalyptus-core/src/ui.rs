mod button;
mod utils;
mod text;
mod align;
mod checkbox;

use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use ::jni::JNIEnv;
use ::jni::objects::JObject;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use yakui::{Alignment, MainAxisSize, Yakui};
use dropbear_engine::utils::ResourceReference;
use dropbear_macro::SerializableComponent;
use dropbear_traits::SerializableComponent;
use crate::scripting::jni::utils::{FromJObject};
use crate::scripting::result::DropbearNativeResult;
use crate::ui::align::{AlignParser};
use crate::ui::button::ButtonParser;
use crate::ui::checkbox::CheckboxParser;
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
    pub checked: bool,
}

pub trait WidgetParser: Send + Sync {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<UiInstructionType>>;
    fn name(&self) -> String;
}

pub struct UiContext {
    pub yakui_state: Mutex<Yakui>,
    pub instruction_set: Mutex<Vec<UiInstructionType>>,
    pub widget_states: Mutex<HashMap<i64, WidgetState>>,
    pub parsers: Mutex<Vec<Box<dyn WidgetParser>>>,
}

pub fn poll() {
    UI_CONTEXT.with(|v| {
        let ctx = v.borrow();
        let mut instructions = ctx.instruction_set.lock();
        let mut widget_states = ctx.widget_states.lock();

        widget_states.clear();

        let current_instructions = instructions.drain(..).collect::<Vec<UiInstructionType>>();

        let tree = build_tree(current_instructions);

        yakui::widgets::Align::new(Alignment::TOP_LEFT).show(|| {
            yakui::widgets::List::column()
                .main_axis_size(MainAxisSize::Max)
                .show(|| {
                    render_tree(tree, &mut widget_states);
                });
        });
    });
}

fn build_tree(instructions: Vec<UiInstructionType>) -> Vec<UiNode> {
    let mut stack: Vec<UiNode> = Vec::new();
    let mut root = Vec::new();

    for instruction in instructions {
        match &instruction {
            UiInstructionType::Containered(container_ty) => {
                match container_ty {
                    ContaineredWidgetType::Start { .. } => {
                        stack.push(UiNode {
                            instruction,
                            children: Vec::new(),
                        });
                    }
                    ContaineredWidgetType::End { .. } => {
                        if let Some(node) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(node);
                            } else {
                                root.push(node);
                            }
                        }
                    }
                }
            }
            UiInstructionType::Widget(_) => {
                let node = UiNode {
                    instruction,
                    children: Vec::new(),
                };

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    root.push(node);
                }
            }
        }
    }

    root
}

pub fn render_tree(nodes: Vec<UiNode>, widget_state: &mut HashMap<i64, WidgetState>) {
    for node in nodes {
        match node.instruction {
            UiInstructionType::Containered(container_ty) => {
                match container_ty {
                    ContaineredWidgetType::Start { widget, .. } => {
                        widget.render(node.children, widget_state);
                    }
                    ContaineredWidgetType::End { .. } => {
                        // already handled in tree building
                    }
                }
            }
            UiInstructionType::Widget(widget) => {
                widget.render(widget_state);
            }
        }
    }
}

#[derive(Debug)]
pub enum UiInstructionType {
    Containered(ContaineredWidgetType),
    Widget(Box<dyn NativeWidget>),
}

#[derive(Debug)]
pub enum ContaineredWidgetType {
    Start {
        id: i64,
        widget: Box<dyn ContaineredWidget>,
    },
    End {
        id: i64,
    }
}

pub trait NativeWidget: Send + Sync + Debug {
    fn render(self: Box<Self>, state: &mut HashMap<i64, WidgetState>);
    fn id(&self) -> i64;
    fn as_any(&self) -> &dyn Any;
}

pub trait ContaineredWidget: Send + Sync + Debug {
    fn render(self: Box<Self>, children: Vec<UiNode>, state: &mut HashMap<i64, WidgetState>);
    fn as_any(&self) -> &dyn Any;
}

pub struct UiNode {
    pub instruction: UiInstructionType,
    pub children: Vec<UiNode>,
}

impl UiContext {
    pub fn new() -> Self {
        let mut parsers: Vec<Box<dyn WidgetParser>> = Vec::new();

        parsers.push(Box::new(ButtonParser));
        parsers.push(Box::new(TextParser));
        parsers.push(Box::new(AlignParser));
        parsers.push(Box::new(CheckboxParser));

        let yakui = Yakui::new();

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

    use jni::sys::{jlong};
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

        let mut instruction_set = ui.instruction_set.lock();
        instruction_set.clear();
        instruction_set.extend(rust_instructions);
    }
}