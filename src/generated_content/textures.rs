use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashSet;

use super::surface_textures::TerrainSurfaceTextureData;

pub(crate) const PROCEDURAL_TEXTURE_SIZE: u32 = 128;
pub(crate) const TERRAIN_TEXTURE_SIZE: u32 = 512;

pub(crate) fn procedural_surface_texture(
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
) -> Image {
    procedural_srgb_texture(
        procedural_surface_texture_data(primary, secondary, accent, seed, PROCEDURAL_TEXTURE_SIZE),
        PROCEDURAL_TEXTURE_SIZE,
        ImageFilterMode::Linear,
        8,
    )
}

pub(crate) fn procedural_surface_texture_data(
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    size: u32,
) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let noise = texture_noise(x, y, seed);
            let vein = (x.wrapping_mul(5) + y.wrapping_mul(3) + seed).is_multiple_of(31);
            let check = (x / 16 + y / 16 + seed).is_multiple_of(2);
            let mut color = if noise < 74 {
                secondary
            } else if noise > 216 {
                accent
            } else {
                primary
            };

            if check {
                color = mix_rgba(color, primary, 178);
            }
            if vein {
                color = mix_rgba(color, accent, 112);
            }

            data.extend_from_slice(&color);
        }
    }

    data
}

pub(crate) fn procedural_terrain_surface_texture_data(
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    size: u32,
) -> Vec<u8> {
    TerrainSurfaceTextureData::generate(primary, secondary, accent, seed, size).albedo
}

pub(crate) fn procedural_srgb_texture(
    data: Vec<u8>,
    size: u32,
    filter: ImageFilterMode,
    anisotropy_clamp: u16,
) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        anisotropy_clamp,
        ..default()
    });
    image
}

pub(crate) fn procedural_material_map(seed: u32, roughness: f32) -> Image {
    procedural_material_map_with_size(seed, roughness, PROCEDURAL_TEXTURE_SIZE)
}

pub(crate) fn procedural_material_map_with_size(seed: u32, roughness: f32, size: u32) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let noise = texture_noise(x, y, seed) as f32 / 255.0;
            let pore = texture_noise(x / 2, y / 2, seed.wrapping_add(9)) as f32 / 255.0;
            let roughness_value =
                (roughness * (0.82 + noise * 0.28) + pore * 0.08).clamp(0.08, 1.0);
            data.extend_from_slice(&[0, (roughness_value * 255.0) as u8, 0, 255]);
        }
    }

    procedural_data_texture_with_size(data, ImageFilterMode::Linear, size)
}

pub(crate) fn procedural_occlusion_map(seed: u32) -> Image {
    procedural_occlusion_map_with_size(seed, PROCEDURAL_TEXTURE_SIZE)
}

pub(crate) fn procedural_occlusion_map_with_size(seed: u32, size: u32) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let noise = texture_noise(x, y, seed) as u16;
            let large = texture_noise(x / 4, y / 4, seed.wrapping_add(17)) as u16;
            let occlusion = (190 + noise / 5 + large / 7).min(255) as u8;
            data.extend_from_slice(&[occlusion, occlusion, occlusion, 255]);
        }
    }

    procedural_data_texture_with_size(data, ImageFilterMode::Linear, size)
}

pub(crate) fn procedural_depth_map(seed: u32, filter: ImageFilterMode) -> Image {
    procedural_depth_map_with_size(seed, filter, PROCEDURAL_TEXTURE_SIZE)
}

pub(crate) fn procedural_depth_map_with_size(
    seed: u32,
    filter: ImageFilterMode,
    size: u32,
) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let fine = texture_noise(x, y, seed) as u16;
            let broad = texture_noise(x / 4, y / 4, seed.wrapping_add(31)) as u16;
            let ridge = if (x.wrapping_mul(7) + y.wrapping_mul(11) + seed).is_multiple_of(37) {
                18
            } else {
                0
            };
            let depth = (64 + fine / 3 + broad / 4 + ridge).min(255) as u8;
            data.extend_from_slice(&[depth, depth, depth, 255]);
        }
    }

    procedural_data_texture_with_size(data, filter, size)
}

