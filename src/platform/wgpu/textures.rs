//! WGPU D2Array texture creation, mipmap generation, and runtime atlas uploads.

use crate::features::rendering::atlas::{TextureAtlas, TextureId, TEX_SIZE};

use super::types::{TEXTURE_LAYER_COUNT, VulkanContext};

impl VulkanContext {
    /// Maps a [`TextureId`] to its slice index inside the GPU `D2Array` texture.
    #[inline(always)]
    pub const fn texture_layer(id: TextureId) -> u32 {
        id.layer()
    }

    /// Maps a GPU `D2Array` slice index back to its [`TextureId`].
    #[inline(always)]
    pub const fn texture_id_from_layer(layer: u32) -> TextureId {
        TextureId::from_layer(layer)
    }

    /// Re-uploads every atlas layer and mipmap chain after texture settings change.
    pub fn update_textures(&mut self, atlas: &TextureAtlas) {
        for layer in 0..TEXTURE_LAYER_COUNT {
            let texels = &atlas.get(Self::texture_id_from_layer(layer)).texels;
            Self::upload_layer_mipmaps(&self.queue, &self.texture, layer, texels);
        }
    }

    fn downsample_texels_rgba(src: &[[u8; 4]], src_size: usize) -> Vec<[u8; 4]> {
        let dst_size = src_size / 2;
        let mut dst = Vec::with_capacity(dst_size * dst_size);
        for dy in 0..dst_size {
            for dx in 0..dst_size {
                let p00 = src[(dy * 2) * src_size + (dx * 2)];
                let p10 = src[(dy * 2) * src_size + (dx * 2 + 1)];
                let p01 = src[(dy * 2 + 1) * src_size + (dx * 2)];
                let p11 = src[(dy * 2 + 1) * src_size + (dx * 2 + 1)];
                let a0 = p00[3] as u32;
                let a1 = p10[3] as u32;
                let a2 = p01[3] as u32;
                let a3 = p11[3] as u32;
                let total_a = a0 + a1 + a2 + a3;
                if total_a == 0 {
                    dst.push([0, 0, 0, 0]);
                } else {
                    let r = (((p00[0] as u32 * a0
                        + p10[0] as u32 * a1
                        + p01[0] as u32 * a2
                        + p11[0] as u32 * a3)
                        / total_a)
                        .min(255)) as u8;
                    let g = (((p00[1] as u32 * a0
                        + p10[1] as u32 * a1
                        + p01[1] as u32 * a2
                        + p11[1] as u32 * a3)
                        / total_a)
                        .min(255)) as u8;
                    let b = (((p00[2] as u32 * a0
                        + p10[2] as u32 * a1
                        + p01[2] as u32 * a2
                        + p11[2] as u32 * a3)
                        / total_a)
                        .min(255)) as u8;
                    let a = ((total_a + 2) / 4) as u8;
                    dst.push([r, g, b, a]);
                }
            }
        }
        dst
    }

    fn upload_layer_mipmaps(
        queue: &::wgpu::Queue,
        texture: &::wgpu::Texture,
        layer: u32,
        base_texels: &[[f32; 3]],
    ) {
        let mut level_rgba: Vec<[u8; 4]> = base_texels
            .iter()
            .map(|color| {
                if color[0] < 0.0 {
                    let real_r = (-color[0] - 0.001).clamp(0.0, 1.0);
                    [
                        (real_r * 255.0) as u8,
                        (color[1].clamp(0.0, 1.0) * 255.0) as u8,
                        (color[2].clamp(0.0, 1.0) * 255.0) as u8,
                        0,
                    ]
                } else {
                    [
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                        255,
                    ]
                }
            })
            .collect();

        let mut size = TEX_SIZE;
        for mip in 0..5 {
            let bytes: Vec<u8> = level_rgba.iter().flat_map(|c| *c).collect();
            queue.write_texture(
                ::wgpu::ImageCopyTexture {
                    texture,
                    mip_level: mip,
                    origin: ::wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: ::wgpu::TextureAspect::All,
                },
                &bytes,
                ::wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some((size * 4) as u32),
                    rows_per_image: Some(size as u32),
                },
                ::wgpu::Extent3d {
                    width: size as u32,
                    height: size as u32,
                    depth_or_array_layers: 1,
                },
            );
            if mip < 4 {
                level_rgba = Self::downsample_texels_rgba(&level_rgba, size);
                size /= 2;
            }
        }
    }

    pub(super) fn create_texture_array(
        device: &::wgpu::Device,
        queue: &::wgpu::Queue,
        atlas: &TextureAtlas,
    ) -> ::wgpu::Texture {
        let texture = device.create_texture(&::wgpu::TextureDescriptor {
            label: Some("RustCraft texture array"),
            size: ::wgpu::Extent3d {
                width: TEX_SIZE as u32,
                height: TEX_SIZE as u32,
                depth_or_array_layers: TEXTURE_LAYER_COUNT,
            },
            mip_level_count: 5,
            sample_count: 1,
            dimension: ::wgpu::TextureDimension::D2,
            format: ::wgpu::TextureFormat::Rgba8Unorm,
            usage: ::wgpu::TextureUsages::TEXTURE_BINDING | ::wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for layer in 0..TEXTURE_LAYER_COUNT {
            let texels = &atlas.get(Self::texture_id_from_layer(layer)).texels;
            Self::upload_layer_mipmaps(queue, &texture, layer, texels);
        }
        texture
    }
}
