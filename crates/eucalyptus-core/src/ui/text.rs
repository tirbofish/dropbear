use std::any::Any;
use std::borrow::Cow;
use std::collections::HashMap;
use jni::JNIEnv;
use jni::objects::JObject;
use yakui::style::TextStyle;
use yakui::widgets::Pad;
use crate::scripting::jni::utils::FromJObject;
use crate::scripting::result::DropbearNativeResult;
use crate::ui::{NativeWidget, UiInstructionType, WidgetParser, WidgetState};

pub(crate) struct TextParser;

#[derive(Debug)]
pub(crate) struct TextWidget {
    pub id: i64,
    pub text: yakui::widgets::Text,
}

impl WidgetParser for TextParser {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<UiInstructionType>> {
        let class = env.get_object_class(obj)?;
        let name_str_obj = env.call_method(class, "getName", "()Ljava/lang/String;", &[])?.l()?;
        let name_string: String = env.get_string(&name_str_obj.into())?.into();
        // println!("TextParser obj get_string result: {}", name_string);

        if name_string.contains("TextInstruction$Text") {
            let text_obj = env.get_field(obj, "text", "Lcom/dropbear/ui/widgets/Text;")?.l()?;
            let text = yakui::widgets::Text::from_jobject(env, &text_obj)?;

            let id_obj = env.get_field(obj, "id", "Lcom/dropbear/ui/WidgetId;")?.l()?;
            let id = env.get_field(id_obj, "id", "J")?.j()?;

            return Ok(Some(UiInstructionType::Widget(Box::new(TextWidget {
                id,
                text,
            }))))
        }

        Ok(None)
    }

    fn name(&self) -> String {
        String::from("TextParser")
    }
}

impl NativeWidget for TextWidget {
    fn render(self: Box<Self>, _state: &mut HashMap<i64, WidgetState>) {
        let _ = self.text.show(); // no response
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl FromJObject for yakui::widgets::Text {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let text_field = env.get_field(obj, "text", "Ljava/lang/String;")?;
        let text_jstring = text_field.l()?.into();
        let text: String = env.get_string(&text_jstring)?.into();

        let style_obj = env.get_field(obj, "style", "Lcom/dropbear/ui/styling/TextStyle;")?.l()?;
        let style = TextStyle::from_jobject(env, &style_obj)?;

        let f = env.get_field(obj, "padding", "Lcom/dropbear/ui/styling/Padding;")?.l()?;
        let padding = Pad::from_jobject(
            env,
            &f
        )?;
        
        Ok(Self {
            text: Cow::Owned(text),
            style,
            padding,
        })
    }
}