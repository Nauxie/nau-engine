use super::super::{ground_cover_material, textured_material};
use super::constants::{ISLAND_CLIFF_STRATA_BANDS, TERRAIN_UV_TILES_PER_METER};
use bevy::prelude::*;
use nau_engine::world::{
    IslandArtDirection, IslandPaletteFamily, SkyIsland, island_art_directions,
};

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
    let profiles = island_art_directions();
    let profile = profiles[island_index % profiles.len()];
    let base = palette_family_base(profile.palette_family);
    let grass = apply_palette_direction(base.grass, profile);
    let moss = apply_palette_direction(base.moss, profile);
    let meadow = apply_palette_direction(base.meadow, profile);
    let clay = apply_palette_direction(base.clay, profile);
    let rock = apply_palette_direction(base.rock, profile);

    TerrainBiomePalette {
        grass,
        moss,
        meadow,
        clay,
        rock,
        region_tints: [
            grass.lerp(meadow, 0.18),
            meadow.lerp(clay, 0.14),
            moss.lerp(rock, 0.16),
            clay.lerp(rock, 0.34),
        ],
    }
}

#[derive(Clone, Copy)]
struct PaletteFamilyBase {
    grass: Vec3,
    moss: Vec3,
    meadow: Vec3,
    clay: Vec3,
    rock: Vec3,
}

fn palette_family_base(family: IslandPaletteFamily) -> PaletteFamilyBase {
    let (grass, moss, meadow, clay, rock) = match family {
        IslandPaletteFamily::VerdantSun => (
            (0.34, 0.68, 0.35),
            (0.20, 0.48, 0.32),
            (0.73, 0.69, 0.35),
            (0.59, 0.43, 0.28),
            (0.48, 0.47, 0.41),
        ),
        IslandPaletteFamily::CopperOrchard => (
            (0.52, 0.62, 0.29),
            (0.34, 0.45, 0.26),
            (0.82, 0.60, 0.29),
            (0.68, 0.39, 0.24),
            (0.54, 0.44, 0.37),
        ),
        IslandPaletteFamily::StormSlate => (
            (0.31, 0.50, 0.43),
            (0.20, 0.36, 0.38),
            (0.52, 0.57, 0.49),
            (0.43, 0.40, 0.38),
            (0.36, 0.43, 0.49),
        ),
        IslandPaletteFamily::MistJade => (
            (0.28, 0.60, 0.50),
            (0.16, 0.43, 0.43),
            (0.56, 0.68, 0.55),
            (0.45, 0.46, 0.41),
            (0.43, 0.51, 0.53),
        ),
        IslandPaletteFamily::SapphireWetland => (
            (0.25, 0.56, 0.48),
            (0.14, 0.39, 0.43),
            (0.50, 0.65, 0.57),
            (0.42, 0.44, 0.40),
            (0.38, 0.49, 0.57),
        ),
        IslandPaletteFamily::AlpineFrost => (
            (0.43, 0.61, 0.48),
            (0.29, 0.45, 0.43),
            (0.68, 0.73, 0.62),
            (0.50, 0.48, 0.43),
            (0.51, 0.58, 0.64),
        ),
        IslandPaletteFamily::RuinOchre => (
            (0.52, 0.57, 0.28),
            (0.35, 0.41, 0.26),
            (0.78, 0.57, 0.28),
            (0.67, 0.40, 0.25),
            (0.55, 0.46, 0.38),
        ),
        IslandPaletteFamily::CloudSilver => (
            (0.43, 0.59, 0.45),
            (0.30, 0.44, 0.42),
            (0.66, 0.69, 0.57),
            (0.51, 0.49, 0.44),
            (0.52, 0.56, 0.59),
        ),
        IslandPaletteFamily::PlateauBloom => (
            (0.45, 0.67, 0.35),
            (0.28, 0.48, 0.35),
            (0.82, 0.67, 0.43),
            (0.62, 0.45, 0.34),
            (0.51, 0.49, 0.45),
        ),
    };

    PaletteFamilyBase {
        grass: Vec3::from(grass),
        moss: Vec3::from(moss),
        meadow: Vec3::from(meadow),
        clay: Vec3::from(clay),
        rock: Vec3::from(rock),
    }
}

