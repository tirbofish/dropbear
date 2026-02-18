use std::{fs, path::PathBuf, sync::Arc};

use image::GenericImageView;
use serde::{Deserialize, Serialize};
use crate::asset::AssetRegistry;
use crate::graphics::SharedGraphicsContext;
use crate::utils::{ResourceReference, ToPotentialString};

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
    pub label: Option<String>,
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
    pub hash: Option<u64>,
    pub reference: Option<ResourceReference>
}

impl Texture {
    /// Describes the depth format for all Texture related functions in WGPU to use. Makes life easier
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    pub fn create_2d_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        mag_filter: wgpu::FilterMode,
        label: Option<&str>,
    ) -> Self {
        puffin::profile_function!(label.unwrap_or("create 2d texture"));
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        Self::create_texture(
            device,
            label,
            size,
            format,
            usage,
            wgpu::TextureDimension::D2,
            mag_filter,
        )
    }

    pub fn create_texture(
        device: &wgpu::Device,
        label: Option<&str>,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        dimension: wgpu::TextureDimension,
        mag_filter: wgpu::FilterMode,
    ) -> Self {
        puffin::profile_function!(label.unwrap_or("create texture"));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            label: label.and_then(|v| Some(v.to_string())),
            texture,
            view,
            sampler,
            size,
            hash: None,
            reference: None,
        }
    }

    /// Creates a new depth texture. This is an internal function.
    pub fn depth_texture(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        label: Option<&str>,
    ) -> Self {
        puffin::profile_function!(label.unwrap_or("depth texture"));
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
            view,
            label: label.to_potential_string(),
            hash: None,
            reference: None,
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
        puffin::profile_function!(label.unwrap_or("viewport texture"));
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
            format: config.format.add_srgb_suffix(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };

        let texture = device.create_texture(&desc);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            label: label.to_potential_string(),
            texture,
            sampler,
            size,
            view,
            hash: None,
            reference: None,
        }
    }

    /// Loads the texture from a file. 
    pub async fn from_file(
        graphics: Arc<SharedGraphicsContext>,
        path: &PathBuf,
        label: Option<&str>,
    ) -> anyhow::Result<Self> {
        puffin::profile_function!(label.unwrap_or(""));
        let data = fs::read(path)?;
        let mut result = Self::from_bytes(graphics.clone(), &data, label);
        result.reference = Some(ResourceReference::from_path(path)?);
        Ok(result)
    }

    /// Loads the texture from bytes.
    /// 
    /// If you want more customisability in the texture being generated, you can use [Self::from_bytes_verbose]
    pub fn from_bytes(graphics: Arc<SharedGraphicsContext>, bytes: &[u8], label: Option<&str>) -> Self {
        puffin::profile_function!(label.unwrap_or(""));
        Self::from_bytes_verbose_mipmapped(graphics, bytes, None, None, None, label)
    }

    /// Loads the texture from bytes and generates mipmaps on the GPU.
    ///
    /// This is the recommended constructor for any sampled texture used for rendering.
    pub fn from_bytes_verbose_mipmapped(
        graphics: Arc<SharedGraphicsContext>,
        bytes: &[u8],
        dimensions: Option<(u32, u32)>,
        view_descriptor: Option<wgpu::TextureViewDescriptor>,
        sampler: Option<wgpu::SamplerDescriptor>,
        label: Option<&str>,
    ) -> Self {
        puffin::profile_function!(label.unwrap_or(""));
        let texture = Self::from_bytes_verbose(
            graphics.clone(),
            bytes,
            dimensions,
            None,
            view_descriptor,
            sampler,
            label,
        );

        if let Err(err) = graphics
            .mipmapper
            .compute_mipmaps(&graphics.device, &graphics.queue, &texture)
        {
            log_once::warn_once!("Failed to generate mipmaps: {}", err);
        }

        texture
    }

    /// Loads the texture from bytes, with options for more arguments. 
    /// 
    /// Requires more arguments. For a simpler usage, you should use [Self::from_bytes]
    pub fn from_bytes_verbose(
        graphics: Arc<SharedGraphicsContext>,
        bytes: &[u8],
        dimensions: Option<(u32, u32)>,
        _texture_descriptor: Option<wgpu::TextureDescriptor>,
        view_descriptor: Option<wgpu::TextureViewDescriptor>,
        sampler: Option<wgpu::SamplerDescriptor>,
        label: Option<&str>,
    ) -> Self {
        puffin::profile_function!(label.unwrap_or(""));
        if let Some(l) = label {
            log::debug!("Loading texture: {l}");
        }

        let hash = AssetRegistry::hash_bytes(bytes);
        
        let (diffuse_rgba, dimensions) = {
            puffin::profile_scope!("load from memory image");
            match image::load_from_memory(bytes) {
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
                            "Texture [{:?}] decode failed ({:?}); expected {} bytes for raw RGBA ({}x{}), got {}. Falling back.",
                            label,
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
                        "Texture [{:?}] decode failed ({:?}) and no dimensions were provided; falling back to 1x1 magenta.",
                        label,
                        err
                    );
                        (vec![255, 0, 255, 255], (1, 1))
                    }
                }
            }
        };

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let mip_level_count = size.width.min(size.height).ilog2() + 1;
        log::debug!("Mip level count [{:?}]: {}", label, mip_level_count);

        let texture = graphics.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("{:?} diffuse blit texture", label).as_str()),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let unpadded_bytes_per_row = 4 * size.width;
        let padded_bytes_per_row = (unpadded_bytes_per_row + wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1) & !(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1);
        debug_assert!(diffuse_rgba.len() >= (unpadded_bytes_per_row * size.height) as usize);

        if padded_bytes_per_row == unpadded_bytes_per_row {
            puffin::profile_scope!("write to texture");
            graphics.queue.write_texture(
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
            puffin::profile_scope!("write to texture");
            let mut padded = vec![0u8; (padded_bytes_per_row * size.height) as usize];
            let src_stride = unpadded_bytes_per_row as usize;
            let dst_stride = padded_bytes_per_row as usize;
            for row in 0..size.height as usize {
                let src_start = row * src_stride;
                let dst_start = row * dst_stride;
                padded[dst_start..dst_start + src_stride]
                    .copy_from_slice(&diffuse_rgba[src_start..src_start + src_stride]);
            }

            graphics.queue.write_texture(
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
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }
        };

        let sampler = graphics.device.create_sampler(&sampler_desc);

        let view = texture.create_view(&view_descriptor.unwrap_or_default());

        Self {
            label: label.to_potential_string(),
            texture,
            sampler,
            size,
            view,
            hash: Some(hash),
            reference: Some(ResourceReference::from_bytes(bytes)),
        }
    }

    pub fn sampler_from_wrap(wrap: TextureWrapMode) -> wgpu::SamplerDescriptor<'static> {
        wgpu::SamplerDescriptor {
            address_mode_u: wrap.into(),
            address_mode_v: wrap.into(),
            address_mode_w: wrap.into(),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
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
        puffin::profile_function!("generate dropbear engine logo");
        let image = image::load_from_memory(Self::DROPBEAR_ENGINE_LOGO)?.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        Ok((rgba, width, height))
    }
}