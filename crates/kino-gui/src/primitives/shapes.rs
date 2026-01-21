use wgpu::RenderPass;
use wgpu::util::DeviceExt;
use crate::math::{create_orthographic_projection, Colour, Size, Vector2};
use crate::{Widget, WidgetId};
use crate::rendering::{Globals, KinoRenderer};
use crate::utils::UniformBuffer;

pub struct Rectangle {
    pub id: WidgetId,
    pub initial: Vector2,
    pub size: Size,
    pub fill_colour: Colour,

    globals_uniform: Option<UniformBuffer<Globals>>,
    vertex_buffer: Option<wgpu::Buffer>,
}

impl Rectangle {
    pub fn new(id: WidgetId, initial: Vector2, size: Size, fill_colour: Colour) -> Self {
        Self {
            id,
            initial,
            size,
            fill_colour,
            globals_uniform: None,
            vertex_buffer: None,
        }
    }

    pub(crate) fn create_vertex_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let x = self.initial.x;
        let y = self.initial.y;
        let w = self.size.width;
        let h = self.size.height;
        let c = [
            self.fill_colour.r,
            self.fill_colour.g,
            self.fill_colour.b,
            self.fill_colour.a,
        ];

        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct Vertex {
            position: [f32; 3],
            fill_colour: [f32; 4],
        }

        let vertices: [Vertex; 6] = [
            Vertex { position: [x, y, 0.0], fill_colour: c },
            Vertex { position: [x + w, y, 0.0], fill_colour: c },
            Vertex { position: [x + w, y + h, 0.0], fill_colour: c },
            Vertex { position: [x, y, 0.0], fill_colour: c },
            Vertex { position: [x + w, y + h, 0.0], fill_colour: c },
            Vertex { position: [x, y + h, 0.0], fill_colour: c },
        ];

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rectangle vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }
}

impl Widget for Rectangle {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn draw<'a>(&mut self, renderer: &KinoRenderer, pass: &mut RenderPass<'a>) {
        if self.globals_uniform.is_none() {
            let globals = Globals::new(
                create_orthographic_projection(
                    0.0,
                    renderer.context.screen_size.width,
                    renderer.context.screen_size.height,
                    0.0,
                    -1.0,
                    1.0
                ),
            [
                renderer.context.screen_size.width,
                renderer.context.screen_size.height,
                ],
            );

            self.globals_uniform = Some(UniformBuffer::new(
                &renderer.render.device,
                Some(globals),
                Some("rectangle globals uniform"),
            ));
        }

        if self.vertex_buffer.is_none() {
            self.vertex_buffer = Some(self.create_vertex_buffer(&renderer.render.device));
        }

        let globals_bind_group = renderer.render.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rectangle globals bind group"),
            layout: &renderer.render.globals_uniform_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.globals_uniform.as_ref().unwrap().buffer(),
                        offset: 0,
                        size: None,
                    }),
                }
            ],
        });

        pass.set_pipeline(&renderer.render.pipeline);
        pass.set_bind_group(0, &globals_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));

        pass.draw(0..6, 0..1);
    }
}