use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashSet;

mod detail_meshes;
mod island_meshes;

pub(crate) use detail_meshes::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, TREE_CANOPY_CARD_COUNT,
    TREE_TRUNK_SEGMENTS, cloud_cluster_mesh, cloud_filament_ribbon_detail_count,
    glider_airflow_trail_mesh, rock_scatter_mesh, tree_canopy_mesh, tree_trunk_mesh,
    updraft_ribbon_mesh,
};
#[cfg(test)]
pub(crate) use detail_meshes::{
    CLOUD_FILAMENT_RIBBON_VERTICES, CLOUD_FILAMENT_RIBBONS_PER_LOBE, DETAIL_CARD_VERTICES,
    ROCK_MESH_RINGS, ROCK_MESH_SEGMENTS, TREE_BRANCH_COUNT, TREE_BRANCH_SEGMENTS,
    TREE_CANOPY_LATITUDE_SEGMENTS, TREE_CANOPY_LONGITUDE_SEGMENTS,
};
pub(crate) use island_meshes::{
    GROUND_COVER_BLADES_PER_PATCH, GROUND_COVER_PATCHES, ISLAND_BODY_SEGMENTS,
    IslandDetailMaterials, TERRAIN_BIOME_PALETTE_COUNT, VERTICES_PER_GROUND_BLADE,
    biome_detail_color_set, biome_detail_materials, island_cliff_mesh, island_ground_cover_mesh,
    island_impostor_mesh, island_terrain_mesh, island_underside_mesh,
    island_visual_surface_position, mesh_terrain_material_channel_count,
    mesh_terrain_material_region_count, mesh_terrain_material_weight_band_count,
    mesh_vertex_color_band_count, mesh_y_range, terrain_biome_palette,
};
#[cfg(test)]
pub(crate) use island_meshes::{
    INDICES_PER_GROUND_BLADE, ISLAND_CLIFF_RINGS, ISLAND_CLIFF_STRATA_BANDS,
    ISLAND_IMPOSTOR_COLOR_BANDS, ISLAND_IMPOSTOR_SEGMENTS, ISLAND_TERRAIN_COLOR_BANDS,
    ISLAND_TERRAIN_MATERIAL_CHANNELS, ISLAND_TERRAIN_MATERIAL_REGIONS,
    ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS, ISLAND_TERRAIN_RINGS,
    ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS, ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE,
    ISLAND_UNDERSIDE_RINGS, island_terrain_vertex_color,
};

pub(crate) const PROCEDURAL_TEXTURE_SIZE: u32 = 64;
pub(crate) const TERRAIN_TEXTURE_SIZE: u32 = 128;

#[allow(clippy::too_many_arguments)]
pub(crate) fn textured_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    perceptual_roughness: f32,
    reflectance: f32,
) -> Handle<StandardMaterial> {
    let material_seed = seed.wrapping_add(1_337);
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        metallic_roughness_texture: Some(
            images.add(procedural_material_map(material_seed, perceptual_roughness)),
        ),
        occlusion_texture: Some(
            images.add(procedural_occlusion_map(material_seed.wrapping_add(23))),
        ),
        depth_map: Some(images.add(procedural_depth_map(
            material_seed.wrapping_add(47),
            ImageFilterMode::Nearest,
        ))),
        parallax_depth_scale: 0.012,
        max_parallax_layer_count: 8.0,
        perceptual_roughness,
        reflectance,
        ..default()
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn terrain_surface_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    perceptual_roughness: f32,
    reflectance: f32,
) -> (Handle<StandardMaterial>, usize) {
    let material_seed = seed.wrapping_add(1_337);
    let surface_data = procedural_terrain_surface_texture_data(
        primary,
        secondary,
        accent,
        seed,
        TERRAIN_TEXTURE_SIZE,
    );
    let detail_bands = texture_detail_band_count(&surface_data);
    let base_color_texture = procedural_srgb_texture(
        surface_data,
        TERRAIN_TEXTURE_SIZE,
        ImageFilterMode::Linear,
        16,
    );

    (
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(images.add(base_color_texture)),
            metallic_roughness_texture: Some(images.add(procedural_material_map_with_size(
                material_seed,
                perceptual_roughness,
                TERRAIN_TEXTURE_SIZE,
            ))),
            occlusion_texture: Some(images.add(procedural_occlusion_map_with_size(
                material_seed.wrapping_add(23),
                TERRAIN_TEXTURE_SIZE,
            ))),
            depth_map: Some(images.add(procedural_depth_map_with_size(
                material_seed.wrapping_add(47),
                ImageFilterMode::Linear,
                TERRAIN_TEXTURE_SIZE,
            ))),
            parallax_depth_scale: 0.018,
            max_parallax_layer_count: 12.0,
            perceptual_roughness,
            reflectance,
            ..default()
        }),
        detail_bands,
    )
}

pub(crate) fn emissive_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    emissive: LinearRgba,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        emissive,
        emissive_exposure_weight: 0.15,
        perceptual_roughness: 0.7,
        reflectance: 0.38,
        ..default()
    })
}

