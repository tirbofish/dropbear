use dropbear_engine::entity::Transform;
use glam::{DQuat, DVec3};
use crate::scripting::native::exports::dropbear_math::NativeTransform;

pub fn write_native_transform(target: &mut NativeTransform, transform: &Transform) {
    target.position_x = transform.position.x;
    target.position_y = transform.position.y;
    target.position_z = transform.position.z;
    target.rotation_x = transform.rotation.x;
    target.rotation_y = transform.rotation.y;
    target.rotation_z = transform.rotation.z;
    target.rotation_w = transform.rotation.w;
    target.scale_x = transform.scale.x;
    target.scale_y = transform.scale.y;
    target.scale_z = transform.scale.z;
}

pub fn native_transform_to_transform(native: &NativeTransform) -> Transform {
    Transform {
        position: DVec3::new(native.position_x, native.position_y, native.position_z),
        rotation: DQuat::from_xyzw(
            native.rotation_x,
            native.rotation_y,
            native.rotation_z,
            native.rotation_w,
        ),
        scale: DVec3::new(native.scale_x, native.scale_y, native.scale_z),
    }
}