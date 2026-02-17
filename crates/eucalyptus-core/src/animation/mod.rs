use std::sync::Arc;
use egui::{CollapsingHeader, Ui};
use hecs::{Entity, World};
use dropbear_engine::animation::AnimationComponent;
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::graphics::SharedGraphicsContext;
use crate::component::{Component, ComponentDescriptor, SerializedComponent};

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

    async fn first_time(_graphics: Arc<SharedGraphicsContext>) -> anyhow::Result<Self::RequiredComponentTypes>
    where
        Self: Sized
    {
        Ok((Self::default(), ))
    }

    fn init<'a>(
        ser: &'a Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> crate::component::ComponentInitFuture<'a, Self> {
        Box::pin(async move { Ok((ser.clone(), )) })
    }

    fn update_component(&mut self, world: &World, entity: Entity, dt: f32, graphics: Arc<SharedGraphicsContext>) {
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

    fn inspect(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Animation").default_open(true).show(ui, |ui| {
            ui.label("Active animation:");
            let mut enabled = self.active_animation_index.is_some();
            let mut value = self.active_animation_index.unwrap_or(0);

            ui.horizontal(|ui| {
                if ui.checkbox(&mut enabled, "Enable").changed() {
                    self.active_animation_index = if enabled { Some(value) } else { None };
                }

                ui.add_enabled(enabled, egui::DragValue::new(&mut value));

                if enabled && self.active_animation_index != Some(value) {
                    self.active_animation_index = Some(value);
                }
            });

            ui.label("Not implemented yet!");
            // todo: complete some more
        });
    }
}