use crate::buffer::UniformBuffer;
use crate::graphics::SharedGraphicsContext;
use crate::pipelines::globals::Globals;
use wgpu::{BindGroup, Buffer};

/// Bind groups for @group(0)
pub struct SceneGlobalsBindGroup {
    pub bind_group: BindGroup,
}

impl SceneGlobalsBindGroup {
    pub fn new(
        graphics: &SharedGraphicsContext,
        globals_buffer: &UniformBuffer<Globals>,
        camera_buffer: &Buffer,
    ) -> Self {
        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scene globals+camera bind group"),
                layout: &graphics.layouts.scene_globals_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: globals_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: camera_buffer.as_entire_binding(),
                    },
                ],
            });

        Self { bind_group }
    }

    pub fn update(
        &mut self,
        graphics: &SharedGraphicsContext,
        globals_buffer: &UniformBuffer<Globals>,
        camera_buffer: &Buffer,
    ) {
        puffin::profile_function!();
        self.bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scene globals+camera bind group"),
                layout: &graphics.layouts.scene_globals_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: globals_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: camera_buffer.as_entire_binding(),
                    },
                ],
            });
    }

    pub fn as_ref(&self) -> &BindGroup {
        &self.bind_group
    }
}

/// Bind group for @group(2)
pub struct LightSkinBindGroup {
    pub bind_group: BindGroup,
}

impl LightSkinBindGroup {
    pub fn new(
        graphics: &SharedGraphicsContext,
        light_buffer: &Buffer,
        skinning_buffer: &Buffer,
    ) -> Self {
        let bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scene light+skin bind group"),
                layout: &graphics.layouts.scene_light_skin_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: light_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: skinning_buffer.as_entire_binding(),
                    },
                ],
            });

        Self { bind_group }
    }

    pub fn update(
        &mut self,
        graphics: &SharedGraphicsContext,
        light_buffer: &Buffer,
        skinning_buffer: &Buffer,
    ) {
        self.bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("scene light+skin bind group"),
                layout: &graphics.layouts.scene_light_skin_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: light_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: skinning_buffer.as_entire_binding(),
                    },
                ],
            });
    }

    pub fn as_ref(&self) -> &BindGroup {
        &self.bind_group
    }
}