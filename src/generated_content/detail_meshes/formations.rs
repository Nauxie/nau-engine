use super::super::{
    island_playable_normalized_offset, island_visual_surface_position, random_unit,
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::{
    IslandBiome, IslandScaleClass, IslandTerrainArchetype, IslandVisualMotif, SkyIsland,
    authored_island_composition,
};

const MAX_ROCK_FORMATIONS_PER_ISLAND: usize = 2;
const ARRIVAL_LANE_MIN_X: f32 = -0.48;
const ARRIVAL_LANE_MAX_X: f32 = 0.38;
const ARRIVAL_LANE_HALF_WIDTH: f32 = 0.24;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum RockFormationKind {
    BasaltCrown,
    WeatheredArch,
    BoulderSpine,
    StackedMonoliths,
    CrystalOutcrop,
}

impl RockFormationKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::BasaltCrown => "basalt_crown",
            Self::WeatheredArch => "weathered_arch",
            Self::BoulderSpine => "boulder_spine",
            Self::StackedMonoliths => "stacked_monoliths",
            Self::CrystalOutcrop => "crystal_outcrop",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::BasaltCrown => "clustered basalt crown",
            Self::WeatheredArch => "weathered rock arch",
            Self::BoulderSpine => "fractured boulder spine",
            Self::StackedMonoliths => "stacked leaning monoliths",
            Self::CrystalOutcrop => "faceted crystal outcrop",
        }
    }

    #[cfg(test)]
    const COUNT: usize = 5;

    #[cfg(test)]
    fn index(self) -> usize {
        match self {
            Self::BasaltCrown => 0,
            Self::WeatheredArch => 1,
            Self::BoulderSpine => 2,
            Self::StackedMonoliths => 3,
            Self::CrystalOutcrop => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IslandRockFormationSpec {
    pub(crate) kind: RockFormationKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) collision_half_extents: Option<Vec3>,
    pub(crate) camera_half_extents: Option<Vec3>,
    scale: f32,
    seed: u32,
}

impl IslandRockFormationSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        rock_formation_mesh(self.kind, self.scale, self.seed)
    }
}

pub(crate) fn island_rock_formation_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandRockFormationSpec> {
    let Some(composition) = authored_island_composition(island.name) else {
        return Vec::new();
    };
    let kinds = rock_formation_kinds(island, composition.visual_motif);
    let layout_seed = detail_seed(island_index, island, 0xc731_84a9);

    kinds
        .into_iter()
        .take(MAX_ROCK_FORMATIONS_PER_ISLAND)
        .enumerate()
        .map(|(formation_index, kind)| {
            let sample = formation_index as u32;
            let initial_offset = perimeter_offset(
                island,
                formation_index,
                layout_seed,
                composition.visual_motif,
            );
            let placement_angle = initial_offset.y.atan2(initial_offset.x);
            let rotation_y = formation_rotation(kind, placement_angle)
                + (random_unit(layout_seed, sample, 0x3d1) - 0.5) * 0.34;
            let scale = formation_scale(island, kind);
            let (collision_half_extents, camera_half_extents) = formation_bounds(kind, scale);
            let normalized_offset = collision_bounded_offset(
                island,
                initial_offset,
                collision_half_extents
                    .map(|half_extents| rotated_aabb_half_extents(half_extents, rotation_y)),
            );

            IslandRockFormationSpec {
                kind,
                label: kind.visual_name(),
                translation: island_visual_surface_position(island, normalized_offset)
                    + Vec3::Y * 0.035,
                rotation_y,
                collision_half_extents,
                camera_half_extents: Some(camera_half_extents),
                scale,
                seed: mixed_seed(
                    layout_seed
                        ^ sample.wrapping_mul(0x27d4_eb2d)
                        ^ (kind as u32).wrapping_mul(0x1656_67b1),
                ),
            }
        })
        .collect()
}

