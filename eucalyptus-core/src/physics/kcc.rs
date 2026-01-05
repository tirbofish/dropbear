//! Module that relates to the [Kinematic Character Controller](https://rapier.rs/docs/user_guides/rust/character_controller)
//! (or kcc for short) in the [rapier3d] physics engine.

use crate::traits::SerializableComponent;
use rapier3d::control::KinematicCharacterController;
use serde::{Deserialize, Serialize};
use dropbear_macro::SerializableComponent;
use crate::states::Label;

/// The kinematic character controller (kcc) component.
#[derive(Debug, Default, Serialize, Deserialize, Clone, SerializableComponent)]
pub struct KCC {
    pub entity: Label,
    pub controller: KinematicCharacterController,
}

impl KCC {
    pub fn new(label: &Label) -> Self {
        KCC {
            entity: label.clone(),
            controller: KinematicCharacterController::default(),
        }
    }
}