use crate::editor::{DragState, EditorTabDock, EditorTabDockDescriptor, EditorTabViewer, Signal, TABS_GLOBAL, UndoableAction};
use dropbear_engine::asset::ASSET_REGISTRY;
use dropbear_engine::camera::Camera;
use dropbear_engine::entity::{EntityTransform, MeshRenderer, Transform};
use dropbear_engine::lighting::Light;
use eucalyptus_core::camera::CameraComponent;
use eucalyptus_core::hierarchy::EntityTransformExt;
use eucalyptus_core::utils::ViewportMode;
use glam::DVec3;
use hecs::Entity;
use transform_gizmo_egui::{GizmoConfig, GizmoExt, GizmoOrientation};
use eucalyptus_core::input::ndc::NormalisedDeviceCoordinates;
use crate::editor::page::EditorTabVisibility;

impl<'a> EditorTabViewer<'a> {
    fn on_pointer_down(&mut self, touch_pos: [f32; 2], screen_size: [f32; 2], camera: &Camera) {
        if self.gizmo.is_focused() {
            return;
        }

        let inv_proj = camera.proj_mat.inverse().as_mat4();
        let inv_view = camera.view_mat.inverse().as_mat4();
        let (ray_origin, ray_dir) =
            NormalisedDeviceCoordinates::screen_to_ray(touch_pos, screen_size, inv_proj, inv_view);

        let mut closest: Option<(Entity, f32)> = None;

        for (entity, transform, mesh) in
            self.world.query::<(Entity, &EntityTransform, &MeshRenderer)>().iter()
        {
            let (aabb_min, aabb_max) = compute_world_aabb(transform, mesh);
            if let Some(t) =
                NormalisedDeviceCoordinates::ray_aabb(ray_origin, ray_dir, aabb_min, aabb_max)
            {
                if closest.map_or(true, |(_, best)| t < best) {
                    closest = Some((entity, t));
                }
            }
        }

        for (entity, transform, mesh) in
            self.world.query::<(Entity, &Transform, &MeshRenderer)>().iter()
        {
            let (aabb_min, aabb_max) = compute_transform_aabb(transform, mesh);
            if let Some(t) =
                NormalisedDeviceCoordinates::ray_aabb(ray_origin, ray_dir, aabb_min, aabb_max)
            {
                if closest.map_or(true, |(_, best)| t < best) {
                    closest = Some((entity, t));
                }
            }
        }

        if let Some((entity, t)) = closest {
            let hit_point = ray_origin + ray_dir * t;

            let entity_origin: glam::Vec3 =
                if let Ok(et) = self.world.get::<&EntityTransform>(entity) {
                    let p = et.world().position;
                    glam::Vec3::new(p.x as f32, p.y as f32, p.z as f32)
                } else if let Ok(tr) = self.world.get::<&Transform>(entity) {
                    let p = tr.position;
                    glam::Vec3::new(p.x as f32, p.y as f32, p.z as f32)
                } else {
                    hit_point
                };

            let plane_normal = -ray_dir;
            let plane_d = plane_normal.dot(hit_point);
            let pick_offset = hit_point - entity_origin;

            let initial_entity_transform =
                self.world.get::<&EntityTransform>(entity).ok().map(|et| *et);
            let initial_transform = if initial_entity_transform.is_none() {
                self.world.get::<&Transform>(entity).ok().map(|tr| *tr)
            } else {
                None
            };

            *self.selected_entity = Some(entity);
            *self.viewport_drag = Some(DragState {
                entity,
                plane_normal,
                plane_d,
                pick_offset,
                initial_entity_transform,
                initial_transform,
            });
            log::debug!("Pointer down - {:?}", entity);
        } else {
            *self.selected_entity = None;
            *self.viewport_drag = None;
        }
    }

    fn on_pointer_move(&mut self, touch_pos: [f32; 2], screen_size: [f32; 2], camera: &Camera) {
        let drag = match self.viewport_drag.as_ref() {
            Some(d) => d,
            None => return,
        };

        let inv_proj = camera.proj_mat.inverse().as_mat4();
        let inv_view = camera.view_mat.inverse().as_mat4();
        let (ray_origin, ray_dir) =
            NormalisedDeviceCoordinates::screen_to_ray(touch_pos, screen_size, inv_proj, inv_view);

        let new_world_pos = match NormalisedDeviceCoordinates::ray_plane(
            ray_origin,
            ray_dir,
            drag.plane_normal,
            drag.plane_d,
        ) {
            Some(p) => p - drag.pick_offset,
            None => return,
        };

        let entity = drag.entity;
        let new_pos = DVec3::new(
            new_world_pos.x as f64,
            new_world_pos.y as f64,
            new_world_pos.z as f64,
        );

        if let Ok(mut et) = self.world.get::<&mut EntityTransform>(entity) {
            et.world_mut().position = new_pos;
        } else if let Ok(mut tr) = self.world.get::<&mut Transform>(entity) {
            tr.position = new_pos;
        }

        log::debug!("Pointer moved - {:?}", entity);
    }