fn rock_formation_kinds(island: SkyIsland, motif: IslandVisualMotif) -> Vec<RockFormationKind> {
    let primary = match motif {
        IslandVisualMotif::StormStone => Some(RockFormationKind::BasaltCrown),
        IslandVisualMotif::MistArch | IslandVisualMotif::CaveMouth => {
            Some(RockFormationKind::WeatheredArch)
        }
        IslandVisualMotif::CairnShelf
        | IslandVisualMotif::WindRibbon
        | IslandVisualMotif::PlateauRim => Some(RockFormationKind::BoulderSpine),
        IslandVisualMotif::NeedleSpire
        | IslandVisualMotif::CrownPerch
        | IslandVisualMotif::RuinStair => Some(RockFormationKind::StackedMonoliths),
        IslandVisualMotif::GardenRing
        | IslandVisualMotif::LakeBasin
        | IslandVisualMotif::ThermalRing
        | IslandVisualMotif::WaterfallMeadow => Some(RockFormationKind::CrystalOutcrop),
        IslandVisualMotif::OrchardGrove => Some(RockFormationKind::BoulderSpine),
        IslandVisualMotif::LaunchBeacon => match island.terrain_archetype {
            IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard => {
                Some(RockFormationKind::BasaltCrown)
            }
            IslandTerrainArchetype::Needle | IslandTerrainArchetype::CrownRidge => {
                Some(RockFormationKind::StackedMonoliths)
            }
            _ => None,
        },
    }
    .or(match island.terrain_archetype {
        IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard => {
            Some(RockFormationKind::BasaltCrown)
        }
        IslandTerrainArchetype::MistArch | IslandTerrainArchetype::CloudGate => {
            Some(RockFormationKind::WeatheredArch)
        }
        IslandTerrainArchetype::Needle | IslandTerrainArchetype::CrownRidge => {
            Some(RockFormationKind::StackedMonoliths)
        }
        IslandTerrainArchetype::SapphireBasin => Some(RockFormationKind::CrystalOutcrop),
        IslandTerrainArchetype::BrokenStair | IslandTerrainArchetype::TerracedSpur => {
            Some(RockFormationKind::BoulderSpine)
        }
        _ => match island.world_tags.biome {
            IslandBiome::Storm => Some(RockFormationKind::BasaltCrown),
            IslandBiome::Mist => Some(RockFormationKind::WeatheredArch),
            IslandBiome::Alpine | IslandBiome::Ruin => Some(RockFormationKind::StackedMonoliths),
            IslandBiome::Lake | IslandBiome::Garden => Some(RockFormationKind::CrystalOutcrop),
            IslandBiome::Meadow | IslandBiome::Orchard => None,
        },
    });

    let Some(primary) = primary else {
        return Vec::new();
    };
    let mut kinds = vec![primary];
    if matches!(
        island.world_tags.scale_class,
        IslandScaleClass::Large | IslandScaleClass::Vast | IslandScaleClass::HugePlateau
    ) {
        let secondary = match island.world_tags.biome {
            IslandBiome::Storm => RockFormationKind::BoulderSpine,
            IslandBiome::Mist => RockFormationKind::StackedMonoliths,
            IslandBiome::Alpine => RockFormationKind::CrystalOutcrop,
            IslandBiome::Lake => RockFormationKind::BoulderSpine,
            IslandBiome::Ruin => RockFormationKind::WeatheredArch,
            IslandBiome::Garden | IslandBiome::Orchard => RockFormationKind::CrystalOutcrop,
            IslandBiome::Meadow
                if island.terrain_archetype == IslandTerrainArchetype::SkyPlateau =>
            {
                RockFormationKind::StackedMonoliths
            }
            IslandBiome::Meadow => RockFormationKind::BoulderSpine,
        };
        if secondary != primary {
            kinds.push(secondary);
        }
    }
    kinds
}

fn perimeter_offset(
    island: SkyIsland,
    formation_index: usize,
    seed: u32,
    motif: IslandVisualMotif,
) -> Vec2 {
    let sample = formation_index as u32;
    let mut angle = random_unit(seed, sample, 0x291) * std::f32::consts::TAU
        + formation_index as f32 * 2.371
        + motif as u32 as f32 * 0.137
        + island.terrain_archetype.index() as f32 * 0.071;
    let radius = 0.57 + random_unit(seed, sample, 0x29b) * 0.17;
    let mut offset = Vec2::new(angle.cos(), angle.sin()) * radius;

    if occupies_arrival_lane(offset) {
        angle += if random_unit(seed, sample, 0x2a1) > 0.5 {
            0.88
        } else {
            -0.88
        };
        offset = Vec2::new(angle.cos(), angle.sin()) * radius;
    }

    island_playable_normalized_offset(island, offset)
}

fn occupies_arrival_lane(offset: Vec2) -> bool {
    offset.x > ARRIVAL_LANE_MIN_X
        && offset.x < ARRIVAL_LANE_MAX_X
        && offset.y.abs() < ARRIVAL_LANE_HALF_WIDTH
}

fn collision_bounded_offset(
    island: SkyIsland,
    mut offset: Vec2,
    collision_half_extents: Option<Vec3>,
) -> Vec2 {
    let Some(half_extents) = collision_half_extents else {
        return offset;
    };

    for _ in 0..24 {
        if collision_footprint_fits(island, offset, half_extents) {
            return offset;
        }
        offset *= 0.96;
    }
    offset
}

