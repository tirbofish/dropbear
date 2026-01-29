use std::any::Any;
use std::collections::HashMap;
use ::jni::JNIEnv;
use ::jni::objects::JObject;
use yakui::widgets::Checkbox;
use crate::scripting::result::DropbearNativeResult;
use crate::ui::{NativeWidget, UiInstructionType, WidgetParser, WidgetState};

pub(crate) struct CheckboxParser;

#[derive(Debug)]
pub(crate) struct CheckboxWidget {
    pub id: i64,
    pub checkbox: Checkbox,
}

impl WidgetParser for CheckboxParser {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<UiInstructionType>> {
        let class = env.get_object_class(obj)?;
        let name_str_obj = env.call_method(class, "getName", "()Ljava/lang/String;", &[])?.l()?;
        let name_string: String = env.get_string(&name_str_obj.into())?.into();

        if name_string.contains("CheckboxInstruction$Checkbox") {
            let checked = env.get_field(obj, "checked", "Z")?.z()?;
            let checkbox = Checkbox::new(checked);

            let id_obj = env.get_field(obj, "id", "Lcom/dropbear/ui/WidgetId;")?.l()?;
            let id = env.get_field(id_obj, "id", "J")?.j()?;

            return Ok(Some(UiInstructionType::Widget(Box::new(CheckboxWidget {
                id,
                checkbox,
            }))));
        }

        Ok(None)
    }

    fn name(&self) -> String {
        String::from("CheckboxParser")
    }
}

impl NativeWidget for CheckboxWidget {
    fn render(self: Box<Self>, state: &mut HashMap<i64, WidgetState>) {
        let resp = self.checkbox.show();
        state.insert(self.id, WidgetState {
            clicked: false,
            hovering: false,
            checked: resp.checked,
        });

    }

    fn id(&self) -> i64 {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub mod jni {
    #![allow(non_snake_case)]

    use jni::JNIEnv;
    use jni::objects::JClass;
    use jni::sys::{jboolean, jlong};
    use crate::convert_ptr;
    use crate::ui::UiContext;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_widgets_CheckboxNative_getChecked(
        _env: JNIEnv,
        _: JClass,
        ui_buf_ptr: jlong,
        id: jlong,
    ) -> jboolean {
        let ui = convert_ptr!(ui_buf_ptr => UiContext);

        if let Some(v) = ui.widget_states.lock().get(&(id as i64)) {
            return v.checked.into();
        }
        false.into()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_dropbear_ui_widgets_CheckboxNative_hasCheckedState(
        _env: JNIEnv,
        _: JClass,
        ui_buf_ptr: jlong,
        id: jlong,
    ) -> jboolean {
        let ui = convert_ptr!(ui_buf_ptr => UiContext);

        ui.widget_states.lock().contains_key(&(id as i64)).into()
    }
}