    fn on_pointer_up(&mut self) {
        let drag = match self.viewport_drag.take() {
            Some(d) => d,
            None => return,
        };

        if let Some(initial_et) = drag.initial_entity_transform {
            if let Ok(current_et) = self.world.get::<&EntityTransform>(drag.entity) {
                if *current_et != initial_et {
                    self.undo_stack
                        .push(UndoableAction::EntityTransform(drag.entity, initial_et));
                    log::debug!("Pushed viewport drag entity-transform to undo stack");
                }
            }
        } else if let Some(initial_tr) = drag.initial_transform {
            if let Ok(current_tr) = self.world.get::<&Transform>(drag.entity) {
                if *current_tr != initial_tr {
                    self.undo_stack
                        .push(UndoableAction::Transform(drag.entity, initial_tr));
                    log::debug!("Pushed viewport drag transform to undo stack");
                }
            }
        }
    }

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

        // let (_rect, _response) =
        //     ui.allocate_exact_size(available_size, egui::Sense::click_and_drag());


        let (_full, _) = ui.allocate_exact_size(available_size, egui::Sense::hover());

        ui.painter().image(
            self.view,
            image_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        let image_response = ui.interact(
            image_rect,
            egui::Id::new("viewport_image_interaction"),
            egui::Sense::click_and_drag(),
        );

        let snapping = ui.input(|input| input.modifiers.shift);
        let active_camera_entity: Option<Entity> = *self.active_camera.lock();
        if let Some(active_camera) = active_camera_entity {
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

                // viewport 3d clicking / dragging
                if image_response.clicked() || image_response.drag_started() {
                    if let Some(pos) = image_response.interact_pointer_pos() {
                        let local_pos = pos - image_rect.min;
                        let touch_pos = [local_pos.x, local_pos.y];
                        let screen_size = [image_rect.width(), image_rect.height()];
                        self.on_pointer_down(touch_pos, screen_size, &camera);
                    }
                }

                if image_response.dragged() {
                    if let Some(pos) = image_response.interact_pointer_pos() {
                        let local_pos = pos - image_rect.min;
                        let touch_pos = [local_pos.x, local_pos.y];
                        let screen_size = [image_rect.width(), image_rect.height()];
                        self.on_pointer_move(touch_pos, screen_size, &camera);
                    }
                }

                if image_response.drag_stopped() {
                    self.on_pointer_up();
                }
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

/// Compute a world-space AABB for an entity that has an [`EntityTransform`].
fn compute_world_aabb(transform: &EntityTransform, mesh: &MeshRenderer) -> (glam::Vec3, glam::Vec3) {
    aabb_for_world_matrix(transform.world().matrix().as_mat4(), mesh)
}

/// Compute a world-space AABB for an entity that only has a plain [`Transform`].
fn compute_transform_aabb(transform: &Transform, mesh: &MeshRenderer) -> (glam::Vec3, glam::Vec3) {
    aabb_for_world_matrix(transform.matrix().as_mat4(), mesh)
}

/// Transform the model's local AABB by `world_mat` and return the resulting world AABB.
///
/// If the model hasn't loaded yet a unit box centred on the origin is used as a fallback.
fn aabb_for_world_matrix(
    world_mat: glam::Mat4,
    mesh: &MeshRenderer,
) -> (glam::Vec3, glam::Vec3) {
    use glam::Vec3;

    let (local_min, local_max) = {
        let registry = ASSET_REGISTRY.read();
        let bounds = registry.get_model(mesh.model()).and_then(|model| {
            let mut lo = Vec3::splat(f32::INFINITY);
            let mut hi = Vec3::splat(f32::NEG_INFINITY);
            for m in &model.meshes {
                for v in &m.vertices {
                    let p = Vec3::from(v.position);
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
            }
            if lo.x <= hi.x { Some((lo, hi)) } else { None }
        });
        bounds.unwrap_or((Vec3::splat(-0.5), Vec3::splat(0.5)))
    };

    let s = mesh.import_scale();
    let local_min = local_min * s;
    let local_max = local_max * s;

    let corners = [
        Vec3::new(local_min.x, local_min.y, local_min.z),
        Vec3::new(local_max.x, local_min.y, local_min.z),
        Vec3::new(local_min.x, local_max.y, local_min.z),
        Vec3::new(local_max.x, local_max.y, local_min.z),
        Vec3::new(local_min.x, local_min.y, local_max.z),
        Vec3::new(local_max.x, local_min.y, local_max.z),
        Vec3::new(local_min.x, local_max.y, local_max.z),
        Vec3::new(local_max.x, local_max.y, local_max.z),
    ];

    let mut world_min = Vec3::splat(f32::INFINITY);
    let mut world_max = Vec3::splat(f32::NEG_INFINITY);
    for &corner in &corners {
        let wc = world_mat.transform_point3(corner);
        world_min = world_min.min(wc);
        world_max = world_max.max(wc);
    }
    (world_min, world_max)
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