use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDataOrder, TextureDimension, TextureFormat};
use std::collections::HashSet;

const NORMAL_STRENGTH: f32 = 7.0;
#[cfg(test)]
const DETAIL_OFFSETS: [u32; 3] = [1, 4, 16];
#[cfg(test)]
const EDGE_THRESHOLD: u8 = 18;
#[cfg(test)]
const HIGH_FREQUENCY_THRESHOLD: u8 = 32;
const WATER_NORMAL_SIZES: [u32; 2] = [128, 256];
const CROSSING_WATER_WAVES: [(i32, i32, f32); 8] = [
    (2, 1, 0.28),
    (5, 2, 0.17),
    (9, 4, 0.10),
    (15, 7, 0.06),
    (-1, 3, 0.26),
    (-3, 7, 0.16),
    (-5, 12, 0.09),
    (-8, 19, 0.05),
];

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(super) struct TerrainSurfaceTextureData {
    size: u32,
    height: Vec<f32>,
    moisture: Vec<f32>,
    material_weights: Vec<[f32; 3]>,
    pub(super) albedo: Vec<u8>,
    normal: Vec<u8>,
    orm: Vec<u8>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
struct TerrainSurfaceTextureMetrics {
    detail_band_count: usize,
    edge_promille: usize,
    high_frequency_promille: usize,
    isolated_edge_promille: usize,
    detail_energy: [f32; 3],
    height_range: f32,
    moisture_range: f32,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct TerrainSurfaceTextureSet {
    pub(crate) albedo: Image,
    pub(crate) normal: Image,
    pub(crate) orm: Image,
    pub(crate) albedo_data: Vec<u8>,
    pub(crate) detail_bands: usize,
}

impl TerrainSurfaceTextureSet {
    pub(crate) fn generate(
        primary: [u8; 4],
        secondary: [u8; 4],
        accent: [u8; 4],
        seed: u32,
        size: u32,
    ) -> Self {
        let data = TerrainSurfaceTextureData::generate(primary, secondary, accent, seed, size);
        let albedo = srgb_image_with_mips(&data.albedo, size);
        let normal = normal_image_with_mips(&data.normal, size);
        let orm = linear_image_with_mips(&data.orm, size);
        let detail_bands = detail_band_count(&data.albedo);

        Self {
            albedo,
            normal,
            orm,
            albedo_data: data.albedo,
            detail_bands,
        }
    }
}

impl TerrainSurfaceTextureData {
    pub(super) fn generate(
        primary: [u8; 4],
        secondary: [u8; 4],
        accent: [u8; 4],
        seed: u32,
        size: u32,
    ) -> Self {
        assert!(size > 0, "terrain surface textures need a non-zero size");

        let pixel_count = (size * size) as usize;
        let (mut height, mut moisture) = periodic_surface_field(seed, size);
        normalize_field(&mut height);
        normalize_field(&mut moisture);

        let mut material_weights = Vec::with_capacity(pixel_count);
        let mut albedo = Vec::with_capacity(pixel_count * 4);
        let mut normal = Vec::with_capacity(pixel_count * 4);
        let mut orm = Vec::with_capacity(pixel_count * 4);

        for y in 0..size {
            for x in 0..size {
                let index = field_index(x, y, size);
                let h = height[index];
                let wet = moisture[index];
                let left = height[field_index(wrapped_sub(x, size), y, size)];
                let right = height[field_index(wrapped_add(x, size), y, size)];
                let up = height[field_index(x, wrapped_sub(y, size), size)];
                let down = height[field_index(x, wrapped_add(y, size), size)];
                let dx = (right - left) * 0.5;
                let dy = (down - up) * 0.5;
                let slope = (dx.mul_add(dx, dy * dy)).sqrt();
                let slope_weight = (slope * NORMAL_STRENGTH * 1.25).clamp(0.0, 1.0);
                let cavity = ((left + right + up + down) * 0.25 - h).clamp(-0.2, 0.2);
                let patch_step = (size / 64).clamp(4, 8) as i32;
                let height_patch =
                    (h - field_cardinal_average(&height, x, y, size, patch_step)) * 8.0;
                let moisture_patch =
                    (wet - field_cardinal_average(&moisture, x, y, size, patch_step)) * 8.0;

                let accent_weight = (0.14
                    + (h - 0.5) * 0.06
                    + height_patch * 0.50
                    + slope_weight * 0.18
                    + (0.5 - wet) * 0.02)
                    .clamp(0.04, 0.52);
                let secondary_weight = (0.22 + (0.5 - wet) * 0.10 - moisture_patch * 0.42
                    + (0.5 - h) * 0.02
                    + cavity.max(0.0) * 0.30)
                    .clamp(0.06, 0.58)
                    * (1.0 - accent_weight * 0.35);
                let primary_weight = (1.0 - accent_weight - secondary_weight).max(0.0);
                let weight_total = primary_weight + secondary_weight + accent_weight;
                let weights = [
                    primary_weight / weight_total,
                    secondary_weight / weight_total,
                    accent_weight / weight_total,
                ];
                material_weights.push(weights);

                let fiber_step = (size / 128).clamp(2, 4) as i32;
                let fiber_cross_step = (fiber_step / 2).max(1);
                let fiber_forward = height[field_index(
                    wrapped_offset(x, fiber_step, size),
                    wrapped_offset(y, fiber_cross_step, size),
                    size,
                )];
                let fiber_backward = height[field_index(
                    wrapped_offset(x, -fiber_step, size),
                    wrapped_offset(y, -fiber_cross_step, size),
                    size,
                )];
                let cross_forward = moisture[field_index(
                    wrapped_offset(x, -fiber_cross_step, size),
                    wrapped_offset(y, fiber_step, size),
                    size,
                )];
                let cross_backward = moisture[field_index(
                    wrapped_offset(x, fiber_cross_step, size),
                    wrapped_offset(y, -fiber_step, size),
                    size,
                )];
                let directional_erosion = (fiber_forward - fiber_backward) * 18.0
                    + (cross_forward - cross_backward) * 8.0;
                let micro_relief = (h - (left + right + up + down) * 0.25) * 260.0;
                let shade = (((h - 0.5) * 1.6 + (wet - 0.5) * 1.1 + height_patch * 8.0
                    - moisture_patch * 5.0
                    + directional_erosion * 0.85
                    + micro_relief * 0.65
                    - cavity * 14.0)
                    * 1.12)
                    .clamp(-18.0, 18.0);
                albedo
                    .extend_from_slice(&weighted_color(primary, secondary, accent, weights, shade));

                let tangent_normal =
                    Vec3::new(-dx * NORMAL_STRENGTH, -dy * NORMAL_STRENGTH, 1.0).normalize();
                normal.extend_from_slice(&encode_normal(tangent_normal));

                let ambient_occlusion =
                    (0.96 - cavity.max(0.0) * 2.2 - slope_weight * 0.08).clamp(0.58, 1.0);
                let roughness = (0.68 + wet * 0.24 + (1.0 - accent_weight) * 0.08
                    - slope_weight * 0.06)
                    .clamp(0.48, 1.0);
                orm.extend_from_slice(&[
                    normalized_byte(ambient_occlusion),
                    normalized_byte(roughness),
                    0,
                    255,
                ]);
            }
        }

        Self {
            size,
            height,
            moisture,
            material_weights,
            albedo,
            normal,
            orm,
        }
    }
}

#[cfg(test)]
impl TerrainSurfaceTextureMetrics {
    fn from_data(data: &TerrainSurfaceTextureData) -> Self {
        Self {
            detail_band_count: detail_band_count(&data.albedo),
            edge_promille: edge_promille(&data.albedo, data.size, EDGE_THRESHOLD),
            high_frequency_promille: high_frequency_promille(
                &data.albedo,
                data.size,
                HIGH_FREQUENCY_THRESHOLD,
            ),
            isolated_edge_promille: isolated_edge_promille(
                &data.albedo,
                data.size,
                HIGH_FREQUENCY_THRESHOLD,
            ),
            detail_energy: DETAIL_OFFSETS
                .map(|offset| detail_energy(&data.albedo, data.size, offset)),
            height_range: field_range(&data.height),
            moisture_range: field_range(&data.moisture),
        }
    }
}

pub(crate) fn srgb_image_with_mips(base_level: &[u8], size: u32) -> Image {
    let (data, mip_level_count) = build_mip_chain(base_level, size, downsample_srgb);
    mipmapped_image(data, size, mip_level_count, TextureFormat::Rgba8UnormSrgb)
}

pub(crate) fn linear_image_with_mips(base_level: &[u8], size: u32) -> Image {
    let (data, mip_level_count) = build_mip_chain(base_level, size, downsample_linear);
    mipmapped_image(data, size, mip_level_count, TextureFormat::Rgba8Unorm)
}

pub(crate) fn procedural_water_normal_texture(seed: u32, size: u32) -> Image {
    assert!(
        WATER_NORMAL_SIZES.contains(&size),
        "water normal textures must be 128 or 256 pixels square"
    );

    let mut normal_data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let uv = Vec2::new(x as f32 / size as f32, y as f32 / size as f32);
            let mut slope = Vec2::ZERO;
            for (wave_index, &(wave_x, wave_y, strength)) in CROSSING_WATER_WAVES.iter().enumerate()
            {
                let wave = Vec2::new(wave_x as f32, wave_y as f32);
                let phase_offset =
                    hash_unit(wave_index as u32, 0x7f4a_7c15, seed) * std::f32::consts::TAU;
                let phase = std::f32::consts::TAU * wave.dot(uv) + phase_offset;
                slope += wave.normalize() * phase.cos() * strength;
            }
            normal_data.extend_from_slice(&encode_normal(
                Vec3::new(-slope.x, -slope.y, 1.0).normalize(),
            ));
        }
    }

    normal_image_with_mips(&normal_data, size)
}

