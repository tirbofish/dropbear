use glam::Vec2;
use wgpu::{BufferUsages, IndexFormat};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::camera::{CameraRendering, CameraUniform};
use crate::rendering::pipeline::KinoRendererPipeline;
use crate::rendering::vertex::Vertex;

pub mod pipeline;
pub mod texture;
pub mod vertex;

pub struct KinoWGPURenderer {
    pipeline: KinoRendererPipeline,
    default_texture: texture::Texture,
    pub size: Vec2,

    camera: CameraRendering,
}

impl KinoWGPURenderer {
    /// Creates a new `wgpu` renderer for the kino ui system.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        size: [f32; 2],
    ) -> Self {
        log::debug!("Creating KinoWGPURenderer");
        let pipeline = KinoRendererPipeline::new(device, surface_format);

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&CameraUniform {
                view_proj: [
                    [1.0, 0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0, 0.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let default_texture = texture::Texture::create_default(&device, &queue, &pipeline.texture_bind_group_layout);
        log::debug!("Created KinoWGPURenderer");
        Self {
            pipeline,
            default_texture,
            size: Vec2::from_array(size),
            camera: CameraRendering {
                buffer: camera_buffer,
                bind_group: camera_bind_group,
            },
        }
    }

    pub fn upload_camera_matrix(&mut self, queue: &wgpu::Queue, view_proj: [[f32; 4]; 4]) {
        queue.write_buffer(
            &self.camera.buffer,
            0,
            bytemuck::bytes_of(&CameraUniform { view_proj }),
        );
    }

    pub fn draw_batch(
        &self,
        r_pass: &mut wgpu::RenderPass<'_>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        batch: &mut VertexBatch,
        texture: Option<&texture::Texture>,
    ) {
        if batch.is_empty() {
            return;
        }
        batch.upload(&device, &queue);

        let texture = texture.unwrap_or(&self.default_texture);

        texture.bind(r_pass, 0);

        r_pass.set_pipeline(&self.pipeline.pipeline);
        r_pass.set_bind_group(1, &self.camera.bind_group, &[]);

        batch.draw(r_pass);
        batch.clear();
    }
}

/// Describes a primitive shape.
#[derive(Debug)]
pub struct VertexBatch {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    vertices_dirty: bool,
    indices_dirty: bool,
}

impl Default for VertexBatch {
    fn default() -> Self {
        Self {
            vertices: Vec::with_capacity(Self::MAX_VERTICES),
            indices: Vec::with_capacity(Self::MAX_INDICES),
            vertex_buffer: None,
            index_buffer: None,
            vertices_dirty: false,
            indices_dirty: false,
        }
    }
}

impl VertexBatch {
    const MAX_VERTICES: usize = u16::MAX as usize;
    const MAX_INDICES: usize = Self::MAX_VERTICES * 6;

    /// Returns true if adding verts/indices would exceed max allowed
    fn would_overflow(&self, vert_count: usize, idx_count: usize) -> bool {
        self.vertices.len() + vert_count > Self::MAX_VERTICES
            || self.indices.len() + idx_count > Self::MAX_INDICES
    }

    /// Adds vertices/indices, returns false if it would overflow
    pub fn push(&mut self, verts: &[Vertex], indices: &[u16]) -> bool {
        if self.would_overflow(verts.len(), indices.len()) {
            return false;
        }

        let idx_offset = self.vertices.len() as u16;
        self.vertices.extend_from_slice(verts);
        self.indices.extend(indices.iter().map(|i| *i + idx_offset));

        self.vertices_dirty = true;
        self.indices_dirty = true;

        true
    }

    fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.is_empty() || (!self.vertices_dirty && !self.indices_dirty) {
            return;
        }

        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("kino vertex buffer"),
                size: (Self::MAX_VERTICES * std::mem::size_of::<Vertex>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        if self.index_buffer.is_none() {
            self.index_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("kino index buffer"),
                size: (Self::MAX_INDICES * std::mem::size_of::<u16>()) as u64,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        if self.vertices_dirty {
            queue.write_buffer(
                self.vertex_buffer.as_ref().unwrap(),
                0,
                bytemuck::cast_slice(&self.vertices),
            );
            self.vertices_dirty = false;
        }
        if self.indices_dirty {
            let mut indices_bytes: Vec<u8> = bytemuck::cast_slice(&self.indices).to_vec();
            let remainder = indices_bytes.len() % wgpu::COPY_BUFFER_ALIGNMENT as usize;
            if remainder != 0 {
                let pad_len = wgpu::COPY_BUFFER_ALIGNMENT as usize - remainder;
                indices_bytes.extend_from_slice(&vec![0u8; pad_len]);
            }

            queue.write_buffer(self.index_buffer.as_ref().unwrap(), 0, &indices_bytes);
            self.indices_dirty = false;
        }
    }

    fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.vertices_dirty = true;
        self.indices_dirty = true;
    }

    fn draw(&self, pass: &mut wgpu::RenderPass) {
        if self.is_empty() {
            return;
        }

        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        pass.set_index_buffer(
            self.index_buffer.as_ref().unwrap().slice(..),
            IndexFormat::Uint16,
        );
        pass.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
    }
}