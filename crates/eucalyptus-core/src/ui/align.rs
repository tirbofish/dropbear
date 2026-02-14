use std::any::Any;
use std::collections::HashMap;
use jni::JNIEnv;
use jni::objects::JObject;
use yakui::Alignment;
use crate::scripting::jni::utils::FromJObject;
use crate::scripting::result::DropbearNativeResult;
use crate::ui::{ContaineredWidget, ContaineredWidgetType, UiInstructionType, UiNode, WidgetParser, WidgetState};

pub(crate) struct AlignParser;

#[derive(Debug, Clone)]
pub(crate) struct AlignWidget {
    pub alignment: Alignment,
}

impl WidgetParser for AlignParser {
    fn parse(&self, env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Option<UiInstructionType>> {
        let class = env.get_object_class(obj)?;
        let name_str_obj = env.call_method(class, "getName", "()Ljava/lang/String;", &[])?.l()?;
        let name_string: String = env.get_string(&name_str_obj.into())?.into();

        if name_string.contains("AlignmentInstruction$StartAlignmentBlock") {
            let align_obj = env.get_field(obj, "align", "Lcom/dropbear/ui/widgets/Align;")?.l()?;
            let align = yakui::widgets::Align::from_jobject(env, &align_obj)?;

            let id_obj = env.get_field(obj, "id", "Lcom/dropbear/ui/WidgetId;")?.l()?;
            let id = env.get_field(id_obj, "id", "J")?.j()?;

            return Ok(Some(UiInstructionType::Containered(
                ContaineredWidgetType::Start {
                    id,
                    widget: Box::new(AlignWidget {
                        alignment: align.alignment,
                    }),
                }
            )));
        }

        if name_string.contains("AlignmentInstruction$EndAlignmentBlock") {
            let id_obj = env.get_field(obj, "id", "Lcom/dropbear/ui/WidgetId;")?.l()?;
            let id = env.get_field(id_obj, "id", "J")?.j()?;

            return Ok(Some(UiInstructionType::Containered(
                ContaineredWidgetType::End { id }
            )));
        }

        Ok(None)
    }

    fn name(&self) -> String {
        String::from("AlignParser")
    }
}

impl ContaineredWidget for AlignWidget {
    fn render(self: Box<Self>, children: Vec<UiNode>, state: &mut HashMap<i64, WidgetState>) {
        yakui::widgets::Align::new(self.alignment).show(|| {
            super::render_tree(children, state);
        });
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl FromJObject for yakui::widgets::Align {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let align_obj = env.get_field(obj, "align", "Lcom/dropbear/ui/styling/Alignment;")?.l()?;
        let alignment = Alignment::from_jobject(env, &align_obj)?;

        Ok(Self {
            alignment,
        })
    }
}