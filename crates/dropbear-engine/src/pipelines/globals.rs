use std::sync::Arc;

use crate::buffer::UniformBuffer;
use crate::graphics::SharedGraphicsContext;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub num_lights: u32,
    pub ambient_strength: f32,
    pub _padding: [u32; 2],
}

impl Default for Globals {
    fn default() -> Self {
        Self {
            num_lights: 0,
            ambient_strength: 0.8,
            _padding: [0; 2],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalsUniform {
    pub data: Globals,
    pub buffer: UniformBuffer<Globals>,
    pub bind_group: wgpu::BindGroup,
}

impl GlobalsUniform {
    pub fn new(graphics: Arc<SharedGraphicsContext>, label: Option<&str>) -> Self {
        let label = label.unwrap_or("shader globals");

        let buffer = UniformBuffer::new(&graphics.device, label);

        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &graphics.layouts.shader_globals_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.buffer().as_entire_binding(),
                }],
                label: Some(label),
            });

        let data = Globals::default();
        buffer.write(&graphics.queue, &data);

        Self {
            data,
            buffer,
            bind_group,
        }
    }

    pub fn write(&mut self, queue: &wgpu::Queue) {
        self.buffer.write(queue, &self.data);
    }

    pub fn set_num_lights(&mut self, num_lights: u32) {
        self.data.num_lights = num_lights;
    }

    pub fn set_ambient_strength(&mut self, ambient_strength: f32) {
        self.data.ambient_strength = ambient_strength;
    }
}
