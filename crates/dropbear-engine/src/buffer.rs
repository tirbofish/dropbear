//! Vertices and different buffers used for wgpu

use std::marker::PhantomData;

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
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            label: label.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, value: &T) {
        puffin::profile_function!(&self.label);
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// A wrapper to a [wgpu::Buffer] that stores
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
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            label: label.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, value: &T) {
        puffin::profile_function!(self.label());
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

    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T]) {
        puffin::profile_function!(&self.label);
        if data.is_empty() {
            return;
        }

        if data.len() > self.capacity {
            self.capacity = data.len().max(self.capacity * 2);

            let new_size = (self.capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress;

            log::debug!(
                "Resizing buffer '{}' to hold {} items",
                self.label,
                self.capacity
            );

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
