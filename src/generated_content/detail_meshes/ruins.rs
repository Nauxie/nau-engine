use super::super::{
    island_playable_normalized_offset, island_visual_surface_position, random_unit,
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::{
    IslandBiome, IslandLandmarkRole, IslandScaleClass, IslandTerrainArchetype, IslandVisualMotif,
    SkyIsland, authored_island_composition,
};

const MAX_RUIN_COMPLEXES_PER_ISLAND: usize = 2;
const ARRIVAL_LANE_MIN_X: f32 = -0.48;
const ARRIVAL_LANE_MAX_X: f32 = 0.38;
const ARRIVAL_LANE_HALF_WIDTH: f32 = 0.24;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum RuinComplexKind {
    Colonnade,
    SunkenSanctum,
    Watchtower,
    BrokenAqueduct,
    ProcessionalStairs,
}

impl RuinComplexKind {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Colonnade => "colonnade",
            Self::SunkenSanctum => "sunken_sanctum",
            Self::Watchtower => "watchtower",
            Self::BrokenAqueduct => "broken_aqueduct",
            Self::ProcessionalStairs => "processional_stairs",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::Colonnade => "ruined perimeter colonnade",
            Self::SunkenSanctum => "sunken open-air sanctum",
            Self::Watchtower => "collapsed watchtower",
            Self::BrokenAqueduct => "broken aqueduct arcade",
            Self::ProcessionalStairs => "processional ruin stairs",
        }
    }

    #[cfg(test)]
    const COUNT: usize = 5;

    #[cfg(test)]
    fn index(self) -> usize {
        match self {
            Self::Colonnade => 0,
            Self::SunkenSanctum => 1,
            Self::Watchtower => 2,
            Self::BrokenAqueduct => 3,
            Self::ProcessionalStairs => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IslandRuinComplexSpec {
    pub(crate) kind: RuinComplexKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) collision_half_extents: Option<Vec3>,
    pub(crate) camera_half_extents: Option<Vec3>,
    scale: f32,
    seed: u32,
}

impl IslandRuinComplexSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        ruin_complex_mesh(self.kind, self.scale, self.seed)
    }
}

pub(crate) fn island_ruin_complex_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandRuinComplexSpec> {
    let Some(composition) = authored_island_composition(island.name) else {
        return Vec::new();
    };
    let kinds = ruin_complex_kinds(island, composition.visual_motif);
    let layout_seed = detail_seed(island_index, island, 0x9d34_5a71);

    kinds
        .into_iter()
        .take(MAX_RUIN_COMPLEXES_PER_ISLAND)
        .enumerate()
        .map(|(complex_index, kind)| {
            let sample = complex_index as u32;
            let initial_offset =
                perimeter_offset(island, complex_index, layout_seed, composition.visual_motif);
            let placement_angle = initial_offset.y.atan2(initial_offset.x);
            let rotation_y = kind_rotation(kind, placement_angle)
                + (random_unit(layout_seed, sample, 0x4b1) - 0.5) * 0.24;
            let scale = ruin_scale(island, kind);
            let (collision_half_extents, camera_half_extents) = ruin_complex_bounds(kind, scale);
            let normalized_offset = collision_bounded_offset(
                island,
                initial_offset,
                collision_half_extents
                    .map(|half_extents| rotated_aabb_half_extents(half_extents, rotation_y)),
            );

            IslandRuinComplexSpec {
                kind,
                label: kind.visual_name(),
                translation: island_visual_surface_position(island, normalized_offset)
                    + Vec3::Y * 0.04,
                rotation_y,
                collision_half_extents,
                camera_half_extents: Some(camera_half_extents),
                scale,
                seed: mixed_seed(
                    layout_seed
                        ^ sample.wrapping_mul(0x85eb_ca6b)
                        ^ (kind as u32).wrapping_mul(0xc2b2_ae35),
                ),
            }
        })
        .collect()
}

