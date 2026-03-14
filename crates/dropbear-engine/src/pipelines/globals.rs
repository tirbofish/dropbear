use std::sync::Arc;
use dropbear_utils::Dirty;
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
    pub data: Dirty<Globals>,
    pub buffer: UniformBuffer<Globals>,
}

impl GlobalsUniform {
    pub fn new(graphics: Arc<SharedGraphicsContext>, label: Option<&str>) -> Self {
        let label = label.unwrap_or("shader globals");

        let buffer: UniformBuffer<Globals> = UniformBuffer::new(&graphics.device, label);

        let data = Dirty::new(Globals::default());
        buffer.write(&graphics.queue, &data);

        Self {
            data,
            buffer,
        }
    }

    pub fn write(&mut self, queue: &wgpu::Queue) {
        if self.data.is_dirty() {
            self.buffer.write(queue, &self.data);
        }

    }

    pub fn set_num_lights(&mut self, num_lights: u32) {
        self.data.num_lights = num_lights;
    }

    pub fn set_ambient_strength(&mut self, ambient_strength: f32) {
        self.data.ambient_strength = ambient_strength;
    }
}
