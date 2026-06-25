use super::super::{ground_cover_material, textured_material};
use super::constants::{
    ISLAND_CLIFF_STRATA_BANDS, TERRAIN_BIOME_PALETTE_COUNT, TERRAIN_UV_TILES_PER_METER,
};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

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