fn ruin_complex_kinds(island: SkyIsland, motif: IslandVisualMotif) -> Vec<RuinComplexKind> {
    let scale_class = island.world_tags.scale_class;
    let primary = match motif {
        IslandVisualMotif::RuinStair => Some(RuinComplexKind::ProcessionalStairs),
        IslandVisualMotif::MistArch | IslandVisualMotif::WaterfallMeadow => {
            Some(RuinComplexKind::BrokenAqueduct)
        }
        IslandVisualMotif::PlateauRim => Some(RuinComplexKind::Colonnade),
        IslandVisualMotif::LakeBasin
            if matches!(
                scale_class,
                IslandScaleClass::Large | IslandScaleClass::Vast | IslandScaleClass::HugePlateau
            ) =>
        {
            Some(RuinComplexKind::SunkenSanctum)
        }
        IslandVisualMotif::CrownPerch
            if !matches!(
                scale_class,
                IslandScaleClass::Tiny | IslandScaleClass::Small
            ) =>
        {
            Some(RuinComplexKind::Watchtower)
        }
        _ if island.world_tags.landmark_role == IslandLandmarkRole::RuinArch => {
            Some(match island.terrain_archetype {
                IslandTerrainArchetype::BrokenStair | IslandTerrainArchetype::TerracedSpur => {
                    RuinComplexKind::ProcessionalStairs
                }
                IslandTerrainArchetype::MistArch | IslandTerrainArchetype::CloudGate => {
                    RuinComplexKind::BrokenAqueduct
                }
                _ => RuinComplexKind::Colonnade,
            })
        }
        _ if island.world_tags.biome == IslandBiome::Ruin => {
            Some(RuinComplexKind::ProcessionalStairs)
        }
        _ if island.terrain_archetype == IslandTerrainArchetype::SkyPlateau => {
            Some(RuinComplexKind::Colonnade)
        }
        _ => None,
    };

    let Some(primary) = primary else {
        return Vec::new();
    };
    let mut kinds = vec![primary];
    if matches!(
        scale_class,
        IslandScaleClass::Vast | IslandScaleClass::HugePlateau
    ) {
        let secondary = match primary {
            RuinComplexKind::Colonnade => RuinComplexKind::Watchtower,
            RuinComplexKind::SunkenSanctum => RuinComplexKind::Colonnade,
            RuinComplexKind::Watchtower => RuinComplexKind::Colonnade,
            RuinComplexKind::BrokenAqueduct => RuinComplexKind::Watchtower,
            RuinComplexKind::ProcessionalStairs => RuinComplexKind::Colonnade,
        };
        kinds.push(secondary);
    }
    kinds
}

