use std::sync::Arc;
use eucalyptus_core::component::{
    Component, ComponentDescriptor, ComponentInitFuture, DisabilityFlags, InspectableComponent,
    SerializedComponent,
};
use eucalyptus_core::component::{Serialize, Deserialize};
use eucalyptus_core::engine::graphics::SharedGraphicsContext;
use eucalyptus_core::physics::PhysicsState;
use eucalyptus_core::third_party::{egui, hecs};
use dropbear_engine::asset::{ASSET_REGISTRY, Handle};
use dropbear_engine::entity::MeshRenderer;
use dropbear_engine::model::ModelVertex;
use dropbear_engine::procedural::{ProcObjType, ProcedurallyGeneratedObject};
use ndshape::RuntimeShape;
use crate::surface_nets::{SurfaceNetsBuffer, surface_nets as compute_surface_nets};

/// Persistent configuration for a [`SurfaceNets`] component, stored in scene files.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SerializedSurfaceNets {
    /// Number of voxels along each axis, including the required 1-voxel boundary padding on each side.
    pub resolution: [u32; 3],
    /// The isosurface threshold; voxels with SDF values below this level are considered inside the surface.
    pub iso_level: f32,
}

#[typetag::serde]
impl SerializedComponent for SerializedSurfaceNets {}

/// An ECS component that generates and maintains a Surface Nets isosurface mesh from a signed
/// distance field.
///
/// On every frame where [`dirty`] is `true`, the component recomputes the isosurface, converts
/// it to a [`MeshRenderer`]-compatible procedural mesh, and uploads it to the GPU.
///
/// To update the surface shape at runtime: write your new SDF values into [`sdf`], then set
/// [`dirty = true`].
pub struct SurfaceNets {
    /// The isosurface threshold — voxels with SDF < iso_level are inside the surface.
    pub iso_level: f32,
    /// Per-axis voxel grid resolution, including the required 1-voxel boundary padding.
    pub resolution: [u32; 3],
    /// The signed distance field, laid out in row-major (x-major) order.
    /// Length must equal `resolution[0] * resolution[1] * resolution[2]`.
    pub sdf: Vec<f32>,
    /// Pre-allocated output buffers reused each time the mesh is regenerated.
    pub buffer: SurfaceNetsBuffer,
    /// Set to `true` whenever [`sdf`] changes and the mesh must be rebuilt.
    pub dirty: bool,
}

impl SurfaceNets {
    /// Writes a default sphere SDF into `self.sdf` for the current `resolution` and marks the
    /// component dirty so it rebuilds on the next frame.
    pub fn reset_to_sphere(&mut self) {
        let [rx, ry, rz] = self.resolution;
        let cx = rx as f32 / 2.0;
        let cy = ry as f32 / 2.0;
        let cz = rz as f32 / 2.0;
        let radius = (cx.min(cy).min(cz) - 2.0).max(1.0);
        let total = (rx * ry * rz) as usize;

        self.sdf.resize(total, 0.0);
        for i in 0..total as u32 {
            let z = i / (rx * ry);
            let rem = i % (rx * ry);
            let y = rem / rx;
            let x = rem % rx;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dz = z as f32 - cz;
            self.sdf[i as usize] = (dx * dx + dy * dy + dz * dz).sqrt() - radius;
        }
        self.dirty = true;
    }
}

impl Component for SurfaceNets {
    type SerializedForm = SerializedSurfaceNets;
    /// Both [`SurfaceNets`] and a [`MeshRenderer`] are inserted at spawn time so that
    /// `update_component` can drive the renderer without needing `&mut World`.
    type RequiredComponentTypes = (Self, MeshRenderer);

    fn descriptor() -> ComponentDescriptor {
        ComponentDescriptor {
            fqtn: "surface_nets_plugin::component::SurfaceNets".to_string(),
            type_name: "SurfaceNets".to_string(),
            category: Some("Rendering".to_string()),
            description: Some(
                "Generates an isosurface mesh from a signed distance field \
                 using the Surface Nets algorithm."
                    .to_string(),
            ),
            disabled_flags: DisabilityFlags::Disabled,
            internal: false,
        }
    }

