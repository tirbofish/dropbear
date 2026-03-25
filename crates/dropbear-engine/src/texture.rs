use std::sync::Arc;

use crate::asset::AssetRegistry;
use crate::graphics::SharedGraphicsContext;
use crate::utils::{ResourceReference};
use image::{DynamicImage, GenericImageView, RgbaImage};
use uuid::Uuid;
use rkyv::Archive;
use serde::{Deserialize, Serialize};
use wgpu::{SamplerDescriptor, TextureAspect, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension};
use crate::multisampling::{AntiAliasingMode};

/// Describes a texture, like an image of some sort. Can be a normal texture on a model or a viewport or depth texture.
pub struct Texture {
    pub label: Option<String>,
    pub texture: wgpu::Texture,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub view: wgpu::TextureView,
    pub hash: Option<u64>,
    pub reference: Option<ResourceReference>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TextureBuilder<'a> {
    #[serde(skip)]
    device: Option<&'a wgpu::Device>,
    #[serde(skip)]
    graphics: Option<Arc<SharedGraphicsContext>>,

    width: u32,
    height: u32,
    depth_or_array_layers: u32,

    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    dimension: wgpu::TextureDimension,
    sample_count: u32,
    mip_level_count: u32,
    auto_mip: bool,

    mag_filter: wgpu::FilterMode,
    min_filter: wgpu::FilterMode,
    mipmap_filter: wgpu::FilterMode,
    wrap_mode: TextureWrapMode,
    compare: Option<wgpu::CompareFunction>,
    lod_min_clamp: f32,
    lod_max_clamp: f32,

    view_descriptor: Option<SerTextureViewDescriptor>,

    label: Option<&'a str>,
    mime_type: Option<String>,

    source: TextureSource,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
struct SerTextureViewDescriptor {
    pub label: Option<String>,
    pub format: Option<TextureFormat>,
    pub dimension: Option<TextureViewDimension>,
    pub usage: Option<TextureUsages>,
    pub aspect: TextureAspect,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
}

impl<'a> From<wgpu::TextureViewDescriptor<'a>> for SerTextureViewDescriptor {
    fn from(value: TextureViewDescriptor<'a>) -> Self {
        Self {
            label: value.label.map(|s| s.to_string()),
            format: value.format,
            dimension: value.dimension,
            usage: value.usage,
            aspect: value.aspect,
            base_mip_level: value.base_mip_level,
            mip_level_count: value.mip_level_count,
            base_array_layer: value.base_array_layer,
            array_layer_count: value.array_layer_count,
        }
    }
}

impl<'a> From<SerTextureViewDescriptor> for TextureViewDescriptor<'a> {
    fn from(value: SerTextureViewDescriptor) -> Self {
        Self {
            label: value.label.map(|v| Box::leak(v.into_boxed_str()) as &str),
            format: value.format,
            dimension: value.dimension,
            usage: value.usage,
            aspect: value.aspect,
            base_mip_level: value.base_mip_level,
            mip_level_count: value.mip_level_count,
            base_array_layer: value.base_array_layer,
            array_layer_count: value.array_layer_count,
        }
    }
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
enum TextureSource {
    #[default]
    Empty,
    Image {
        image: Image,
        hash: u64,
        reference: ResourceReference,
    },
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Image {
    width: u32,
    height: u32,
    pixel_data: Arc<[u8]>,
}

impl Image {
    fn from_dynamic(image: &DynamicImage) -> Self {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Self {
            width,
            height,
            pixel_data: Arc::<[u8]>::from(rgba.into_raw()),
        }
    }

