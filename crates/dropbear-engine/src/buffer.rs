//! Vertices and different buffers used for wgpu

use std::marker::PhantomData;
use std::ops::Range;
use bytemuck::NoUninit;
use dropbear_utils::Dirty;

pub trait WritableBuffer<T> {
    fn write(&self, queue: &wgpu::Queue, value: &T);
    fn buffer(&self) -> &wgpu::Buffer;
}


#[derive(Debug, Clone, PartialEq)]
pub struct UniformBuffer<T> {
    buffer: wgpu::Buffer,
    label: String,
    _marker: PhantomData<T>,
}

impl<T: NoUninit> WritableBuffer<T> for UniformBuffer<T> {
    fn write(&self, queue: &wgpu::Queue, value: &T) {
        puffin::profile_function!(&self.label);
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl<T: NoUninit> UniformBuffer<T> {
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

impl<T: NoUninit> WritableBuffer<T> for StorageBuffer<T> {
    fn write(&self, queue: &wgpu::Queue, value: &T) {
        puffin::profile_function!(self.label());
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }

    fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl<T: NoUninit> StorageBuffer<T> {
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
            wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
        };

        let size = (std::mem::size_of::<T>() as wgpu::BufferAddress).max(16);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        log::debug!(
            "Registered new storage buffer: {:?} (read_only: {})",
            label,
            read_only
        );
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

impl<T: NoUninit> StorageBuffer<T> {
    pub fn new_slice(device: &wgpu::Device, label: &str, count: usize, read_only: bool) -> Self {
        let usage = if read_only {
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
        } else {
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
        };

        let size = ((std::mem::size_of::<T>() * count) as wgpu::BufferAddress).max(16);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });

        Self { buffer, label: label.to_string(), _marker: PhantomData }
    }

    pub fn write_slice(&self, queue: &wgpu::Queue, values: &[T]) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(values));
    }
}

#[derive(Clone, PartialEq)]
pub struct MutableDataBuffer<T, B: WritableBuffer<T> = UniformBuffer<T>> {
    data: Dirty<T>,
    pub buffer: B,
}

impl<T: NoUninit, B: WritableBuffer<T>> MutableDataBuffer<T, B> {
    pub fn new(data: Dirty<T>, buffer: B) -> Self {
        Self {
            data,
            buffer,
        }
    }
    
    pub fn write(&mut self, queue: &wgpu::Queue) {
        if let Some(value) = self.data.get_if_dirty() {
            self.buffer.write(queue, value);
        }
    }
    
    pub fn get_data(&self) -> &T {
        self.data.get()
    }
    
    pub fn set_data(&mut self, value: T) {
        self.data.set(value);
    }
}

pub struct DynamicBuffer<T> {
    data: Vec<T>,
    dirty_range: Option<Range<usize>>,
    buffer: wgpu::Buffer,
    capacity: usize,
    usage: wgpu::BufferUsages,
    label: String,
    _marker: PhantomData<T>,
}

impl<T: Clone> Clone for DynamicBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            dirty_range: self.dirty_range.clone(),
            buffer: self.buffer.clone(),
            capacity: self.capacity,
            usage: self.usage,
            label: self.label.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for DynamicBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicBuffer")
            .field("label", &self.label)
            .field("capacity", &self.capacity)
            .field("len", &self.data.len())
            .field("dirty_range", &self.dirty_range)
            .finish()
    }
}