fn collision_footprint_fits(island: SkyIsland, offset: Vec2, half_extents: Vec3) -> bool {
    const FOOTPRINT_MARGIN_M: f32 = 0.04;
    let normalized_half_extents = Vec2::new(
        half_extents.x / island.half_extents.x.max(0.001),
        half_extents.z / island.half_extents.y.max(0.001),
    );
    let normalized_margin = FOOTPRINT_MARGIN_M / island.half_extents.min_element().max(0.001);

    [-1.0_f32, 0.0, 1.0].into_iter().all(|x_sign| {
        [-1.0_f32, 0.0, 1.0].into_iter().all(|z_sign| {
            let sample = offset
                + Vec2::new(
                    normalized_half_extents.x * x_sign,
                    normalized_half_extents.y * z_sign,
                );
            sample.length()
                <= island.playable_silhouette_scale(sample.y.atan2(sample.x)) - normalized_margin
        })
    })
}

fn rotated_aabb_half_extents(half_extents: Vec3, rotation_y: f32) -> Vec3 {
    let cosine = rotation_y.cos().abs();
    let sine = rotation_y.sin().abs();
    Vec3::new(
        cosine * half_extents.x + sine * half_extents.z,
        half_extents.y,
        sine * half_extents.x + cosine * half_extents.z,
    )
}

fn formation_rotation(kind: RockFormationKind, placement_angle: f32) -> f32 {
    match kind {
        RockFormationKind::WeatheredArch | RockFormationKind::BoulderSpine => {
            placement_angle + std::f32::consts::FRAC_PI_2
        }
        RockFormationKind::BasaltCrown
        | RockFormationKind::StackedMonoliths
        | RockFormationKind::CrystalOutcrop => placement_angle + std::f32::consts::PI,
    }
}

fn formation_scale(island: SkyIsland, kind: RockFormationKind) -> f32 {
    let class_scale = match island.world_tags.scale_class {
        IslandScaleClass::Tiny => 0.80,
        IslandScaleClass::Small => 0.90,
        IslandScaleClass::Medium => 1.0,
        IslandScaleClass::Large => 1.08,
        IslandScaleClass::Vast => 1.16,
        IslandScaleClass::HugePlateau => 1.24,
    };
    let biome_scale = match island.world_tags.biome {
        IslandBiome::Storm | IslandBiome::Alpine => 1.10,
        IslandBiome::Ruin => 1.05,
        IslandBiome::Lake | IslandBiome::Mist => 1.0,
        IslandBiome::Meadow | IslandBiome::Garden | IslandBiome::Orchard => 0.94,
    };
    let kind_scale = match kind {
        RockFormationKind::WeatheredArch => 0.95,
        RockFormationKind::CrystalOutcrop => 0.88,
        RockFormationKind::BasaltCrown
        | RockFormationKind::BoulderSpine
        | RockFormationKind::StackedMonoliths => 1.0,
    };

    ((island.half_extents.min_element() * 0.075).clamp(1.35, 4.8)
        * class_scale
        * biome_scale
        * kind_scale)
        .clamp(1.05, 6.2)
}

fn formation_bounds(kind: RockFormationKind, scale: f32) -> (Option<Vec3>, Vec3) {
    // Runtime centers each bound at translation + Vec3::Y * half_extents.y.
    match kind {
        RockFormationKind::BasaltCrown => (None, Vec3::new(3.2, 3.8, 3.1) * scale),
        RockFormationKind::WeatheredArch => (None, Vec3::new(3.5, 3.0, 1.7) * scale),
        RockFormationKind::BoulderSpine => (None, Vec3::new(4.4, 1.9, 2.0) * scale),
        RockFormationKind::StackedMonoliths => (
            Some(Vec3::new(0.95, 2.90, 0.90) * scale),
            Vec3::new(2.8, 4.1, 2.4) * scale,
        ),
        RockFormationKind::CrystalOutcrop => (
            Some(Vec3::new(0.88, 2.10, 0.88) * scale),
            Vec3::new(2.5, 3.6, 2.4) * scale,
        ),
    }
}

fn rock_formation_mesh(kind: RockFormationKind, scale: f32, seed: u32) -> Mesh {
    let mut mesh = MeshBuffers::default();
    match kind {
        RockFormationKind::BasaltCrown => build_basalt_crown(&mut mesh, scale, seed),
        RockFormationKind::WeatheredArch => build_weathered_arch(&mut mesh, scale, seed),
        RockFormationKind::BoulderSpine => build_boulder_spine(&mut mesh, scale, seed),
        RockFormationKind::StackedMonoliths => build_stacked_monoliths(&mut mesh, scale, seed),
        RockFormationKind::CrystalOutcrop => build_crystal_outcrop(&mut mesh, scale, seed),
    }
    mesh.build()
}

