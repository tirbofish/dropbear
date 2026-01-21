#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    /// States the left most corner of the rectangle in reference to the window
    pub initial: Vector2,
    /// The width and height that extends from the initial point, with the width extending
    /// left and the height extending down. 
    pub size: Size,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Into<Size> for Vector2 {
    fn into(self) -> Size {
        Size {
            width: self.x,
            height: self.y,
        }
    }
}

impl Into<[f32; 2]> for Vector2 {
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Into<Vector2> for [f32; 2] {
    fn into(self) -> Vector2 {
        Vector2 {
            x: self[0],
            y: self[1],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32
}

impl Default for Size {
    fn default() -> Self {
        Self {
            // cant be zero otherwise wgpu panics
            width: 800.0,
            height: 600.0,
        }
    }
}

impl Into<Vector2> for Size {
    fn into(self) -> Vector2 {
        Vector2 {
            x: self.width,
            y: self.height,
        }
    }
}

impl Into<[f32; 2]> for Size {
    fn into(self) -> [f32; 2] {
        [self.width, self.height]
    }
}

impl Into<Size> for [f32; 2] {
    fn into(self) -> Size {
        Size {
            width: self[0],
            height: self[1],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub fn create_orthographic_projection(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    let width = right - left;
    let height = top - bottom;
    let depth = far - near;

    [
        [2.0 / width, 0.0, 0.0, 0.0],
        [0.0, 2.0 / height, 0.0, 0.0],
        [0.0, 0.0, -2.0 / depth, 0.0],
        [-(right + left) / width, -(bottom + top) / height, -(far + near) / depth, 1.0],
    ]
}