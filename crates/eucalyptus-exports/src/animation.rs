use hecs::{Entity, World};
use dropbear_engine::animation::{AnimationComponent, AnimationSettings};
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::MeshRenderer;
use eucalyptus_core::ptr::WorldPtr;
use eucalyptus_core::scripting::native::DropbearNativeError;
use eucalyptus_core::scripting::result::DropbearNativeResult;

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "animationComponentExistsForEntity"
    ),
    c
)]
fn exists_for_entity(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    Ok(world.get::<&AnimationComponent>(entity).is_ok())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "getActiveAnimationIndex"
    ),
    c
)]
fn get_active_animation_index(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<Option<i32>> {
    let component = world
        .get::<&AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(component.active_animation_index.map(|index| index as i32))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "setActiveAnimationIndex"
    ),
    c
)]
fn set_active_animation_index(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    index: &Option<i32>,
) -> DropbearNativeResult<()> {
    let mut component = world
        .get::<&mut AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    let index = match index {
        Some(value) if *value >= 0 => Some(*value as usize),
        Some(_) => return Err(DropbearNativeError::InvalidArgument),
        None => None,
    };

    if let Some(value) = index {
        if !component.available_animations.is_empty()
            && value >= component.available_animations.len()
        {
            return Err(DropbearNativeError::InvalidArgument);
        }
    }

    component.active_animation_index = index;
    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "getTime"
    ),
    c
)]
fn get_time(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<f64> {
    let component = world
        .get::<&AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    if let Some(index) = component.active_animation_index {
        if let Some(settings) = component.animation_settings.get(&index) {
            return Ok(settings.time as f64);
        }
    }

    Ok(component.time as f64)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "setTime"
    ),
    c
)]
fn set_time(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    value: f64,
) -> DropbearNativeResult<()> {
    let mut component = world
        .get::<&mut AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    component.time = value as f32;

    if let Some(index) = component.active_animation_index {
        let (time, speed, looping, is_playing) = (
            component.time,
            component.speed,
            component.looping,
            component.is_playing,
        );
        let settings = component
            .animation_settings
            .entry(index)
            .or_insert_with(|| AnimationSettings {
                time,
                speed,
                looping,
                is_playing,
            });
        settings.time = time;
    }

    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "getSpeed"
    ),
    c
)]
fn get_speed(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<f64> {
    let component = world
        .get::<&AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    if let Some(index) = component.active_animation_index {
        if let Some(settings) = component.animation_settings.get(&index) {
            return Ok(settings.speed as f64);
        }
    }

    Ok(component.speed as f64)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "setSpeed"
    ),
    c
)]
fn set_speed(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    value: f64,
) -> DropbearNativeResult<()> {
    let mut component = world
        .get::<&mut AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    component.speed = value as f32;

    if let Some(index) = component.active_animation_index {
        let (time, speed, looping, is_playing) = (
            component.time,
            component.speed,
            component.looping,
            component.is_playing,
        );
        let settings = component
            .animation_settings
            .entry(index)
            .or_insert_with(|| AnimationSettings {
                time,
                speed,
                looping,
                is_playing,
            });
        settings.speed = speed;
    }

    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "getLooping"
    ),
    c
)]
fn get_looping(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    let component = world
        .get::<&AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    if let Some(index) = component.active_animation_index {
        if let Some(settings) = component.animation_settings.get(&index) {
            return Ok(settings.looping);
        }
    }

    Ok(component.looping)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "setLooping"
    ),
    c
)]
fn set_looping(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    value: bool,
) -> DropbearNativeResult<()> {
    let mut component = world
        .get::<&mut AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    component.looping = value;

    if let Some(index) = component.active_animation_index {
        let (time, speed, looping, is_playing) = (
            component.time,
            component.speed,
            component.looping,
            component.is_playing,
        );
        let settings = component
            .animation_settings
            .entry(index)
            .or_insert_with(|| AnimationSettings {
                time,
                speed,
                looping,
                is_playing,
            });
        settings.looping = value;
    }

    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "getIsPlaying"
    ),
    c
)]
fn get_is_playing(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<bool> {
    let component = world
        .get::<&AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    if let Some(index) = component.active_animation_index {
        if let Some(settings) = component.animation_settings.get(&index) {
            return Ok(settings.is_playing);
        }
    }

    Ok(component.is_playing)
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "setIsPlaying"
    ),
    c
)]
fn set_is_playing(
    #[dropbear_macro::define(WorldPtr)] world: &mut World,
    #[dropbear_macro::entity] entity: Entity,
    value: bool,
) -> DropbearNativeResult<()> {
    let mut component = world
        .get::<&mut AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    component.is_playing = value;

    if let Some(index) = component.active_animation_index {
        let (time, speed, looping, is_playing) = (
            component.time,
            component.speed,
            component.looping,
            component.is_playing,
        );
        let settings = component
            .animation_settings
            .entry(index)
            .or_insert_with(|| AnimationSettings {
                time,
                speed,
                looping,
                is_playing,
            });
        settings.is_playing = value;
    }

    Ok(())
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "getIndexFromString"
    ),
    c
)]
fn get_index_from_string(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
    name: String,
) -> DropbearNativeResult<Option<i32>> {
    let component = world
        .get::<&AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;

    Ok(component
        .available_animations
        .iter()
        .enumerate()
        .find_map(|(i, l)| if *l == name { Some(i as i32) } else { None }))
}

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "getAvailableAnimations"
    ),
    c
)]
fn get_available_animations(
    #[dropbear_macro::define(WorldPtr)] world: &World,
    #[dropbear_macro::entity] entity: Entity,
) -> DropbearNativeResult<Vec<String>> {
    let component = world
        .get::<&AnimationComponent>(entity)
        .map_err(|_| DropbearNativeError::MissingComponent)?;
    Ok(collect_available_animations(world, entity, &component))
}

// ---------------------- helpers ----------------------

fn collect_available_animations(
    world: &World,
    entity: Entity,
    component: &AnimationComponent,
) -> Vec<String> {
    if !component.available_animations.is_empty() {
        return component.available_animations.clone();
    }

    let Ok(renderer) = world.get::<&MeshRenderer>(entity) else {
        return Vec::new();
    };

    let handle = renderer.model();
    if handle.is_null() {
        return Vec::new();
    }

    let registry = ASSET_REGISTRY.read();
    let Some(model) = registry.get_model(handle) else {
        return Vec::new();
    };

    model
        .animations
        .iter()
        .map(|animation| animation.name.clone())
        .collect()
}