fn build_basalt_crown(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    for column in 0..11 {
        let t = column as f32 / 11.0;
        let angle =
            t * std::f32::consts::TAU + (random_unit(seed, column as u32, 0x401) - 0.5) * 0.20;
        let ring = if column < 8 { 1.0 } else { 0.44 };
        let radius_from_center = (1.72 + random_unit(seed, column as u32, 0x407) * 0.72) * ring;
        let height = if column < 8 {
            2.15 + random_unit(seed, column as u32, 0x40d) * 2.25
        } else {
            3.25 + random_unit(seed, column as u32, 0x413) * 1.40
        };
        let radius = 0.34 + random_unit(seed, column as u32, 0x419) * 0.20;
        let lean = Vec2::new(
            (random_unit(seed, column as u32, 0x421) - 0.5) * 0.30,
            (random_unit(seed, column as u32, 0x427) - 0.5) * 0.30,
        );
        mesh.append_irregular_spire(
            Vec3::new(
                angle.cos() * radius_from_center,
                0.0,
                angle.sin() * radius_from_center,
            ) * scale,
            radius * scale,
            height * scale,
            6,
            seed ^ (column as u32).wrapping_mul(0x42d),
            lean * scale,
        );
    }
    append_rock_cluster(mesh, Vec3::ZERO, 7, 1.75, scale, seed ^ 0x439);
}

fn build_weathered_arch(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    for side in [-1.0_f32, 1.0] {
        let side_index = usize::from(side > 0.0);
        mesh.append_irregular_spire(
            Vec3::new(side * 2.08, 0.0, 0.0) * scale,
            0.82 * scale,
            (3.15 + random_unit(seed, side_index as u32, 0x501) * 0.35) * scale,
            8,
            seed ^ (side_index as u32).wrapping_mul(0x50b),
            Vec2::new(-side * 0.20, 0.04) * scale,
        );
        mesh.append_irregular_spire(
            Vec3::new(side * 2.55, 0.0, 0.20) * scale,
            0.52 * scale,
            1.80 * scale,
            7,
            seed ^ (side_index as u32).wrapping_mul(0x511) ^ 0x517,
            Vec2::new(side * 0.10, -0.08) * scale,
        );
    }
    append_arch_blocks(
        mesh,
        Vec3::ZERO,
        2.10 * scale,
        2.68 * scale,
        1.42 * scale,
        1.18 * scale,
        11,
        seed ^ 0x523,
    );
    append_rock_cluster(
        mesh,
        Vec3::new(1.25, 0.0, 0.42),
        7,
        1.35,
        scale,
        seed ^ 0x52f,
    );
}

fn build_boulder_spine(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    for boulder in 0..10 {
        let t = boulder as f32 / 9.0;
        let x = (t - 0.5) * 7.60;
        let z = (t * std::f32::consts::PI * 1.7).sin() * 0.62
            + (random_unit(seed, boulder as u32, 0x601) - 0.5) * 0.32;
        let radius = 0.48 + random_unit(seed, boulder as u32, 0x607) * 0.34;
        let height = 0.78 + random_unit(seed, boulder as u32, 0x60d) * 1.05;
        mesh.append_irregular_spire(
            Vec3::new(x, 0.0, z) * scale,
            radius * scale,
            height * scale,
            7,
            seed ^ (boulder as u32).wrapping_mul(0x613),
            Vec2::new(
                (random_unit(seed, boulder as u32, 0x619) - 0.5) * 0.32,
                (random_unit(seed, boulder as u32, 0x61f) - 0.5) * 0.20,
            ) * scale,
        );
    }
    for fin in 0..5 {
        let t = fin as f32 / 4.0;
        let half_height = 0.82 + t * 0.36;
        mesh.append_box(
            Vec3::new((t - 0.5) * 5.80, half_height, (t * 5.2).sin() * 0.48) * scale,
            Vec3::new(0.20, half_height, 0.66) * scale,
            Quat::from_euler(
                EulerRot::YXZ,
                0.18 + t * 0.24,
                (random_unit(seed, fin as u32, 0x62b) - 0.5) * 0.28,
                (random_unit(seed, fin as u32, 0x631) - 0.5) * 0.22,
            ),
        );
    }
}

