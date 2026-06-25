use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use nau_engine::world::SkyIsland;
use std::collections::HashSet;

mod detail_meshes;

pub(crate) use detail_meshes::{
    CLOUD_BANK_LOBES, CLOUD_VEIL_LOBES, CLOUD_WISP_CARDS_PER_LOBE, TREE_CANOPY_CARD_COUNT,
    TREE_TRUNK_SEGMENTS, cloud_cluster_mesh, glider_airflow_trail_mesh, rock_scatter_mesh,
    tree_canopy_mesh, tree_trunk_mesh, updraft_ribbon_mesh,
};
#[cfg(test)]
pub(crate) use detail_meshes::{
    DETAIL_CARD_VERTICES, ROCK_MESH_RINGS, ROCK_MESH_SEGMENTS, TREE_BRANCH_COUNT,
    TREE_BRANCH_SEGMENTS, TREE_CANOPY_LATITUDE_SEGMENTS, TREE_CANOPY_LONGITUDE_SEGMENTS,
};

pub(crate) const PROCEDURAL_TEXTURE_SIZE: u32 = 64;
pub(crate) const TERRAIN_TEXTURE_SIZE: u32 = 128;
pub(crate) const TERRAIN_UV_TILES_PER_METER: f32 = 1.0 / 12.0;
pub(crate) const TERRAIN_BIOME_PALETTE_COUNT: usize = 5;
pub(crate) const GROUND_COVER_PATCHES: usize = 44;
pub(crate) const GROUND_COVER_BLADES_PER_PATCH: usize = 5;
pub(crate) const VERTICES_PER_GROUND_BLADE: usize = 5;
pub(crate) const INDICES_PER_GROUND_BLADE: usize = 9;

#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_COLOR_BANDS: usize = 5;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS: usize = 12;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_MATERIAL_CHANNELS: usize = 3;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_MATERIAL_REGIONS: usize = 4;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS: usize = 44;
#[cfg(test)]
pub(crate) const ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE: usize = 240;
#[cfg(test)]
pub(crate) const ISLAND_IMPOSTOR_COLOR_BANDS: usize = 18;
pub(crate) const ISLAND_CLIFF_STRATA_BANDS: usize = 9;

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

pub(crate) fn island_visual_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let x = island.center.x + island.half_extents.x * normalized_offset.x;
    let z = island.center.z + island.half_extents.y * normalized_offset.y;

    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
}

pub(crate) const ISLAND_TERRAIN_RINGS: usize = 24;
pub(crate) const ISLAND_BODY_SEGMENTS: usize = 96;
pub(crate) const ISLAND_IMPOSTOR_SEGMENTS: usize = 48;
pub(crate) const ISLAND_CLIFF_RINGS: usize = 8;
pub(crate) const ISLAND_UNDERSIDE_RINGS: usize = 7;

pub(crate) fn island_silhouette_scale(island_index: usize, angle: f32) -> f32 {
    let phase = island_index as f32 * 0.73;
    (1.0 + 0.09 * (angle * 3.0 + phase).sin()
        + 0.055 * (angle * 7.0 - phase * 0.4).cos()
        + 0.032 * (angle * 11.0 + phase * 1.7).sin())
    .clamp(0.82, 1.18)
}

pub(crate) fn island_playable_silhouette_scale(island_index: usize, angle: f32) -> f32 {
    island_silhouette_scale(island_index, angle).min(1.0)
}

pub(crate) fn island_polar_position(
    island: SkyIsland,
    angle: f32,
    radius_scale: f32,
    y: f32,
) -> [f32; 3] {
    [
        island.center.x + angle.cos() * island.half_extents.x * radius_scale,
        y,
        island.center.z + angle.sin() * island.half_extents.y * radius_scale,
    ]
}

pub(crate) fn mesh_y_range(mesh: &Mesh) -> f32 {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return 0.0;
    };
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for position in positions {
        min_y = min_y.min(position[1]);
        max_y = max_y.max(position[1]);
    }
    if min_y.is_finite() && max_y.is_finite() {
        max_y - min_y
    } else {
        0.0
    }
}

