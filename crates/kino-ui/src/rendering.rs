use glam::Vec2;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use batching::VertexBatch;
use crate::camera::{CameraRendering, CameraUniform};
use crate::rendering::pipeline::KinoRendererPipeline;
// use crate::rendering::text::KinoTextRenderer;

pub mod pipeline;
pub mod texture;
pub mod vertex;
pub mod batching;
// pub mod text;

pub struct KinoWGPURenderer {
    pipeline: KinoRendererPipeline,
    default_texture: texture::Texture,
    pub format: wgpu::TextureFormat,
    pub size: Vec2,
    // pub text: KinoTextRenderer,

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

        let default_texture = texture::Texture::create_default(&device, &queue, &pipeline.texture_bind_group_layout, surface_format);
        // let text = KinoTextRenderer::new(&device, &queue, surface_format);

        log::debug!("Created KinoWGPURenderer");
        Self {
            pipeline,
            default_texture,
            format: surface_format,
            size: Vec2::from_array(size),
            // text,
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

    pub(crate) fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.pipeline.texture_bind_group_layout
    }
}