fn normal_image_with_mips(base_level: &[u8], size: u32) -> Image {
    let (data, mip_level_count) = build_mip_chain(base_level, size, downsample_normal);
    mipmapped_image(data, size, mip_level_count, TextureFormat::Rgba8Unorm)
}

fn mipmapped_image(data: Vec<u8>, size: u32, mip_level_count: u32, format: TextureFormat) -> Image {
    let mut image = Image::new_uninit(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        format,
        RenderAssetUsages::default(),
    );
    image.data = Some(data);
    image.data_order = TextureDataOrder::MipMajor;
    image.texture_descriptor.mip_level_count = mip_level_count;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        anisotropy_clamp: 16,
        ..default()
    });
    image
}

fn periodic_surface_field(seed: u32, size: u32) -> (Vec<f32>, Vec<f32>) {
    let mut height = Vec::with_capacity((size * size) as usize);
    let mut moisture = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            let u = x as f32 / size as f32;
            let v = y as f32 / size as f32;
            let warp_x = periodic_value_noise(u, v, 3, seed.wrapping_add(0x51ed_270b)) - 0.5;
            let warp_y = periodic_value_noise(u, v, 3, seed.wrapping_add(0x9e37_79b9)) - 0.5;
            let warped_u = u + warp_x * 0.075;
            let warped_v = v + warp_y * 0.075;
            let base_height =
                periodic_fbm(warped_u, warped_v, seed.wrapping_add(0x85eb_ca6b), size);
            let ridge_noise =
                periodic_value_noise(warped_u, warped_v, 6, seed.wrapping_add(0xc2b2_ae35));
            let ridges = 1.0 - (ridge_noise * 2.0 - 1.0).abs();
            let h = base_height * 0.84 + ridges * 0.16;
            let wet = periodic_fbm(
                u - warp_y * 0.055,
                v + warp_x * 0.055,
                seed.wrapping_add(0x27d4_eb2f),
                size,
            ) * 0.82
                + (1.0 - h) * 0.18;
            height.push(h);
            moisture.push(wet);
        }
    }
    (height, moisture)
}