fn build_stacked_monoliths(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    let stack_positions = [
        Vec3::new(-1.45, 0.0, -0.65),
        Vec3::new(0.0, 0.0, 0.15),
        Vec3::new(1.45, 0.0, -0.35),
        Vec3::new(0.65, 0.0, 1.20),
    ];
    for (stack_index, base) in stack_positions.into_iter().enumerate() {
        let block_count = if stack_index == 1 { 4 } else { 3 };
        let mut y = 0.0;
        for block in 0..block_count {
            let sample = (stack_index * 7 + block) as u32;
            let height = 0.72 + random_unit(seed, sample, 0x701) * 0.48;
            let half_extents = Vec3::new(
                0.48 - block as f32 * 0.045,
                height * 0.5,
                0.40 - block as f32 * 0.030,
            ) * scale;
            mesh.append_box(
                (base
                    + Vec3::new(
                        (random_unit(seed, sample, 0x707) - 0.5) * 0.18,
                        y + height * 0.5,
                        (random_unit(seed, sample, 0x70d) - 0.5) * 0.16,
                    ))
                    * scale,
                half_extents,
                Quat::from_euler(
                    EulerRot::YXZ,
                    (random_unit(seed, sample, 0x713) - 0.5) * 0.30,
                    (random_unit(seed, sample, 0x719) - 0.5) * 0.16,
                    (random_unit(seed, sample, 0x71f) - 0.5) * 0.20,
                ),
            );
            y += height * 0.92;
        }
        mesh.append_irregular_spire(
            (base + Vec3::new(0.0, y, 0.0)) * scale,
            0.34 * scale,
            (0.82 + stack_index as f32 * 0.12) * scale,
            6,
            seed ^ (stack_index as u32).wrapping_mul(0x727),
            Vec2::new(0.08, -0.05) * scale,
        );
    }
    append_rock_cluster(
        mesh,
        Vec3::new(0.0, 0.0, 0.20),
        7,
        1.70,
        scale,
        seed ^ 0x72d,
    );
}

fn build_crystal_outcrop(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    append_rock_cluster(mesh, Vec3::ZERO, 8, 1.75, scale, seed ^ 0x801);
    for crystal in 0..9 {
        let sample = crystal as u32;
        let angle =
            crystal as f32 / 9.0 * std::f32::consts::TAU + random_unit(seed, sample, 0x807) * 0.30;
        let radius = if crystal < 3 {
            0.35 + crystal as f32 * 0.20
        } else {
            0.90 + random_unit(seed, sample, 0x80d) * 0.92
        };
        let height = if crystal < 3 {
            2.65 + random_unit(seed, sample, 0x813) * 1.25
        } else {
            1.05 + random_unit(seed, sample, 0x819) * 1.35
        };
        mesh.append_crystal(
            Vec3::new(angle.cos() * radius, 0.10, angle.sin() * radius) * scale,
            (0.24 + random_unit(seed, sample, 0x821) * 0.18) * scale,
            height * scale,
            6,
            Vec2::new(
                (random_unit(seed, sample, 0x827) - 0.5) * 0.30,
                (random_unit(seed, sample, 0x82d) - 0.5) * 0.30,
            ) * scale,
        );
    }
}