pub(crate) fn mesh_vertex_color_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        return 0;
    };
    let mut bands = HashSet::new();
    for color in colors {
        bands.insert([
            (color[0].clamp(0.0, 1.0) * 31.0).round() as u8,
            (color[1].clamp(0.0, 1.0) * 31.0).round() as u8,
            (color[2].clamp(0.0, 1.0) * 31.0).round() as u8,
        ]);
    }
    bands.len()
}

pub(crate) fn mesh_terrain_material_weight_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    let mut bands = HashSet::new();
    for weight in weights {
        bands.insert([
            (weight[0].clamp(0.0, 1.0) * 15.0).round() as u8,
            (weight[1].clamp(0.0, 1.0) * 15.0).round() as u8,
        ]);
    }
    bands.len()
}

pub(crate) fn mesh_terrain_material_channel_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    let base = weights
        .iter()
        .any(|weight| weight[0] < 0.18 && weight[1] < 0.18);
    let lush = weights.iter().any(|weight| weight[0] > 0.18);
    let exposed = weights.iter().any(|weight| weight[1] > 0.18);
    usize::from(base) + usize::from(lush) + usize::from(exposed)
}

pub(crate) fn terrain_material_region_id(weight: [f32; 2]) -> u8 {
    let lush_highland = weight[0].clamp(0.0, 1.0);
    let exposed_edge = weight[1].clamp(0.0, 1.0);

    if exposed_edge >= 0.48 {
        3
    } else if lush_highland >= 0.42 {
        2
    } else if lush_highland >= 0.24 || exposed_edge >= 0.10 {
        1
    } else {
        0
    }
}

pub(crate) fn mesh_terrain_material_region_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    weights
        .iter()
        .map(|weight| terrain_material_region_id(*weight))
        .collect::<HashSet<_>>()
        .len()
}