fn periodic_fbm(u: f32, v: f32, seed: u32, size: u32) -> f32 {
    let max_cells = (size / 2).max(2);
    let mut cells = 2;
    let mut amplitude = 1.0;
    let mut total = 0.0;
    let mut amplitude_total = 0.0;
    let mut octave = 0u32;
    while cells <= max_cells {
        total += periodic_value_noise(
            u,
            v,
            cells,
            seed.wrapping_add(octave.wrapping_mul(0x9e37_79b9)),
        ) * amplitude;
        amplitude_total += amplitude;
        amplitude *= 0.54;
        cells *= 2;
        octave += 1;
    }
    total / amplitude_total
}

fn periodic_value_noise(u: f32, v: f32, cells: u32, seed: u32) -> f32 {
    let x = u * cells as f32;
    let y = v * cells as f32;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = smootherstep(x - x.floor());
    let ty = smootherstep(y - y.floor());
    let sample = |grid_x: i32, grid_y: i32| {
        hash_unit(
            grid_x.rem_euclid(cells as i32) as u32,
            grid_y.rem_euclid(cells as i32) as u32,
            seed,
        )
    };
    let north = lerp(sample(x0, y0), sample(x0 + 1, y0), tx);
    let south = lerp(sample(x0, y0 + 1), sample(x0 + 1, y0 + 1), tx);
    lerp(north, south, ty)
}

fn hash_unit(x: u32, y: u32, seed: u32) -> f32 {
    let mut value = x
        .wrapping_mul(0x9e37_79b1)
        .wrapping_add(y.wrapping_mul(0x85eb_ca77))
        .wrapping_add(seed.wrapping_mul(0xc2b2_ae3d));
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^= value >> 16;
    value as f32 / u32::MAX as f32
}

