use slank::{include_slang, utils::WgpuUtils};

use crate::{texture::Texture};

pub struct MipMapper {
    blit_mipmap: wgpu::RenderPipeline,
    blit_sampler: wgpu::Sampler,
    compute_pipeline: wgpu::ComputePipeline,
    storage_texture_layout: wgpu::BindGroupLayout,
    pub enabled: bool,
}

impl MipMapper {
    pub fn new(device: &wgpu::Device) -> Self {
        puffin::profile_function!();
        let blit_shader = device.create_shader_module(slank::CompiledSlangShader::from_bytes(
            "mipmap blit_shader", 
            include_slang!("blit_shader")
        ).create_wgpu_shader());

        // Keep this SRGB so we can render directly into the SRGB textures we create for materials.
        let blit_format = Texture::TEXTURE_FORMAT;
        let blit_mipmap = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mipmap blit render pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: None,
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: blit_format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw, // change if issues
                cull_mode: Some(wgpu::Face::Back),
                strip_index_format: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            cache: None,
            multiview: None,
        });

        let storage_texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Mipmapper::texture_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&storage_texture_layout],
            push_constant_ranges: &[],
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mipmap compute shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mipmap.wgsl").into()),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Mipmapper"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: Some("compute_mipmap"),
            compilation_options: Default::default(),
            cache: None,
        });

        let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            storage_texture_layout,
            compute_pipeline,
            blit_mipmap,
            blit_sampler,
            enabled: true,
        }
    }

    pub fn blit_mipmaps(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &Texture,
    ) -> anyhow::Result<()> {
        puffin::profile_function!();
        let texture = &texture.texture;

        match texture.format() {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {},
            _ => anyhow::bail!("Unsupported format {:?}", texture.format()),
        }

        if texture.mip_level_count() == 1 {
            return Ok(());
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("blit mipmap encoder"),
        });

        // We need to render to this texture, so if the supplied texture
        // isn't setup for rendering, we need to create a temporary one.
        let (mut src_view, maybe_temp) = if texture.usage().contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
            (
                texture.create_view(&wgpu::TextureViewDescriptor {
                    base_mip_level: 0,
                    mip_level_count: Some(1),
                    ..Default::default()
                }),
                None,
            )
        } else {
            // Create a temporary texture that can be rendered to since the
            // supplied texture can't be rendered to. It will be basically
            // identical to the original apart from the usage field and removing
            // sRGB from the format if it's present.
            let temp = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Mipmapper::blit_mipmaps::temp"),
                size: texture.size(),
                mip_level_count: texture.mip_level_count(),
                sample_count: texture.sample_count(),
                dimension: texture.dimension(),
                format: texture.format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            encoder.copy_texture_to_texture(
                texture.as_image_copy(),
                temp.as_image_copy(),
                temp.size(),
            );

            (
                temp.create_view(&wgpu::TextureViewDescriptor {
                    mip_level_count: Some(1),
                    ..Default::default()
                }),
                Some(temp),
            )
        };

        for mip in 1..texture.mip_level_count() {
            let dst_view = src_view
                .texture()
                .create_view(&wgpu::TextureViewDescriptor {
                    // What mip we want to render to
                    base_mip_level: mip,
                    // Like src_view we need to ignore other mips
                    mip_level_count: Some(1),
                    ..Default::default()
                });

            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &self.blit_mipmap.get_bind_group_layout(0),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.blit_sampler),
                    },
                ],
            });

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dst_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.blit_mipmap);
            pass.set_bind_group(0, &texture_bind_group, &[]);
            pass.draw(0..3, 0..1);

            // Make sure that we use the mip we just generated for the
            // next iteration.
            src_view = dst_view;
        }

        // If we created a temporary texture, now we need to copy it back
        // into the original.
        if let Some(temp) = maybe_temp {
            let mut size = temp.size();
            for mip_level in 0..temp.mip_level_count() {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        mip_level,
                        ..temp.as_image_copy()
                    },
                    wgpu::TexelCopyTextureInfo {
                        mip_level,
                        ..texture.as_image_copy()
                    },
                    size,
                );

                // Each mipmap is half the size of the original,
                // so we need to half the copy size as well.
                size.width /= 2;
                size.height /= 2;
            }
        }

        // Submit directly (this is a standalone utility).
        let command_buffer = encoder.finish();
        queue.submit(std::iter::once(command_buffer));

        Ok(())
    }

    pub fn compute_mipmaps(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &Texture,
    ) -> anyhow::Result<()> {
        puffin::profile_function!();
        let texture = &texture.texture;

        match texture.format() {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => {}
            _ => anyhow::bail!("Unsupported format {:?}", texture.format()),
        }

        if texture.mip_level_count() == 1 {
            return Ok(());
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("compute mipmap encoder"),
        });

        // Create temp texture if supplied texture isn't setup for use
        // as a storage texture
        let (mut src_view, maybe_temp) = if texture
            .usage()
            .contains(wgpu::TextureUsages::STORAGE_BINDING)
        {
            (
                texture.create_view(&wgpu::TextureViewDescriptor {
                    mip_level_count: Some(1),
                    ..Default::default()
                }),
                None,
            )
        } else {
            let temp = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Mipmapper::compute_mipmaps::temp"),
                size: texture.size(),
                mip_level_count: texture.mip_level_count(),
                sample_count: texture.sample_count(),
                dimension: texture.dimension(),
                format: texture.format().remove_srgb_suffix(),
                usage: wgpu::TextureUsages::STORAGE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });

            encoder.copy_texture_to_texture(
                texture.as_image_copy(),
                temp.as_image_copy(),
                temp.size(),
            );

            (
                temp.create_view(&wgpu::TextureViewDescriptor {
                    mip_level_count: Some(1),
                    ..Default::default()
                }),
                Some(temp),
            )
        };

        let dispatch_x = texture.width().div_ceil(16);
        let dispatch_y = texture.height().div_ceil(16);

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.compute_pipeline);
            for mip in 1..texture.mip_level_count() {
                let dst_view = src_view
                    .texture()
                    .create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: mip,
                        mip_level_count: Some(1),
                        ..Default::default()
                    });
                let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.storage_texture_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&dst_view),
                        },
                    ],
                });
                pass.set_bind_group(0, &texture_bind_group, &[]);
                pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);

                src_view = dst_view;
            }
        }

        if let Some(temp) = maybe_temp {
            let mut size = temp.size();
            for mip_level in 0..temp.mip_level_count() {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        mip_level,
                        ..temp.as_image_copy()
                    },
                    wgpu::TexelCopyTextureInfo {
                        mip_level,
                        ..texture.as_image_copy()
                    },
                    size,
                );

                // Each mipmap is half the size of the original
                size.width /= 2;
                size.height /= 2;
            }
        }

        let command_buffer = encoder.finish();
        queue.submit(std::iter::once(command_buffer));

        Ok(())
    }
}