pub(crate) fn color_array(color: Vec3) -> [f32; 4] {
    [
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
        1.0,
    ]
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerrainBiomePalette {
    pub(crate) grass: Vec3,
    pub(crate) moss: Vec3,
    pub(crate) meadow: Vec3,
    pub(crate) clay: Vec3,
    pub(crate) rock: Vec3,
    pub(crate) region_tints: [Vec3; 4],
}

pub(crate) fn terrain_biome_palette(island_index: usize) -> TerrainBiomePalette {
    match island_index % TERRAIN_BIOME_PALETTE_COUNT {
        1 => TerrainBiomePalette {
            grass: Vec3::new(0.30, 0.56, 0.24),
            moss: Vec3::new(0.20, 0.38, 0.24),
            meadow: Vec3::new(0.62, 0.56, 0.30),
            clay: Vec3::new(0.50, 0.38, 0.27),
            rock: Vec3::new(0.43, 0.42, 0.38),
            region_tints: [
                Vec3::new(0.26, 0.50, 0.22),
                Vec3::new(0.57, 0.53, 0.28),
                Vec3::new(0.18, 0.34, 0.25),
                Vec3::new(0.40, 0.36, 0.30),
            ],
        },
        2 => TerrainBiomePalette {
            grass: Vec3::new(0.36, 0.49, 0.24),
            moss: Vec3::new(0.25, 0.34, 0.25),
            meadow: Vec3::new(0.61, 0.45, 0.24),
            clay: Vec3::new(0.56, 0.32, 0.20),
            rock: Vec3::new(0.48, 0.39, 0.33),
            region_tints: [
                Vec3::new(0.34, 0.46, 0.22),
                Vec3::new(0.59, 0.42, 0.23),
                Vec3::new(0.24, 0.32, 0.24),
                Vec3::new(0.43, 0.32, 0.27),
            ],
        },
        3 => TerrainBiomePalette {
            grass: Vec3::new(0.18, 0.48, 0.42),
            moss: Vec3::new(0.12, 0.34, 0.38),
            meadow: Vec3::new(0.42, 0.52, 0.44),
            clay: Vec3::new(0.35, 0.36, 0.34),
            rock: Vec3::new(0.38, 0.44, 0.46),
            region_tints: [
                Vec3::new(0.16, 0.44, 0.36),
                Vec3::new(0.40, 0.50, 0.40),
                Vec3::new(0.10, 0.30, 0.36),
                Vec3::new(0.34, 0.40, 0.42),
            ],
        },
        4 => TerrainBiomePalette {
            grass: Vec3::new(0.42, 0.52, 0.25),
            moss: Vec3::new(0.30, 0.39, 0.23),
            meadow: Vec3::new(0.62, 0.55, 0.30),
            clay: Vec3::new(0.48, 0.39, 0.25),
            rock: Vec3::new(0.43, 0.40, 0.34),
            region_tints: [
                Vec3::new(0.36, 0.48, 0.23),
                Vec3::new(0.59, 0.52, 0.29),
                Vec3::new(0.28, 0.36, 0.22),
                Vec3::new(0.42, 0.36, 0.28),
            ],
        },
        _ => TerrainBiomePalette {
            grass: Vec3::new(0.22, 0.58, 0.29),
            moss: Vec3::new(0.15, 0.42, 0.32),
            meadow: Vec3::new(0.55, 0.52, 0.28),
            clay: Vec3::new(0.48, 0.36, 0.25),
            rock: Vec3::new(0.42, 0.40, 0.36),
            region_tints: [
                Vec3::new(0.19, 0.52, 0.24),
                Vec3::new(0.50, 0.49, 0.25),
                Vec3::new(0.14, 0.36, 0.30),
                Vec3::new(0.39, 0.34, 0.29),
            ],
        },
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BiomeDetailColorSet {
    pub(crate) trunk_primary: [u8; 4],
    pub(crate) trunk_secondary: [u8; 4],
    pub(crate) trunk_accent: [u8; 4],
    pub(crate) foliage_primary: [u8; 4],
    pub(crate) foliage_secondary: [u8; 4],
    pub(crate) foliage_accent: [u8; 4],
    pub(crate) ground_primary: [u8; 4],
    pub(crate) ground_secondary: [u8; 4],
    pub(crate) ground_accent: [u8; 4],
    pub(crate) stone_primary: [u8; 4],
    pub(crate) stone_secondary: [u8; 4],
    pub(crate) stone_accent: [u8; 4],
}

#[derive(Clone)]
pub(crate) struct IslandDetailMaterials {
    pub(crate) trunk: Handle<StandardMaterial>,
    pub(crate) foliage: Handle<StandardMaterial>,
    pub(crate) ground_cover: Handle<StandardMaterial>,
    pub(crate) stone: Handle<StandardMaterial>,
}

pub(crate) fn rgba8(color: Vec3) -> [u8; 4] {
    [
        (color.x.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.y.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.z.clamp(0.0, 1.0) * 255.0).round() as u8,
        255,
    ]
}

pub(crate) fn biome_detail_color_set(island_index: usize) -> BiomeDetailColorSet {
    let palette = terrain_biome_palette(island_index);
    let bark_base = palette.clay.lerp(Vec3::new(0.25, 0.14, 0.08), 0.46);
    let foliage_base = palette.grass.lerp(palette.moss, 0.54);
    let ground_base = palette.grass.lerp(palette.meadow, 0.24);
    let stone_base = palette.rock.lerp(palette.clay, 0.28);

    BiomeDetailColorSet {
        trunk_primary: rgba8(bark_base),
        trunk_secondary: rgba8(bark_base * 0.58),
        trunk_accent: rgba8(bark_base.lerp(Vec3::new(0.72, 0.46, 0.26), 0.38)),
        foliage_primary: rgba8(foliage_base),
        foliage_secondary: rgba8(palette.moss * 0.72),
        foliage_accent: rgba8(foliage_base.lerp(palette.meadow, 0.34)),
        ground_primary: rgba8(ground_base),
        ground_secondary: rgba8(palette.moss.lerp(palette.rock, 0.16)),
        ground_accent: rgba8(palette.meadow.lerp(Vec3::new(0.92, 0.78, 0.38), 0.22)),
        stone_primary: rgba8(stone_base),
        stone_secondary: rgba8(palette.rock * 0.68),
        stone_accent: rgba8(stone_base.lerp(Vec3::splat(0.76), 0.22)),
    }
}

pub(crate) fn biome_detail_materials(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    island_index: usize,
) -> IslandDetailMaterials {
    let colors = biome_detail_color_set(island_index);
    let seed_base = 211 + island_index as u32 * 41;

    IslandDetailMaterials {
        trunk: textured_material(
            images,
            materials,
            colors.trunk_primary,
            colors.trunk_secondary,
            colors.trunk_accent,
            seed_base,
            0.96,
            0.16,
        ),
        foliage: textured_material(
            images,
            materials,
            colors.foliage_primary,
            colors.foliage_secondary,
            colors.foliage_accent,
            seed_base + 7,
            0.88,
            0.22,
        ),
        ground_cover: ground_cover_material(
            images,
            materials,
            colors.ground_primary,
            colors.ground_secondary,
            colors.ground_accent,
            seed_base + 13,
        ),
        stone: textured_material(
            images,
            materials,
            colors.stone_primary,
            colors.stone_secondary,
            colors.stone_accent,
            seed_base + 19,
            0.98,
            0.18,
        ),
    }
}

pub(crate) fn island_terrain_material_factors(
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> (f32, f32, f32, f32) {
    let phase = island_index as f32 * 0.49;
    let inner_meadow = ((0.42 - radius) / 0.42).clamp(0.0, 1.0);
    let exposed_edge = ((radius - 0.72) / 0.28).clamp(0.0, 1.0);
    let highland = ((relief_m + 0.18) / 0.82).clamp(0.0, 1.0);
    let dapple = (angle * 13.0 + phase).sin() * 0.025
        + (angle * 29.0 - phase * 0.6).cos() * 0.015
        + (radius * 31.0 + phase).sin() * 0.018;
    (inner_meadow, exposed_edge, highland, dapple)
}

pub(crate) fn island_terrain_material_weights(
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> [f32; 2] {
    let (inner_meadow, exposed_edge, highland, _) =
        island_terrain_material_factors(island_index, radius, angle, relief_m);
    [
        (highland * 0.72 + inner_meadow * 0.28).clamp(0.0, 1.0),
        exposed_edge.clamp(0.0, 1.0),
    ]
}

pub(crate) fn island_terrain_vertex_color(
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> [f32; 4] {
    let palette = terrain_biome_palette(island_index);
    let (inner_meadow, exposed_edge, highland, dapple) =
        island_terrain_material_factors(island_index, radius, angle, relief_m);
    let region = terrain_material_region_id(island_terrain_material_weights(
        island_index,
        radius,
        angle,
        relief_m,
    ));
    let color = palette
        .grass
        .lerp(palette.meadow, inner_meadow * 0.36)
        .lerp(palette.moss, highland * 0.42)
        .lerp(palette.clay, exposed_edge * 0.38)
        .lerp(palette.rock, exposed_edge.powf(1.7) * 0.48)
        .lerp(palette.region_tints[region as usize], 0.32)
        + Vec3::splat(dapple);
    color_array(color)
}

pub(crate) fn island_rock_vertex_color(
    island_index: usize,
    angle: f32,
    t: f32,
    underside: bool,
) -> [f32; 4] {
    let phase = island_index as f32 * 0.61;
    let band = ((t * ISLAND_CLIFF_STRATA_BANDS as f32 + phase * 0.13).floor() as usize)
        % ISLAND_CLIFF_STRATA_BANDS;
    let band_tint = band as f32 / (ISLAND_CLIFF_STRATA_BANDS - 1) as f32;
    let vertical_stain = (angle * 17.0 + phase + t * 4.0).sin().abs() * 0.08;
    let base = if underside {
        Vec3::new(0.25, 0.22, 0.18)
    } else {
        Vec3::new(0.38, 0.35, 0.3)
    };
    let warm = Vec3::new(0.48, 0.39, 0.29);
    let cool = Vec3::new(0.28, 0.3, 0.31);
    let color = base
        .lerp(warm, band_tint * 0.32)
        .lerp(cool, ((band % 3) as f32 / 2.0) * 0.22)
        - Vec3::splat(vertical_stain + if underside { 0.07 } else { 0.0 });
    color_array(color)
}

pub(crate) fn island_terrain_uv(
    island_index: usize,
    island: SkyIsland,
    x: f32,
    z: f32,
) -> [f32; 2] {
    let island_offset = island_index as f32 * 0.173;
    [
        (x - island.center.x) * TERRAIN_UV_TILES_PER_METER + 0.5 + island_offset,
        (z - island.center.z) * TERRAIN_UV_TILES_PER_METER + 0.5 + island_offset * 1.37,
    ]
}

pub(crate) fn island_terrain_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let vertex_count = 1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut material_weights = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(
        ISLAND_BODY_SEGMENTS * 3 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS * 6,
    );

    let center_y = island.mesh_top_y_at(island.center);
    positions.push([island.center.x, center_y, island.center.z]);
    uvs.push(island_terrain_uv(
        island_index,
        island,
        island.center.x,
        island.center.z,
    ));
    colors.push(island_terrain_vertex_color(
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));
    material_weights.push(island_terrain_material_weights(
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));

    for ring in 1..=ISLAND_TERRAIN_RINGS {
        let radius = ring as f32 / ISLAND_TERRAIN_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_scale = island_playable_silhouette_scale(island_index, angle);
            let radius_scale = radius * (1.0 + radius.powf(1.35) * (edge_scale - 1.0));
            let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
            let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
            let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));

            positions.push([x, y, z]);
            uvs.push(island_terrain_uv(island_index, island, x, z));
            colors.push(island_terrain_vertex_color(
                island_index,
                radius,
                angle,
                y - island.mesh_top_y(),
            ));
            material_weights.push(island_terrain_material_weights(
                island_index,
                radius,
                angle,
                y - island.mesh_top_y(),
            ));
        }
    }

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (1 + (ring - 1) * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for segment in 0..ISLAND_BODY_SEGMENTS {
        indices.extend([0, ring_index(1, segment + 1), ring_index(1, segment)]);
    }

    for ring in 1..ISLAND_TERRAIN_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let inner_current = ring_index(ring, segment);
            let inner_next = ring_index(ring, segment + 1);
            let outer_current = ring_index(ring + 1, segment);
            let outer_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                inner_current,
                inner_next,
                outer_current,
                inner_next,
                outer_next,
                outer_current,
            ]);
        }
    }

    let normals = smooth_normals_from_triangles(&positions, &indices);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, material_weights)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn island_cliff_surface_position(
    island_index: usize,
    island: SkyIsland,
    angle: f32,
    t: f32,
) -> [f32; 3] {
    let phase = island_index as f32 * 0.73;
    let shelf_variation = 1.0
        + t * 0.035 * (angle * 5.0 + phase + t * 1.7).sin()
        + t * 0.025 * (angle * 13.0 - phase * 0.3 + t * 2.1).cos();
    let ledge_phase = (t * ISLAND_CLIFF_STRATA_BANDS as f32 + phase * 0.11).fract();
    let ledge_shelf = (1.0 - (ledge_phase - 0.5).abs() * 2.0).max(0.0).powf(2.2);
    let radius_scale = island_playable_silhouette_scale(island_index, angle)
        * (1.0 - t.powf(1.18) * 0.34)
        * shelf_variation
        * (1.0 + ledge_shelf * 0.028);
    let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
    let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
    let vertical_fracture = t
        * ((angle * 8.0 + phase).sin() * (0.45 + t) + (angle * 17.0 - phase).cos() * 0.22).abs()
        * island.thickness
        * 0.045;
    let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z))
        - 0.06
        - island.thickness * (t * 0.78)
        - ledge_shelf * island.thickness * 0.018
        - vertical_fracture;

    [x, y, z]
}