    fn to_dynamic(&self) -> DynamicImage {
        if let Some(rgba) = RgbaImage::from_raw(self.width, self.height, self.pixel_data.to_vec()) {
            DynamicImage::ImageRgba8(rgba)
        } else {
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                1,
                1,
                image::Rgba([255, 0, 255, 255]),
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TextureReference {
    /// A solid RGBA colour literal — no file backing needed.
    RGBAColour([f32; 4]),
    /// UUID of an asset tracked by a `.eucmeta` sidecar file.
    AssetUuid(Uuid),
}

impl<'a> TextureBuilder<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self {
            device: Some(device),
            graphics: None,
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
            format: Texture::TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            dimension: wgpu::TextureDimension::D2,
            sample_count: 1,
            mip_level_count: 1,
            auto_mip: false,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            wrap_mode: TextureWrapMode::Repeat,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            view_descriptor: None,
            label: None,
            mime_type: None,
            source: TextureSource::Empty,
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn size_from_config(mut self, config: &'a wgpu::SurfaceConfiguration) -> Self {
        self.width = config.width.max(1);
        self.height = config.height.max(1);
        self
    }

    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.format = format;
        self
    }

    pub fn usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.usage = usage;
        self
    }

    /// preset: depth_texture()
    pub fn depth(mut self, config: &'a wgpu::SurfaceConfiguration, antialiasing: AntiAliasingMode) -> Self {
        self.source = TextureSource::Empty;
        self.width = config.width.max(1);
        self.height = config.height.max(1);
        self.format = Texture::DEPTH_FORMAT;
        self.usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        self.sample_count = antialiasing.into();
        self.mag_filter = wgpu::FilterMode::Linear;
        self.min_filter = wgpu::FilterMode::Linear;
        self.mipmap_filter = wgpu::FilterMode::Nearest;
        self.compare = Some(wgpu::CompareFunction::LessEqual);
        self.lod_min_clamp = 0.0;
        self.lod_max_clamp = 100.0;
        self
    }

    /// preset: viewport()
    pub fn viewport(mut self, config: &'a wgpu::SurfaceConfiguration) -> Self {
        self.source = TextureSource::Empty;
        self.width = config.width.max(1);
        self.height = config.height.max(1);
        self.format = config.format.add_srgb_suffix();
        self.usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        self.mag_filter = wgpu::FilterMode::Linear;
        self.min_filter = wgpu::FilterMode::Linear;
        self.mipmap_filter = wgpu::FilterMode::Nearest;
        self
    }

    pub fn render_target(mut self) -> Self {
        self.usage = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        self
    }

    pub fn mag_filter(mut self, filter: wgpu::FilterMode) -> Self {
        self.mag_filter = filter;
        self
    }

    pub fn wrap_mode(mut self, wrap: TextureWrapMode) -> Self {
        self.wrap_mode = wrap;
        self
    }

    pub fn antialiasing(mut self, aa: AntiAliasingMode) -> Self {
        self.sample_count = aa.into();
        self
    }

    pub fn sampler(mut self, sampler: SamplerDescriptor) -> Self {
        self.wrap_mode = sampler.address_mode_u.into();
        self.mag_filter = sampler.mag_filter;
        self.min_filter = sampler.min_filter;
        self.compare = sampler.compare;
        self.lod_max_clamp = sampler.lod_max_clamp;
        self.lod_min_clamp = sampler.lod_min_clamp;
        self
    }