pub(crate) fn procedural_data_texture_with_size(
    data: Vec<u8>,
    filter: ImageFilterMode,
    size: u32,
) -> Image {
    let anisotropy_clamp = if filter == ImageFilterMode::Linear {
        8
    } else {
        1
    };
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        anisotropy_clamp,
        ..default()
    });
    image
}

pub(crate) fn texture_detail_band_count(data: &[u8]) -> usize {
    data.chunks_exact(4)
        .map(|pixel| [pixel[0] / 16, pixel[1] / 16, pixel[2] / 16])
        .collect::<HashSet<_>>()
        .len()
}

pub(crate) fn texture_detail_energy_promille(data: &[u8], size: u32, offset: u32) -> usize {
    if !has_complete_rgba_level(data, size) {
        return 0;
    }
    let offset = offset % size;
    if offset == 0 {
        return 0;
    }

    let mut total_delta = 0u64;
    for y in 0..size {
        for x in 0..size {
            let luma = texture_pixel_luma(data, size, x, y);
            total_delta +=
                u64::from(luma.abs_diff(texture_pixel_luma(data, size, (x + offset) % size, y)));
            total_delta +=
                u64::from(luma.abs_diff(texture_pixel_luma(data, size, x, (y + offset) % size)));
        }
    }

    let sample_count = u64::from(size) * u64::from(size) * 2;
    (total_delta * 1000 / (sample_count * 255)) as usize
}

pub(crate) fn texture_isolated_edge_promille(data: &[u8], size: u32) -> usize {
    if !has_complete_rgba_level(data, size) {
        return 0;
    }

    const HIGH_CONTRAST_DELTA: u8 = 24;
    let mut isolated = 0usize;
    for y in 0..size {
        for x in 0..size {
            let center = texture_pixel_luma(data, size, x, y);
            let mut above_all = true;
            let mut below_all = true;
            let mut high_contrast = true;
            for offset_y in -1..=1 {
                for offset_x in -1..=1 {
                    if offset_x == 0 && offset_y == 0 {
                        continue;
                    }
                    let neighbor = texture_pixel_luma(
                        data,
                        size,
                        wrapped_texture_coordinate(x, offset_x, size),
                        wrapped_texture_coordinate(y, offset_y, size),
                    );
                    above_all &= center > neighbor;
                    below_all &= center < neighbor;
                    high_contrast &= center.abs_diff(neighbor) >= HIGH_CONTRAST_DELTA;
                }
            }
            isolated += usize::from(high_contrast && (above_all || below_all));
        }
    }

    isolated * 1000 / (size as usize * size as usize)
}

pub(crate) fn texture_edge_promille(data: &[u8], size: u32) -> usize {
    if size < 2 {
        return 0;
    }
    let stride = size as usize * 4;
    let mut edge_count = 0usize;
    let mut sample_count = 0usize;
    for y in 0..size as usize {
        for x in 0..size as usize {
            let offset = y * stride + x * 4;
            let luma = texture_luma(&data[offset..offset + 3]);
            if x + 1 < size as usize {
                let right = texture_luma(&data[offset + 4..offset + 7]);
                edge_count += usize::from(luma.abs_diff(right) >= 18);
                sample_count += 1;
            }
            if y + 1 < size as usize {
                let down_offset = offset + stride;
                let down = texture_luma(&data[down_offset..down_offset + 3]);
                edge_count += usize::from(luma.abs_diff(down) >= 18);
                sample_count += 1;
            }
        }
    }

    (edge_count * 1000).checked_div(sample_count).unwrap_or(0)
}

pub(crate) fn texture_luma(rgb: &[u8]) -> u8 {
    ((u16::from(rgb[0]) * 77 + u16::from(rgb[1]) * 150 + u16::from(rgb[2]) * 29) / 256) as u8
}

fn has_complete_rgba_level(data: &[u8], size: u32) -> bool {
    size > 0
        && usize::try_from(u64::from(size) * u64::from(size) * 4)
            .is_ok_and(|expected_len| data.len() >= expected_len)
}

fn texture_pixel_luma(data: &[u8], size: u32, x: u32, y: u32) -> u8 {
    let offset = ((y * size + x) * 4) as usize;
    texture_luma(&data[offset..offset + 3])
}

