use glam::Quat;
use dropbear_engine::debug::DebugDraw;
use crate::physics::collider::ColliderShape;

/// Extension traits for [`DebugDraw`](dropbear_engine::debug::DebugDraw)
pub trait DebugDrawExt {
    fn draw_collider(&mut self, shape: &ColliderShape, translation: glam::Vec3, scale: glam::Vec3, rotation: Quat, colour: [f32; 4]);
}

impl DebugDrawExt for DebugDraw {
    fn draw_collider(&mut self, shape: &ColliderShape, translation: glam::Vec3, scale: glam::Vec3, rotation: Quat, colour: [f32; 4]) {
        match &shape {
            ColliderShape::Box { half_extents } => {
                let he = glam::Vec3::new(
                    half_extents.x as f32 * scale.x,
                    half_extents.y as f32 * scale.y,
                    half_extents.z as f32 * scale.z,
                );
                self.draw_obb(translation, he, rotation, colour);
            }
            ColliderShape::Sphere { radius } => {
                let r = radius * scale.x.max(scale.y).max(scale.z);
                self.draw_sphere(translation, r, colour);
            }
            ColliderShape::Capsule {
                half_height,
                radius,
            } => {
                let axis = rotation * glam::Vec3::Y;
                let top = translation + axis * (half_height * scale.y);
                let bottom = translation - axis * (half_height * scale.y);
                self.draw_capsule(
                    bottom,
                    top,
                    radius * scale.x.max(scale.z),
                    colour,
                );
            }
            ColliderShape::Cylinder {
                half_height,
                radius,
            } => {
                let axis = rotation * glam::Vec3::Y;
                self.draw_cylinder(
                    translation,
                    half_height * scale.y,
                    radius * scale.x.max(scale.z),
                    axis,
                    colour,
                );
            }
            ColliderShape::Cone {
                half_height,
                radius,
            } => {
                let axis = rotation * glam::Vec3::Y;
                let apex = translation + axis * (half_height * scale.y);
                let height = 2.0 * half_height * scale.y;
                let r = radius * scale.x.max(scale.z);
                self.draw_cone(
                    apex,
                    -(rotation * glam::Vec3::Y),
                    (r / height).atan(),
                    height,
                    colour,
                );
            }
        }
    }
}
