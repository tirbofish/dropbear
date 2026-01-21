use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct UniformBuffer<T> {
    pub inner: T,
    buffer: wgpu::Buffer,
}

impl<T: bytemuck::Pod + Default> UniformBuffer<T> {
    pub fn new(device: &wgpu::Device, uniform: Option<T>, label: Option<&str>) -> Self {
        let uniform = if let Some(uniform) = uniform {
            uniform
        } else {
            T::default()
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            inner: uniform,
            buffer,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, uniform: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
    
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}