impl<T: NoUninit> DynamicBuffer<T> {
    /// Allocate an empty GPU buffer with `initial_capacity` element slots.
    pub fn new(
        device: &wgpu::Device,
        initial_capacity: usize,
        usage: wgpu::BufferUsages,
        label: &str,
    ) -> Self {
        let size = ((initial_capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress).max(16);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        });
        log::debug!("Registered new dynamic buffer: {:?} (usage={:?})", label, usage);
        Self {
            data: Vec::with_capacity(initial_capacity),
            dirty_range: None,
            buffer,
            capacity: initial_capacity,
            usage,
            label: label.to_string(),
            _marker: PhantomData,
        }
    }

    /// Create a buffer pre-populated with `data`. The initial contents are immediately
    /// uploaded via `create_buffer_init`, so no flush is needed after construction.
    pub fn from_slice(
        device: &wgpu::Device,
        data: &[T],
        usage: wgpu::BufferUsages,
        label: &str,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage,
        });
        log::debug!("Registered new dynamic buffer (from_slice): {:?} (usage={:?})", label, usage);
        Self {
            capacity: data.len(),
            data: data.to_vec(),
            dirty_range: None,
            buffer,
            usage,
            label: label.to_string(),
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Borrow the CPU-side data slice.
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Consume the buffer and return the owned CPU-side data.
    pub fn into_data(self) -> Vec<T> {
        self.data
    }

    /// Read an element without marking anything dirty.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Overwrite a single element and mark it dirty.
    pub fn update(&mut self, index: usize, value: T) {
        self.data[index] = value;
        self.expand_dirty(index..index + 1);
    }

    /// Overwrite a contiguous slice of elements starting at `start` and mark them dirty.
    pub fn update_range(&mut self, start: usize, values: &[T])
    where
        T: Copy,
    {
        let end = start + values.len();
        self.data[start..end].copy_from_slice(values);
        self.expand_dirty(start..end);
    }

    /// Append an element to the CPU buffer and mark the new slot dirty.
    /// The GPU buffer will be reallocated on the next [`flush`](DynamicBuffer::flush) if needed.
    pub fn push(&mut self, value: T) {
        let idx = self.data.len();
        self.data.push(value);
        self.expand_dirty(idx..idx + 1);
    }

    /// Truncate the CPU buffer. Does not shrink the GPU allocation; the GPU buffer
    /// will simply have unused capacity at the end until re-populated.
    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
        if let Some(ref mut r) = self.dirty_range {
            r.end = r.end.min(len);
            if r.start >= r.end {
                self.dirty_range = None;
            }
        }
    }

    /// Upload only the dirty element range to the GPU.
    ///
    /// If the CPU buffer has grown beyond the current GPU capacity the GPU buffer is
    /// reallocated (doubling strategy) and the full contents are re-uploaded.
    /// Cheap no-op when nothing is dirty.
    pub fn flush(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        puffin::profile_function!(&self.label);
        let Some(dirty) = self.dirty_range.take() else { return };

        if self.data.len() > self.capacity {
            self.capacity = self.data.len().max(self.capacity * 2);
            let new_size =
                ((self.capacity * std::mem::size_of::<T>()) as wgpu::BufferAddress).max(16);
            log::debug!("Reallocating dynamic buffer '{}' to {} elements", self.label, self.capacity);
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&self.label),
                size: new_size,
                usage: self.usage,
                mapped_at_creation: false,
            });
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
        } else {
            let stride = std::mem::size_of::<T>();
            let byte_offset = (dirty.start * stride) as wgpu::BufferAddress;
            queue.write_buffer(
                &self.buffer,
                byte_offset,
                bytemuck::cast_slice(&self.data[dirty]),
            );
        }
    }

    /// Convenience: replace all CPU data with `data` and immediately flush to the GPU.
    ///
    /// Equivalent to clearing, extending, marking everything dirty, then calling
    /// [`flush`](DynamicBuffer::flush). Use this for buffers that are fully rewritten
    /// every frame (instance buffers, debug geometry, etc.).
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[T])
    where
        T: Copy,
    {
        self.data.clear();
        self.data.extend_from_slice(data);
        if !data.is_empty() {
            self.dirty_range = Some(0..data.len());
        }
        self.flush(device, queue);
    }

    /// The underlying `wgpu::Buffer` — use this when binding to a render pass.
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// A `BufferSlice` covering the first `count` elements (by byte length).
    ///
    /// Mirrors `ResizableBuffer::slice` for render-pass binding of partially-filled buffers.
    pub fn slice(&self, count: usize) -> wgpu::BufferSlice<'_> {
        let byte_len = (count * std::mem::size_of::<T>()) as wgpu::BufferAddress;
        self.buffer.slice(0..byte_len)
    }

    /// A `BufferSlice` covering exactly the live elements (`0..len`).
    pub fn full_slice(&self) -> wgpu::BufferSlice<'_> {
        let byte_len = (self.data.len() * std::mem::size_of::<T>()) as wgpu::BufferAddress;
        self.buffer.slice(0..byte_len)
    }

    fn expand_dirty(&mut self, range: Range<usize>) {
        self.dirty_range = Some(match self.dirty_range.take() {
            None => range,
            Some(existing) => {
                existing.start.min(range.start)..existing.end.max(range.end)
            }
        });
    }
}
