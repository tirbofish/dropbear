use jni::JNIEnv;
use jni::objects::JObject;
use yakui::{Alignment, BorderRadius};
use yakui::widgets::{Button, Pad, DynamicButtonStyle};
use crate::scripting::jni::utils::FromJObject;
use crate::scripting::result::DropbearNativeResult;
use std::borrow::Cow;
use std::collections::HashMap;
use crate::ui::{NativeWidget, WidgetParser, WidgetState, WrapperWidget};

pub(crate) struct ButtonParser;

impl WidgetParser for ButtonParser {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<Box<dyn NativeWidget>>> {
        let class = env.get_object_class(obj)?;
        let name_str_obj = env.call_method(class, "getName", "()Ljava/lang/String;", &[])?.l()?;
        let name_string: String = env.get_string(&name_str_obj.into())?.into();
        // println!("ButtonParser obj get_string result: {}", name_string);

        if name_string.contains("ButtonInstruction$Button") {
            let button_obj = env.get_field(obj, "button", "Lcom/dropbear/ui/widgets/Button;")?.l()?;
            let btn = yakui::widgets::Button::from_jobject(env, &button_obj)?;

            let id_obj = env.get_field(obj, "id", "Lcom/dropbear/ui/WidgetId;")?.l()?;
            let id = env.get_field(id_obj, "id", "J")?.j()?;

            return Ok(Some(Box::new(WrapperWidget {
                id,
                widget: btn,
            })));
        }

        Ok(None)
    }

    fn name(&self) -> String {
        String::from("ButtonParser")
    }
}

impl NativeWidget for WrapperWidget<yakui::widgets::Button> {
    fn build(self: Box<Self>, states: &mut HashMap<i64, WidgetState>) {
        let res = self.widget.show();
        states.insert(self.id, WidgetState {
            clicked: res.clicked,
            hovering: res.hovering,
        });
    }
}

impl FromJObject for Button {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let text_field = env.get_field(obj, "text", "Ljava/lang/String;")?;
        let text_jstring = text_field.l()?.into();
        let text: String = env.get_string(&text_jstring)?.into();

        let f = env.get_field(obj, "alignment", "Lcom/dropbear/ui/styling/Alignment;")?.l()?;
        let alignment = Alignment::from_jobject(
            env,
            &f
        )?;

        let f = env.get_field(obj, "padding", "Lcom/dropbear/ui/styling/Padding;")?.l()?;
        let padding = Pad::from_jobject(
            env,
            &f
        )?;

        let f = env.get_field(obj, "borderRadius", "Lcom/dropbear/ui/styling/BorderRadius;")?.l()?;
        let border_radius = BorderRadius::from_jobject(
            env,
            &f
        )?;
        
        let f = env.get_field(obj, "style", "Lcom/dropbear/ui/styling/DynamicButtonStyle;")?.l()?;
        let style = DynamicButtonStyle::from_jobject(
            env,
            &f
        )?;

        let f = env.get_field(obj, "hoverStyle", "Lcom/dropbear/ui/styling/DynamicButtonStyle;")?.l()?;
        let hover_style = DynamicButtonStyle::from_jobject(
            env,
            &f
        )?;

        let f = env.get_field(obj, "downStyle", "Lcom/dropbear/ui/styling/DynamicButtonStyle;")?.l()?;
        let down_style = DynamicButtonStyle::from_jobject(
            env,
            &f
        )?;

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
