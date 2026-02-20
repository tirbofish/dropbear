use std::hash::{DefaultHasher, Hash, Hasher};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, Device,
    Extent3d, Origin3d, Queue, RenderPass, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

/// A GPU texture that can be bound in shaders for rendering
///
/// Wraps a `wgpu::Texture`, its view, sampler, & bind group
#[derive(Debug, PartialEq)]
pub struct Texture {
    pub(crate) hash: u64,
    bind_group: BindGroup,
}

impl Texture {
    fn bytes_per_pixel(format: TextureFormat) -> Option<u32> {
        match format {
            TextureFormat::Rgba8Unorm
            | TextureFormat::Rgba8UnormSrgb
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Bgra8UnormSrgb => Some(4),
            TextureFormat::Rgba16Float
            | TextureFormat::Rgba16Unorm
            | TextureFormat::Rgba16Snorm
            | TextureFormat::Rgba16Uint
            | TextureFormat::Rgba16Sint => Some(8),
            TextureFormat::Rgba32Float | TextureFormat::Rgba32Uint | TextureFormat::Rgba32Sint => {
                Some(16)
            }
            _ => None,
        }
    }

    /// Creates a new texture from raw RGBA image data,
    /// uploads the data, & builds the bind group using the layout
    ///
    /// - `data`: Must be in tightly packed 8-bit RGBA format
    /// - `width`, `height`: Dimensions of the image in pixels
    pub fn from_bytes(
        device: &Device,
        queue: &Queue,
        bind_group_layout: &BindGroupLayout,
        data: &[u8],
        width: u32,
        height: u32,
        texture_format: TextureFormat,
    ) -> Self {
        log::debug!("Creating new texture");

        let bytes_per_pixel = Self::bytes_per_pixel(texture_format).unwrap_or(4);
        let expected_len = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(bytes_per_pixel as usize);
        if data.len() != expected_len {
            log::error!(
                "Texture data length {} does not match expected {} for {:?}",
                data.len(),
                expected_len,
                texture_format
            );
        }

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: texture_format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_pixel * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&Default::default());
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();

        log::debug!("Created new texture [{}]", hash);

        Self { hash, bind_group }
    }

    /// Creates a 1×1 white fallback texture
    ///
    /// Used when no valid texture is provided for a draw call
    pub fn create_default(
        device: &Device,
        queue: &Queue,
        layout: &BindGroupLayout,
        texture_format: TextureFormat,
    ) -> Self {
        log::debug!("Creating standard white texture");
        Self::from_bytes(
            device,
            queue,
            layout,
            &[255u8, 255, 255, 255],
            1,
            1,
            texture_format,
        )
    }

    /// Binds this texture at the given index in the render pass
    ///
    /// - `index` must match the bind group index used in the pipeline layout
    pub fn bind(&self, pass: &mut RenderPass, index: u32) {
        pass.set_bind_group(index, &self.bind_group, &[]);
    }
}
