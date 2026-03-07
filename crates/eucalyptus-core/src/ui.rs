use std::sync::Arc;
use egui::{CollapsingHeader, Ui};
use hecs::{Entity, World};
use dropbear_engine::graphics::SharedGraphicsContext;
use crate::component::{Component, ComponentDescriptor, ComponentInitFuture, InspectableComponent, SerializedComponent};
use crate::physics::PhysicsState;

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct HUDComponent {
    pub ui_tree: kino_ui::WidgetTree,
}

impl HUDComponent {
    pub fn tree(&self) -> &kino_ui::WidgetTree {
        &self.ui_tree
    }

    pub fn tree_mut(&mut self) -> &mut kino_ui::WidgetTree {
        &mut self.ui_tree
    }
}

#[typetag::serde]
impl SerializedComponent for HUDComponent {}

impl Component for HUDComponent {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self, );

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "eucalyptus_core::ui::HUDComponent".to_string(),
            type_name: "HUD".to_string(),
            category: Some("UI".to_string()),
            description: Some("Renders a camera-facing textured quad, typically used for a HUD or 2D context".to_string()),
        }
    }

    fn init(ser: &'_ Self::SerializedForm, _graphics: Arc<SharedGraphicsContext>) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move { Ok((ser.clone(),)) })
    }

    fn update_component(&mut self, _world: &World, _physics: &mut PhysicsState, _entity: Entity, _dt: f32, _graphics: Arc<SharedGraphicsContext>) {

    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for HUDComponent {
    fn inspect(&mut self, _world: &World, entity: Entity, ui: &mut Ui, _graphics: Arc<SharedGraphicsContext>) {
        CollapsingHeader::new("HUD")
            .default_open(true)
            .id_salt(format!("HUD {}", entity.to_bits()))
            .show(ui, |ui| {
            if ui.button("Edit in UI Editor").clicked() {
                ui.ctx().data_mut(|d| {
                    d.insert_temp::<Option<Entity>>(egui::Id::new("open_ui_editor"), Some(entity));
                });
            }
        });
    }
}