fn wrapped_texture_coordinate(value: u32, offset: i32, size: u32) -> u32 {
    (i64::from(value) + i64::from(offset)).rem_euclid(i64::from(size)) as u32
}

pub(crate) fn texture_noise(x: u32, y: u32, seed: u32) -> u8 {
    let mut value = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(2_654_435_761));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 0xff) as u8
}

pub(crate) fn mix_rgba(source: [u8; 4], target: [u8; 4], target_weight: u16) -> [u8; 4] {
    let source_weight = 255 - target_weight;
    [
        ((source[0] as u16 * source_weight + target[0] as u16 * target_weight) / 255) as u8,
        ((source[1] as u16 * source_weight + target[1] as u16 * target_weight) / 255) as u8,
        ((source[2] as u16 * source_weight + target[2] as u16 * target_weight) / 255) as u8,
        ((source[3] as u16 * source_weight + target[3] as u16 * target_weight) / 255) as u8,
    ]
}

pub(crate) fn mix_color(source: Color, target: Color, target_weight: f32) -> Color {
    source.mix(&target, target_weight.clamp(0.0, 1.0))
}

pub(crate) fn random_unit(seed: u32, x: u32, salt: u32) -> f32 {
    texture_noise(x.wrapping_mul(17).wrapping_add(salt), salt, seed) as f32 / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn procedural_texture_sizes_keep_materials_inspectable() {
        let prop_data = procedural_surface_texture_data(
            [80, 142, 72, 255],
            [45, 96, 64, 255],
            [164, 144, 82, 255],
            211,
            PROCEDURAL_TEXTURE_SIZE,
        );
        let terrain_data = procedural_terrain_surface_texture_data(
            [80, 142, 72, 255],
            [45, 96, 64, 255],
            [164, 144, 82, 255],
            311,
            TERRAIN_TEXTURE_SIZE,
        );

        assert!(prop_data.len() >= 128 * 128 * 4);
        assert!(terrain_data.len() >= 512 * 512 * 4);
    }

    #[test]
    fn terrain_surface_texture_delegates_to_coherent_albedo() {
        let primary = [80, 142, 72, 255];
        let secondary = [45, 96, 64, 255];
        let accent = [164, 144, 82, 255];
        let data = procedural_terrain_surface_texture_data(
            primary,
            secondary,
            accent,
            311,
            TERRAIN_TEXTURE_SIZE,
        );
        let coherent = TerrainSurfaceTextureData::generate(
            primary,
            secondary,
            accent,
            311,
            TERRAIN_TEXTURE_SIZE,
        );

        assert_eq!(
            data.len(),
            (TERRAIN_TEXTURE_SIZE * TERRAIN_TEXTURE_SIZE * 4) as usize
        );
        assert_eq!(data, coherent.albedo);
        assert!(texture_detail_band_count(&data) >= 7);
    }

    #[test]
    fn detail_energy_uses_wrapped_normalized_luma_deltas() {
        let size = 8u32;
        let mut checker = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let value = if (x + y).is_multiple_of(2) { 0 } else { 255 };
                checker.extend_from_slice(&[value, value, value, 255]);
            }
        }

        assert_eq!(texture_detail_energy_promille(&checker, size, 1), 1000);
        assert_eq!(texture_detail_energy_promille(&checker, size, 2), 0);
        assert_eq!(texture_detail_energy_promille(&checker, size, size), 0);
        assert_eq!(texture_detail_energy_promille(&[], size, 1), 0);
    }

    #[test]
    fn isolated_edge_metric_counts_sparse_salt_and_pepper_not_regions() {
        let size = 10u32;
        let mut data = vec![80; (size * size * 4) as usize];
        for alpha in data.iter_mut().skip(3).step_by(4) {
            *alpha = 255;
        }
        let isolated_offset = ((4 * size + 4) * 4) as usize;
        data[isolated_offset..isolated_offset + 3].fill(255);

        assert_eq!(texture_isolated_edge_promille(&data, size), 10);

        for y in 3..=5 {
            for x in 3..=5 {
                let offset = ((y * size + x) * 4) as usize;
                data[offset..offset + 3].fill(255);
            }
        }
        assert_eq!(texture_isolated_edge_promille(&data, size), 0);
    }
}
