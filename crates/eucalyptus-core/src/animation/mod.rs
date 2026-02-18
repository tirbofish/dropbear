use std::sync::Arc;
use egui::{CollapsingHeader, ComboBox, Ui};
use hecs::{Entity, World};
use dropbear_engine::animation::{AnimationComponent, AnimationSettings};
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::graphics::SharedGraphicsContext;
use crate::component::{Component, ComponentDescriptor, InspectableComponent, SerializedComponent};

#[typetag::serde]
impl SerializedComponent for AnimationComponent {}

impl Component for AnimationComponent {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self, );

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
        Box::pin(async move { Ok((ser.clone(), )) })
    }

    fn update_component(&mut self, world: &World, _physics: &mut crate::physics::PhysicsState, entity: Entity, dt: f32, graphics: Arc<SharedGraphicsContext>) {
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
        CollapsingHeader::new("Animation").default_open(true).show(ui, |ui| {
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

            ComboBox::from_label("Animation")
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for (index, name) in self.available_animations.iter().enumerate() {
                        ui.selectable_value(&mut selected_index, index, name);
                    }
                });

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
                    ui.add(egui::DragValue::new(&mut settings.speed).speed(0.01).range(0.0..=10.0));
                });

                ui.horizontal(|ui| {
                    ui.label("Start Time");
                    ui.add(egui::DragValue::new(&mut settings.time).speed(0.01).range(0.0..=1_000_000.0));
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