pub(crate) fn island_cliff_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let mut positions = Vec::with_capacity((ISLAND_CLIFF_RINGS + 1) * ISLAND_BODY_SEGMENTS);
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut colors = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(ISLAND_CLIFF_RINGS * ISLAND_BODY_SEGMENTS * 6);

    for ring in 0..=ISLAND_CLIFF_RINGS {
        let t = ring as f32 / ISLAND_CLIFF_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            positions.push(island_cliff_surface_position(
                island_index,
                island,
                angle,
                t,
            ));
            uvs.push([segment as f32 / ISLAND_BODY_SEGMENTS as f32 * 4.0, t]);
            colors.push(island_rock_vertex_color(island_index, angle, t, false));
        }
    }

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (ring * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_CLIFF_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let upper_current = ring_index(ring, segment);
            let upper_next = ring_index(ring, segment + 1);
            let lower_current = ring_index(ring + 1, segment);
            let lower_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                upper_current,
                upper_next,
                lower_current,
                upper_next,
                lower_next,
                lower_current,
            ]);
        }
    }

    let normals = smooth_normals_from_triangles_oriented(&positions, &indices, Vec3::Z, false);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn island_underside_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let ring_vertex_count = (ISLAND_UNDERSIDE_RINGS + 1) * ISLAND_BODY_SEGMENTS;
    let bottom_index = ring_vertex_count as u32;
    let mut positions = Vec::with_capacity(ring_vertex_count + 1);
    let mut uvs = Vec::with_capacity(ring_vertex_count + 1);
    let mut colors = Vec::with_capacity(ring_vertex_count + 1);
    let mut indices = Vec::with_capacity(
        ISLAND_UNDERSIDE_RINGS * ISLAND_BODY_SEGMENTS * 6 + ISLAND_BODY_SEGMENTS * 3,
    );
    let phase = island_index as f32 * 0.73;
    let top_y = island.mesh_top_y();

    for ring in 0..=ISLAND_UNDERSIDE_RINGS {
        let t = ring as f32 / ISLAND_UNDERSIDE_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            if ring == 0 {
                positions.push(island_cliff_surface_position(
                    island_index,
                    island,
                    angle,
                    1.0,
                ));
                uvs.push([0.5 + angle.cos() * 0.34, 0.5 + angle.sin() * 0.34]);
                colors.push(island_rock_vertex_color(island_index, angle, t, true));
                continue;
            }

            let twist = 0.045 * (angle * 6.0 + phase + t * 2.4).sin();
            let radius_scale = island_playable_silhouette_scale(island_index, angle)
                * (0.66 * (1.0 - t).powf(1.35) + 0.18 * t)
                * (1.0 + twist);
            let y = top_y
                - island.thickness * (0.82 + t * 0.58)
                - island.thickness * 0.06 * (angle * 5.0 - phase).sin().abs();

            positions.push(island_polar_position(island, angle, radius_scale, y));
            uvs.push([
                0.5 + angle.cos() * (0.34 - t * 0.19),
                0.5 + angle.sin() * (0.34 - t * 0.19),
            ]);
            colors.push(island_rock_vertex_color(island_index, angle, t, true));
        }
    }

    positions.push([
        island.center.x,
        top_y - island.thickness * 1.58,
        island.center.z,
    ]);
    uvs.push([0.5, 0.5]);
    colors.push(island_rock_vertex_color(island_index, 0.0, 1.0, true));

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (ring * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_UNDERSIDE_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let upper_current = ring_index(ring, segment);
            let upper_next = ring_index(ring, segment + 1);
            let lower_current = ring_index(ring + 1, segment);
            let lower_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                upper_current,
                upper_next,
                lower_current,
                upper_next,
                lower_next,
                lower_current,
            ]);
        }
    }
    for segment in 0..ISLAND_BODY_SEGMENTS {
        indices.extend([
            ring_index(ISLAND_UNDERSIDE_RINGS, segment),
            ring_index(ISLAND_UNDERSIDE_RINGS, segment + 1),
            bottom_index,
        ]);
    }

    let normals = smooth_normals_from_triangles_oriented(&positions, &indices, Vec3::NEG_Y, false);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn smooth_normals_from_triangles(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<[f32; 3]> {
    smooth_normals_from_triangles_oriented(positions, indices, Vec3::Y, true)
}

pub(crate) fn smooth_normals_from_triangles_oriented(
    positions: &[[f32; 3]],
    indices: &[u32],
    fallback: Vec3,
    force_positive_y: bool,
) -> Vec<[f32; 3]> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    let fallback = fallback.normalize_or_zero();
    let fallback = if fallback.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        fallback
    };

    for triangle in indices.chunks_exact(3) {
        let a_index = triangle[0] as usize;
        let b_index = triangle[1] as usize;
        let c_index = triangle[2] as usize;
        let a = Vec3::from_array(positions[a_index]);
        let b = Vec3::from_array(positions[b_index]);
        let c = Vec3::from_array(positions[c_index]);
        let mut face_normal = (b - a).cross(c - a).normalize_or_zero();

        if force_positive_y && face_normal.y < 0.0 {
            face_normal = -face_normal;
        }
        if face_normal.length_squared() <= f32::EPSILON {
            face_normal = fallback;
        }

        normals[a_index] += face_normal;
        normals[b_index] += face_normal;
        normals[c_index] += face_normal;
    }

    normals
        .into_iter()
        .map(|normal| {
            if normal.length_squared() <= f32::EPSILON {
                fallback.to_array()
            } else {
                normal.normalize().to_array()
            }
        })
        .collect()
}

