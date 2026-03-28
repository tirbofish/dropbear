use crate::component::{
    Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent,
    SerializedComponent,
};
use crate::physics::PhysicsState;
use dropbear_engine::graphics::SharedGraphicsContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Serialized form of [`EntityStatus`], stored as part of the scene file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SerializedEntityStatus {
    /// When `true` the entity's [`crate::entity::MeshRenderer`] is not submitted to the GPU but
    /// logic (scripts, physics) still runs normally.
    #[serde(default)]
    pub hidden: bool,
    /// When `true` **all** component updates for this entity are suppressed — no rendering,
    /// no physics, no script execution.
    #[serde(default)]
    pub disabled: bool,
}

#[typetag::serde]
impl SerializedComponent for SerializedEntityStatus {}

/// Runtime entity-level visibility / activation flags.
///
/// - **Hidden** — the entity is not rendered but logic still executes.
/// - **Disabled** — the entity is fully inert: no rendering, no physics, no script updates.
///
/// This component is optionally added to any entity. Entities without it behave as
/// if they have `hidden = false, disabled = false`.
#[derive(Debug, Clone)]
pub struct EntityStatus {
    pub hidden: bool,
    pub disabled: bool,
}

impl Component for EntityStatus {
    type SerializedForm = SerializedEntityStatus;
    type RequiredComponentTypes = (Self,);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "eucalyptus_core::entity_status::EntityStatus".to_string(),
            type_name: "EntityStatus".to_string(),
            category: None,
            description: Some("Controls entity visibility and activation".to_string()),
            disabled_flags: DisabilityFlags::Never,
            internal: true,
        }
    }

    fn init(
        ser: &'_ Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self> {
        let hidden = ser.hidden;
        let disabled = ser.disabled;
        Box::pin(async move { Ok((Self { hidden, disabled },)) })
    }

    fn update_component(
        &mut self,
        _world: &hecs::World,
        _physics: &mut PhysicsState,
        _entity: hecs::Entity,
        _dt: f32,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
    }

    fn save(&self, _world: &hecs::World, _entity: hecs::Entity) -> Box<dyn SerializedComponent> {
        Box::new(SerializedEntityStatus {
            hidden: self.hidden,
            disabled: self.disabled,
        })
    }
}

impl InspectableComponent for EntityStatus {
    fn inspect(
        &mut self,
        _world: &hecs::World,
        _entity: hecs::Entity,
        _ui: &mut egui::Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        // nothing...
    }
}
