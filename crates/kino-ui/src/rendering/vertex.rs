use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct Vertex {
    pub position: [f32; 2],
    pub colour: [f32; 4],
    pub tex_coord: [f32; 2],
}

impl Vertex {
    pub fn new(
        position: impl Into<[f32; 2]>,
        colour: impl Into<[f32; 4]>,
        tex_coord: impl Into<[f32; 2]>,
    ) -> Self {
        Self {
            position: position.into(),
            colour: colour.into(),
            tex_coord: tex_coord.into(),
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                // colour
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                // tex_coords
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
            ],
        }
    }
}
