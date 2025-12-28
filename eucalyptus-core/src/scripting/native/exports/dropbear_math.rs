/// Represents a [`Vec2`] in a C struct form.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

/// Represents a [`Vec3`] in a C struct form.
#[repr(C)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Represents a [`Transform`] in a C struct form. 
#[repr(C)]
pub struct NativeTransform {
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub rotation_x: f64,
    pub rotation_y: f64,
    pub rotation_z: f64,
    pub rotation_w: f64,
    pub scale_x: f64,
    pub scale_y: f64,
    pub scale_z: f64,
}

/// Represents an [`EntityTransform`] in a C struct form. 
#[repr(C)]
pub struct NativeEntityTransform {
    pub local: NativeTransform,
    pub world: NativeTransform,
}