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

        log::debug!("Registered new resizable buffer: {:?} (usage={:?})", label, usage);
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

        log::debug!("Registered new uniform buffer: {:?}", label);
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
    pub fn new_read_only(device: &wgpu::Device, label: &str) -> Self {
        Self::new(device, label, true)
    }

    pub fn new_read_write(device: &wgpu::Device, label: &str) -> Self {
        Self::new(device, label, false)
    }

    fn new(device: &wgpu::Device, label: &str, read_only: bool) -> Self {
        let usage = if read_only {
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
        } else {
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
        };

        let size = (std::mem::size_of::<T>() as wgpu::BufferAddress).max(16);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        log::debug!("Registered new storage buffer: {:?} (read_only: {})", label, read_only);
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