    pub fn with_bytes(mut self, graphics: Arc<SharedGraphicsContext>, bytes: &'a [u8]) -> Self {
        self.graphics = Some(graphics);
        let hash = AssetRegistry::hash_bytes(bytes);
        let requested_dimensions = Some((self.width, self.height)).filter(|&d| d != (1, 1));

        let image = match image::load_from_memory(bytes) {
            Ok(image) => image,
            Err(err) => {
                if let Some((width, height)) = requested_dimensions {
                    let expected_len = (width as usize)
                        .saturating_mul(height as usize)
                        .saturating_mul(4);
                    if bytes.len() == expected_len {
                        if let Some(rgba) = RgbaImage::from_raw(width, height, bytes.to_vec()) {
                            DynamicImage::ImageRgba8(rgba)
                        } else {
                            log::error!(
                                "Texture [{:?}] decode failed ({:?}); raw RGBA reconstruction failed for dimensions {}x{}. Falling back.",
                                self.label,
                                err,
                                width,
                                height
                            );
                            DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                                1,
                                1,
                                image::Rgba([255, 0, 255, 255]),
                            ))
                        }
                    } else {
                        log::error!(
                            "Texture [{:?}] decode failed ({:?}); expected {} bytes for raw RGBA ({}x{}), got {}. Falling back.",
                            self.label,
                            err,
                            expected_len,
                            width,
                            height,
                            bytes.len()
                        );
                        DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                            1,
                            1,
                            image::Rgba([255, 0, 255, 255]),
                        ))
                    }
                } else {
                    log::error!(
                        "Texture [{:?}] decode failed ({:?}) and no dimensions were provided; falling back to 1x1 magenta.",
                        self.label,
                        err
                    );
                    DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                        1,
                        1,
                        image::Rgba([255, 0, 255, 255]),
                    ))
                }
            }
        };

        self.source = TextureSource::Image {
            image: Image::from_dynamic(&image),
            hash,
            reference: ResourceReference::from_bytes(bytes),
        };
        self.auto_mip = true;
        self.mipmap_filter = wgpu::FilterMode::Linear;
        self.usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC;
        self
    }

    pub fn with_raw_pixels(mut self, graphics: Arc<SharedGraphicsContext>, pixels: &'a [u8]) -> Self {
        self.graphics = Some(graphics);
        let hash = AssetRegistry::hash_bytes(pixels);

        let dimensions = (self.width, self.height);
        let expected_len = (dimensions.0 as usize)
            .saturating_mul(dimensions.1 as usize)
            .saturating_mul(4);

        let image = if pixels.len() == expected_len {
            if let Some(rgba) = RgbaImage::from_raw(dimensions.0, dimensions.1, pixels.to_vec()) {
                DynamicImage::ImageRgba8(rgba)
            } else {
                log::error!(
                    "Texture [{:?}] raw RGBA reconstruction failed for dimensions ({}x{}). Falling back to 1x1 magenta.",
                    self.label,
                    dimensions.0,
                    dimensions.1
                );
                DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                    1,
                    1,
                    image::Rgba([255, 0, 255, 255]),
                ))
            }
        } else {
            log::error!(
                "Texture [{:?}] raw pixel byte length {} does not match expected {} for RGBA8 ({}x{}). Falling back.",
                self.label,
                pixels.len(),
                expected_len,
                dimensions.0,
                dimensions.1
            );
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                1,
                1,
                image::Rgba([255, 0, 255, 255]),
            ))
        };

        self.source = TextureSource::Image {
            image: Image::from_dynamic(&image),
            hash,
            reference: ResourceReference::from_bytes(pixels),
        };
        self.auto_mip = true;
        self.mipmap_filter = wgpu::FilterMode::Linear;
        self.usage = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC;
        self
    }

    pub fn with_auto_mip(mut self) -> Self {
        self.auto_mip = true;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn mime_type(mut self, mime: &str) -> Self {
        self.mime_type = Some(mime.to_string());
        self
    }

    pub fn view_descriptor(mut self, desc: wgpu::TextureViewDescriptor<'a>) -> Self {
        self.view_descriptor = Some(desc.into());
        self
    }

    pub fn build(self) -> Texture {
        puffin::profile_function!(self.label.unwrap_or("TextureBuilder::build"));

        let view_desc: Option<wgpu::TextureViewDescriptor<'_>> = self.view_descriptor.clone().and_then(|v| Some(v.into()));
        let Some(device) = self.device else {
            panic!("TextureBuilder::build() requires a device, and it should be provided to have this to exist. weird...")
        };

        match &self.source {
            TextureSource::Image { image, hash, reference } => {
                let graphics = self
                    .graphics
                    .as_ref()
                    .expect("with_data() requires graphics context");
                let requested_dimensions = Some((self.width, self.height)).filter(|&d| d != (1, 1));

                let mut image = image.to_dynamic();
                if let Some((width, height)) = requested_dimensions {
                    if image.width() != width || image.height() != height {
                        image = image.resize_exact(width, height, image::imageops::FilterType::Triangle);
                    }
                }

                let rgba = image.to_rgba8().into_raw();
                let dimensions = image.dimensions();

                let size = wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                };

                let mip_level_count = self.compute_mip_level_count(size);
                let texture = self.create_texture(&graphics.device, size, self.format, mip_level_count);
                Self::upload_level0(&graphics.queue, &texture, size, &rgba, 4);
                self.finish_uploaded_texture(
                    &graphics,
                    texture,
                    size,
                    *hash,
                    reference.clone(),
                )
            }
            _ => {
                let size = wgpu::Extent3d {
                    width: self.width,
                    height: self.height,
                    depth_or_array_layers: self.depth_or_array_layers,
                };

                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: self.label,
                    size,
                    mip_level_count: self.mip_level_count,
                    sample_count: self.sample_count,
                    dimension: self.dimension,
                    format: self.format,
                    usage: self.usage,
                    view_formats: &[],
                });

                let view = texture.create_view(
                    &view_desc.unwrap_or_default()
                );
                let sampler = device.create_sampler(&self.build_sampler_desc());

                Texture {
                    label: self.label.map(|s| s.to_string()),
                    texture,
                    view,
                    sampler,
                    size,
                    hash: None,
                    reference: None,
                }
            }
        }
    }

    fn compute_mip_level_count(&self, size: wgpu::Extent3d) -> u32 {
        if self.auto_mip {
            size.width.min(size.height).ilog2() + 1
        } else {
            self.mip_level_count
        }
    }

    fn create_texture(
        &self,
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        mip_level_count: u32,
    ) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: self.label,
            size,
            mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format,
            usage: self.usage,
            view_formats: &[],
        })
    }

    fn upload_level0(
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        size: wgpu::Extent3d,
        pixels: &[u8],
        bytes_per_pixel: u32,
    ) {
        let unpadded_bytes_per_row = bytes_per_pixel * size.width;
        let padded_bytes_per_row = (unpadded_bytes_per_row + wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            - 1)
            & !(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT - 1);

        if padded_bytes_per_row == unpadded_bytes_per_row {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                pixels,
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
                    .copy_from_slice(&pixels[src_start..src_start + src_stride]);
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
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
    }

    fn finish_uploaded_texture(
        &self,
        graphics: &SharedGraphicsContext,
        texture: wgpu::Texture,
        size: wgpu::Extent3d,
        hash: u64,
        reference: ResourceReference,
    ) -> Texture {
        let sampler_desc = self.build_sampler_desc();
        let view_descriptor: wgpu::TextureViewDescriptor<'_> = self.view_descriptor.clone().and_then(|v| Some(v.into())).unwrap_or_default();
        let view = texture.create_view(&view_descriptor);
        let sampler = graphics.device.create_sampler(&sampler_desc);

        let built = Texture {
            label: self.label.map(|s| s.to_string()),
            texture,
            sampler,
            size,
            view,
            hash: Some(hash),
            reference: Some(reference),
        };

        if self.auto_mip {
            if let Err(err) = graphics
                .mipmapper
                .compute_mipmaps(&graphics.device, &graphics.queue, &built)
            {
                log_once::warn_once!("Failed to generate mipmaps: {}", err);
            }
        }

        built
    }

    fn build_sampler_desc(&self) -> wgpu::SamplerDescriptor<'static> {
        let addr: wgpu::AddressMode = self.wrap_mode.into();
        wgpu::SamplerDescriptor {
            address_mode_u: addr,
            address_mode_v: addr,
            address_mode_w: addr,
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_filter: self.mipmap_filter,
            compare: self.compare,
            lod_min_clamp: self.lod_min_clamp,
            lod_max_clamp: self.lod_max_clamp,
            ..Default::default()
        }
    }
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const TEXTURE_FORMAT_BASE: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
    pub const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Archive, rkyv::Serialize, rkyv::Deserialize)]
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

impl Into<TextureWrapMode> for wgpu::AddressMode {
    fn into(self) -> TextureWrapMode {
        match self {
            wgpu::AddressMode::ClampToEdge => TextureWrapMode::Clamp,
            wgpu::AddressMode::Repeat => TextureWrapMode::Repeat,
            wgpu::AddressMode::MirrorRepeat => TextureWrapMode::Repeat,
            wgpu::AddressMode::ClampToBorder => TextureWrapMode::Clamp,
        }
    }
}

pub struct DropbearEngineLogo;

impl DropbearEngineLogo {
    /// Note: image size is 256x256
    pub const DROPBEAR_ENGINE_LOGO: &[u8] =
        include_bytes!("../../../resources/eucalyptus-editor.png");

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
