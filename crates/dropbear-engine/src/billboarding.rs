use crate::asset::ASSET_REGISTRY;
use crate::buffer::{DynamicBuffer, UniformBuffer, WritableBuffer};
use crate::graphics::SharedGraphicsContext;
use crate::shader::Shader;
use glam::Mat4;
use std::sync::Arc;
use wgpu::MultisampleState;
use wgpu::util::{DeviceExt};

pub struct BillboardPipeline {
    pipeline: wgpu::RenderPipeline,
    transform_buffer: UniformBuffer<Mat4>,
    projection_buffer: UniformBuffer<Mat4>,
    position_buffer: DynamicBuffer<[f32; 3]>,
    tex_coord_buffer: DynamicBuffer<[f32; 2]>,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl BillboardPipeline {
    pub fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        puffin::profile_function!();
        log::debug!("Initialising billboard pipeline");
        let shader = Shader::new(
            graphics.clone(),
            include_str!("shaders/billboard.wgsl"),
            Some("billboard shader"),
        );

        let positions: [[f32; 3]; 4] = [
            [10.0, -10.0, 0.0],
            [-10.0, -10.0, 0.0],
            [10.0, 10.0, 0.0],
            [-10.0, 10.0, 0.0],
        ];

        let mut position_buffer = DynamicBuffer::new(
            &graphics.device,
            positions.len(),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            "billboard position buffer"
        );
        position_buffer.write(&graphics.device, &graphics.queue, &positions);

        let tex_coords: [[f32; 2]; 4] = [[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];

        let mut tex_coord_buffer = DynamicBuffer::new(
            &graphics.device,
            tex_coords.len(),
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            "billboard tex coords buffer"
        );
        tex_coord_buffer.write(
            &graphics.device,
            &graphics.queue,
            bytemuck::cast_slice(&tex_coords)
        );

        let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("billboard sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let uniform_bind_group_layout =
            graphics
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("billboard group layout"),
                    entries: &[
                        // transform
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // projection
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // t_diffuse
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        // s_diffuse
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let transform_buffer: UniformBuffer<Mat4> =
            UniformBuffer::new(&graphics.device, "billboard transform buffer");
        let projection_buffer: UniformBuffer<Mat4> =
            UniformBuffer::new(&graphics.device, "billboard projection buffer");

        {
            let mut registry = ASSET_REGISTRY.write();
            let handle = registry.solid_texture_rgba8(graphics.clone(), [0, 0, 0, 0], None); // make transparent for now
            let _ = registry.get_texture(handle);
        }

        let pipeline_layout =
            graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("billboard pipeline layout"),
                    bind_group_layouts: &[Some(&uniform_bind_group_layout)],
                    immediate_size: 0,
                });

        let format = { graphics.hdr.read().format().clone() };
        let pipeline = graphics
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("billboard render pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[
                        // positions
                        wgpu::VertexBufferLayout {
                            array_stride: (std::mem::size_of::<f32>() * 3) as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3],
                        },
                        // tex coords
                        wgpu::VertexBufferLayout {
                            array_stride: (std::mem::size_of::<f32>() * 2) as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: graphics.depth_texture.texture.format(),
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Greater),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: MultisampleState {
                    count: (*graphics.antialiasing.read()).into(),
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                cache: None,
                multiview_mask: None,
            });

        log::debug!("Created billboard pipeline");

        Self {
            pipeline,
            transform_buffer,
            projection_buffer,
            position_buffer,
            tex_coord_buffer,
            uniform_bind_group_layout,
            sampler,
        }
    }

    pub fn draw(
        &self,
        graphics: Arc<SharedGraphicsContext>,
        render_pass: &mut wgpu::RenderPass<'_>,
        transform: Mat4,
        projection: Mat4,
        texture_view: &wgpu::TextureView,
    ) {
        self.transform_buffer.write(&graphics.queue, &transform);
        self.projection_buffer.write(&graphics.queue, &projection);

        let uniform_bind_group = graphics
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("billboard bind group"),
                layout: &self.uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.transform_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.projection_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.position_buffer.buffer().slice(..));
        render_pass.set_vertex_buffer(1, self.tex_coord_buffer.buffer().slice(..));
        render_pass.set_bind_group(0, &uniform_bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
