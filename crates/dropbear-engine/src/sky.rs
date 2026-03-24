use crate::graphics::SharedGraphicsContext;
use crate::pipelines::{create_render_pipeline_ex};
use crate::texture::{Texture, TextureBuilder};
use image::codecs::hdr::HdrDecoder;
use std::io::Cursor;
use std::sync::Arc;

pub const DEFAULT_SKY_TEXTURE: &[u8] =
    include_bytes!("../../../resources/textures/kloofendal_48d_partly_cloudy_puresky_4k.hdr");

pub struct CubeTexture {
    texture: wgpu::Texture,
    sampler: wgpu::Sampler,
    view: wgpu::TextureView,
}

impl CubeTexture {
    pub fn create_2d(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        mip_level_count: u32,
        usage: wgpu::TextureUsages,
        mag_filter: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        puffin::profile_function!();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                // A cube has 6 sides, so we need 6 layers
                depth_or_array_layers: 6,
            },
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            array_layer_count: Some(6),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            texture,
            sampler,
            view,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }
}

pub struct HdrLoader {
    src_format: wgpu::TextureFormat,
    dst_format: wgpu::TextureFormat,
    equirect_layout: wgpu::BindGroupLayout,
    equirect_to_cubemap: wgpu::ComputePipeline,
    mip_gen_layout: wgpu::BindGroupLayout,
    mip_gen_pipeline: wgpu::ComputePipeline,
}

impl HdrLoader {
    pub fn new(device: &wgpu::Device) -> Self {
        puffin::profile_function!();
        let module =
            device.create_shader_module(wgpu::include_wgsl!("shaders/equirectangular.wgsl"));
        let src_format = wgpu::TextureFormat::Rgba32Float;
        let dst_format = wgpu::TextureFormat::Rgba16Float;
        let equirect_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("HdrLoader::equirect_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: dst_format,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&equirect_layout],
            push_constant_ranges: &[],
        });

