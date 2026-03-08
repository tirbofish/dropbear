use egui::Ui;
use glam::Vec2;
use crate::editor::{EditorTabDock, EditorTabDockDescriptor, EditorTabViewer};
use crate::editor::page::EditorTabVisibility;

pub struct UIViewport;

impl EditorTabDock for UIViewport {
    fn desc() -> EditorTabDockDescriptor {
        EditorTabDockDescriptor {
            id: "UI Viewport",
            title: "UI Viewport".to_string(),
            visibility: EditorTabVisibility::UIEditor,
        }
    }

    fn display(viewer: &mut EditorTabViewer<'_>, ui: &mut Ui) {
        let available_rect = ui.available_rect_before_wrap();
        let available_size = available_rect.size();
        let pixels_per_point = ui.ctx().pixels_per_point();

        let desired_width = (available_size.x * pixels_per_point).max(1.0).round() as u32;
        let desired_height = (available_size.y * pixels_per_point).max(1.0).round() as u32;

        viewer
            .ui_editor
            .render(viewer.graphics.clone(), desired_width, desired_height);

        if let Some(texture_id) = viewer.ui_editor.texture_id() {
            let response = ui.add_sized(
                [available_size.x, available_size.y],
                egui::Image::new((texture_id, available_size)).sense(egui::Sense::drag()),
            );

            let ppp = ui.ctx().pixels_per_point();
            let viewport_pixels = Vec2::new(available_size.x * ppp, available_size.y * ppp);

            if response.hovered() {
                let scroll_y = ui.ctx().input(|i| i.raw_scroll_delta.y);
                if scroll_y.abs() > 0.0 {
                    let zoom_delta = scroll_y * 0.0025;
                    viewer.ui_editor.zoom_by(zoom_delta);
                }
            }

            let is_middle_down = ui
                .ctx()
                .input(|i| i.pointer.button_down(egui::PointerButton::Middle));

            if response.hovered() && is_middle_down {
                let pointer_delta_points = ui.ctx().input(|i| i.pointer.delta());
                if pointer_delta_points != egui::Vec2::ZERO {
                    let delta_pixels = Vec2::new(
                        pointer_delta_points.x * ppp,
                        pointer_delta_points.y * ppp,
                    );
                    viewer.ui_editor.pan_by_pixels(delta_pixels);
                }
            }

            let cursor_info = response.hover_pos().map(|hover_pos| {
                let local = hover_pos - response.rect.min;
                let pixel = Vec2::new(local.x * ppp, local.y * ppp);
                viewer.ui_editor.world_from_screen_pixels(pixel, viewport_pixels)
            });

            let zoom_percent = viewer.ui_editor.zoom() * 100.0;
            let hud = if let Some(world) = cursor_info {
                format!(
                    "Coords: ({:.1}, {:.1}) u | zoom: {:.0}%",
                    world.x,
                    world.y,
                    zoom_percent
                )
            } else {
                format!(
                    "Coords: (-, -) u | zoom: {:.0}%",
                    zoom_percent
                )
            };

            let text_pos = response.rect.left_top() + egui::vec2(8.0, 8.0);
            ui.painter().text(
                text_pos,
                egui::Align2::LEFT_TOP,
                hud,
                egui::TextStyle::Monospace.resolve(ui.style()),
                egui::Color32::WHITE,
            );
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("UI viewport texture is initialising...");
            });
        }
    }
}