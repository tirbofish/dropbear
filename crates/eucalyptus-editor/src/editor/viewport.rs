use transform_gizmo_egui::{GizmoConfig, GizmoExt, GizmoOrientation};
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, Transform};
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::utils::ViewportMode;
use crate::editor::{EditorTabViewer, Signal, TABS_GLOBAL, UndoableAction};

impl<'a> EditorTabViewer<'a> {
    pub(crate) fn viewport_tab(&mut self, ui: &mut egui::Ui) {
        let mut cfg = TABS_GLOBAL.lock();
        
        log_once::debug_once!("Viewport focused");

        let available_rect = ui.available_rect_before_wrap();
        let available_size = available_rect.size();

        *self.signal = Signal::UpdateViewportSize((available_size.x, available_size.y));

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

        // Note to self: fuck you >:(
        // Note to self: ok wow thats pretty rude im trying my best ＞﹏＜
        // Note to self: finally holy shit i got it working
        let active_cam = self.active_camera.lock();
        if let Some(active_camera) = *active_cam {
            let camera_data = {
                if let Ok((cam, _comp)) = self
                    .world
                    .query_one::<(&Camera, &CameraComponent)>(active_camera).get()
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

            if let Ok(entity_transform) = self.world.query_one::<&mut EntityTransform>(*entity_id).get()
            {
                let was_focused = cfg.is_focused;
                cfg.is_focused = self.gizmo.is_focused();

                if cfg.is_focused && !was_focused {
                    cfg.entity_transform_original = Some(*entity_transform);
                }

                let synced = entity_transform.sync();
                let gizmo_transform =
                    transform_gizmo_egui::math::Transform::from_scale_rotation_translation(
                        synced.scale,
                        synced.rotation,
                        synced.position,
                    );

                if let Some((_result, new_transforms)) =
                    self.gizmo.interact(ui, &[gizmo_transform])
                    && let Some(new_transform) = new_transforms.first()
                {
                    let new_synced_pos: glam::DVec3 = new_transform.translation.into();
                    let new_synced_rot: glam::DQuat = new_transform.rotation.into();
                    let new_synced_scale: glam::DVec3 = new_transform.scale.into();

                    match *self.gizmo_orientation {
                        GizmoOrientation::Global => {
                            let local = *entity_transform.local();

                            let safe_local_scale = glam::DVec3::new(
                                if local.scale.x.abs() < 1e-6 { 1.0 } else { local.scale.x },
                                if local.scale.y.abs() < 1e-6 { 1.0 } else { local.scale.y },
                                if local.scale.z.abs() < 1e-6 { 1.0 } else { local.scale.z },
                            );

                            let new_world_scale = new_synced_scale / safe_local_scale;
                            let new_world_rot = new_synced_rot * local.rotation.inverse();

                            let scaled_local_pos = local.position * new_world_scale;
                            let rotated_local_pos = new_world_rot * scaled_local_pos;
                            let new_world_pos = new_synced_pos - rotated_local_pos;

                            let world_transform = entity_transform.world_mut();
                            world_transform.position = new_world_pos;
                            world_transform.rotation = new_world_rot;
                            world_transform.scale = new_world_scale;
                        }
                        GizmoOrientation::Local => {
                            let world_transform = entity_transform.world();
                            let world_scale = world_transform.scale;
                            let world_rot = world_transform.rotation;
                            let world_pos = world_transform.position;

                            let safe_world_scale = glam::DVec3::new(
                                if world_scale.x.abs() < 1e-6 { 1.0 } else { world_scale.x },
                                if world_scale.y.abs() < 1e-6 { 1.0 } else { world_scale.y },
                                if world_scale.z.abs() < 1e-6 { 1.0 } else { world_scale.z },
                            );

                            let local_transform = entity_transform.local_mut();
                            local_transform.scale = new_synced_scale / safe_world_scale;
                            local_transform.rotation = world_rot.inverse() * new_synced_rot;

                            let delta_pos = new_synced_pos - world_pos;
                            let unrotated_delta = world_rot.inverse() * delta_pos;
                            local_transform.position = unrotated_delta / safe_world_scale;
                        }
                    }
                }

                if was_focused && !cfg.is_focused {
                    if let Some(original) = cfg.entity_transform_original {
                        if original != *entity_transform {
                            UndoableAction::push_to_undo(
                                self.undo_stack,
                                UndoableAction::EntityTransform(*entity_id, original),
                            );
                            log::debug!("Pushed entity transform action to stack");
                        }
                    }
                }
                handled = true;
            }

            if !handled {
                if let Ok(transform) = self.world.query_one::<&mut Transform>(*entity_id).get()
                {
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
                        transform.scale = new_transform.scale.into();
                    }

                    if was_focused && !cfg.is_focused {
                        let transform_changed = cfg.old_pos.position != transform.position
                            || cfg.old_pos.rotation != transform.rotation
                            || cfg.old_pos.scale != transform.scale;

                        if transform_changed {
                            UndoableAction::push_to_undo(
                                self.undo_stack,
                                UndoableAction::Transform(*entity_id, cfg.old_pos),
                            );
                            log::debug!("Pushed transform action to stack");
                        }
                    }
                }
            }
        }
    }
}