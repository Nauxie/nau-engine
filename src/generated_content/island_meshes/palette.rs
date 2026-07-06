use super::super::{ground_cover_material, textured_material};
use super::constants::{
    ISLAND_CLIFF_STRATA_BANDS, TERRAIN_BIOME_PALETTE_COUNT, TERRAIN_UV_TILES_PER_METER,
};
use bevy::prelude::*;
use nau_engine::world::{IslandShapeLanguage, SkyIsland};

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
            grass: Vec3::new(0.44, 0.67, 0.36),
            moss: Vec3::new(0.28, 0.48, 0.38),
            meadow: Vec3::new(0.78, 0.68, 0.38),
            clay: Vec3::new(0.62, 0.48, 0.34),
            rock: Vec3::new(0.50, 0.50, 0.46),
            region_tints: [
                Vec3::new(0.38, 0.62, 0.32),
                Vec3::new(0.72, 0.62, 0.34),
                Vec3::new(0.24, 0.44, 0.36),
                Vec3::new(0.48, 0.44, 0.38),
            ],
        },
        2 => TerrainBiomePalette {
            grass: Vec3::new(0.50, 0.60, 0.31),
            moss: Vec3::new(0.34, 0.43, 0.32),
            meadow: Vec3::new(0.78, 0.56, 0.30),
            clay: Vec3::new(0.66, 0.39, 0.27),
            rock: Vec3::new(0.54, 0.45, 0.39),
            region_tints: [
                Vec3::new(0.48, 0.56, 0.28),
                Vec3::new(0.74, 0.51, 0.29),
                Vec3::new(0.30, 0.39, 0.31),
                Vec3::new(0.51, 0.38, 0.34),
            ],
        },
        3 => TerrainBiomePalette {
            grass: Vec3::new(0.29, 0.60, 0.54),
            moss: Vec3::new(0.18, 0.44, 0.48),
            meadow: Vec3::new(0.55, 0.66, 0.55),
            clay: Vec3::new(0.45, 0.46, 0.43),
            rock: Vec3::new(0.44, 0.52, 0.56),
            region_tints: [
                Vec3::new(0.24, 0.54, 0.46),
                Vec3::new(0.50, 0.62, 0.50),
                Vec3::new(0.14, 0.38, 0.44),
                Vec3::new(0.39, 0.48, 0.52),
            ],
        },
        4 => TerrainBiomePalette {
            grass: Vec3::new(0.56, 0.63, 0.32),
            moss: Vec3::new(0.39, 0.49, 0.29),
            meadow: Vec3::new(0.80, 0.68, 0.38),
            clay: Vec3::new(0.58, 0.47, 0.31),
            rock: Vec3::new(0.50, 0.47, 0.40),
            region_tints: [
                Vec3::new(0.50, 0.58, 0.30),
                Vec3::new(0.76, 0.63, 0.36),
                Vec3::new(0.34, 0.44, 0.28),
                Vec3::new(0.48, 0.42, 0.34),
            ],
        },
        _ => TerrainBiomePalette {
            grass: Vec3::new(0.36, 0.68, 0.38),
            moss: Vec3::new(0.22, 0.50, 0.40),
            meadow: Vec3::new(0.72, 0.66, 0.36),
            clay: Vec3::new(0.58, 0.44, 0.30),
            rock: Vec3::new(0.49, 0.48, 0.43),
            region_tints: [
                Vec3::new(0.30, 0.62, 0.32),
                Vec3::new(0.66, 0.60, 0.32),
                Vec3::new(0.18, 0.44, 0.36),
                Vec3::new(0.46, 0.40, 0.34),
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

pub(crate) fn island_terrain_material_weights_for_shape(
    shape_language: IslandShapeLanguage,
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> [f32; 2] {
    let (inner_meadow, exposed_edge, highland, dapple) =
        island_terrain_material_factors(island_index, radius, angle, relief_m);
    let shoulder = smoothstep(0.22, 0.50, radius) * (1.0 - smoothstep(0.74, 0.90, radius));
    let center_clearing = (1.0 - smoothstep(0.14, 0.34, radius)) * 0.18;
    let rim_highland = smoothstep(0.50, 0.70, radius) * (1.0 - smoothstep(0.78, 0.94, radius));
    let phase = island_index as f32 * 0.49;
    let material_grain = (angle * 11.0 + phase).sin() * 0.018
        + (radius * 23.0 - phase * 0.4).cos() * 0.012
        + dapple * 0.35;
    if shape_language == IslandShapeLanguage::NeedlePerch {
        let center_clear_unit = 1.0 - smoothstep(0.12, 0.34, radius);
        let needle_grain =
            (angle * 17.0 - phase).sin() * 0.055 + (radius * 19.0 + phase).cos() * 0.035;
        let lush_highland =
            (0.30 + highland * 0.18 + rim_highland * 0.16 + shoulder * 0.02 + needle_grain
                - center_clear_unit * 0.32)
                .clamp(0.0, 1.0);

        return [lush_highland, exposed_edge.clamp(0.0, 1.0)];
    }

    let mut lush_highland =
        highland * 0.68 + inner_meadow * 0.30 + shoulder * 0.06 - center_clearing + material_grain;

    lush_highland += match shape_language {
        IslandShapeLanguage::TerraceMesa => rim_highland * 0.12 - center_clearing * 0.45,
        IslandShapeLanguage::BrokenCrescent => rim_highland * 0.55 - center_clearing * 0.25,
        IslandShapeLanguage::LakeBasin => {
            rim_highland * 0.20 + shoulder * 0.30 - center_clearing * 0.25
        }
        IslandShapeLanguage::NeedlePerch => unreachable!("needle perches return above"),
        IslandShapeLanguage::RingGarden => rim_highland * 0.30,
        IslandShapeLanguage::CliffSlab => rim_highland * 0.24 - center_clearing * 0.35,
        IslandShapeLanguage::UndercutCaveIsland => rim_highland * 0.28 - center_clearing * 0.35,
        IslandShapeLanguage::RuinFoundation => rim_highland * 0.25 - center_clearing * 0.80,
        IslandShapeLanguage::SpireCluster => rim_highland * 0.12 - center_clearing * 1.15,
        IslandShapeLanguage::SteppedStairIsland => rim_highland * 0.32 - center_clearing * 0.80,
        IslandShapeLanguage::BridgeRemnant => rim_highland * 0.18 - center_clearing * 0.80,
        IslandShapeLanguage::WaterfallShelf => rim_highland * 0.28 - center_clearing * 0.25,
        IslandShapeLanguage::PlateauFragment => rim_highland * 0.18 - center_clearing * 0.65,
        IslandShapeLanguage::MeadowShelf => rim_highland * 0.28 - center_clearing * 0.35,
    };

    [lush_highland.clamp(0.0, 1.0), exposed_edge.clamp(0.0, 1.0)]
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

pub(crate) fn island_terrain_vertex_color_for_shape(
    shape_language: IslandShapeLanguage,
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> [f32; 4] {
    let palette = terrain_biome_palette(island_index);
    let (inner_meadow, exposed_edge, highland, dapple) =
        island_terrain_material_factors(island_index, radius, angle, relief_m);
    let region = terrain_material_region_id(island_terrain_material_weights_for_shape(
        shape_language,
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

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0).max(f32::EPSILON)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