fn append_rock_cluster(
    mesh: &mut MeshBuffers,
    center: Vec3,
    count: usize,
    radius: f32,
    scale: f32,
    seed: u32,
) {
    for rock in 0..count {
        let sample = rock as u32;
        let angle = random_unit(seed, sample, 0x901) * std::f32::consts::TAU;
        let distance = random_unit(seed, sample, 0x907).sqrt() * radius;
        mesh.append_irregular_spire(
            (center + Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance)) * scale,
            (0.24 + random_unit(seed, sample, 0x90d) * 0.34) * scale,
            (0.34 + random_unit(seed, sample, 0x913) * 0.62) * scale,
            7,
            seed ^ sample.wrapping_mul(0x919),
            Vec2::new(
                (random_unit(seed, sample, 0x91f) - 0.5) * 0.12,
                (random_unit(seed, sample, 0x925) - 0.5) * 0.12,
            ) * scale,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_arch_blocks(
    mesh: &mut MeshBuffers,
    center: Vec3,
    half_width: f32,
    spring_y: f32,
    crown_height: f32,
    depth: f32,
    block_count: usize,
    seed: u32,
) {
    let arc_piece = std::f32::consts::PI * (half_width + crown_height) * 0.25 / block_count as f32;
    for block in 0..block_count {
        if block == block_count / 2 + 2 {
            continue;
        }
        let t = (block as f32 + 0.5) / block_count as f32;
        let angle = std::f32::consts::PI * (1.0 - t);
        let chip = (random_unit(seed, block as u32, 0xa01) - 0.5) * 0.11;
        mesh.append_box(
            center
                + Vec3::new(
                    angle.cos() * half_width,
                    spring_y + angle.sin() * crown_height,
                    chip * depth,
                ),
            Vec3::new(arc_piece * 0.55, crown_height * 0.20, depth * 0.50),
            Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2 + chip),
        );
    }
}

#[derive(Default)]
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl MeshBuffers {
    fn append_box(&mut self, center: Vec3, half_extents: Vec3, rotation: Quat) {
        let half_extents = Vec3::new(
            half_extents.x.max(0.01),
            half_extents.y.max(0.01),
            half_extents.z.max(0.01),
        );
        let vertical_radius = (rotation * Vec3::X).y.abs() * half_extents.x
            + (rotation * Vec3::Y).y.abs() * half_extents.y
            + (rotation * Vec3::Z).y.abs() * half_extents.z;
        let center = center + Vec3::Y * (vertical_radius - center.y).max(0.0);
        let corner = |x: f32, y: f32, z: f32| {
            center
                + rotation * Vec3::new(x * half_extents.x, y * half_extents.y, z * half_extents.z)
        };
        let faces = [
            (
                Vec3::X,
                [
                    corner(1.0, -1.0, -1.0),
                    corner(1.0, -1.0, 1.0),
                    corner(1.0, 1.0, 1.0),
                    corner(1.0, 1.0, -1.0),
                ],
            ),
            (
                Vec3::NEG_X,
                [
                    corner(-1.0, -1.0, 1.0),
                    corner(-1.0, -1.0, -1.0),
                    corner(-1.0, 1.0, -1.0),
                    corner(-1.0, 1.0, 1.0),
                ],
            ),
            (
                Vec3::Y,
                [
                    corner(-1.0, 1.0, -1.0),
                    corner(1.0, 1.0, -1.0),
                    corner(1.0, 1.0, 1.0),
                    corner(-1.0, 1.0, 1.0),
                ],
            ),
            (
                Vec3::NEG_Y,
                [
                    corner(-1.0, -1.0, 1.0),
                    corner(1.0, -1.0, 1.0),
                    corner(1.0, -1.0, -1.0),
                    corner(-1.0, -1.0, -1.0),
                ],
            ),
            (
                Vec3::Z,
                [
                    corner(1.0, -1.0, 1.0),
                    corner(-1.0, -1.0, 1.0),
                    corner(-1.0, 1.0, 1.0),
                    corner(1.0, 1.0, 1.0),
                ],
            ),
            (
                Vec3::NEG_Z,
                [
                    corner(-1.0, -1.0, -1.0),
                    corner(1.0, -1.0, -1.0),
                    corner(1.0, 1.0, -1.0),
                    corner(-1.0, 1.0, -1.0),
                ],
            ),
        ];

        for (local_normal, face_positions) in faces {
            let start = self.positions.len() as u32;
            let normal = (rotation * local_normal).normalize();
            for (position, uv) in
                face_positions
                    .into_iter()
                    .zip([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])
            {
                self.positions.push(position.to_array());
                self.normals.push(normal.to_array());
                self.uvs.push(uv);
            }
            self.indices
                .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
        }
    }

    fn append_irregular_spire(
        &mut self,
        base_center: Vec3,
        radius: f32,
        height: f32,
        segments: usize,
        seed: u32,
        lean: Vec2,
    ) {
        const RINGS: [(f32, f32); 4] = [(0.0, 0.78), (0.30, 1.0), (0.70, 0.66), (1.0, 0.12)];
        let bottom_center = self.positions.len() as u32;
        self.positions.push(base_center.to_array());
        self.normals.push(Vec3::NEG_Y.to_array());
        self.uvs.push([0.5, 0.0]);

        for (ring_index, (t, profile)) in RINGS.into_iter().enumerate() {
            let ring_center = base_center + Vec3::new(lean.x * t, height * t, lean.y * t);
            for segment in 0..segments {
                let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
                let fracture = (random_unit(seed, (ring_index * segments + segment) as u32, 0xb01)
                    - 0.5)
                    * 0.20;
                let radial = radius * profile * (1.0 + fracture);
                let normal =
                    Vec3::new(angle.cos(), 0.16 + (1.0 - t) * 0.18, angle.sin()).normalize();
                self.positions.push(
                    (ring_center + Vec3::new(angle.cos(), 0.0, angle.sin()) * radial).to_array(),
                );
                self.normals.push(normal.to_array());
                self.uvs
                    .push([segment as f32 / segments as f32, t.clamp(0.0, 1.0)]);
            }
        }

        let first_ring = bottom_center + 1;
        for segment in 0..segments {
            let next = ((segment + 1) % segments) as u32;
            self.indices.extend([
                bottom_center,
                first_ring + next,
                first_ring + segment as u32,
            ]);
        }
        for ring in 0..RINGS.len() - 1 {
            let start = first_ring + (ring * segments) as u32;
            let next_start = start + segments as u32;
            for segment in 0..segments {
                let next = ((segment + 1) % segments) as u32;
                let a = start + segment as u32;
                let b = start + next;
                let c = next_start + segment as u32;
                let d = next_start + next;
                self.indices.extend([a, c, b, b, c, d]);
            }
        }

        let top_center = self.positions.len() as u32;
        self.positions
            .push((base_center + Vec3::new(lean.x, height + radius * 0.08, lean.y)).to_array());
        self.normals.push(Vec3::Y.to_array());
        self.uvs.push([0.5, 1.0]);
        let top_ring = first_ring + ((RINGS.len() - 1) * segments) as u32;
        for segment in 0..segments {
            let next = ((segment + 1) % segments) as u32;
            self.indices
                .extend([top_center, top_ring + segment as u32, top_ring + next]);
        }
    }

    fn append_crystal(
        &mut self,
        base_center: Vec3,
        radius: f32,
        height: f32,
        segments: usize,
        lean: Vec2,
    ) {
        let base_start = self.positions.len() as u32;
        for segment in 0..segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
            self.positions
                .push((base_center + radial * radius).to_array());
            self.normals.push(radial.to_array());
            self.uvs.push([segment as f32 / segments as f32, 0.0]);
        }
        let shoulder_start = self.positions.len() as u32;
        for segment in 0..segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
            self.positions.push(
                (base_center
                    + Vec3::new(lean.x * 0.72, height * 0.72, lean.y * 0.72)
                    + radial * radius * 0.76)
                    .to_array(),
            );
            self.normals
                .push((radial + Vec3::Y * 0.10).normalize().to_array());
            self.uvs.push([segment as f32 / segments as f32, 0.72]);
        }
        let tip = self.positions.len() as u32;
        self.positions
            .push((base_center + Vec3::new(lean.x, height, lean.y)).to_array());
        self.normals.push(Vec3::Y.to_array());
        self.uvs.push([0.5, 1.0]);

        for segment in 0..segments {
            let next = ((segment + 1) % segments) as u32;
            let base_a = base_start + segment as u32;
            let base_b = base_start + next;
            let shoulder_a = shoulder_start + segment as u32;
            let shoulder_b = shoulder_start + next;
            self.indices
                .extend([base_a, shoulder_a, base_b, base_b, shoulder_a, shoulder_b]);
            self.indices.extend([shoulder_a, tip, shoulder_b]);
        }
    }

    fn build(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(self.indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
    }
}