fn normalize_field(field: &mut [f32]) {
    let (min, max) = field
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
            (min.min(*value), max.max(*value))
        });
    let range = (max - min).max(f32::EPSILON);
    for value in field {
        *value = ((*value - min) / range).clamp(0.0, 1.0);
    }
}

fn weighted_color(
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    weights: [f32; 3],
    shade: f32,
) -> [u8; 4] {
    let palettes = [primary, secondary, accent];
    let mut color = [0u8; 4];
    for channel in 0..4 {
        let value = palettes
            .iter()
            .zip(weights)
            .map(|(palette, weight)| palette[channel] as f32 * weight)
            .sum::<f32>();
        color[channel] = if channel == 3 {
            value.round().clamp(0.0, 255.0) as u8
        } else {
            (value + shade).round().clamp(0.0, 255.0) as u8
        };
    }
    color
}

fn encode_normal(normal: Vec3) -> [u8; 4] {
    [
        normalized_byte(normal.x * 0.5 + 0.5),
        normalized_byte(normal.y * 0.5 + 0.5),
        normalized_byte(normal.z * 0.5 + 0.5),
        255,
    ]
}

fn decode_normal(pixel: &[u8]) -> Vec3 {
    Vec3::new(
        pixel[0] as f32 / 255.0 * 2.0 - 1.0,
        pixel[1] as f32 / 255.0 * 2.0 - 1.0,
        pixel[2] as f32 / 255.0 * 2.0 - 1.0,
    )
}

fn normalized_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn build_mip_chain(
    base_level: &[u8],
    size: u32,
    downsample: fn(&[u8], u32) -> Vec<u8>,
) -> (Vec<u8>, u32) {
    assert!(size > 0, "mipmapped images need a non-zero size");
    assert_eq!(
        base_level.len(),
        (size * size * 4) as usize,
        "RGBA base level must exactly match the requested image size"
    );

    let mip_level_count = size.ilog2() + 1;
    let mut data = Vec::with_capacity(mip_chain_byte_len(size));
    let mut level = base_level.to_vec();
    let mut level_size = size;
    data.extend_from_slice(&level);
    while level_size > 1 {
        level = downsample(&level, level_size);
        level_size = (level_size / 2).max(1);
        data.extend_from_slice(&level);
    }
    (data, mip_level_count)
}

fn downsample_srgb(source: &[u8], source_size: u32) -> Vec<u8> {
    downsample_rgba(source, source_size, |samples| {
        let mut pixel = [0u8; 4];
        for channel in 0..3 {
            let linear = samples
                .iter()
                .map(|sample| srgb_to_linear(sample[channel]))
                .sum::<f32>()
                * 0.25;
            pixel[channel] = linear_to_srgb(linear);
        }
        pixel[3] = ((samples.iter().map(|sample| sample[3] as u16).sum::<u16>() + 2) / 4) as u8;
        pixel
    })
}

fn downsample_linear(source: &[u8], source_size: u32) -> Vec<u8> {
    downsample_rgba(source, source_size, |samples| {
        let mut pixel = [0u8; 4];
        for channel in 0..4 {
            pixel[channel] = ((samples
                .iter()
                .map(|sample| sample[channel] as u16)
                .sum::<u16>()
                + 2)
                / 4) as u8;
        }
        pixel
    })
}

fn downsample_normal(source: &[u8], source_size: u32) -> Vec<u8> {
    downsample_rgba(source, source_size, |samples| {
        let sum = samples
            .iter()
            .map(|sample| decode_normal(sample))
            .sum::<Vec3>();
        encode_normal(sum.try_normalize().unwrap_or(Vec3::Z))
    })
}

fn downsample_rgba(
    source: &[u8],
    source_size: u32,
    filter: impl Fn([[u8; 4]; 4]) -> [u8; 4],
) -> Vec<u8> {
    let target_size = (source_size / 2).max(1);
    let mut target = Vec::with_capacity((target_size * target_size * 4) as usize);
    for y in 0..target_size {
        for x in 0..target_size {
            let source_x = x * 2;
            let source_y = y * 2;
            let samples = [
                rgba_pixel(source, source_size, source_x, source_y),
                rgba_pixel(source, source_size, source_x + 1, source_y),
                rgba_pixel(source, source_size, source_x, source_y + 1),
                rgba_pixel(source, source_size, source_x + 1, source_y + 1),
            ];
            target.extend_from_slice(&filter(samples));
        }
    }
    target
}

fn rgba_pixel(data: &[u8], size: u32, x: u32, y: u32) -> [u8; 4] {
    let offset = field_index(x % size, y % size, size) * 4;
    data[offset..offset + 4].try_into().expect("RGBA pixel")
}

