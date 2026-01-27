use jni::JNIEnv;
use jni::objects::JObject;
use yakui::{Alignment, BorderRadius};
use yakui::widgets::{Button, Pad, DynamicButtonStyle};
use crate::scripting::jni::utils::FromJObject;
use crate::scripting::result::DropbearNativeResult;
use std::borrow::Cow;

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
