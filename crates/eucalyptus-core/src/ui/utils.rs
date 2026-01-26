use jni::JNIEnv;
use jni::objects::{JObject, JByteArray};
use crate::scripting::jni::utils::FromJObject;
use crate::scripting::result::DropbearNativeResult;
use yakui::Color;
use yakui::widgets::{Pad, DynamicButtonStyle};
use yakui::{Alignment, BorderRadius};
use yakui::style::{TextStyle, TextAlignment};
use yakui::cosmic_text::{Attrs, AttrsOwned, FamilyOwned, Weight, Style, Stretch, CacheKeyFlags, FontFeatures, Feature, FeatureTag};

impl FromJObject for FeatureTag {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let tag_obj = env.call_method(obj, "asBytes", "()[B", &[])?.l()?;
        let tag_bytes = env.convert_byte_array(JByteArray::from(tag_obj))?;
        
        let mut fixed_tag = [0u8; 4];
        let len = tag_bytes.len().min(4);
        fixed_tag[..len].copy_from_slice(&tag_bytes[..len]);
        
        Ok(FeatureTag::new(&fixed_tag))
    }
}

impl FromJObject for Feature {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let tag_obj = env.get_field(obj, "tag", "Lcom/dropbear/ui/styling/fonts/FeatureTag;")?.l()?;
        let tag = FeatureTag::from_jobject(env, &tag_obj)?;
        
        let value_obj = env.get_field(obj, "value", "Lcom/dropbear/ui/styling/fonts/UInt;")?.l()?;
        let value = env.get_field(&value_obj, "value", "I")?.i()? as u32;

        Ok(Feature {
            tag,
            value,
        })
    }
}

impl FromJObject for FontFeatures {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let features_list_obj = env.get_field(obj, "features", "Ljava/util/List;")?.l()?;
        
        let size = env.call_method(&features_list_obj, "size", "()I", &[])?.i()?;
        let mut features = Vec::with_capacity(size as usize);
        
        for i in 0..size {
            let item = env.call_method(&features_list_obj, "get", "(I)Ljava/lang/Object;", &[i.into()])?.l()?;
            if !item.is_null() {
                features.push(Feature::from_jobject(env, &item)?);
            }
        }
        
        Ok(FontFeatures { features })
    }
}

impl FromJObject for Color {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let r = env.get_field(obj, "r", "B")?.b()? as u8;
        let g = env.get_field(obj, "g", "B")?.b()? as u8;
        let b = env.get_field(obj, "b", "B")?.b()? as u8;
        let a = env.get_field(obj, "a", "B")?.b()? as u8;
        Ok(Color::rgba(r, g, b, a))
    }
}

impl FromJObject for Pad {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let left = env.get_field(obj, "left", "D")?.d()? as f32;
        let right = env.get_field(obj, "right", "D")?.d()? as f32;
        let top = env.get_field(obj, "top", "D")?.d()? as f32;
        let bottom = env.get_field(obj, "bottom", "D")?.d()? as f32;
        Ok(Pad {
            left,
            right,
            top,
            bottom,
        })
    }
}

impl FromJObject for BorderRadius {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let top_left = env.get_field(obj, "topLeft", "D")?.d()? as f32;
        let top_right = env.get_field(obj, "topRight", "D")?.d()? as f32;
        let bottom_left = env.get_field(obj, "bottomLeft", "D")?.d()? as f32;
        let bottom_right = env.get_field(obj, "bottomRight", "D")?.d()? as f32;
        Ok(BorderRadius {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        })
    }
}

impl FromJObject for Alignment {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let x = env.get_field(obj, "x", "D")?.d()? as f32;
        let y = env.get_field(obj, "y", "D")?.d()? as f32;
        Ok(Alignment::new(x, y))
    }
}

impl FromJObject for TextAlignment {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let name_obj = env.call_method(obj, "name", "()Ljava/lang/String;", &[])?.l()?;
        let name: String = env.get_string(&name_obj.into())?.into();
        match name.as_str() {
            "Start" => Ok(TextAlignment::Start),
            "Center" => Ok(TextAlignment::Center),
            "End" => Ok(TextAlignment::End),
            _ => Ok(TextAlignment::Start),
        }
    }
}

impl FromJObject for Weight {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let val = env.get_field(obj, "value", "I")?.i()? as u16;
        Ok(Weight(val))
    }
}

impl FromJObject for Style {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let name_obj = env.call_method(obj, "name", "()Ljava/lang/String;", &[])?.l()?;
        let name: String = env.get_string(&name_obj.into())?.into();
        match name.as_str() {
            "Normal" => Ok(Style::Normal),
            "Italic" => Ok(Style::Italic),
            "Oblique" => Ok(Style::Oblique),
            _ => Ok(Style::Normal),
        }
    }
}

impl FromJObject for Stretch {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let name_obj = env.call_method(obj, "name", "()Ljava/lang/String;", &[])?.l()?;
        let name: String = env.get_string(&name_obj.into())?.into();
        match name.as_str() {
            "UltraCondensed" => Ok(Stretch::UltraCondensed),
            "ExtraCondensed" => Ok(Stretch::ExtraCondensed),
            "Condensed" => Ok(Stretch::Condensed),
            "SemiCondensed" => Ok(Stretch::SemiCondensed),
            "Normal" => Ok(Stretch::Normal),
            "SemiExpanded" => Ok(Stretch::SemiExpanded),
            "Expanded" => Ok(Stretch::Expanded),
            "ExtraExpanded" => Ok(Stretch::ExtraExpanded),
            "UltraExpanded" => Ok(Stretch::UltraExpanded),
            _ => Ok(Stretch::Normal),
        }
    }
}

