use std::sync::Arc;
use hecs::{Entity, World};
use dropbear_engine::entity::inspect_rotation_quat;
use dropbear_engine::graphics::SharedGraphicsContext;
use crate::component::{Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent, SerializedComponent};
use crate::physics::PhysicsState;
use egui::{CollapsingHeader, Ui};

fn default_world_size() -> glam::Vec2 {
    glam::Vec2::ONE
}

fn default_enabled() -> bool {
    true
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BillboardComponent {
    #[serde(default)]
    pub offset: glam::Vec3,
    #[serde(default)]
    pub rotation: Option<glam::Quat>, // if None, billboard is facing the camera
    #[serde(default = "default_world_size")]
    pub world_size: glam::Vec2,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub ui_tree: kino_ui::WidgetTree,
}

impl Default for BillboardComponent {
    fn default() -> Self {
        Self {
            offset: glam::Vec3::ZERO,
            rotation: None,
            world_size: glam::Vec2::ONE,
            enabled: true,
            ui_tree: Default::default(),
        }
    }
}

#[typetag::serde]
impl SerializedComponent for BillboardComponent {}

impl Component for BillboardComponent {
    type SerializedForm = Self;
    type RequiredComponentTypes = (Self, );

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
            fqtn: "eucalyptus_core::billboard::BillboardComponent".to_string(),
            type_name: "Billboard".to_string(),
            category: Some("UI".to_string()),
            description: Some("Renders a camera-facing textured quad".to_string()),
        }
    } 

    fn init(ser: &'_ Self::SerializedForm, _graphics: Arc<SharedGraphicsContext>) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move {
            Ok((ser.clone(),))
        })
    }

    fn update_component(&mut self, _world: &World, _physics: &mut PhysicsState, _entity: Entity, _dt: f32, _graphics: Arc<SharedGraphicsContext>) {
    }

    fn save(&self, _world: &World, _entity: Entity) -> Box<dyn SerializedComponent> {
        Box::new(self.clone())
    }
}

impl InspectableComponent for BillboardComponent {
    fn inspect(
        &mut self,
        _world: &World,
        entity: Entity,
        ui: &mut Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        CollapsingHeader::new("Billboard")
            .default_open(true)
            .id_salt(format!("Billboard {}", entity.to_bits()))
            .show(ui, |ui| {
                if ui.button("Edit in UI Editor").clicked() {
                    ui.ctx().data_mut(|d| {
                        d.insert_temp::<Option<Entity>>(egui::Id::new("open_ui_editor"), Some(entity));
                    });
                }

                ui.checkbox(&mut self.enabled, "Enabled");

                ui.horizontal(|ui| {
                    ui.label("Size");
                    ui.add(egui::DragValue::new(&mut self.world_size.x).speed(0.01));
                    ui.add(egui::DragValue::new(&mut self.world_size.y).speed(0.01));
                });
                self.world_size.x = self.world_size.x.max(0.001);
                self.world_size.y = self.world_size.y.max(0.001);

                ui.horizontal(|ui| {
                    ui.label("Offset");
                    ui.add(egui::DragValue::new(&mut self.offset.x).speed(0.01));
                    ui.add(egui::DragValue::new(&mut self.offset.y).speed(0.01));
                    ui.add(egui::DragValue::new(&mut self.offset.z).speed(0.01));
                });

                let mut face_camera = self.rotation.is_none();
                if ui.checkbox(&mut face_camera, "Face Camera").changed() {
                    if face_camera {
                        self.rotation = None;
                    } else {
                        self.rotation = Some(glam::Quat::IDENTITY);
                    }
                }

                if let Some(rotation) = &mut self.rotation {
                    ui.label("Rotation");
                    let _ = inspect_rotation_quat(
                        ui,
                        ("billboard_rotation", entity.to_bits()),
                        rotation,
                    );
                }

            });
    }
}