    fn init(
        ser: &'_ Self::SerializedForm,
        _graphics: Arc<SharedGraphicsContext>,
    ) -> ComponentInitFuture<'_, Self> {
        Box::pin(async move {
            let mut component = SurfaceNets {
                iso_level: ser.iso_level,
                resolution: ser.resolution,
                sdf: Vec::new(),
                buffer: SurfaceNetsBuffer::default(),
                dirty: true,
            };
            component.reset_to_sphere();
            Ok((component, MeshRenderer::from_handle(Handle::NULL)))
        })
    }

    fn update_component(
        &mut self,
        world: &hecs::World,
        _physics: &mut PhysicsState,
        entity: hecs::Entity,
        _dt: f32,
        graphics: Arc<SharedGraphicsContext>,
    ) {
        if !self.dirty {
            return;
        }

        let [rx, ry, rz] = self.resolution;
        let expected_len = (rx * ry * rz) as usize;
        if self.sdf.len() != expected_len || expected_len == 0 {
            return;
        }

        let shape = RuntimeShape::<u32, 3>::new([rx, ry, rz]);
        compute_surface_nets(
            &self.sdf,
            &shape,
            [0; 3],
            [rx.saturating_sub(1), ry.saturating_sub(1), rz.saturating_sub(1)],
            &mut self.buffer,
        );

        if !self.buffer.indices.is_empty() {
            let vertices: Vec<ModelVertex> = self
                .buffer
                .positions
                .iter()
                .zip(self.buffer.normals.iter())
                .map(|(pos, norm)| ModelVertex {
                    position: *pos,
                    normal: *norm,
                    tangent: [1.0, 0.0, 0.0, 1.0],
                    tex_coords0: [0.0, 0.0],
                    tex_coords1: [0.0, 0.0],
                    colour0: [1.0, 1.0, 1.0, 1.0],
                    joints0: [0, 0, 0, 0],
                    weights0: [1.0, 0.0, 0.0, 0.0],
                })
                .collect();

            let proc_obj = ProcedurallyGeneratedObject {
                vertices,
                indices: self.buffer.indices.clone(),
                ty: ProcObjType::Cuboid,
            };

            let handle = proc_obj.build_model(graphics, None, None, ASSET_REGISTRY.clone());

            if let Ok(mut renderer) = world.get::<&mut MeshRenderer>(entity) {
                renderer.set_model(handle);
            }
        }

        self.dirty = false;
    }

    fn save(&self, _world: &hecs::World, _entity: hecs::Entity) -> Box<dyn SerializedComponent> {
        Box::new(SerializedSurfaceNets {
            resolution: self.resolution,
            iso_level: self.iso_level,
        })
    }
}

impl InspectableComponent for SurfaceNets {
    fn inspect(
        &mut self,
        _world: &hecs::World,
        entity: hecs::Entity,
        ui: &mut egui::Ui,
        _graphics: Arc<SharedGraphicsContext>,
    ) {
        egui::CollapsingHeader::new("Surface Nets")
            .default_open(true)
            .id_salt(format!("surface_nets_{}", entity.to_bits()))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Iso Level:");
                    if ui
                        .add(egui::DragValue::new(&mut self.iso_level).speed(0.01))
                        .changed()
                    {
                        self.dirty = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Resolution:");
                    let [ref mut rx, ref mut ry, ref mut rz] = self.resolution;
                    let mut changed = false;
                    changed |= ui.add(egui::DragValue::new(rx).speed(1.0)).changed();
                    changed |= ui.add(egui::DragValue::new(ry).speed(1.0)).changed();
                    changed |= ui.add(egui::DragValue::new(rz).speed(1.0)).changed();
                    if changed {
                        self.reset_to_sphere();
                    }
                });

                if ui.button("Rebuild Mesh").clicked() {
                    self.dirty = true;
                }

                if ui.button("Reset to Sphere").clicked() {
                    self.reset_to_sphere();
                }
            });
    }
}