        let equirect_to_cubemap =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("equirect_to_cubemap"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some("compute_equirect_to_cubemap"),
                compilation_options: Default::default(),
                cache: None,
            });

        let mip_gen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("HdrLoader::mip_gen_layout"),
            entries: &[
                // Source mip level – read via textureLoad (no ReadOnly storage needed,
                // avoids the poor cross-backend support for storage-image reads).
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                // Destination mip level – write via textureStore (storage WriteOnly).
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: dst_format,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
            ],
        });

        let mip_gen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&mip_gen_layout],
                push_constant_ranges: &[],
            });

        let mip_gen_module =
            device.create_shader_module(wgpu::include_wgsl!("shaders/mip_generator.wgsl"));

        let mip_gen_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("env cubemap mip generator"),
                layout: Some(&mip_gen_pipeline_layout),
                module: &mip_gen_module,
                entry_point: Some("generate_mip"),
                compilation_options: Default::default(),
                cache: None,
            });

        Self {
            equirect_to_cubemap,
            src_format,
            dst_format,
            equirect_layout,
            mip_gen_layout,
            mip_gen_pipeline,
        }
    }

    pub fn from_equirectangular_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        dst_size: u32,
        label: Option<&str>,
    ) -> anyhow::Result<CubeTexture> {
        puffin::profile_function!();
        let loader = Self::new(device);

        let hdr_decoder = HdrDecoder::new(Cursor::new(data))?;
        let meta = hdr_decoder.metadata();

        #[cfg(not(target_arch = "wasm32"))]
        let pixels = {
            let dec = image::DynamicImage::from_decoder(hdr_decoder)?;
            let pixels: Vec<[f32; 4]> = dec
                .into_rgba32f()
                .pixels()
                .map(|p| p.0)
                .collect();
            pixels
        };
        #[cfg(target_arch = "wasm32")]
        let pixels = hdr_decoder
            .read_image_native()?
            .into_iter()
            .map(|pix| {
                let rgb = pix.to_hdr();
                [rgb.0[0], rgb.0[1], rgb.0[2], 1.0f32]
            })
            .collect::<Vec<_>>();

        let src = TextureBuilder::new(&device)
            .size(meta.width, meta.height)
            .format(loader.src_format)
            .usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
            .mag_filter(wgpu::FilterMode::Linear)
            .build();

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &src.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &bytemuck::cast_slice(&pixels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(src.size.width * std::mem::size_of::<[f32; 4]>() as u32),
                rows_per_image: Some(src.size.height),
            },
            src.size,
        );

        // Calculate full mip chain count: floor(log2(dst_size)) + 1
        let mip_count = (dst_size as f32).log2().floor() as u32 + 1;

        let dst = CubeTexture::create_2d(
            device,
            dst_size,
            dst_size,
            loader.dst_format,
            mip_count,
            wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            wgpu::FilterMode::Linear,
            label,
        );

        let dst_view = dst.texture().create_view(&wgpu::TextureViewDescriptor {
            label,
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &loader.equirect_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&src.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&dst_view),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label,
            timestamp_writes: None,
        });

        let num_workgroups = (dst_size + 15) / 16;
        pass.set_pipeline(&loader.equirect_to_cubemap);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(num_workgroups, num_workgroups, 6);

        drop(pass);

        queue.submit([encoder.finish()]);

        for level in 1..mip_count {
            let src_view = dst.texture().create_view(&wgpu::TextureViewDescriptor {
                label: Some("mip src view"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_mip_level: level - 1,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(6),
                ..Default::default()
            });
            let dst_mip_view = dst.texture().create_view(&wgpu::TextureViewDescriptor {
                label: Some("mip dst view"),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                base_mip_level: level,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(6),
                ..Default::default()
            });

            let mip_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("mip gen bind group"),
                layout: &loader.mip_gen_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&dst_mip_view),
                    },
                ],
            });

            let mut mip_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("mip gen encoder"),
                });
            {
                let mut pass = mip_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("mip gen pass"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&loader.mip_gen_pipeline);
                pass.set_bind_group(0, &mip_bind_group, &[]);
                let mip_dim = (dst_size >> level).max(1);
                let workgroups = (mip_dim + 7) / 8;
                pass.dispatch_workgroups(workgroups, workgroups, 6);
            }
            queue.submit([mip_encoder.finish()]);
        }

        Ok(dst)
    }
}

pub struct SkyPipeline {
    pub texture: CubeTexture,
    pub pipeline: wgpu::RenderPipeline,
    pub camera_layout: wgpu::BindGroupLayout,
    pub environment_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    pub environment_bind_group: wgpu::BindGroup,
}

impl SkyPipeline {
    pub fn new(
        graphics: Arc<SharedGraphicsContext>,
        sky_texture: CubeTexture,
        camera_buffer: &wgpu::Buffer,
    ) -> Self {
        puffin::profile_function!();
        let camera_layout = graphics.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky camera bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let environment_layout = graphics.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky environment bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
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
            ],
        });

        let camera_bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky camera bind group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let environment_bind_group = graphics.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky environment bind group"),
            layout: &environment_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(sky_texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sky_texture.sampler()),
                },
            ],
        });

        let sky_pipeline = {
            let layout = graphics
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Sky Pipeline Layout"),
                    bind_group_layouts: &[
                        &camera_layout,
                        &environment_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let shader = wgpu::include_wgsl!("shaders/sky.wgsl");
            create_render_pipeline_ex(
                Some("sky render pipeline"),
                &graphics.device,
                &layout,
                graphics.hdr.read().format(),
                Some(Texture::DEPTH_FORMAT),
                &[],
                wgpu::PrimitiveTopology::TriangleList,
                shader,
                false,
                wgpu::CompareFunction::GreaterEqual,
                (*graphics.antialiasing.read()).into(),
            )
        };

        Self {
            texture: sky_texture,
            pipeline: sky_pipeline,
            camera_layout,
            environment_layout,
            camera_bind_group,
            environment_bind_group,
        }
    }
}
