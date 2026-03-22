use crate::editor::{EditorTabDock, EditorTabDockDescriptor, EditorTabViewer, Signal, TABS_GLOBAL, UndoableAction};
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, Transform};
use dropbear_engine::lighting::Light;
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::utils::ViewportMode;
use glam::DVec3;
use transform_gizmo_egui::{GizmoConfig, GizmoExt, GizmoOrientation};
use eucalyptus_core::hierarchy::EntityTransformExt;
use crate::editor::page::EditorTabVisibility;

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn viewport_tab(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();

        log_once::debug_once!("Viewport focused");

        let available_rect = ui.available_rect_before_wrap();
        let available_size = available_rect.size();
        let pixels_per_point = ui.ctx().pixels_per_point();

        let desired_width = (available_size.x * pixels_per_point).max(1.0).round() as u32;
        let desired_height = (available_size.y * pixels_per_point).max(1.0).round() as u32;
        if self.tex_size.width != desired_width || self.tex_size.height != desired_height {
            if self.signal.is_empty() {
                self.signal.push_back(Signal::UpdateViewportSize((desired_width as f32, desired_height as f32)));
            }
        }

        let tex_aspect = self.tex_size.width as f32 / self.tex_size.height as f32;
        let available_aspect = available_size.x / available_size.y;

        let (display_width, display_height) = if available_aspect > tex_aspect {
            let height = available_size.y;
            let width = height * tex_aspect;
            (width, height)
        } else {
            let width = available_size.x;
            let height = width / tex_aspect;
            (width, height)
        };

        let center_x = available_rect.center().x;
        let center_y = available_rect.center().y;

        let image_rect = egui::Rect::from_center_size(
            egui::pos2(center_x, center_y),
            egui::vec2(display_width, display_height),
        );

        let (_rect, _response) =
            ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());

        let _image_response = ui.allocate_rect(image_rect, egui::Sense::click_and_drag());

        ui.scope_builder(egui::UiBuilder::new().max_rect(image_rect), |ui| {
            ui.add_sized(
                [display_width, display_height],
                egui::Image::new((self.view, [display_width, display_height].into()))
                    .fit_to_exact_size([display_width, display_height].into()),
            )
        });

        let snapping = ui.input(|input| input.modifiers.shift);

        let active_cam = self.active_camera.lock();
        if let Some(active_camera) = *active_cam {
            let camera_data = {
                if let Ok((cam, _comp)) = self
                    .world
                    .query_one::<(&Camera, &CameraComponent)>(active_camera)
                    .get()
                {
                    Some(cam.clone())
                } else {
                    log::warn!("Queried camera but found no value");
                    None
                }
            };

            if let Some(camera) = camera_data {
                self.gizmo.update_config(GizmoConfig {
                    view_matrix: camera.view_mat.into(),
                    projection_matrix: camera.proj_mat.into(),
                    viewport: image_rect,
                    modes: *self.gizmo_mode,
                    orientation: *self.gizmo_orientation,
                    snapping,
                    snap_distance: 1.0,
                    ..Default::default()
                });
            }
        }

        if !matches!(self.viewport_mode, ViewportMode::None)
            && let Some(entity_id) = self.selected_entity
        {
            let mut handled = false;
            let mut updated_light_transform: Option<Transform> = None;

            if let Ok(mut entity_transform) = self
                .world
                .get::<&mut EntityTransform>(*entity_id)
            {
                let was_focused = cfg.is_focused;
                cfg.is_focused = self.gizmo.is_focused();

                if cfg.is_focused && !was_focused {
                    cfg.entity_transform_original = Some(*entity_transform);
                }

                let synced = entity_transform.propagate(&self.world, *entity_id);
                let gizmo_transform =
                    transform_gizmo_egui::math::Transform::from_scale_rotation_translation(
                        synced.scale,
                        synced.rotation,
                        synced.position,
                    );

                if let Some((_result, new_transforms)) = self.gizmo.interact(ui, &[gizmo_transform])
                    && let Some(new_transform) = new_transforms.first()
                {
                    let new_synced_pos: glam::DVec3 = new_transform.translation.into();
                    let new_synced_rot: glam::DQuat = new_transform.rotation.into();
                    let new_synced_scale: glam::DVec3 = new_transform.scale.into();

                    let prev_world_pos = synced.position;
                    let prev_world_rot = synced.rotation;
                    let prev_world_scale = synced.scale;

                    let safe = |v: f64| if v.abs() < 1e-9 { 1.0 } else { v };

                    let delta_pos   = new_synced_pos - prev_world_pos;
                    let delta_rot   = new_synced_rot * prev_world_rot.inverse();
                    let delta_scale = glam::DVec3::new(
                        new_synced_scale.x / safe(prev_world_scale.x),
                        new_synced_scale.y / safe(prev_world_scale.y),
                        new_synced_scale.z / safe(prev_world_scale.z),
                    );

                    match *self.gizmo_orientation {
                        GizmoOrientation::Global => {
                            let world = entity_transform.world_mut();
                            world.position += delta_pos;
                            world.rotation  = delta_rot * world.rotation;
                            world.scale    *= delta_scale;
                        }
                        GizmoOrientation::Local => {
                            let world_rot   = entity_transform.world().rotation;
                            let world_scale = entity_transform.world().scale;

                            let safe_ws = glam::DVec3::new(
                                safe(world_scale.x),
                                safe(world_scale.y),
                                safe(world_scale.z),
                            );

                            let local = entity_transform.local_mut();

                            local.position += world_rot.inverse() * delta_pos / safe_ws;

                            local.rotation  = world_rot.inverse() * delta_rot * world_rot * local.rotation;

                            local.scale    *= delta_scale;
                        }
                    }

                    updated_light_transform = Some(entity_transform.sync());
                }

                if was_focused && !cfg.is_focused {
                    if let Some(original) = cfg.entity_transform_original {
                        if original != *entity_transform {
                            self.undo_stack.push(UndoableAction::EntityTransform(*entity_id, original));
                            log::debug!("Pushed entity transform action to stack");
                        }
                    }
                }

                handled = true;
            }

            if !handled {
                if let Ok(transform) = self.world.query_one::<&mut Transform>(*entity_id).get() {
                    let was_focused = cfg.is_focused;
                    cfg.is_focused = self.gizmo.is_focused();

                    if cfg.is_focused && !was_focused {
                        cfg.old_pos = *transform;
                    }

                    let gizmo_transform =
                        transform_gizmo_egui::math::Transform::from_scale_rotation_translation(
                            transform.scale,
                            transform.rotation,
                            transform.position,
                        );

                    if let Some((_result, new_transforms)) =
                        self.gizmo.interact(ui, &[gizmo_transform])
                        && let Some(new_transform) = new_transforms.first()
                    {
                        transform.position = new_transform.translation.into();
                        transform.rotation = new_transform.rotation.into();
                        transform.scale    = new_transform.scale.into();
                        updated_light_transform = Some(*transform);
                    }

                    if was_focused && !cfg.is_focused {
                        let transform_changed = cfg.old_pos.position != transform.position
                            || cfg.old_pos.rotation != transform.rotation
                            || cfg.old_pos.scale    != transform.scale;

                        if transform_changed {
                            self.undo_stack.push(
                                UndoableAction::Transform(*entity_id, cfg.old_pos)
                            );
                            log::debug!("Pushed transform action to stack");
                        }
                    }
                }
            }

            if let Some(updated_transform) = updated_light_transform {
                if let Ok(mut light) = self.world.get::<&mut Light>(*entity_id) {
                    let forward = DVec3::new(0.0, -1.0, 0.0);
                    light.component.position  = updated_transform.position;
                    light.component.direction =
                        (updated_transform.rotation * forward).normalize_or_zero();
                }
            }
        }
    }
}

pub struct ViewportDock;

impl EditorTabDock for ViewportDock {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            id: "viewport",
            title: "Viewport".to_string(),
            visibility: EditorTabVisibility::GameEditor,
        }
    }

    fn display(viewer: &mut EditorTabViewer<'_>, ui: &mut egui::Ui) {
        viewer.viewport_tab(ui);
    }
}