fn perimeter_offset(
    island: SkyIsland,
    complex_index: usize,
    seed: u32,
    motif: IslandVisualMotif,
) -> Vec2 {
    let sample = complex_index as u32;
    let mut angle = random_unit(seed, sample, 0x2a9) * std::f32::consts::TAU
        + complex_index as f32 * 2.176
        + motif as u32 as f32 * 0.173;
    let radius = 0.60 + random_unit(seed, sample, 0x2b7) * 0.13;
    let mut offset = Vec2::new(angle.cos(), angle.sin()) * radius;

    if occupies_arrival_lane(offset) {
        angle += if random_unit(seed, sample, 0x2c3) > 0.5 {
            0.82
        } else {
            -0.82
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

fn kind_rotation(kind: RuinComplexKind, placement_angle: f32) -> f32 {
    match kind {
        RuinComplexKind::Colonnade | RuinComplexKind::BrokenAqueduct => {
            placement_angle + std::f32::consts::FRAC_PI_2
        }
        RuinComplexKind::SunkenSanctum
        | RuinComplexKind::Watchtower
        | RuinComplexKind::ProcessionalStairs => placement_angle + std::f32::consts::PI,
    }
}

fn ruin_scale(island: SkyIsland, kind: RuinComplexKind) -> f32 {
    let class_scale = match island.world_tags.scale_class {
        IslandScaleClass::Tiny => 0.78,
        IslandScaleClass::Small => 0.90,
        IslandScaleClass::Medium => 1.0,
        IslandScaleClass::Large => 1.08,
        IslandScaleClass::Vast => 1.16,
        IslandScaleClass::HugePlateau => 1.25,
    };
    let kind_scale = match kind {
        RuinComplexKind::Watchtower => 0.86,
        RuinComplexKind::BrokenAqueduct => 0.94,
        RuinComplexKind::Colonnade
        | RuinComplexKind::SunkenSanctum
        | RuinComplexKind::ProcessionalStairs => 1.0,
    };

    ((island.half_extents.min_element() * 0.09).clamp(1.7, 5.8) * class_scale * kind_scale)
        .clamp(1.5, 7.0)
}

fn ruin_complex_bounds(kind: RuinComplexKind, scale: f32) -> (Option<Vec3>, Vec3) {
    // Runtime centers each bound at translation + Vec3::Y * half_extents.y.
    match kind {
        RuinComplexKind::Colonnade => (None, Vec3::new(3.9, 2.5, 2.4) * scale),
        RuinComplexKind::SunkenSanctum => (None, Vec3::new(3.4, 2.1, 3.4) * scale),
        RuinComplexKind::Watchtower => (
            Some(Vec3::new(0.92, 2.45, 0.92) * scale),
            Vec3::new(1.7, 3.7, 1.7) * scale,
        ),
        RuinComplexKind::BrokenAqueduct => (None, Vec3::new(4.5, 2.9, 1.5) * scale),
        RuinComplexKind::ProcessionalStairs => (None, Vec3::new(2.6, 2.0, 4.0) * scale),
    }
}

fn ruin_complex_mesh(kind: RuinComplexKind, scale: f32, seed: u32) -> Mesh {
    let mut mesh = MeshBuffers::default();
    match kind {
        RuinComplexKind::Colonnade => build_colonnade(&mut mesh, scale, seed),
        RuinComplexKind::SunkenSanctum => build_sunken_sanctum(&mut mesh, scale, seed),
        RuinComplexKind::Watchtower => build_watchtower(&mut mesh, scale, seed),
        RuinComplexKind::BrokenAqueduct => build_broken_aqueduct(&mut mesh, scale, seed),
        RuinComplexKind::ProcessionalStairs => build_processional_stairs(&mut mesh, scale, seed),
    }
    mesh.build()
}

fn build_colonnade(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    mesh.append_box(
        Vec3::new(0.0, 0.16, 0.0) * scale,
        Vec3::new(3.75, 0.16, 2.15) * scale,
        Quat::IDENTITY,
    );
    mesh.append_box(
        Vec3::new(0.0, 0.38, 0.0) * scale,
        Vec3::new(3.35, 0.08, 1.76) * scale,
        Quat::IDENTITY,
    );

    let column_x = [-2.72_f32, -0.92, 0.92, 2.72];
    for (row_index, z) in [-1.36_f32, 1.36].into_iter().enumerate() {
        for (column_index, x) in column_x.into_iter().enumerate() {
            let index = row_index * column_x.len() + column_index;
            if index == 5 {
                continue;
            }
            let lean = (random_unit(seed, index as u32, 0x501) - 0.5) * 0.055;
            append_ruin_column(
                mesh,
                Vec3::new(x, 0.46, z) * scale,
                0.24 * scale,
                (2.80 + random_unit(seed, index as u32, 0x50b) * 0.25) * scale,
                lean,
            );
        }
    }

    for (lintel_index, x) in [-1.82_f32, 1.82].into_iter().enumerate() {
        let tilt = if lintel_index == 0 { -0.025 } else { 0.055 };
        for z in [-1.36_f32, 1.36] {
            mesh.append_box(
                Vec3::new(x, 3.62, z) * scale,
                Vec3::new(0.86, 0.18, 0.30) * scale,
                Quat::from_rotation_z(tilt),
            );
        }
    }
    mesh.append_box(
        Vec3::new(0.60, 0.58, 0.36) * scale,
        Vec3::new(0.78, 0.17, 0.28) * scale,
        Quat::from_euler(EulerRot::YXZ, 0.34, 0.12, -0.18),
    );
    append_rubble(mesh, Vec3::new(-0.45, 0.18, 0.10), 7, scale, seed ^ 0x517);
}

fn build_sunken_sanctum(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    let wall_segments = [
        (Vec3::new(0.0, 0.22, -2.75), Vec3::new(3.15, 0.22, 0.28)),
        (Vec3::new(-2.88, 0.22, 0.0), Vec3::new(0.28, 0.22, 2.48)),
        (Vec3::new(2.88, 0.22, 0.0), Vec3::new(0.28, 0.22, 2.48)),
        (Vec3::new(-2.10, 0.22, 2.75), Vec3::new(0.75, 0.22, 0.28)),
        (Vec3::new(2.10, 0.22, 2.75), Vec3::new(0.75, 0.22, 0.28)),
    ];
    for (center, half_extents) in wall_segments {
        mesh.append_box(center * scale, half_extents * scale, Quat::IDENTITY);
    }
    mesh.append_box(
        Vec3::new(0.0, 0.06, 0.0) * scale,
        Vec3::new(2.42, 0.06, 2.20) * scale,
        Quat::IDENTITY,
    );
    mesh.append_box(
        Vec3::new(0.0, 0.10, -0.30) * scale,
        Vec3::new(0.78, 0.10, 0.66) * scale,
        Quat::from_rotation_y(0.12),
    );

    for step in 0..5 {
        let t = step as f32 / 4.0;
        mesh.append_box(
            Vec3::new(0.0, 0.18 - t * 0.10, 2.58 - t * 0.66) * scale,
            Vec3::new(0.74 + t * 0.06, 0.08, 0.34) * scale,
            Quat::IDENTITY,
        );
    }
    for (column_index, (x, z)) in [
        (-2.35_f32, -2.12_f32),
        (2.35, -2.12),
        (-2.35, 2.12),
        (2.35, 2.12),
    ]
    .into_iter()
    .enumerate()
    {
        let height = if column_index == 2 { 1.65 } else { 2.35 };
        append_ruin_column(
            mesh,
            Vec3::new(x, 0.44, z) * scale,
            0.22 * scale,
            height * scale,
            (random_unit(seed, column_index as u32, 0x601) - 0.5) * 0.06,
        );
    }
    mesh.append_box(
        Vec3::new(0.0, 0.62, -0.28) * scale,
        Vec3::new(0.70, 0.36, 0.58) * scale,
        Quat::from_rotation_y(-0.08),
    );
    append_rubble(mesh, Vec3::new(1.15, 0.12, 1.30), 8, scale, seed ^ 0x61d);
}

fn build_watchtower(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    mesh.append_box(
        Vec3::new(0.0, 0.18, 0.0) * scale,
        Vec3::new(1.42, 0.18, 1.42) * scale,
        Quat::from_rotation_y(0.10),
    );
    mesh.append_tapered_cylinder(
        Vec3::new(0.0, 1.68, 0.0) * scale,
        1.08 * scale,
        0.88 * scale,
        3.0 * scale,
        10,
    );
    mesh.append_tapered_cylinder(
        Vec3::new(0.04, 3.50, -0.02) * scale,
        0.94 * scale,
        0.76 * scale,
        0.72 * scale,
        10,
    );
    mesh.append_box(
        Vec3::new(0.04, 3.92, -0.02) * scale,
        Vec3::new(1.18, 0.16, 1.18) * scale,
        Quat::from_rotation_y(0.06),
    );

    for battlement in 0..8 {
        if battlement == 2 || battlement == 5 {
            continue;
        }
        let angle = battlement as f32 / 8.0 * std::f32::consts::TAU;
        mesh.append_box(
            Vec3::new(angle.cos() * 0.92, 4.35, angle.sin() * 0.92) * scale,
            Vec3::new(0.25, 0.28, 0.23) * scale,
            Quat::from_rotation_y(-angle),
        );
    }
    for buttress in 0..4 {
        let angle = buttress as f32 * std::f32::consts::FRAC_PI_2 + 0.16;
        mesh.append_box(
            Vec3::new(angle.cos() * 1.12, 0.86, angle.sin() * 1.12) * scale,
            Vec3::new(0.28, 0.86, 0.48) * scale,
            Quat::from_rotation_y(-angle),
        );
    }
    mesh.append_box(
        Vec3::new(0.85, 0.32, -1.22) * scale,
        Vec3::new(0.62, 0.26, 0.36) * scale,
        Quat::from_euler(EulerRot::YXZ, -0.42, 0.14, 0.20),
    );
    append_rubble(mesh, Vec3::new(-0.65, 0.15, 0.72), 6, scale, seed ^ 0x70d);
}

fn build_broken_aqueduct(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    let bay_width = 2.20_f32;
    for pillar in 0..4 {
        let x = (pillar as f32 - 1.5) * bay_width;
        let height = if pillar == 3 { 2.45 } else { 3.35 };
        mesh.append_box(
            Vec3::new(x, 0.16, 0.0) * scale,
            Vec3::new(0.50, 0.16, 0.76) * scale,
            Quat::from_rotation_y((random_unit(seed, pillar as u32, 0x801) - 0.5) * 0.05),
        );
        mesh.append_tapered_cylinder(
            Vec3::new(x, 1.82, 0.0) * scale,
            0.42 * scale,
            0.34 * scale,
            height * scale,
            8,
        );
        mesh.append_box(
            Vec3::new(x, height + 0.24, 0.0) * scale,
            Vec3::new(0.54, 0.16, 0.68) * scale,
            Quat::IDENTITY,
        );
    }

    for bay in 0..3 {
        let center_x = (bay as f32 - 1.0) * bay_width;
        append_arch_blocks(
            mesh,
            Vec3::new(center_x, 0.0, 0.0) * scale,
            0.78 * scale,
            2.15 * scale,
            1.18 * scale,
            1.18 * scale,
            9,
            seed ^ (bay as u32).wrapping_mul(0x811),
            bay == 2,
        );
    }
    for channel in 0..3 {
        if channel == 2 {
            continue;
        }
        mesh.append_box(
            Vec3::new((channel as f32 - 0.5) * bay_width, 4.04, 0.0) * scale,
            Vec3::new(1.02, 0.20, 0.58) * scale,
            Quat::from_rotation_z(if channel == 0 { -0.025 } else { 0.04 }),
        );
    }
    mesh.append_box(
        Vec3::new(2.62, 0.42, 0.32) * scale,
        Vec3::new(0.88, 0.22, 0.44) * scale,
        Quat::from_euler(EulerRot::YXZ, 0.38, 0.09, -0.28),
    );
    append_rubble(mesh, Vec3::new(2.30, 0.16, -0.45), 7, scale, seed ^ 0x83d);
}

fn build_processional_stairs(mesh: &mut MeshBuffers, scale: f32, seed: u32) {
    let step_count = 11;
    let step_depth = 0.58_f32;
    for step in 0..step_count {
        let t = step as f32 / (step_count - 1) as f32;
        let width = 2.10 - t * 0.28;
        let height = 0.12 + t * 1.76;
        mesh.append_box(
            Vec3::new(
                (random_unit(seed, step as u32, 0x901) - 0.5) * 0.05,
                height * 0.5,
                -3.20 + step as f32 * step_depth,
            ) * scale,
            Vec3::new(width, height * 0.5, step_depth * 0.49) * scale,
            Quat::from_rotation_y((random_unit(seed, step as u32, 0x907) - 0.5) * 0.025),
        );
    }
    mesh.append_box(
        Vec3::new(0.0, 1.10, 3.20) * scale,
        Vec3::new(2.45, 0.18, 1.02) * scale,
        Quat::IDENTITY,
    );

    for side in [-1.0_f32, 1.0] {
        for segment in 0..5 {
            let t = segment as f32 / 4.0;
            mesh.append_box(
                Vec3::new(
                    side * (2.20 - t * 0.22),
                    0.42 + t * 1.18,
                    -2.75 + segment as f32 * 1.18,
                ) * scale,
                Vec3::new(0.22, 0.40 + t * 0.12, 0.54) * scale,
                Quat::from_rotation_x(-0.12),
            );
        }
    }
    for (column_index, (x, z)) in [
        (-1.82_f32, 2.78_f32),
        (1.82, 2.78),
        (-2.12, -2.82),
        (2.12, -2.82),
    ]
    .into_iter()
    .enumerate()
    {
        let height = if column_index == 3 { 1.25 } else { 2.15 };
        append_ruin_column(
            mesh,
            Vec3::new(x, if z > 0.0 { 1.28 } else { 0.12 }, z) * scale,
            0.20 * scale,
            height * scale,
            (random_unit(seed, column_index as u32, 0x929) - 0.5) * 0.05,
        );
    }
    mesh.append_box(
        Vec3::new(0.0, 1.52, 3.22) * scale,
        Vec3::new(0.72, 0.26, 0.60) * scale,
        Quat::from_rotation_y(0.08),
    );
    append_rubble(mesh, Vec3::new(1.05, 0.18, 2.10), 7, scale, seed ^ 0x93d);
}

fn append_ruin_column(
    mesh: &mut MeshBuffers,
    base_center: Vec3,
    radius: f32,
    height: f32,
    lean: f32,
) {
    let lean_rotation = Quat::from_rotation_z(lean);
    mesh.append_box(
        base_center + Vec3::Y * radius * 0.28,
        Vec3::new(radius * 1.45, radius * 0.28, radius * 1.45),
        lean_rotation,
    );
    mesh.append_box(
        base_center + Vec3::Y * radius * 0.72,
        Vec3::new(radius * 1.18, radius * 0.18, radius * 1.18),
        lean_rotation,
    );
    mesh.append_tapered_cylinder(
        base_center + Vec3::Y * (radius * 0.90 + height * 0.5),
        radius,
        radius * 0.82,
        height,
        10,
    );
    let capital_y = radius * 0.90 + height;
    mesh.append_box(
        base_center + Vec3::Y * (capital_y + radius * 0.20),
        Vec3::new(radius * 1.22, radius * 0.20, radius * 1.22),
        lean_rotation,
    );
    mesh.append_box(
        base_center + Vec3::Y * (capital_y + radius * 0.48),
        Vec3::new(radius * 1.58, radius * 0.12, radius * 1.48),
        lean_rotation,
    );
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
    broken: bool,
) {
    let arc_piece = std::f32::consts::PI * (half_width + crown_height) * 0.25 / block_count as f32;
    for block in 0..block_count {
        if broken && (block == block_count / 2 || block + 2 == block_count) {
            continue;
        }
        let t = (block as f32 + 0.5) / block_count as f32;
        let angle = std::f32::consts::PI * (1.0 - t);
        let chip = (random_unit(seed, block as u32, 0xa01) - 0.5) * 0.08;
        let block_center = center
            + Vec3::new(
                angle.cos() * half_width,
                spring_y + angle.sin() * crown_height,
                chip * depth,
            );
        mesh.append_box(
            block_center,
            Vec3::new(arc_piece * 0.50, crown_height * 0.16, depth * 0.50),
            Quat::from_rotation_z(angle - std::f32::consts::FRAC_PI_2 + chip),
        );
    }
}

fn append_rubble(mesh: &mut MeshBuffers, center: Vec3, count: usize, scale: f32, seed: u32) {
    for block in 0..count {
        let sample = block as u32;
        let angle = random_unit(seed, sample, 0xb01) * std::f32::consts::TAU;
        let radius = 0.34 + random_unit(seed, sample, 0xb07) * 1.28;
        let half_extents = Vec3::new(
            0.18 + random_unit(seed, sample, 0xb0b) * 0.30,
            0.12 + random_unit(seed, sample, 0xb11) * 0.20,
            0.16 + random_unit(seed, sample, 0xb17) * 0.26,
        ) * scale;
        let block_center = (center
            + Vec3::new(
                angle.cos() * radius,
                half_extents.y / scale,
                angle.sin() * radius,
            ))
            * scale;
        mesh.append_box(
            block_center,
            half_extents,
            Quat::from_euler(
                EulerRot::YXZ,
                angle,
                (random_unit(seed, sample, 0xb1d) - 0.5) * 0.32,
                (random_unit(seed, sample, 0xb23) - 0.5) * 0.38,
            ),
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

    fn append_tapered_cylinder(
        &mut self,
        center: Vec3,
        bottom_radius: f32,
        top_radius: f32,
        height: f32,
        segments: usize,
    ) {
        let half_height = height * 0.5;
        for segment in 0..segments {
            let angle_0 = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let angle_1 = (segment + 1) as f32 / segments as f32 * std::f32::consts::TAU;
            let radial_0 = Vec3::new(angle_0.cos(), 0.0, angle_0.sin());
            let radial_1 = Vec3::new(angle_1.cos(), 0.0, angle_1.sin());
            let bottom_0 = center + radial_0 * bottom_radius - Vec3::Y * half_height;
            let bottom_1 = center + radial_1 * bottom_radius - Vec3::Y * half_height;
            let top_0 = center + radial_0 * top_radius + Vec3::Y * half_height;
            let top_1 = center + radial_1 * top_radius + Vec3::Y * half_height;
            let slope = (bottom_radius - top_radius) / height.max(0.01);
            let normal_0 = (radial_0 + Vec3::Y * slope).normalize();
            let normal_1 = (radial_1 + Vec3::Y * slope).normalize();

            let side_start = self.positions.len() as u32;
            for (position, normal, uv) in [
                (bottom_0, normal_0, [0.0, 0.0]),
                (bottom_1, normal_1, [1.0, 0.0]),
                (top_1, normal_1, [1.0, 1.0]),
                (top_0, normal_0, [0.0, 1.0]),
            ] {
                self.positions.push(position.to_array());
                self.normals.push(normal.to_array());
                self.uvs.push(uv);
            }
            self.indices.extend([
                side_start,
                side_start + 1,
                side_start + 2,
                side_start,
                side_start + 2,
                side_start + 3,
            ]);

            let top_start = self.positions.len() as u32;
            for (position, uv) in [
                (center + Vec3::Y * half_height, [0.5, 0.5]),
                (top_1, [1.0, 1.0]),
                (top_0, [0.0, 1.0]),
            ] {
                self.positions.push(position.to_array());
                self.normals.push(Vec3::Y.to_array());
                self.uvs.push(uv);
            }
            self.indices
                .extend([top_start, top_start + 1, top_start + 2]);

            let bottom_start = self.positions.len() as u32;
            for (position, uv) in [
                (center - Vec3::Y * half_height, [0.5, 0.5]),
                (bottom_0, [0.0, 0.0]),
                (bottom_1, [1.0, 0.0]),
            ] {
                self.positions.push(position.to_array());
                self.normals.push(Vec3::NEG_Y.to_array());
                self.uvs.push(uv);
            }
            self.indices
                .extend([bottom_start, bottom_start + 1, bottom_start + 2]);
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
            _ => panic!("ruin mesh should expose Float32x3 positions"),
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
    fn ruin_complex_specs_are_deterministic_and_capped() {
        let route = SkyRoute::default();
        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let first = island_ruin_complex_specs(island_index, island);
            let second = island_ruin_complex_specs(island_index, island);
            assert_eq!(first, second, "{} should be deterministic", island.name);
            assert!(
                first.len() <= MAX_RUIN_COMPLEXES_PER_ISLAND,
                "{} exceeded the ruin-complex cap",
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
    fn authored_route_covers_every_ruin_complex_kind() {
        let route = SkyRoute::default();
        let mut seen = [false; RuinComplexKind::COUNT];

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_ruin_complex_specs(island_index, island) {
                seen[spec.kind.index()] = true;
                assert!(!spec.kind.label().is_empty());
                assert!(!spec.kind.visual_name().is_empty());
            }
        }

        assert!(seen.into_iter().all(|covered| covered));
    }

    #[test]
    fn ruin_complexes_stay_playable_perimeter_biased_and_clear_of_arrival_lane() {
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_ruin_complex_specs(island_index, island) {
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
    fn open_ruin_silhouettes_do_not_publish_blocking_coarse_collision() {
        for kind in [
            RuinComplexKind::Colonnade,
            RuinComplexKind::SunkenSanctum,
            RuinComplexKind::BrokenAqueduct,
            RuinComplexKind::ProcessionalStairs,
        ] {
            assert!(ruin_complex_bounds(kind, 2.0).0.is_none());
        }
        assert!(
            ruin_complex_bounds(RuinComplexKind::Watchtower, 2.0)
                .0
                .is_some()
        );
    }

    #[test]
    fn every_ruin_mesh_is_complex_bounded_and_spatially_readable() {
        let route = SkyRoute::default();
        let mut examples = [None; RuinComplexKind::COUNT];
        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_ruin_complex_specs(island_index, island) {
                examples[spec.kind.index()].get_or_insert(spec);
            }
        }

        for spec in examples.into_iter().map(Option::unwrap) {
            let mesh = spec.build_mesh();
            let mesh_positions = positions(&mesh);
            assert!(
                (360..=5_000).contains(&mesh.count_vertices()),
                "{} has an unexpected vertex cost: {}",
                spec.label,
                mesh.count_vertices()
            );
            assert!(axis_range(mesh_positions, 0) > spec.scale * 2.0);
            assert!(axis_range(mesh_positions, 1) > spec.scale * 1.5);
            assert!(axis_range(mesh_positions, 2) > spec.scale * 1.2);
            let (_, max_y) = vertical_span(mesh_positions);
            let camera_half_extents = spec
                .camera_half_extents
                .expect("ruin complexes should publish camera bounds");
            assert!(max_y <= camera_half_extents.y * 2.0 + 0.001);
        }
    }

    #[test]
    fn solid_ruin_bounds_use_grounded_base_origin_and_cover_mesh_height() {
        let route = SkyRoute::default();
        let mut solid_count = 0;

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_ruin_complex_specs(island_index, island) {
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
                    .expect("ruin complexes should publish camera bounds");
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