pub(crate) fn island_impostor_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let top_center_y = island.mesh_top_y() - 0.16;
    let shoulder_center_y = top_center_y - island.thickness * 0.30;
    let lower_center_y = top_center_y - island.thickness * 0.62;
    let bottom_y = top_center_y - island.thickness * 0.92;
    let phase = island_index as f32 * 0.71;
    let top_ring_start = 1;
    let shoulder_ring_start = top_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let lower_ring_start = shoulder_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let bottom_index = lower_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let mut positions = Vec::with_capacity(bottom_index + 1);
    let mut uvs = Vec::with_capacity(bottom_index + 1);
    let mut colors = Vec::with_capacity(bottom_index + 1);
    let mut indices = Vec::with_capacity(ISLAND_IMPOSTOR_SEGMENTS * 18);

    positions.push([island.center.x, top_center_y, island.center.z]);
    uvs.push([0.5, 0.5]);
    colors.push(island_terrain_vertex_color(island_index, 0.0, 0.0, 0.0));

    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
        let edge_variation =
            1.0 + 0.09 * (angle * 3.0 + phase).sin() + 0.045 * (angle * 7.0 - phase).cos();
        let radius_x = island.half_extents.x * 0.9 * edge_variation;
        let radius_z = island.half_extents.y * 0.9 * edge_variation;
        let x = island.center.x + angle.cos() * radius_x;
        let z = island.center.z + angle.sin() * radius_z;
        let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) - 0.18;

        positions.push([x, y, z]);
        uvs.push([0.5 + angle.cos() * 0.45, 0.5 + angle.sin() * 0.45]);
        colors.push(island_terrain_vertex_color(
            island_index,
            0.9,
            angle,
            y - island.mesh_top_y(),
        ));
    }

    for (ring, (center_y, radius_scale, t, underside)) in [
        (shoulder_center_y, 0.72, 0.34, false),
        (lower_center_y, 0.48, 0.78, true),
    ]
    .into_iter()
    .enumerate()
    {
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_variation =
                1.0 + 0.08 * (angle * 4.0 + phase).sin() - 0.035 * (angle * 8.0).cos();
            let radius_x = island.half_extents.x * radius_scale * edge_variation;
            let radius_z = island.half_extents.y * radius_scale * edge_variation;
            let x = island.center.x + angle.cos() * radius_x;
            let z = island.center.z + angle.sin() * radius_z;
            let y = center_y - island.thickness * 0.05 * (angle * 5.0 + phase).sin().abs();

            positions.push([x, y, z]);
            uvs.push([
                0.5 + angle.cos() * (0.35 - ring as f32 * 0.11),
                0.78 + angle.sin() * 0.11 + ring as f32 * 0.14,
            ]);
            colors.push(island_rock_vertex_color(island_index, angle, t, underside));
        }
    }

    positions.push([island.center.x, bottom_y, island.center.z]);
    uvs.push([0.5, 1.0]);
    colors.push(island_rock_vertex_color(island_index, 0.0, 1.0, true));

    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let next = (segment + 1) % ISLAND_IMPOSTOR_SEGMENTS;
        let top_current = (top_ring_start + segment) as u32;
        let top_next = (top_ring_start + next) as u32;
        let shoulder_current = (shoulder_ring_start + segment) as u32;
        let shoulder_next = (shoulder_ring_start + next) as u32;
        let lower_current = (lower_ring_start + segment) as u32;
        let lower_next = (lower_ring_start + next) as u32;
        let bottom = bottom_index as u32;

        indices.extend([0, top_next, top_current]);
        indices.extend([top_current, top_next, shoulder_current]);
        indices.extend([top_next, shoulder_next, shoulder_current]);
        indices.extend([shoulder_current, shoulder_next, lower_current]);
        indices.extend([shoulder_next, lower_next, lower_current]);
        indices.extend([lower_current, lower_next, bottom]);
    }

    let normals = smooth_normals_from_triangles(&positions, &indices);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

