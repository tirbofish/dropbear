//! Vertices and different buffers used for wgpu

use std::marker::PhantomData;

use wgpu::BufferUsages;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResizableBuffer<T> {
    buffer: wgpu::Buffer,
    capacity: usize,
    usage: wgpu::BufferUsages,
    label: String,
    _marker: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct UniformBuffer<T> {
    buffer: wgpu::Buffer,
    label: String,
    _marker: PhantomData<T>,
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    pub fn new(device: &wgpu::Device, label: &str) -> Self {
        let size = (std::mem::size_of::<T>() as wgpu::BufferAddress).max(16);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            label: label.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, value: &T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Debug, Clone)]
pub struct StorageBuffer<T> {
    buffer: wgpu::Buffer,
    label: String,
    _marker: PhantomData<T>,
}

impl<T: bytemuck::Pod> StorageBuffer<T> {
    /// Creates a storage buffer intended to be written by the CPU and read by the GPU.
    ///
    /// Note: whether it is bound as read-only is controlled by the bind group layout
    /// (`BufferBindingType::Storage { read_only: true }`).
    pub fn new(device: &wgpu::Device, label: &str) -> Self {
        let size = (std::mem::size_of::<T>() as wgpu::BufferAddress).max(16);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            label: label.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, value: &T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl<T: bytemuck::Pod> ResizableBuffer<T> {
    pub fn new(
        device: &wgpu::Device,
        initial_capacity: usize,
        usage: wgpu::BufferUsages,
        label: &str,
    ) -> Self {
        let size = (initial_capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: size.max(16),
            usage,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            capacity: initial_capacity,
            usage,
            label: label.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn uniform(
        device: &wgpu::Device,
        label: &str,
    ) -> Self {
        Self::new(device, 1, BufferUsages::UNIFORM | BufferUsages::COPY_DST, label)
    }

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        if data.is_empty() {
            return;
        }

        if data.len() > self.capacity {
            self.capacity = data.len().max(self.capacity * 2);
            
            let new_size = (self.capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress;
            
            log::debug!("Resizing buffer '{}' to hold {} items", self.label, self.capacity);

            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&self.label),
                size: new_size,
                usage: self.usage,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(data));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    
    pub fn slice(&self, count: usize) -> wgpu::BufferSlice<'_> {
        let byte_count = (count * std::mem::size_of::<T>()) as wgpu::BufferAddress;
        self.buffer.slice(0..byte_count)
    }
}