fn detail_seed(island_index: usize, island: SkyIsland, salt: u32) -> u32 {
    let mut seed = (island_index as u32)
        .wrapping_mul(0x9e37_79b9)
        .wrapping_add(salt);
    for byte in island.name.bytes() {
        seed = (seed ^ byte as u32).wrapping_mul(0x0100_0193);
    }
    seed ^= island.half_extents.x.to_bits().rotate_left(7);
    seed ^= island.half_extents.y.to_bits().rotate_left(17);
    seed ^= island.thickness.to_bits().rotate_left(23);
    mixed_seed(seed)
}

fn mixed_seed(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use nau_engine::world::SkyRoute;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values,
            _ => panic!("formation mesh should expose Float32x3 positions"),
        }
    }

    fn axis_range(positions: &[[f32; 3]], axis: usize) -> f32 {
        let (min, max) = positions.iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(min, max), position| (min.min(position[axis]), max.max(position[axis])),
        );
        max - min
    }

    fn normalized_offset(island: SkyIsland, translation: Vec3) -> Vec2 {
        Vec2::new(
            (translation.x - island.center.x) / island.half_extents.x,
            (translation.z - island.center.z) / island.half_extents.y,
        )
    }

    #[test]
    fn rock_formation_specs_are_deterministic_and_capped() {
        let route = SkyRoute::default();
        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let first = island_rock_formation_specs(island_index, island);
            let second = island_rock_formation_specs(island_index, island);
            assert_eq!(first, second, "{} should be deterministic", island.name);
            assert!(
                first.len() <= MAX_ROCK_FORMATIONS_PER_ISLAND,
                "{} exceeded the formation cap",
                island.name
            );
            for spec in first {
                let first_mesh = spec.build_mesh();
                let second_mesh = spec.build_mesh();
                assert_eq!(positions(&first_mesh), positions(&second_mesh));
            }
        }
    }

    #[test]
    fn authored_route_covers_every_rock_formation_kind() {
        let route = SkyRoute::default();
        let mut seen = [false; RockFormationKind::COUNT];

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_rock_formation_specs(island_index, island) {
                seen[spec.kind.index()] = true;
                assert!(!spec.kind.label().is_empty());
                assert!(!spec.kind.visual_name().is_empty());
            }
        }

        assert!(seen.into_iter().all(|covered| covered));
    }

    #[test]
    fn formations_stay_playable_perimeter_biased_and_clear_of_arrival_lane() {
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_rock_formation_specs(island_index, island) {
                let offset = normalized_offset(island, spec.translation);
                assert!(
                    island.contains_horizontal(spec.translation),
                    "{} placed {} outside its playable footprint",
                    island.name,
                    spec.label
                );
                assert!(
                    offset.length() >= 0.49,
                    "{} placed {} too close to the central route lane",
                    island.name,
                    spec.label
                );
                assert!(
                    !occupies_arrival_lane(offset),
                    "{} placed {} inside the arrival lane",
                    island.name,
                    spec.label
                );
                if let Some(half_extents) = spec.collision_half_extents {
                    assert!(
                        collision_footprint_fits(
                            island,
                            offset,
                            rotated_aabb_half_extents(half_extents, spec.rotation_y),
                        ),
                        "{} placed {} with collision beyond the playable footprint",
                        island.name,
                        spec.label
                    );
                }
            }
        }
    }

    #[test]
    fn open_formation_silhouettes_do_not_publish_blocking_coarse_collision() {
        for kind in [
            RockFormationKind::BasaltCrown,
            RockFormationKind::WeatheredArch,
            RockFormationKind::BoulderSpine,
        ] {
            assert!(formation_bounds(kind, 2.0).0.is_none());
        }
        for kind in [
            RockFormationKind::StackedMonoliths,
            RockFormationKind::CrystalOutcrop,
        ] {
            assert!(formation_bounds(kind, 2.0).0.is_some());
        }
    }

    #[test]
    fn every_formation_mesh_clears_landmark_floor_and_has_a_bounded_silhouette() {
        let route = SkyRoute::default();
        let mut examples = [None; RockFormationKind::COUNT];
        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_rock_formation_specs(island_index, island) {
                examples[spec.kind.index()].get_or_insert(spec);
            }
        }

        for spec in examples.into_iter().map(Option::unwrap) {
            let mesh = spec.build_mesh();
            let mesh_positions = positions(&mesh);
            assert!(
                (60..=3_500).contains(&mesh.count_vertices()),
                "{} has an unexpected vertex cost: {}",
                spec.label,
                mesh.count_vertices()
            );
            assert!(axis_range(mesh_positions, 0) > spec.scale * 1.8);
            assert!(axis_range(mesh_positions, 1) > spec.scale * 1.4);
            assert!(axis_range(mesh_positions, 2) > spec.scale * 1.0);
            let (_, max_y) = vertical_span(mesh_positions);
            let camera_half_extents = spec
                .camera_half_extents
                .expect("rock formations should publish camera bounds");
            assert!(max_y <= camera_half_extents.y * 2.0 + 0.001);
        }
    }

    #[test]
    fn solid_formation_bounds_use_grounded_base_origin_and_cover_mesh_height() {
        let route = SkyRoute::default();
        let mut solid_count = 0;

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_rock_formation_specs(island_index, island) {
                let mesh = spec.build_mesh();
                let mesh_positions = positions(&mesh);
                let (min_y, max_y) = vertical_span(mesh_positions);
                assert!(
                    min_y >= -0.001,
                    "{} sinks below its base origin",
                    spec.label
                );
                let camera_half_extents = spec
                    .camera_half_extents
                    .expect("rock formations should publish camera bounds");
                assert!(max_y <= camera_half_extents.y * 2.0 + 0.001);

                if let Some(half_extents) = spec.collision_half_extents {
                    solid_count += 1;
                    let bounds_center_y = half_extents.y;
                    let bounds_min_y = bounds_center_y - half_extents.y;
                    let bounds_max_y = bounds_center_y + half_extents.y;
                    assert!(bounds_min_y.abs() <= 0.001);
                    assert!(min_y >= bounds_min_y - 0.001);
                    assert!(
                        max_y <= bounds_max_y + 0.001,
                        "{} rises above its solid bound: {max_y} > {bounds_max_y}",
                        spec.label
                    );
                }
            }
        }

        assert!(solid_count > 0);
    }

    fn vertical_span(positions: &[[f32; 3]]) -> (f32, f32) {
        positions.iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(min, max), position| (min.min(position[1]), max.max(position[1])),
        )
    }
}