fn apply_palette_direction(color: Vec3, profile: IslandArtDirection) -> Vec3 {
    let hue_shifted = shift_rgb_hue(color, f32::from(profile.palette_hue_shift_degrees));
    let warmth = f32::from(profile.palette_warmth_percent) / 100.0;
    let warmed = hue_shifted + Vec3::new(0.12, 0.035, -0.10) * warmth;
    let contrast = f32::from(profile.terrain_contrast_percent) / 100.0;

    (Vec3::splat(0.5) + (warmed - Vec3::splat(0.5)) * contrast)
        .clamp(Vec3::splat(0.08), Vec3::splat(0.92))
}

fn shift_rgb_hue(color: Vec3, shift_degrees: f32) -> Vec3 {
    let max = color.max_element();
    let min = color.min_element();
    let delta = max - min;
    if delta <= f32::EPSILON {
        return color;
    }

    let hue_sector = if max == color.x {
        ((color.y - color.z) / delta).rem_euclid(6.0)
    } else if max == color.y {
        (color.z - color.x) / delta + 2.0
    } else {
        (color.x - color.y) / delta + 4.0
    };
    let hue = (hue_sector / 6.0 + shift_degrees / 360.0).rem_euclid(1.0);
    let saturation = delta / max;
    hsv_to_rgb(hue, saturation, max)
}

fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> Vec3 {
    let sector = hue * 6.0;
    let index = sector.floor() as u8;
    let fraction = sector - f32::from(index);
    let low = value * (1.0 - saturation);
    let descending = value * (1.0 - fraction * saturation);
    let ascending = value * (1.0 - (1.0 - fraction) * saturation);

    match index % 6 {
        0 => Vec3::new(value, ascending, low),
        1 => Vec3::new(descending, value, low),
        2 => Vec3::new(low, value, ascending),
        3 => Vec3::new(low, descending, value),
        4 => Vec3::new(ascending, low, value),
        _ => Vec3::new(value, low, descending),
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
    let bark_base = palette.clay.lerp(Vec3::new(0.34, 0.20, 0.12), 0.34);
    let foliage_base = palette.grass.lerp(palette.moss, 0.42);
    let ground_base = palette.grass.lerp(palette.meadow, 0.30);
    let stone_base = palette.rock.lerp(palette.clay, 0.20);

    BiomeDetailColorSet {
        trunk_primary: rgba8(bark_base),
        trunk_secondary: rgba8(bark_base * 0.66),
        trunk_accent: rgba8(bark_base.lerp(Vec3::new(0.80, 0.56, 0.32), 0.42)),
        foliage_primary: rgba8(foliage_base),
        foliage_secondary: rgba8(palette.moss * 0.82),
        foliage_accent: rgba8(foliage_base.lerp(palette.meadow, 0.42)),
        ground_primary: rgba8(ground_base),
        ground_secondary: rgba8(palette.moss.lerp(palette.rock, 0.12)),
        ground_accent: rgba8(palette.meadow.lerp(Vec3::new(0.96, 0.82, 0.46), 0.28)),
        stone_primary: rgba8(stone_base),
        stone_secondary: rgba8(palette.rock * 0.74),
        stone_accent: rgba8(stone_base.lerp(Vec3::new(0.78, 0.74, 0.66), 0.28)),
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
    let (inner_meadow, exposed_edge, highland, dapple) =
        island_terrain_material_factors(island_index, radius, angle, relief_m);
    [
        (highland * 0.68 + inner_meadow * 0.30 + dapple * 2.0).clamp(0.0, 1.0),
        exposed_edge.clamp(0.0, 1.0),
    ]
}

pub(crate) fn balance_terrain_material_weights(weights: &mut [[f32; 2]]) {
    if weights.is_empty() {
        return;
    }

    let target_counts = [
        min_region_count(weights.len(), 130),
        min_region_count(weights.len(), 300),
        min_region_count(weights.len(), 130),
        min_region_count(weights.len(), 160),
    ];
    let representatives = [[0.12, 0.04], [0.31, 0.06], [0.52, 0.08], [0.20, 0.62]];

    for _ in 0..weights.len() {
        let counts = terrain_material_region_counts(weights);
        let Some(target_region) = counts
            .iter()
            .enumerate()
            .find_map(|(region, count)| (*count < target_counts[region]).then_some(region))
        else {
            break;
        };
        let Some(donor_region) = counts
            .iter()
            .enumerate()
            .filter(|(region, count)| **count > target_counts[*region])
            .max_by_key(|(region, count)| *count - target_counts[*region])
            .map(|(region, _)| region)
        else {
            break;
        };
        let Some(index) = best_material_region_donor(weights, donor_region, target_region) else {
            break;
        };

        weights[index] = representatives[target_region];
    }
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

fn min_region_count(vertex_count: usize, promille: usize) -> usize {
    (vertex_count * promille).div_ceil(1000)
}

fn terrain_material_region_counts(weights: &[[f32; 2]]) -> [usize; 4] {
    let mut counts = [0; 4];
    for weight in weights {
        counts[terrain_material_region_id(*weight) as usize] += 1;
    }
    counts
}

fn best_material_region_donor(
    weights: &[[f32; 2]],
    donor_region: usize,
    target_region: usize,
) -> Option<usize> {
    let target_weight = match target_region {
        0 => [0.12, 0.04],
        1 => [0.31, 0.06],
        2 => [0.52, 0.08],
        _ => [0.20, 0.62],
    };

    weights
        .iter()
        .enumerate()
        .filter(|(_, weight)| terrain_material_region_id(**weight) as usize == donor_region)
        .min_by(|(_, a), (_, b)| {
            material_region_target_distance(**a, target_weight)
                .total_cmp(&material_region_target_distance(**b, target_weight))
        })
        .map(|(index, _)| index)
}

fn material_region_target_distance(weight: [f32; 2], target: [f32; 2]) -> f32 {
    (weight[0] - target[0]).abs() + (weight[1] - target[1]).abs()
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
    let watercolor_wash = Vec3::new(0.86, 0.74, 0.48) * (inner_meadow * 0.035 + highland * 0.018);
    let color = palette
        .grass
        .lerp(palette.meadow, inner_meadow * 0.36)
        .lerp(palette.moss, highland * 0.42)
        .lerp(palette.clay, exposed_edge * 0.32)
        .lerp(palette.rock, exposed_edge.powf(1.7) * 0.48)
        .lerp(palette.region_tints[region as usize], 0.32)
        + watercolor_wash
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
    let vertical_stain = (angle * 17.0 + phase + t * 4.0).sin().abs() * 0.07;
    let sun_wash = Vec3::new(0.040, 0.024, -0.014) * (angle * 5.0 + phase).sin();
    let cool_wash = Vec3::new(-0.018, 0.014, 0.034) * (angle * 11.0 - phase * 0.7 + t).cos();
    let stratum_wash = Vec3::new(0.026, 0.012, -0.010) * ((band % 4) as f32 / 3.0);
    let base = if underside {
        Vec3::new(0.30, 0.28, 0.25)
    } else {
        Vec3::new(0.46, 0.42, 0.35)
    };
    let warm = Vec3::new(0.62, 0.49, 0.34);
    let cool = Vec3::new(0.34, 0.38, 0.42);
    let color = base
        .lerp(warm, band_tint * 0.32)
        .lerp(cool, ((band % 3) as f32 / 2.0) * 0.22)
        + sun_wash
        + cool_wash
        + stratum_wash
        - Vec3::splat(vertical_stain + if underside { 0.04 } else { 0.0 });
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
