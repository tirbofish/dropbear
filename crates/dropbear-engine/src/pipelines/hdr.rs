use crate::pipelines::create_render_pipeline;
use crate::texture::{Texture, TextureBuilder};
use wgpu::{Operations, ShaderModuleDescriptor};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PostProcessUniforms {
    gamma: f32,
    _pad: [f32; 3],
}

pub struct HdrPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: Texture,
    msaa_texture: Option<Texture>,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    layout: wgpu::BindGroupLayout,
    antialiasing: crate::multisampling::AntiAliasingMode,
    gamma: f32,
    gamma_buffer: wgpu::Buffer,
}

impl HdrPipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        output_format: wgpu::TextureFormat,
        antialiasing: crate::multisampling::AntiAliasingMode,
    ) -> Self {
        let width = config.width;
        let height = config.height;

        // We could use `Rgba32Float`, but that requires some extra
        // features to be enabled for rendering.
        let format = wgpu::TextureFormat::Rgba16Float;

        let texture = TextureBuilder::new(device)
            .size(width, height)
            .format(format)
            .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT)
            .mag_filter(wgpu::FilterMode::Nearest)
            .label("Hdr::texture")
            .build();

        let msaa_texture = match antialiasing {
            crate::multisampling::AntiAliasingMode::None => None,
            _ => Some(
                TextureBuilder::new(device)
                    .size(width, height)
                    .format(format)
                    .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
                    .mag_filter(wgpu::FilterMode::Nearest)
                    .label("Hdr::texture")
                    .antialiasing(antialiasing)
                    .build(),
            ),
        };

        let default_gamma = 2.2_f32;
        let gamma_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Hdr::gamma_buffer"),
            contents: bytemuck::bytes_of(&PostProcessUniforms {
                gamma: default_gamma,
                _pad: [0.0; 3],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hdr::layout"),
            entries: &[
                // This is the HDR texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: gamma_buffer.as_entire_binding(),
                },
            ],
        });

        // We'll cover the shader next
        let source = wesl::Wesl::new("src/shaders")
            .add_package(&crate::shader::code::PACKAGE)
            .compile(&"dropbear_shaders::hdr".parse().unwrap())
            .inspect_err(|e| {
                panic!("{e}");
            })
            .unwrap()
            .to_string();


        let shader = ShaderModuleDescriptor {
            label: Some("hdr shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let pipeline = create_render_pipeline(
            Some("hdr render pipeline"),
            device,
            &pipeline_layout,
            output_format,
            None,
            // We'll use some math to generate the vertex data in
            // the shader, so we don't need any vertex buffers
            &[],
            wgpu::PrimitiveTopology::TriangleList,
            shader,
            1,
        );

        Self {
            pipeline,
            bind_group,
            layout,
            texture,
            msaa_texture,
            width,
            height,
            format,
            antialiasing,
            gamma: default_gamma,
            gamma_buffer,
        }
    }

    /// Returns the current gamma exponent.
    pub fn gamma(&self) -> f32 {
        self.gamma
    }

    /// Sets the gamma correction exponent applied after ACES tonemapping.
    ///
    /// - `2.2` — standard gamma for non-sRGB render targets (default).
    /// - `1.0` — no correction; use when the render target is an sRGB-format
    ///   texture so the GPU handles gamma encoding automatically.
    pub fn set_gamma(&mut self, queue: &wgpu::Queue, gamma: f32) {
        self.gamma = gamma;
        queue.write_buffer(
            &self.gamma_buffer,
            0,
            bytemuck::bytes_of(&PostProcessUniforms {
                gamma,
                _pad: [0.0; 3],
            }),
        );
    }

    /// Resize the HDR texture
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        antialiasing: Option<crate::multisampling::AntiAliasingMode>,
    ) {
        self.antialiasing = antialiasing.unwrap_or(self.antialiasing);

        self.texture = TextureBuilder::new(device)
            .size(width, height)
            .format(self.format)
            .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT)
            .mag_filter(wgpu::FilterMode::Nearest)
            .label("Hdr::texture")
            .build();

        self.msaa_texture = match self.antialiasing {
            crate::multisampling::AntiAliasingMode::None => None,
            _ => Some(
                TextureBuilder::new(device)
                    .size(width, height)
                    .format(self.format)
                    .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
                    .mag_filter(wgpu::FilterMode::Nearest)
                    .label("Hdr::texture")
                    .antialiasing(self.antialiasing)
                    .build(),
            ),
        };
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.gamma_buffer.as_entire_binding(),
                },
            ],
        });
        self.width = width;
        self.height = height;
    }

    /// The view to render INTO (MSAA if enabled, otherwise the HDR texture directly)
    pub fn render_view(&self) -> &wgpu::TextureView {
        match &self.msaa_texture {
            Some(msaa) => &msaa.view,
            None => &self.texture.view,
        }
    }

    /// The resolve target — only Some() when MSAA is active
    pub fn resolve_target(&self) -> Option<&wgpu::TextureView> {
        match &self.msaa_texture {
            Some(_) => Some(&self.texture.view),
            None => None,
        }
    }

    /// The resolved HDR texture for post-processing (always single-sample)
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture.view
    }

    /// The format of the HDR texture
    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// This renders the internal HDR texture to the [TextureView]
    /// supplied as parameter.
    pub fn process(&self, encoder: &mut wgpu::CommandEncoder, output: &wgpu::TextureView) {
        puffin::profile_function!();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hdr::process"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