fn detail_band_count(data: &[u8]) -> usize {
    data.chunks_exact(4)
        .map(|pixel| [pixel[0] / 16, pixel[1] / 16, pixel[2] / 16])
        .collect::<HashSet<_>>()
        .len()
}

#[cfg(test)]
fn edge_promille(data: &[u8], size: u32, threshold: u8) -> usize {
    let mut edges = 0usize;
    let samples = (size * size * 2) as usize;
    for y in 0..size {
        for x in 0..size {
            let luma = pixel_luma(data, size, x, y);
            edges += usize::from(
                luma.abs_diff(pixel_luma(data, size, wrapped_add(x, size), y)) >= threshold,
            );
            edges += usize::from(
                luma.abs_diff(pixel_luma(data, size, x, wrapped_add(y, size))) >= threshold,
            );
        }
    }
    edges * 1000 / samples.max(1)
}

#[cfg(test)]
fn high_frequency_promille(data: &[u8], size: u32, threshold: u8) -> usize {
    let mut count = 0usize;
    for y in 0..size {
        for x in 0..size {
            let center = pixel_luma(data, size, x, y) as i16;
            let neighbor_average = [
                pixel_luma(data, size, wrapped_sub(x, size), y),
                pixel_luma(data, size, wrapped_add(x, size), y),
                pixel_luma(data, size, x, wrapped_sub(y, size)),
                pixel_luma(data, size, x, wrapped_add(y, size)),
            ]
            .into_iter()
            .map(i16::from)
            .sum::<i16>()
                / 4;
            count += usize::from(center.abs_diff(neighbor_average) >= threshold as u16);
        }
    }
    count * 1000 / (size * size).max(1) as usize
}

#[cfg(test)]
fn isolated_edge_promille(data: &[u8], size: u32, threshold: u8) -> usize {
    let mut isolated = 0usize;
    for y in 0..size {
        for x in 0..size {
            let center = pixel_luma(data, size, x, y);
            let neighbors = [
                pixel_luma(data, size, wrapped_sub(x, size), y),
                pixel_luma(data, size, wrapped_add(x, size), y),
                pixel_luma(data, size, x, wrapped_sub(y, size)),
                pixel_luma(data, size, x, wrapped_add(y, size)),
            ];
            let separated = neighbors
                .iter()
                .all(|neighbor| center.abs_diff(*neighbor) >= threshold);
            let extremum = neighbors.iter().all(|neighbor| center > *neighbor)
                || neighbors.iter().all(|neighbor| center < *neighbor);
            isolated += usize::from(separated && extremum);
        }
    }
    isolated * 1000 / (size * size).max(1) as usize
}

#[cfg(test)]
fn detail_energy(data: &[u8], size: u32, offset: u32) -> f32 {
    let offset = offset % size.max(1);
    let mut difference = 0u64;
    for y in 0..size {
        for x in 0..size {
            let luma = pixel_luma(data, size, x, y);
            difference += u64::from(luma.abs_diff(pixel_luma(data, size, (x + offset) % size, y)));
            difference += u64::from(luma.abs_diff(pixel_luma(data, size, x, (y + offset) % size)));
        }
    }
    difference as f32 / ((size * size * 2).max(1) as f32 * 255.0)
}

#[cfg(test)]
fn pixel_luma(data: &[u8], size: u32, x: u32, y: u32) -> u8 {
    let offset = field_index(x, y, size) * 4;
    ((u16::from(data[offset]) * 77
        + u16::from(data[offset + 1]) * 150
        + u16::from(data[offset + 2]) * 29)
        / 256) as u8
}

#[cfg(test)]
fn field_range(field: &[f32]) -> f32 {
    let (min, max) = field
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
            (min.min(*value), max.max(*value))
        });
    max - min
}

fn field_index(x: u32, y: u32, size: u32) -> usize {
    (y * size + x) as usize
}

fn wrapped_add(value: u32, size: u32) -> u32 {
    (value + 1) % size
}

fn wrapped_sub(value: u32, size: u32) -> u32 {
    (value + size - 1) % size
}

fn wrapped_offset(value: u32, offset: i32, size: u32) -> u32 {
    (value as i64 + i64::from(offset)).rem_euclid(i64::from(size)) as u32
}

fn field_cardinal_average(field: &[f32], x: u32, y: u32, size: u32, radius: i32) -> f32 {
    [
        field[field_index(wrapped_offset(x, radius, size), y, size)],
        field[field_index(wrapped_offset(x, -radius, size), y, size)],
        field[field_index(x, wrapped_offset(y, radius, size), size)],
        field[field_index(x, wrapped_offset(y, -radius, size), size)],
    ]
    .into_iter()
    .sum::<f32>()
        * 0.25
}

