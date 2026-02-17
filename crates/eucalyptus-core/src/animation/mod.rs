use std::sync::Arc;
use egui::{CollapsingHeader, Ui};
use hecs::{Entity, World};
use dropbear_engine::animation::AnimationComponent;
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
            let mut enabled = self.active_animation_index.is_some();
            let mut value = self.active_animation_index.unwrap_or(0);

            ui.horizontal(|ui| {
                ui.label("Active Animation");
                if ui.checkbox(&mut enabled, "Enable").changed() {
                    self.active_animation_index = if enabled { Some(value) } else { None };
                }

                let response = ui.add_enabled(
                    enabled,
                    egui::DragValue::new(&mut value).speed(1.0).range(0..=1_000_000),
                );

                if enabled && response.changed() {
                    self.active_animation_index = Some(value);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Playing");
                ui.checkbox(&mut self.is_playing, "");
            });

            ui.horizontal(|ui| {
                ui.label("Looping");
                ui.checkbox(&mut self.looping, "");
            });

            ui.horizontal(|ui| {
                ui.label("Speed");
                ui.add(egui::DragValue::new(&mut self.speed).speed(0.01).range(0.0..=10.0));
            });

            ui.horizontal(|ui| {
                ui.label("Time");
                ui.add(egui::DragValue::new(&mut self.time).speed(0.01).range(0.0..=1_000_000.0));
                if ui.button("Reset").clicked() {
                    self.time = 0.0;
                }
            });
        });
    }
}