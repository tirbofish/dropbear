use std::sync::Arc;
use glam::{Mat4};
use crate::animation::{MorphTargetInfo, MAX_MORPH_WEIGHTS, MAX_SKINNING_MATRICES};
use crate::buffer::{StorageBuffer, UniformBuffer, WritableBuffer};
use crate::graphics::SharedGraphicsContext;

pub struct AnimationDefaults {
    pub skinning_buffer: StorageBuffer<Mat4>,
    pub morph_deltas_buffer: StorageBuffer<f32>,
    pub morph_weights_buffer: StorageBuffer<f32>,
    pub morph_info_buffer: UniformBuffer<MorphTargetInfo>,
    pub animation_bind_group: wgpu::BindGroup,
}

impl AnimationDefaults {
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let identity = vec![Mat4::IDENTITY; MAX_SKINNING_MATRICES];
        let skinning_buffer = StorageBuffer::new_slice(
            &graphics.device, "editor default skinning buffer", MAX_SKINNING_MATRICES, true
        );
        skinning_buffer.write_slice(&graphics.queue, &identity);

        // mapped to default value
        let morph_deltas_buffer: StorageBuffer<f32> = StorageBuffer::new_read_only(
            &graphics.device, "editor default morph deltas buffer"
        );

        let morph_weights_buffer: StorageBuffer<f32> = StorageBuffer::new_slice(
            &graphics.device, "editor default skinning buffer", MAX_MORPH_WEIGHTS, true
        );

        let morph_info = MorphTargetInfo::default();
        let morph_info_buffer = UniformBuffer::new(
            &graphics.device, "editor default morph info buffer"
        );
        morph_info_buffer.write(&graphics.queue, &morph_info);

        let animation_bind_group = graphics.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("editor default animation bind group"),
                layout: &graphics.layouts.animation_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: skinning_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: morph_deltas_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: morph_weights_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: morph_info_buffer.buffer().as_entire_binding(),
                    },
                ],
            },
        );

        Self {
            skinning_buffer,
            morph_deltas_buffer,
            morph_weights_buffer,
            morph_info_buffer,
            animation_bind_group,
        }
    }
}