use std::any::Any;
use std::collections::HashMap;
use yakui::{Alignment, BorderRadius};
use yakui::widgets::{Button, Pad, DynamicButtonStyle};
use crate::scripting::jni::utils::FromJObject;
use crate::scripting::result::DropbearNativeResult;
use std::borrow::Cow;
use ::jni::JNIEnv;
use ::jni::objects::JObject;
use crate::ui::{NativeWidget, UiInstructionType, WidgetParser, WidgetState};

pub(crate) struct ButtonParser;

#[derive(Debug)]
pub(crate) struct ButtonWidget {
    pub id: i64,
    pub button: yakui::widgets::Button,
}

impl WidgetParser for ButtonParser {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<UiInstructionType>> {
        let class = env.get_object_class(obj)?;
        let name_str_obj = env.call_method(class, "getName", "()Ljava/lang/String;", &[])?.l()?;
        let name_string: String = env.get_string(&name_str_obj.into())?.into();

        if name_string.contains("ButtonInstruction$Button") {
            let button_obj = env.get_field(obj, "button", "Lcom/dropbear/ui/widgets/Button;")?.l()?;
            let button = yakui::widgets::Button::from_jobject(env, &button_obj)?;

            let id_obj = env.get_field(obj, "id", "Lcom/dropbear/ui/WidgetId;")?.l()?;
            let id = env.get_field(id_obj, "id", "J")?.j()?;

            return Ok(Some(UiInstructionType::Widget(Box::new(ButtonWidget {
                id,
                button,
            }))));
        }

        Ok(None)
    }

    fn name(&self) -> String {
        String::from("ButtonParser")
    }
}

impl NativeWidget for ButtonWidget {
    fn render(self: Box<Self>, states: &mut HashMap<i64, WidgetState>) {
        let res = self.button.show();
        states.insert(self.id, WidgetState {
            clicked: res.clicked,
            hovering: res.hovering,
            checked: false, // always be false because it is not a checkbox, obv
        });
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl FromJObject for Button {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let text_field = env.get_field(obj, "text", "Ljava/lang/String;")?;
        let text_jstring = text_field.l()?.into();
        let text: String = env.get_string(&text_jstring)?.into();

        let f = env.get_field(obj, "alignment", "Lcom/dropbear/ui/styling/Alignment;")?.l()?;
        let alignment = Alignment::from_jobject(env, &f)?;

        let f = env.get_field(obj, "padding", "Lcom/dropbear/ui/styling/Padding;")?.l()?;
        let padding = Pad::from_jobject(env, &f)?;

        let f = env.get_field(obj, "borderRadius", "Lcom/dropbear/ui/styling/BorderRadius;")?.l()?;
        let border_radius = BorderRadius::from_jobject(env, &f)?;

        let f = env.get_field(obj, "style", "Lcom/dropbear/ui/styling/DynamicButtonStyle;")?.l()?;
        let style = DynamicButtonStyle::from_jobject(env, &f)?;

        let f = env.get_field(obj, "hoverStyle", "Lcom/dropbear/ui/styling/DynamicButtonStyle;")?.l()?;
        let hover_style = DynamicButtonStyle::from_jobject(env, &f)?;

        let f = env.get_field(obj, "downStyle", "Lcom/dropbear/ui/styling/DynamicButtonStyle;")?.l()?;
        let down_style = DynamicButtonStyle::from_jobject(env, &f)?;

        Ok(Self {
            text: Cow::Owned(text),
            alignment,
            padding,
            border_radius,
            style,
            hover_style,
            down_style,
        })
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