pub(crate) fn island_ground_cover_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let blade_count = GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH;
    let mut positions = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut normals = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut uvs = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut indices = Vec::with_capacity(blade_count * INDICES_PER_GROUND_BLADE);
    let seed = island_index as u32 * 41 + 503;

    for patch in 0..GROUND_COVER_PATCHES {
        let base_angle = random_unit(seed, patch as u32, 3) * std::f32::consts::TAU;
        let radius = random_unit(seed, patch as u32, 11).sqrt() * 0.90;
        let jitter = Vec2::new(
            (random_unit(seed, patch as u32, 17) - 0.5) * 0.08,
            (random_unit(seed, patch as u32, 23) - 0.5) * 0.08,
        );
        let normalized_offset = Vec2::new(base_angle.cos(), base_angle.sin()) * radius + jitter;
        let x = island.center.x + normalized_offset.x * island.half_extents.x;
        let z = island.center.z + normalized_offset.y * island.half_extents.y;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) + 0.08;

        for blade in 0..GROUND_COVER_BLADES_PER_PATCH {
            let blade_phase = base_angle
                + blade as f32 * std::f32::consts::TAU / GROUND_COVER_BLADES_PER_PATCH as f32;
            let width = 0.14 + random_unit(seed, patch as u32, 31 + blade as u32) * 0.15;
            let height = 0.72 + random_unit(seed, patch as u32, 43 + blade as u32) * 0.86;
            let lean = Vec3::new(blade_phase.cos(), 0.0, blade_phase.sin())
                * (0.1 + random_unit(seed, patch as u32, 53 + blade as u32) * 0.24);
            push_ground_cover_blade(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                Vec3::new(x, surface_y, z),
                blade_phase,
                width,
                height,
                lean,
                patch,
            );
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_ground_cover_blade(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    origin: Vec3,
    angle: f32,
    width: f32,
    height: f32,
    lean: Vec3,
    patch: usize,
) {
    let right = Vec3::new(angle.cos(), 0.0, angle.sin());
    let side = right * (width * 0.5);
    let mid_side = right * (width * 0.26);
    let mid = origin + Vec3::Y * (height * 0.54) + lean * 0.42;
    let tip = origin + Vec3::Y * height + lean;
    let blade_normal = Vec3::new(right.z * 0.35, 0.8, -right.x * 0.35).normalize();
    let start = positions.len() as u32;

    positions.extend([
        (origin - side).to_array(),
        (origin + side).to_array(),
        (mid - mid_side).to_array(),
        (mid + mid_side).to_array(),
        tip.to_array(),
    ]);
    normals.extend([blade_normal.to_array(); VERTICES_PER_GROUND_BLADE]);
    let uv_offset = if patch.is_multiple_of(2) { 0.0 } else { 0.5 };
    uvs.extend([
        [uv_offset, 1.0],
        [uv_offset + 0.42, 1.0],
        [uv_offset + 0.10, 0.46],
        [uv_offset + 0.32, 0.46],
        [uv_offset + 0.21, 0.0],
    ]);
    indices.extend([
        start,
        start + 1,
        start + 2,
        start + 1,
        start + 3,
        start + 2,
        start + 2,
        start + 3,
        start + 4,
    ]);
}

pub(crate) fn random_unit(seed: u32, x: u32, salt: u32) -> f32 {
    texture_noise(x.wrapping_mul(17).wrapping_add(salt), salt, seed) as f32 / 255.0
}
