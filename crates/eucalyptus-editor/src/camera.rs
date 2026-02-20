// impl InspectableComponent for Camera {
//     fn inspect(
//         &mut self,
//         _entity: &mut Entity,
//         _cfg: &mut StaticallyKept,
//         ui: &mut Ui,
//         _undo_stack: &mut Vec<UndoableAction>,
//         _signal: &mut Signal,
//         _label: &mut String,
//     ) {
//         ui.vertical(|ui| {
//             ui.horizontal(|ui| {
//                 ui.label("Position:");
//                 ui.label(format!(
//                     "{:.2}, {:.2}, {:.2}",
//                     self.eye.x, self.eye.y, self.eye.z
//                 ));
//                 if ui.button("Reset").clicked() {
//                     self.eye = glam::DVec3::ZERO;
//                 }
//             });
//
//             ui.horizontal(|ui| {
//                 ui.label("Target:");
//                 ui.label(format!(
//                     "{:.2}, {:.2}, {:.2}",
//                     self.target.x, self.target.y, self.target.z
//                 ));
//                 if ui.button("Reset").clicked() {
//                     self.target = glam::DVec3::ZERO;
//                 }
//             });
//         });
//     }
// }
//
// impl InspectableComponent for CameraComponent {
//     fn inspect(
//         &mut self,
//         _entity: &mut Entity,
//         _cfg: &mut StaticallyKept,
//         ui: &mut Ui,
//         _undo_stack: &mut Vec<UndoableAction>,
//         _signal: &mut Signal,
//         _label: &mut String,
//     ) {
//         ui.vertical(|ui| {
//             ui.horizontal(|ui| {
//                 ui.label("Speed:");
//                 ui.add(
//                     egui::DragValue::new(&mut self.settings.speed)
//                         .speed(0.1),
//                 );
//             });
//
//             ui.horizontal(|ui| {
//                 ui.label("Sensitivity:");
//                 ui.add(
//                     egui::DragValue::new(&mut self.settings.sensitivity)
//                         .speed(0.0001)
//                         .range(0.0001..=1.5),
//                 );
//             });
//
//             ui.horizontal(|ui| {
//                 ui.label("FOV:");
//                 ui.add(
//                     egui::Slider::new(&mut self.settings.fov_y, 10.0..=120.0).suffix("°"),
//                 );
//             });
//         });
//     }
// }
