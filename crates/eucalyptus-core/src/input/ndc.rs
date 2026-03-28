use glam::{Mat4, Vec3, Vec4};

/// Helpers for converting from a 2D display to a 3D object, such as billboard ui and some
/// other applications.
// viewport aint even work lmaoooo
pub struct NormalisedDeviceCoordinates;

impl NormalisedDeviceCoordinates {
    pub fn screen_to_ray(
        touch_pos: impl Into<[f32; 2]>,
        screen_dims: impl Into<[f32; 2]>,
        inv_proj: Mat4,
        inv_view: Mat4,
    ) -> (Vec3, Vec3) {
        let [touch_x, touch_y] = touch_pos.into();
        let [screen_width, screen_height] = screen_dims.into();

        let ndc_x = (touch_x / screen_width) * 2.0 - 1.0;
        let ndc_y = -(touch_y / screen_height) * 2.0 + 1.0; // flip Y, screen Y is down

        // unproject to view space
        let clip_near = Vec4::new(ndc_x, ndc_y, 0.0, 1.0); // z=0 = near plane
        let mut view_near = inv_proj * clip_near;
        view_near /= view_near.w;

        // to world space
        let world_near = inv_view * view_near;

        // ray origin = camera position, direction = toward unprojected point
        let ray_origin = (inv_view * Vec4::new(0.0, 0.0, 0.0, 1.0)).truncate();
        let ray_dir = (world_near.truncate() - ray_origin).normalize();

        (ray_origin, ray_dir)
    }

    pub fn ray_aabb(origin: Vec3, dir: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Option<f32> {
        let inv_dir = 1.0 / dir;
        let t1 = (aabb_min - origin) * inv_dir;
        let t2 = (aabb_max - origin) * inv_dir;
        let t_min = t1.min(t2).max_element();
        let t_max = t1.max(t2).min_element();
        if t_max >= t_min && t_max > 0.0 {
            Some(t_min)
        } else {
            None
        }
    }

    pub fn ray_plane(origin: Vec3, dir: Vec3, plane_normal: Vec3, plane_d: f32) -> Option<Vec3> {
        let denom = plane_normal.dot(dir);
        if denom.abs() < 1e-10 {
            return None;
        } // ray parallel to plane
        let t = (plane_d - plane_normal.dot(origin)) / denom;
        if t < 0.0 {
            return None;
        } // intersection behind ray
        Some(origin + dir * t)
    }
}
