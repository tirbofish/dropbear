use egui::{ComboBox, Ui};
use hecs::Entity;
use eucalyptus_core::physics::collider::{Collider, ColliderShape};
use eucalyptus_core::states::Label;
use crate::editor::{Signal, StaticallyKept, UndoableAction};

impl InspectableComponent for Collider {
    fn inspect(
        &mut self,
        _entity: &mut Entity,
        _cfg: &mut StaticallyKept,
        ui: &mut Ui,
        _undo_stack: &mut Vec<UndoableAction>,
        _signal: &mut Signal,
        label: &mut String
    ) {
        
    }
}