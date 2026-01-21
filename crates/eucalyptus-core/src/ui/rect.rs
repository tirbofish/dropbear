use jni::JNIEnv;
use jni::objects::JObject;
use crate::scripting::jni::utils::{FromJObject, ToJObject};
use crate::scripting::result::DropbearNativeResult;
use crate::scripting::native::DropbearNativeError;
use kino_gui::prelude::*;
use kino_gui::prelude::shapes::Rectangle;
use kino_gui::WidgetId;

impl FromJObject for Rectangle {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self>
    where
        Self: Sized
    {
        let class = env
            .find_class("com/dropbear/ui/primitive/Rectangle")
            .map_err(|_| DropbearNativeError::JNIClassNotFound)?;

        if !env
            .is_instance_of(obj, &class)
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
        {
            return Err(DropbearNativeError::InvalidArgument);
        }

        // Get the ID field
        let id_obj = env
            .get_field(obj, "id", "Lcom/dropbear/utils/ID;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;
        
        let id = env
            .call_method(&id_obj, "getId", "()J", &[])
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .j()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u64;

        // Get initial (Vector2d)
        let initial_obj = env
            .get_field(obj, "initial", "Lcom/dropbear/math/Vector2d;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let initial_x = env
            .get_field(&initial_obj, "x", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        let initial_y = env
            .get_field(&initial_obj, "y", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        // Get width and height
        let width = env
            .get_field(obj, "width", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        let height = env
            .get_field(obj, "height", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        // Get corner radius
        let corner_radius = env
            .get_field(obj, "cornerRadius", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        // Get stroke width
        let stroke_width = env
            .get_field(obj, "stroke", "D")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .d()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as f32;

        // Get fill colour (Colour object)
        let fill_colour_obj = env
            .get_field(obj, "fillColour", "Lcom/dropbear/utils/Colour;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let r = env
            .get_field(&fill_colour_obj, "r", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        let g = env
            .get_field(&fill_colour_obj, "g", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        let b = env
            .get_field(&fill_colour_obj, "b", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        let a = env
            .get_field(&fill_colour_obj, "a", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        // let fill = egui::Color32::from_rgba_unmultiplied(r, g, b, a);
        let fill = Colour {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        };
        
        // Get stroke colour
        let stroke_colour_obj = env
            .get_field(obj, "strokeColour", "Lcom/dropbear/utils/Colour;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let stroke_r = env
            .get_field(&stroke_colour_obj, "r", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        let stroke_g = env
            .get_field(&stroke_colour_obj, "g", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        let stroke_b = env
            .get_field(&stroke_colour_obj, "b", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        let stroke_a = env
            .get_field(&stroke_colour_obj, "a", "B")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .b()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)? as u8;

        let stroke_color = egui::Color32::from_rgba_unmultiplied(stroke_r, stroke_g, stroke_b, stroke_a);
        // let stroke = Stroke::new(stroke_width, stroke_color);

        // Get stroke kind (enum)
        let stroke_kind_obj = env
            .get_field(obj, "strokeKind", "Lcom/dropbear/ui/StrokeKind;")
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let stroke_kind_name = env
            .call_method(&stroke_kind_obj, "name", "()Ljava/lang/String;", &[])
            .map_err(|_| DropbearNativeError::JNIFailedToGetField)?
            .l()
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?;

        let stroke_kind_str: String = env
            .get_string(&stroke_kind_name.into())
            .map_err(|_| DropbearNativeError::JNIUnwrapFailed)?
            .into();

        // let stroke_kind = match stroke_kind_str.as_str() {
        //     "Inside" => StrokeKind::Inside,
        //     "Middle" => StrokeKind::Middle,
        //     "Outside" => StrokeKind::Outside,
        //     _ => StrokeKind::Middle, // default
        // };

        let rect = Rectangle::new(
            WidgetId::new(id),
            [initial_x, initial_y].into(),
            [width, height].into(),
            fill
        );
        
        Ok(rect)
    }
}

impl ToJObject for Rectangle {
    fn to_jobject<'a>(&self, _env: &mut JNIEnv<'a>) -> DropbearNativeResult<JObject<'a>> {
        todo!()
    }
}