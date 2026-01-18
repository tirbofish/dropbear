use std::{fs, path::PathBuf, sync::Arc};

use image::GenericImageView;
use serde::{Deserialize, Serialize};

use crate::graphics::SharedGraphicsContext;


/// As defined in `shaders.wgsl` as 
/// ```
/// @group(0) @binding(0)
/// var t_diffuse: texture_2d<f32>;
/// @group(0) @binding(1)
/// var s_diffuse: sampler;
/// @group(0) @binding(2)
/// var t_normal: texture_2d<f32>;
/// @group(0) @binding(3)
/// var s_normal: sampler;
/// ```
pub const TEXTURE_BIND_GROUP_LAYOUT: wgpu::BindGroupLayoutDescriptor<'_> = 
    wgpu::BindGroupLayoutDescriptor {
        entries: &[
            // t_diffuse
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            // s_diffuse
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            // t_normal
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            // s_normal
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("texture bind group layout"),
    };

#[derive(Clone)]
/// Describes a texture, like an image of some sort. Can be a normal texture on a model or a viewport or depth texture.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
}

impl Texture {
    /// Describes the depth format for all Texture related functions in WGPU to use. Makes life easier
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    /// Creates a new depth texture. This is an internal function.
    pub fn depth_texture(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1, // leave me alone
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            sampler,
            size,
            view
        }
    }

    /// Creates a viewport texture.
    ///  
    /// This is an internal function. 
    pub fn viewport(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1, // leave me alone
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            sampler,
            size,
            view,
        }
    }

    /// Loads the texture from a file. 
    pub async fn from_file(
        graphics: Arc<SharedGraphicsContext>,
        path: &PathBuf,
    ) -> anyhow::Result<Self> {
        let data = fs::read(path)?;
        Ok(Self::from_bytes(graphics.clone(), &data))
    }

    /// Loads the texture from bytes.
    /// 
    /// If you want more customisability in the texture being generated, you can use [Self::from_bytes_verbose]
    pub fn from_bytes(graphics: Arc<SharedGraphicsContext>, bytes: &[u8]) -> Self {
        Self::from_bytes_verbose(
            &graphics.device, 
            &graphics.queue, 
            bytes, 
            None, 
            None, 
            None,
            None
        )
    }

    /// Loads the texture from bytes, with options for more arguments. 
    /// 
    /// Requires more arguments. For a simpler usage, you should use [Self::from_bytes]
    pub fn from_bytes_verbose(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        dimensions: Option<(u32, u32)>,
        texture_descriptor: Option<wgpu::TextureDescriptor>,
        view_descriptor: Option<wgpu::TextureViewDescriptor>,
        sampler: Option<wgpu::SamplerDescriptor>,
    ) -> Self {
        let (diffuse_rgba, dimensions) = match image::load_from_memory(bytes) {
            Ok(image) => {
                let rgba = image.to_rgba8().into_raw();
                let dims = dimensions.unwrap_or_else(|| image.dimensions());
                (rgba, dims)
            }
            Err(err) => {
                if let Some(dims) = dimensions {
                    let expected_len = (dims.0 as usize)
                        .saturating_mul(dims.1 as usize)
                        .saturating_mul(4);
                    if bytes.len() == expected_len {
                        (bytes.to_vec(), dims)
                    } else {
                        log::error!(
                            "Texture decode failed ({:?}); expected {} bytes for raw RGBA ({}x{}), got {}. Falling back.",
                            err,
                            expected_len,
                            dims.0,
                            dims.1,
                            bytes.len()
                        );
                        (vec![255, 0, 255, 255], (1, 1))
                    }
                } else {
                    log::error!(
                        "Texture decode failed ({:?}) and no dimensions were provided; falling back to 1x1 magenta.",
                        err
                    );
                    (vec![255, 0, 255, 255], (1, 1))
                }
            }
        };
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let desc = texture_descriptor.unwrap_or_else(|| 
            wgpu::TextureDescriptor {
                label: Some("diffuse_texture"),
                size: size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Texture::TEXTURE_FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }
        );

        let texture = device.create_texture(&desc);

        let unpadded_bytes_per_row = 4 * size.width;
        let padded_bytes_per_row = (unpadded_bytes_per_row + wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1) & !(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1);
        debug_assert!(diffuse_rgba.len() >= (unpadded_bytes_per_row * size.height) as usize);

        if padded_bytes_per_row == unpadded_bytes_per_row {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &diffuse_rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(unpadded_bytes_per_row),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        } else {
            let mut padded = vec![0u8; (padded_bytes_per_row * size.height) as usize];
            let src_stride = unpadded_bytes_per_row as usize;
            let dst_stride = padded_bytes_per_row as usize;
            for row in 0..size.height as usize {
                let src_start = row * src_stride;
                let dst_start = row * dst_stride;
                padded[dst_start..dst_start + src_stride]
                    .copy_from_slice(&diffuse_rgba[src_start..src_start + src_stride]);
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &padded,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        let sampler_desc = if let Some(sampler) = sampler {
            sampler
        } else {
            wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        };

        let sampler = device.create_sampler(&sampler_desc);

        let view = texture.create_view(&view_descriptor.unwrap_or_default());

        Self {
            texture,
            sampler,
            size,
            view,
        }
    }

    pub fn sampler_from_wrap(wrap: TextureWrapMode) -> wgpu::SamplerDescriptor<'static> {
        wgpu::SamplerDescriptor {
            address_mode_u: wrap.into(),
            address_mode_v: wrap.into(),
            address_mode_w: wrap.into(),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize
)]
pub enum TextureWrapMode {
    Repeat,
    Clamp,
}

impl Default for TextureWrapMode {
    fn default() -> Self {
        Self::Repeat
    }
}

impl Into<wgpu::AddressMode> for TextureWrapMode {
    fn into(self) -> wgpu::AddressMode {
        match self {
            TextureWrapMode::Repeat => wgpu::AddressMode::Repeat,
            TextureWrapMode::Clamp => wgpu::AddressMode::ClampToEdge,
        }
    }
}

pub struct DropbearEngineLogo;

impl DropbearEngineLogo {
    /// Note: image size is 256x256
    pub const DROPBEAR_ENGINE_LOGO: &[u8] = include_bytes!("../../../resources/eucalyptus-editor.png");

    /// Generates the dropbear engine logo in a form that [winit::window::Icon] can accept. 
    /// 
    /// Returns (the bytes, width, height) in resp order. 
    pub fn generate() -> anyhow::Result<(Vec<u8>, u32, u32)> {
        let image = image::load_from_memory(Self::DROPBEAR_ENGINE_LOGO)?.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        Ok((rgba, width, height))
    }
}