fn mip_chain_byte_len(size: u32) -> usize {
    let mut total = 0usize;
    let mut level_size = size;
    loop {
        total += (level_size * level_size * 4) as usize;
        if level_size == 1 {
            return total;
        }
        level_size = (level_size / 2).max(1);
    }
}

fn srgb_to_linear(value: u8) -> f32 {
    let value = value as f32 / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    let srgb = if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };
    normalized_byte(srgb)
}

fn smootherstep(value: f32) -> f32 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, weight: f32) -> f32 {
    a + (b - a) * weight
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMARY: [u8; 4] = [80, 142, 72, 255];
    const SECONDARY: [u8; 4] = [45, 96, 64, 255];
    const ACCENT: [u8; 4] = [164, 144, 82, 255];
    const TEST_SIZE: u32 = 64;

    #[test]
    fn generated_images_have_exact_dimensions_formats_and_full_mips() {
        let generated =
            TerrainSurfaceTextureSet::generate(PRIMARY, SECONDARY, ACCENT, 311, TEST_SIZE);
        let expected_mips = TEST_SIZE.ilog2() + 1;
        let expected_bytes = mip_chain_byte_len(TEST_SIZE);

        for (image, format) in [
            (&generated.albedo, TextureFormat::Rgba8UnormSrgb),
            (&generated.normal, TextureFormat::Rgba8Unorm),
            (&generated.orm, TextureFormat::Rgba8Unorm),
        ] {
            assert_eq!(image.texture_descriptor.size.width, TEST_SIZE);
            assert_eq!(image.texture_descriptor.size.height, TEST_SIZE);
            assert_eq!(image.texture_descriptor.mip_level_count, expected_mips);
            assert_eq!(image.texture_descriptor.format, format);
            assert_eq!(image.data_order, TextureDataOrder::MipMajor);
            assert_eq!(image.data.as_ref().map(Vec::len), Some(expected_bytes));
        }
        assert_eq!(
            &generated.albedo.data.as_ref().unwrap()[..generated.albedo_data.len()],
            generated.albedo_data
        );
        assert_eq!(
            TerrainSurfaceTextureSet::generate(PRIMARY, SECONDARY, ACCENT, 311, TEST_SIZE)
                .albedo_data,
            generated.albedo_data
        );
        assert_eq!(
            generated.detail_bands,
            detail_band_count(&generated.albedo_data)
        );
    }

    #[test]
    fn periodic_field_and_maps_cross_the_repeat_seam_continuously() {
        let data = TerrainSurfaceTextureData::generate(PRIMARY, SECONDARY, ACCENT, 419, TEST_SIZE);
        let height_seam = seam_difference_f32(&data.height, TEST_SIZE);
        let height_interior = neighbor_difference_f32(&data.height, TEST_SIZE);
        let albedo_seam = seam_difference_rgba(&data.albedo, TEST_SIZE);
        let albedo_interior = neighbor_difference_rgba(&data.albedo, TEST_SIZE);

        assert!(height_seam <= height_interior * 2.0 + 0.005);
        assert!(albedo_seam <= albedo_interior * 2.0 + 1.0);
    }

    #[test]
    fn tangent_normals_remain_valid_through_every_mip() {
        let generated =
            TerrainSurfaceTextureSet::generate(PRIMARY, SECONDARY, ACCENT, 521, TEST_SIZE);
        for pixel in generated.normal.data.as_ref().unwrap().chunks_exact(4) {
            let normal = decode_normal(pixel);
            assert!((normal.length() - 1.0).abs() < 0.025);
            assert!(normal.z > 0.35);
            assert_eq!(pixel[3], 255);
        }
    }

    #[test]
    fn water_normal_is_deterministic_linear_and_fully_mipped() {
        for size in WATER_NORMAL_SIZES {
            let image = procedural_water_normal_texture(887, size);
            assert_eq!(image.texture_descriptor.size.width, size);
            assert_eq!(image.texture_descriptor.size.height, size);
            assert_eq!(image.texture_descriptor.format, TextureFormat::Rgba8Unorm);
            assert_eq!(image.texture_descriptor.mip_level_count, size.ilog2() + 1);
            assert_eq!(image.data_order, TextureDataOrder::MipMajor);
            assert_eq!(
                image.data.as_ref().map(Vec::len),
                Some(mip_chain_byte_len(size))
            );
        }

        let first = procedural_water_normal_texture(887, 128);
        let repeated = procedural_water_normal_texture(887, 128);
        let different_seed = procedural_water_normal_texture(888, 128);
        assert_eq!(first.data, repeated.data);
        assert_ne!(first.data, different_seed.data);
    }

    #[test]
    fn crossing_water_normal_stays_valid_and_continuous_across_the_seam() {
        let size = 128;
        let image = procedural_water_normal_texture(991, size);
        let data = image.data.as_ref().unwrap();
        let base_level = &data[..(size * size * 4) as usize];
        let mut min_normal = Vec2::splat(f32::INFINITY);
        let mut max_normal = Vec2::splat(f32::NEG_INFINITY);

        for pixel in data.chunks_exact(4) {
            let normal = decode_normal(pixel);
            assert!((normal.length() - 1.0).abs() < 0.025);
            assert!(normal.z > 0.52);
            assert_eq!(pixel[3], 255);
        }
        for pixel in base_level.chunks_exact(4) {
            let normal = decode_normal(pixel);
            min_normal = min_normal.min(normal.xy());
            max_normal = max_normal.max(normal.xy());
        }

        let seam = normal_seam_difference(base_level, size);
        let interior = normal_neighbor_difference(base_level, size);
        assert!(seam <= interior * 1.6 + 0.005);
        assert!(min_normal.x < -0.18 && max_normal.x > 0.18);
        assert!(min_normal.y < -0.18 && max_normal.y > 0.18);
    }

    #[test]
    fn albedo_and_material_channels_follow_height_and_moisture() {
        let data = TerrainSurfaceTextureData::generate(PRIMARY, SECONDARY, ACCENT, 631, TEST_SIZE);
        let mut high_accent = Vec::new();
        let mut low_accent = Vec::new();
        let mut dry_secondary = Vec::new();
        let mut wet_secondary = Vec::new();
        let mut reconstruction_error = 0.0;

        for (index, weights) in data.material_weights.iter().enumerate() {
            let height = data.height[index];
            let moisture = data.moisture[index];
            if height > 0.72 {
                high_accent.push(weights[2]);
            } else if height < 0.28 {
                low_accent.push(weights[2]);
            }
            if moisture < 0.28 {
                dry_secondary.push(weights[1]);
            } else if moisture > 0.72 {
                wet_secondary.push(weights[1]);
            }

            let pixel = &data.albedo[index * 4..index * 4 + 3];
            for channel in 0..3 {
                let expected = PRIMARY[channel] as f32 * weights[0]
                    + SECONDARY[channel] as f32 * weights[1]
                    + ACCENT[channel] as f32 * weights[2];
                reconstruction_error += (pixel[channel] as f32 - expected).abs();
            }
        }

        assert!(mean(&high_accent) > mean(&low_accent) + 0.28);
        assert!(mean(&dry_secondary) > mean(&wet_secondary) + 0.18);
        assert!(reconstruction_error / ((data.material_weights.len() * 3) as f32) < 13.0);
        assert!(data.orm.chunks_exact(4).all(|pixel| pixel[2] == 0));
    }

    #[test]
    fn coherent_surface_keeps_detail_at_multiple_scales_without_rewarding_noise() {
        let data = TerrainSurfaceTextureData::generate(PRIMARY, SECONDARY, ACCENT, 733, 128);
        let metrics = TerrainSurfaceTextureMetrics::from_data(&data);
        let [fine, medium, coarse] = metrics.detail_energy;

        assert!(metrics.detail_band_count >= 12);
        assert!(fine > 0.002);
        assert!(medium > fine * 2.0);
        assert!(coarse > medium * 1.05);
        assert!(coarse < medium * 1.5);
        assert!(fine < 0.015);
        assert!(metrics.edge_promille < 40);
        assert!(metrics.high_frequency_promille < 40);
        assert!(metrics.isolated_edge_promille < 5);
        assert!(height_bin_luma_reversal_count(&data, 24) <= 5);
        assert!(metrics.height_range > 0.95);
        assert!(metrics.moisture_range > 0.95);
    }

    fn mean(values: &[f32]) -> f32 {
        values.iter().sum::<f32>() / values.len().max(1) as f32
    }

    fn height_bin_luma_reversal_count(data: &TerrainSurfaceTextureData, bin_count: usize) -> usize {
        let mut residual_sums = vec![0.0; bin_count];
        let mut sample_counts = vec![0usize; bin_count];
        for (index, weights) in data.material_weights.iter().enumerate() {
            let bin = ((data.height[index] * bin_count as f32) as usize).min(bin_count - 1);
            let pixel = &data.albedo[index * 4..index * 4 + 3];
            let actual_luma = pixel_luma(&[pixel[0], pixel[1], pixel[2], 255], 1, 0, 0) as f32;
            let expected = [
                PRIMARY[0] as f32 * weights[0]
                    + SECONDARY[0] as f32 * weights[1]
                    + ACCENT[0] as f32 * weights[2],
                PRIMARY[1] as f32 * weights[0]
                    + SECONDARY[1] as f32 * weights[1]
                    + ACCENT[1] as f32 * weights[2],
                PRIMARY[2] as f32 * weights[0]
                    + SECONDARY[2] as f32 * weights[1]
                    + ACCENT[2] as f32 * weights[2],
            ];
            let expected_luma =
                (expected[0] * 77.0 + expected[1] * 150.0 + expected[2] * 29.0) / 256.0;
            residual_sums[bin] += actual_luma - expected_luma;
            sample_counts[bin] += 1;
        }

        let means = residual_sums
            .into_iter()
            .zip(sample_counts)
            .map(|(sum, count)| sum / count.max(1) as f32)
            .collect::<Vec<_>>();
        means
            .windows(3)
            .filter(|window| {
                let incoming = window[1] - window[0];
                let outgoing = window[2] - window[1];
                incoming.abs() >= 0.35
                    && outgoing.abs() >= 0.35
                    && incoming.signum() != outgoing.signum()
            })
            .count()
    }

    fn seam_difference_f32(data: &[f32], size: u32) -> f32 {
        let mut difference = 0.0;
        for coordinate in 0..size {
            difference += (data[field_index(0, coordinate, size)]
                - data[field_index(size - 1, coordinate, size)])
            .abs();
            difference += (data[field_index(coordinate, 0, size)]
                - data[field_index(coordinate, size - 1, size)])
            .abs();
        }
        difference / (size * 2) as f32
    }

    fn neighbor_difference_f32(data: &[f32], size: u32) -> f32 {
        let mut difference = 0.0;
        let mut samples = 0;
        for y in 0..size {
            for x in 0..size - 1 {
                difference +=
                    (data[field_index(x, y, size)] - data[field_index(x + 1, y, size)]).abs();
                difference +=
                    (data[field_index(y, x, size)] - data[field_index(y, x + 1, size)]).abs();
                samples += 2;
            }
        }
        difference / samples as f32
    }

    fn seam_difference_rgba(data: &[u8], size: u32) -> f32 {
        let mut difference = 0u64;
        for coordinate in 0..size {
            difference += u64::from(pixel_luma(data, size, 0, coordinate).abs_diff(pixel_luma(
                data,
                size,
                size - 1,
                coordinate,
            )));
            difference += u64::from(pixel_luma(data, size, coordinate, 0).abs_diff(pixel_luma(
                data,
                size,
                coordinate,
                size - 1,
            )));
        }
        difference as f32 / (size * 2) as f32
    }

    fn neighbor_difference_rgba(data: &[u8], size: u32) -> f32 {
        let mut difference = 0u64;
        let mut samples = 0u64;
        for y in 0..size {
            for x in 0..size - 1 {
                difference += u64::from(pixel_luma(data, size, x, y).abs_diff(pixel_luma(
                    data,
                    size,
                    x + 1,
                    y,
                )));
                difference += u64::from(pixel_luma(data, size, y, x).abs_diff(pixel_luma(
                    data,
                    size,
                    y,
                    x + 1,
                )));
                samples += 2;
            }
        }
        difference as f32 / samples as f32
    }

    fn normal_seam_difference(data: &[u8], size: u32) -> f32 {
        let mut difference = 0.0;
        for coordinate in 0..size {
            difference += normal_pixel(data, size, 0, coordinate).distance(normal_pixel(
                data,
                size,
                size - 1,
                coordinate,
            ));
            difference += normal_pixel(data, size, coordinate, 0).distance(normal_pixel(
                data,
                size,
                coordinate,
                size - 1,
            ));
        }
        difference / (size * 2) as f32
    }

    fn normal_neighbor_difference(data: &[u8], size: u32) -> f32 {
        let mut difference = 0.0;
        let mut samples = 0;
        for y in 0..size {
            for x in 0..size - 1 {
                difference +=
                    normal_pixel(data, size, x, y).distance(normal_pixel(data, size, x + 1, y));
                difference +=
                    normal_pixel(data, size, y, x).distance(normal_pixel(data, size, y, x + 1));
                samples += 2;
            }
        }
        difference / samples as f32
    }

    fn normal_pixel(data: &[u8], size: u32, x: u32, y: u32) -> Vec3 {
        let offset = field_index(x, y, size) * 4;
        decode_normal(&data[offset..offset + 4])
    }
}
