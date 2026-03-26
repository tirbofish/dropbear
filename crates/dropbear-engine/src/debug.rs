use std::sync::Arc;
use wgpu::{RenderPipeline, RenderPipelineDescriptor, VertexState};
use crate::graphics::SharedGraphicsContext;

pub struct DebugLine {

}

pub struct DebugDraw {
    pipeline: Arc<DebugDrawPipeline>,
}

impl DebugDraw {
    pub fn draw_line(&self) {

    }
}

pub struct DebugDrawPipeline {
    pipeline: RenderPipeline,
}

impl DebugDrawPipeline {
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("debug draw pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = graphics.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("debug draw render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &(),
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: None,
            multiview: None,
            cache: None,
        });
    }

    pub fn draw(&self, graphics: Arc<SharedGraphicsContext>) {

    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DebugVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}