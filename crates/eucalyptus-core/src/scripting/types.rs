use crate::component::{
    Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent,
    SerializedComponent,
};
use crate::physics::PhysicsState;
use dropbear_engine::graphics::SharedGraphicsContext;
use egui::{CollapsingHeader, Ui};
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Single [`hecs`] component that represents all Kotlin components on this entity.

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KotlinComponents {
    /// A list of `fully qualified class names`.
    pub fqcns: Vec<String>,
}

#[typetag::serde]
impl SerializedComponent for KotlinComponents {}

impl KotlinComponents {
    pub fn attach(&mut self, fqcn: &str) {
        if !self.fqcns.iter().any(|f| f == fqcn) {
            self.fqcns.push(fqcn.to_owned());
        }
    }

    pub fn detach(&mut self, fqcn: &str) {
        self.fqcns.retain(|f| f != fqcn);
    }

    pub fn has(&self, fqcn: &str) -> bool {
        self.fqcns.iter().any(|f| f == fqcn)
    }
}

impl Component for KotlinComponents {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "eucalyptus_core::scripting::types::KotlinComponents".to_string(),
            type_name: "KotlinComponents".to_string(),
            category: None,
            description: Some(
                "Tracks all Kotlin-defined components attached to this entity.".to_string(),
            ),
            disabled_flags: DisabilityFlags::Never,
            internal: true,
        }
    }

    fn init(
        ser: &'_ Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self> {
        let cloned = ser.clone();
        Box::pin(async move { Ok((cloned,)) })
    }

    fn update_component(
        &mut self,
        _world: &World,
        _physics: &mut PhysicsState,
        _entity: Entity,
        _dt: f32,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for KotlinComponents {
    fn inspect(
        &mut self,
        _world: &World,
        _entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Kotlin Components")
            .default_open(true)
            .show(ui, |ui| {
                for fqcn in &self.fqcns {
                    ui.label(fqcn);
                }

                ui.separator();

                ui.label(
                    "This should not be visible in the editor. this is considered `internal=true`",
                );
            });
    }
}
