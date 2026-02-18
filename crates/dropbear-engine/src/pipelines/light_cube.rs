use std::sync::Arc;
use std::mem::size_of;
use glam::DMat4;
use slank::include_slang;
use wgpu::{BufferAddress, CompareFunction, DepthBiasState, StencilState};
use crate::buffer::{StorageBuffer};
use crate::entity::{EntityTransform, Transform};
use crate::graphics::SharedGraphicsContext;
use crate::lighting::{Light, LightArrayUniform, LightComponent, MAX_LIGHTS};
use crate::model::{ModelVertex, Vertex};
use crate::pipelines::DropbearShaderPipeline;
use crate::shader::Shader;
use crate::texture::Texture;

pub struct LightCubePipeline {
    shader: Shader,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
    storage_buffer: Option<StorageBuffer<LightArrayUniform>>,
    /// Bind group, defined in `shaders/shader.wgsl` as @group(2)
    light_bind_group: wgpu::BindGroup,
}

impl DropbearShaderPipeline for LightCubePipeline {
    fn new(graphics: Arc<SharedGraphicsContext>) -> Self {
        let shader = Shader::from_slang(graphics.clone(), &slank::CompiledSlangShader::from_bytes("light cube", include_slang!("light_cube")));

        let pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("light cube pipeline layout"),
            bind_group_layouts: &[
                &graphics.layouts.camera_bind_group_layout,
                &graphics.layouts.light_cube_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let hdr_format = graphics.hdr.read().format();
        let pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("light cube pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    // model
                    LightCubeVertex::desc(),
                    // instance
                    InstanceInput::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: hdr_format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let storage_buffer = StorageBuffer::new(
            &graphics.device,
            "light cube pipeline storage buffer",
        );

        let light_buffer: &wgpu::Buffer = storage_buffer.buffer();

        let light_bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &graphics.layouts.light_array_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: Some("light array bind group"),
        });

        Self {
            shader,
            pipeline_layout,
            pipeline,
            storage_buffer: Some(storage_buffer),
            light_bind_group,
        }
    }

    fn shader(&self) -> &Shader {
        &self.shader
    }

    fn pipeline_layout(&self) -> &wgpu::PipelineLayout {
        &self.pipeline_layout
    }

    fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

impl LightCubePipeline {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.light_bind_group
    }

    pub fn update(&mut self, graphics: Arc<SharedGraphicsContext>, world: &hecs::World) {
        let mut light_array = LightArrayUniform::default();

        let mut light_index: usize = 0;

        for (light_component, s_trans, e_trans, light) in world
            .query::<(&LightComponent, Option<&Transform>, Option<&EntityTransform>, &mut Light)>()
            .iter()
        {
            let instance: InstanceInput = if let Some(transform) = e_trans {
                let sync_transform = transform.sync();
                sync_transform.matrix().into()
            } else if let Some(transform) = s_trans {
                transform.matrix().into()
            } else {
                light_component.to_transform().matrix().into()
            };

            light.instance_buffer.write(&graphics.device, &graphics.queue, &[instance]);

            if light_component.enabled && light_index < MAX_LIGHTS {
                let uniform = *light.uniform();

                light.buffer.write(&graphics.queue, &uniform);

                light_array.lights[light_index] = uniform;
                light_index += 1;
            }
        }

        light_array.light_count = light_index as u32;

        if let Some(buf) = &self.storage_buffer {
            buf.write(&graphics.queue, &light_array);
        } else {
            panic!("A storage buffer should have been created");
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        if let Some(s) = &self.storage_buffer {
            s.buffer()
        } else {
            panic!("A storage buffer should have been created");
        }
    }
}

pub struct LightCubeVertex;

impl LightCubeVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ModelVertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// As mapped in `shaders/light.slang` as
/// ```wgsl
/// struct InstanceInput {
///     @location(5) model_matrix_0: vec4<f32>,
///     @location(6) model_matrix_1: vec4<f32>,
///     @location(7) model_matrix_2: vec4<f32>,
///     @location(8) model_matrix_3: vec4<f32>,
/// }
/// ```
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceInput {
    pub model_matrix: [[f32; 4]; 4],
}

impl Vertex for InstanceInput {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceInput>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // model_matrix_0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // model_matrix_1
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // model_matrix_2
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // model_matrix_3
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Into<InstanceInput> for DMat4 {
    fn into(self) -> InstanceInput {
        InstanceInput {
            model_matrix: self.as_mat4().to_cols_array_2d(),
        }
    }
}
