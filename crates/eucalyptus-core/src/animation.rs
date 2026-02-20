use crate::component::{Component, ComponentDescriptor, InspectableComponent, SerializedComponent};
use crate::ptr::WorldPtr;
use crate::scripting::native::DropbearNativeError;
use crate::scripting::result::DropbearNativeResult;
use dropbear_engine::animation::{AnimationComponent, AnimationSettings};
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{CollapsingHeader, ComboBox, Ui};
use hecs::{Entity, World};
use std::sync::Arc;

#[typetag::serde]
impl SerializedComponent for AnimationComponent {}

impl Component for AnimationComponent {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "dropbear_engine::animation::AnimationComponent".to_string(),
            type_name: "AnimationComponent".to_string(),
            category: Some("Animation".to_string()),
            description: Some("Animates a 3D MeshRenderer".to_string()),
        }
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> crate::component::ComponentInitFuture<'a, Self> {
        Box::pin(async move { Ok((ser.clone(),)) })
    }

    fn update_component(
        &mut self,
        world: &World,
        _physics: &mut crate::physics::PhysicsState,
        entity: Entity,
        dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        let Ok(renderer) = world.get::<&MeshRenderer>(entity) else {
            return;
        };

        let handle = renderer.model();
        if handle.is_null() {
            return;
        }

        let registry = ASSET_REGISTRY.read();
        let Some(model) = registry.get_model(handle) else {
            return;
        };

        self.update(dt, model);

        self.prepare_gpu_resources(graphics.clone());
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for AnimationComponent {
    fn inspect(&mut self, ui: &mut Ui, _graphics: Arc<SharedGraphicsContext>) {
        CollapsingHeader::new("Animation")
            .default_open(true)
            .show(ui, |ui| {
                let has_animations = !self.available_animations.is_empty();
                let mut enabled = self.active_animation_index.is_some() && has_animations;

                let mut selected_index = self
                    .active_animation_index
                    .unwrap_or(0)
                    .min(self.available_animations.len().saturating_sub(1));

                let selected_label = if has_animations {
                    self.available_animations
                        .get(selected_index)
                        .map(String::as_str)
                        .unwrap_or("Unnamed Animation")
                } else {
                    "No Animations"
                };

                let mut selection_changed = false;
                ComboBox::from_label("Animation")
                    .selected_text(selected_label)
                    .show_ui(ui, |ui| {
                        for (index, name) in self.available_animations.iter().enumerate() {
                            if ui
                                .selectable_value(&mut selected_index, index, name)
                                .changed()
                            {
                                selection_changed = true;
                            }
                        }
                    });

                if selection_changed && has_animations {
                    self.active_animation_index = Some(selected_index);
                    enabled = true;
                }

                ui.horizontal(|ui| {
                    ui.label("Active");

                    if ui.checkbox(&mut enabled, "Enable").changed() {
                        self.active_animation_index = if enabled && has_animations {
                            Some(selected_index)
                        } else {
                            None
                        };
                    }
                });

                if has_animations {
                    let settings = self
                        .animation_settings
                        .entry(selected_index)
                        .or_insert_with(|| AnimationSettings {
                            time: self.time,
                            speed: self.speed,
                            looping: self.looping,
                            is_playing: self.is_playing,
                        });

                    ui.horizontal(|ui| {
                        ui.label("Playing");
                        ui.checkbox(&mut settings.is_playing, "");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Looping");
                        ui.checkbox(&mut settings.looping, "");
                    });

                    ui.horizontal(|ui| {
                        ui.label("Speed");
                        ui.add(
                            egui::DragValue::new(&mut settings.speed)
                                .speed(0.01)
                                .range(0.0..=10.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Start Time");
                        ui.add(
                            egui::DragValue::new(&mut settings.time)
                                .speed(0.01)
                                .range(0.0..=1_000_000.0),
                        );
                        if ui.button("Reset").clicked() {
                            settings.time = 0.0;
                        }
                    });

                    if self.active_animation_index == Some(selected_index) {
                        self.time = settings.time;
                        self.speed = settings.speed;
                        self.looping = settings.looping;
                        self.is_playing = settings.is_playing;
                    }
                }
            });
    }
}

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

#[dropbear_macro::export(
    kotlin(
        class = "com.dropbear.animation.AnimationComponentNative",
        func = "animationComponentExistsForEntity"
    ),
    c
)]
fn animation_component_exists_for_entity(
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