pub(crate) fn water_surface_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.58, 0.86, 0.76),
        base_color_texture: Some(images.add(procedural_surface_texture(
            [54, 154, 210, 210],
            [22, 92, 156, 210],
            [160, 220, 244, 210],
            79,
        ))),
        metallic_roughness_texture: Some(images.add(procedural_material_map(1_079, 0.22))),
        depth_map: Some(images.add(procedural_depth_map(1_113, ImageFilterMode::Linear))),
        parallax_depth_scale: 0.018,
        max_parallax_layer_count: 10.0,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        perceptual_roughness: 0.18,
        reflectance: 0.82,
        clearcoat: 0.85,
        clearcoat_perceptual_roughness: 0.06,
        diffuse_transmission: 0.18,
        specular_transmission: 0.08,
        thickness: 0.08,
        ior: 1.33,
        ..default()
    })
}

pub(crate) fn glider_airflow_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.58, 0.88, 1.0, 0.14),
        emissive: LinearRgba::rgb(0.035, 0.18, 0.42),
        emissive_exposure_weight: 0.12,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.72,
        reflectance: 0.1,
        ..default()
    })
}

pub(crate) fn cloud_surface_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.86, 0.91, 0.96, 0.38),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        reflectance: 0.12,
        diffuse_transmission: 0.18,
        ..default()
    })
}

pub(crate) fn cloud_veil_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.76, 0.84, 0.96, 0.24),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        reflectance: 0.06,
        diffuse_transmission: 0.34,
        ..default()
    })
}

pub(crate) fn updraft_column_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.18, 0.74, 1.0, 0.006),
        emissive: LinearRgba::rgb(0.004, 0.025, 0.045),
        emissive_exposure_weight: 0.12,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.32,
        reflectance: 0.2,
        ..default()
    })
}

pub(crate) fn updraft_ribbon_material(
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.44, 0.92, 1.0, 0.32),
        emissive: LinearRgba::rgb(0.06, 0.9, 1.8),
        emissive_exposure_weight: 0.2,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.4,
        reflectance: 0.18,
        ..default()
    })
}

pub(crate) fn ground_cover_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        metallic_roughness_texture: Some(
            images.add(procedural_material_map(seed.wrapping_add(1_300), 0.94)),
        ),
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 0.94,
        reflectance: 0.2,
        ..default()
    })
}

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
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let fine = texture_noise(x.wrapping_mul(5), y.wrapping_mul(5), seed);
            let grain = texture_noise(x.wrapping_mul(13), y.wrapping_mul(7), seed.wrapping_add(71));
            let broad = smooth_texture_noise(x, y, 22, seed.wrapping_add(19));
            let streak = smooth_texture_noise(
                x.wrapping_mul(2).wrapping_add(y / 2),
                y.wrapping_mul(5).wrapping_add(x / 2),
                12,
                seed.wrapping_add(137),
            );
            let secondary_weight = ((118i16 - broad as i16).max(0) as u16 * 126 / 118)
                .saturating_add((grain > 192) as u16 * 24)
                .min(150);
            let accent_weight = ((broad as i16 - 164).max(0) as u16 * 142 / 91)
                .saturating_add((fine > 222 && grain > 142) as u16 * 70)
                .min(172);
            let vein = (x.wrapping_mul(17) + y.wrapping_mul(29) + seed).is_multiple_of(53);
            let mineral_fleck = fine > 222 && grain > 142;
            let mut color = mix_rgba(primary, secondary, secondary_weight);
            color = mix_rgba(color, accent, accent_weight);

            if vein {
                color = mix_rgba(color, secondary, 104);
            }
            if mineral_fleck {
                color = mix_rgba(color, accent, 96);
            }

            let shade = fine as i16 / 4 + grain as i16 / 7 + streak as i16 / 9 - 82;
            data.extend_from_slice(&shade_rgba(color, shade));
        }
    }

    data
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

pub(crate) fn texture_noise(x: u32, y: u32, seed: u32) -> u8 {
    let mut value = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(2_654_435_761));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 0xff) as u8
}

pub(crate) fn smooth_texture_noise(x: u32, y: u32, cell_size: u32, seed: u32) -> u8 {
    let cell_size = cell_size.max(1);
    let grid_x = x / cell_size;
    let grid_y = y / cell_size;
    let local_x = (x % cell_size) as f32 / cell_size as f32;
    let local_y = (y % cell_size) as f32 / cell_size as f32;
    let weight_x = local_x * local_x * (3.0 - 2.0 * local_x);
    let weight_y = local_y * local_y * (3.0 - 2.0 * local_y);

    let north_west = texture_noise(grid_x, grid_y, seed) as f32;
    let north_east = texture_noise(grid_x + 1, grid_y, seed) as f32;
    let south_west = texture_noise(grid_x, grid_y + 1, seed) as f32;
    let south_east = texture_noise(grid_x + 1, grid_y + 1, seed) as f32;
    let north = north_west + (north_east - north_west) * weight_x;
    let south = south_west + (south_east - south_west) * weight_x;

    (north + (south - north) * weight_y)
        .round()
        .clamp(0.0, 255.0) as u8
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

pub(crate) fn shade_rgba(source: [u8; 4], shade: i16) -> [u8; 4] {
    [
        (source[0] as i16 + shade).clamp(0, 255) as u8,
        (source[1] as i16 + shade).clamp(0, 255) as u8,
        (source[2] as i16 + shade).clamp(0, 255) as u8,
        source[3],
    ]
}

pub(crate) fn mix_color(source: Color, target: Color, target_weight: f32) -> Color {
    source.mix(&target, target_weight.clamp(0.0, 1.0))
}

pub(crate) fn random_unit(seed: u32, x: u32, salt: u32) -> f32 {
    texture_noise(x.wrapping_mul(17).wrapping_add(salt), salt, seed) as f32 / 255.0
}