impl FromJObject for FamilyOwned {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        if env.is_instance_of(obj, "com/dropbear/ui/styling/fonts/Family$Name")? {
            let value_obj = env.call_method(obj, "getValue", "()Ljava/lang/String;", &[])?.l()?;
            let val: String = env.get_string(&value_obj.into())?.into();
            return Ok(FamilyOwned::Name(val.parse().unwrap()));
        }
        if env.is_instance_of(obj, "com/dropbear/ui/styling/fonts/Family$Serif")? { return Ok(FamilyOwned::Serif); }
        if env.is_instance_of(obj, "com/dropbear/ui/styling/fonts/Family$SansSerif")? { return Ok(FamilyOwned::SansSerif); }
        if env.is_instance_of(obj, "com/dropbear/ui/styling/fonts/Family$Cursive")? { return Ok(FamilyOwned::Cursive); }
        if env.is_instance_of(obj, "com/dropbear/ui/styling/fonts/Family$Fantasy")? { return Ok(FamilyOwned::Fantasy); }
        if env.is_instance_of(obj, "com/dropbear/ui/styling/fonts/Family$Monospace")? { return Ok(FamilyOwned::Monospace); }
        
        Ok(FamilyOwned::SansSerif)
    }
}

impl FromJObject for AttrsOwned {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let family_obj = env.get_field(obj, "family", "Lcom/dropbear/ui/styling/fonts/Family;")?.l()?;
        let family_owned = FamilyOwned::from_jobject(env, &family_obj)?;
        let family = family_owned.as_family();
        
        let color_obj = env.get_field(obj, "colourOptions", "Lcom/dropbear/utils/Colour;")?.l()?;
        let color_opt = if !color_obj.is_null() {
            let colour = Color::from_jobject(env, &color_obj)?;
             Some(yakui::cosmic_text::Color::rgba(colour.r, colour.g, colour.b, colour.a))
        } else {
             None
        };
        
        let stretch_obj = env.get_field(obj, "stretch", "Lcom/dropbear/ui/styling/fonts/Stretch;")?.l()?;
        let stretch = Stretch::from_jobject(env, &stretch_obj)?;
        
        let style_obj = env.get_field(obj, "style", "Lcom/dropbear/ui/styling/fonts/FontStyle;")?.l()?;
        let style = Style::from_jobject(env, &style_obj)?;
        
        let weight_obj = env.get_field(obj, "weight", "Lcom/dropbear/ui/styling/fonts/FontWeight;")?.l()?;
        let weight = Weight::from_jobject(env, &weight_obj)?;
        
        let metadata = match env.get_field(obj, "metadata", "I") {
             Ok(val) => val.i()? as usize,
             Err(_) => 0, 
        };

        let letter_spacing_obj = env.get_field(obj, "letterSpacingOptions", "Ljava/lang/Double;")?.l()?;
        let letter_spacing_opt = if !letter_spacing_obj.is_null() {
             let val = env.call_method(&letter_spacing_obj, "doubleValue", "()D", &[])?.d()?;
             Some(yakui::cosmic_text::LetterSpacing(val as f32))
        } else {
             None
        };

        let font_features_obj = env.get_field(obj, "fontFeatures", "Lcom/dropbear/ui/styling/fonts/FontFeatures;")?.l()?;
        let font_features = if !font_features_obj.is_null() {
            FontFeatures::from_jobject(env, &font_features_obj)?
        } else {
            FontFeatures::default()
        };

        Ok(AttrsOwned::new(&Attrs {
            family,
            stretch,
            style,
            weight,
            metadata,
            color_opt,
            cache_key_flags: CacheKeyFlags::empty(),
            metrics_opt: None,
            letter_spacing_opt,
            font_features,
        }))
    }
}

impl FromJObject for TextStyle {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let align_obj = env.get_field(obj, "align", "Lcom/dropbear/ui/styling/fonts/TextAlignment;")?.l()?;
        let align = TextAlignment::from_jobject(env, &align_obj)?;

        let font_size = env.get_field(obj, "fontSize", "D")?.d()? as f32;
        
        let color_obj = env.get_field(obj, "colour", "Lcom/dropbear/utils/Colour;")?.l()?;
        let color = if !color_obj.is_null() {
            Color::from_jobject(env, &color_obj)?
        } else {
            Color::WHITE
        };

        Ok(TextStyle {
            align,
            font_size,
            color,
            ..TextStyle::default()
        })
    }
}

impl FromJObject for DynamicButtonStyle {
    fn from_jobject(env: &mut JNIEnv, obj: &JObject) -> DropbearNativeResult<Self> {
        let text_obj = env.get_field(obj, "text", "Lcom/dropbear/ui/styling/TextStyle;")?.l()?;
        let text = if !text_obj.is_null() {
            TextStyle::from_jobject(env, &text_obj)?
        } else {
            TextStyle::default()
        };

        let fill_obj = env.get_field(obj, "fill", "Lcom/dropbear/utils/Colour;")?.l()?;
        let fill = if !fill_obj.is_null() {
            Color::from_jobject(env, &fill_obj)?
        } else {
            Color::GRAY
        };

        Ok(DynamicButtonStyle {
            text,
            fill,
            border: None,
        })
    }
}
