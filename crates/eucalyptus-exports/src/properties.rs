use eucalyptus_core::properties::{CustomProperties, Value};
use eucalyptus_core::ptr::WorldPtr;
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;
use eucalyptus_core::types::NVector3;
use hecs::{Entity, World};

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "customPropertiesExistsForEntity"
    ),
    c
)]
fn custom_properties_exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&CustomProperties>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "getStringProperty"
    ),
    c
)]
fn get_string_property(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
) -> DropbearNativeResult<Option<String>> {
    let props = world
        .get::<&CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    Ok(props.get_property(&key).and_then(|value| match value {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "getIntProperty"
    ),
    c
)]
fn get_int_property(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
) -> DropbearNativeResult<Option<i32>> {
    let props = world
        .get::<&CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    Ok(props.get_property(&key).and_then(|value| match value {
        Value::Int(v) => Some(*v as i32),
        _ => None,
    }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "getLongProperty"
    ),
    c
)]
fn get_long_property(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
) -> DropbearNativeResult<Option<i64>> {
    let props = world
        .get::<&CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    Ok(props.get_property(&key).and_then(|value| match value {
        Value::Int(v) => Some(*v),
        _ => None,
    }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "getDoubleProperty"
    ),
    c
)]
fn get_double_property(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
) -> DropbearNativeResult<Option<f64>> {
    let props = world
        .get::<&CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    Ok(props.get_property(&key).and_then(|value| match value {
        Value::Double(v) => Some(*v),
        _ => None,
    }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "getFloatProperty"
    ),
    c
)]
fn get_float_property(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
) -> DropbearNativeResult<Option<f32>> {
    let props = world
        .get::<&CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    Ok(props.get_property(&key).and_then(|value| match value {
        Value::Double(v) => Some(*v as f32),
        _ => None,
    }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "getBoolProperty"
    ),
    c
)]
fn get_bool_property(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
) -> DropbearNativeResult<Option<bool>> {
    let props = world
        .get::<&CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    Ok(props.get_property(&key).and_then(|value| match value {
        Value::Bool(v) => Some(*v),
        _ => None,
    }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "getVec3Property"
    ),
    c
)]
fn get_vec3_property(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
) -> DropbearNativeResult<Option<NVector3>> {
    let props = world
        .get::<&CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;

    Ok(props.get_property(&key).and_then(|value| match value {
        Value::Vec3(v) => Some(NVector3::from(*v)),
        _ => None,
    }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "setStringProperty"
    ),
    c
)]
fn set_string_property(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
    value: String,
) -> DropbearNativeResult<()> {
    let mut props = world
        .get::<&mut CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    props.set_property(key, Value::String(value));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "setIntProperty"
    ),
    c
)]
fn set_int_property(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
    value: i32,
) -> DropbearNativeResult<()> {
    let mut props = world
        .get::<&mut CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    props.set_property(key, Value::Int(value as i64));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "setLongProperty"
    ),
    c
)]
fn set_long_property(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
    value: i64,
) -> DropbearNativeResult<()> {
    let mut props = world
        .get::<&mut CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    props.set_property(key, Value::Int(value));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "setDoubleProperty"
    ),
    c
)]
fn set_double_property(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
    value: f64,
) -> DropbearNativeResult<()> {
    let mut props = world
        .get::<&mut CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    props.set_property(key, Value::Double(value));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "setFloatProperty"
    ),
    c
)]
fn set_float_property(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
    value: f64,
) -> DropbearNativeResult<()> {
    let mut props = world
        .get::<&mut CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    props.set_property(key, Value::Double(value));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "setBoolProperty"
    ),
    c
)]
fn set_bool_property(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
    value: bool,
) -> DropbearNativeResult<()> {
    let mut props = world
        .get::<&mut CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    props.set_property(key, Value::Bool(value));
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.components.CustomPropertiesNative",
        func = "setVec3Property"
    ),
    c
)]
fn set_vec3_property(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    key: String,
    value: &NVector3,
) -> DropbearNativeResult<()> {
    let mut props = world
        .get::<&mut CustomProperties>(entity)
        .map_err(|_| DropbearNativeError::NoSuchComponent)?;
    props.set_property(key, Value::Vec3(value.to_array()));
    Ok(())
}
