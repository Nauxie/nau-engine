use super::super::{island_playable_normalized_offset, island_visual_surface_position};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use nau_engine::world::{
    IslandHeroLandmark, IslandScaleClass, SkyIsland, authored_island_art_direction,
};
use std::sync::OnceLock;

const HERO_COLUMN_SEGMENTS: usize = 8;
const HERO_RING_SEGMENTS: usize = 18;
const HERO_LANDMARK_KIND_COUNT: usize = IslandHeroLandmark::ZenithSanctum as usize + 1;
static HERO_SEMANTIC_SAMPLE_CACHE: [OnceLock<[Vec3; 4]>; HERO_LANDMARK_KIND_COUNT] =
    [const { OnceLock::new() }; HERO_LANDMARK_KIND_COUNT];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IslandHeroLandmarkSpec {
    pub(crate) kind: IslandHeroLandmark,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) visual_half_extents: Vec3,
    pub(crate) collision_half_extents: Option<Vec3>,
    scale: f32,
    seed: u32,
}

impl IslandHeroLandmarkSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        hero_landmark_mesh(self.kind, self.scale, self.seed)
    }

    pub(crate) fn semantic_sample_positions(self) -> [Vec3; 4] {
        let rotation = Quat::from_rotation_y(self.rotation_y);
        hero_semantic_sample_local_positions(self.kind)
            .map(|position| self.translation + rotation * (position * self.scale))
    }
}

pub(crate) fn island_hero_landmark_spec(
    island_index: usize,
    island: SkyIsland,
) -> Option<IslandHeroLandmarkSpec> {
    let art = authored_island_art_direction(island.name)?;
    let normalized_anchor =
        island_playable_normalized_offset(island, Vec2::from_array(art.hero_anchor));
    let scale = art.hero_scale * scale_class_multiplier(island.world_tags.scale_class);
    let visual_half_extents = hero_visual_half_extents(art.hero_landmark, scale);

    Some(IslandHeroLandmarkSpec {
        kind: art.hero_landmark,
        label: art.hero_landmark.label(),
        translation: island_visual_surface_position(island, normalized_anchor) + Vec3::Y * 0.04,
        rotation_y: (art.hero_rotation_degrees as f32).to_radians(),
        visual_half_extents,
        // Hero landmarks stay player- and camera-nonblocking until a dedicated obstruction
        // treatment can preserve camera feel without hiding their authored silhouettes.
        collision_half_extents: None,
        scale,
        seed: art.signature_seed ^ (island_index as u32).wrapping_mul(0x9e37_79b9),
    })
}

fn scale_class_multiplier(scale_class: IslandScaleClass) -> f32 {
    match scale_class {
        IslandScaleClass::Tiny => 0.82,
        IslandScaleClass::Small => 1.00,
        IslandScaleClass::Medium => 1.24,
        IslandScaleClass::Large => 1.52,
        IslandScaleClass::Vast => 1.84,
        IslandScaleClass::HugePlateau => 2.40,
    }
}

fn hero_visual_half_extents(kind: IslandHeroLandmark, scale: f32) -> Vec3 {
    let local = match kind {
        IslandHeroLandmark::BeaconCourt => Vec3::new(5.9, 2.9, 5.9),
        IslandHeroLandmark::CairnCauseway => Vec3::new(5.9, 2.1, 5.9),
        IslandHeroLandmark::BloomAmphitheater => Vec3::new(5.9, 1.7, 5.9),
        IslandHeroLandmark::CrownObservatory => Vec3::new(5.9, 4.1, 5.9),
        IslandHeroLandmark::AeolianHarp => Vec3::new(5.9, 3.6, 5.9),
        IslandHeroLandmark::CopperArcade => Vec3::new(5.9, 2.5, 7.0),
        IslandHeroLandmark::SolarGrove => Vec3::new(5.9, 3.1, 5.9),
        IslandHeroLandmark::RefugeCanopy => Vec3::new(5.9, 2.5, 5.9),
        IslandHeroLandmark::StormTotemField => Vec3::new(5.9, 3.0, 5.9),
        IslandHeroLandmark::OrchardPergola => Vec3::new(7.9, 2.5, 7.9),
        IslandHeroLandmark::NeedleHalo => Vec3::new(5.9, 4.2, 5.9),
        IslandHeroLandmark::SapphireShrine => Vec3::new(5.9, 2.5, 5.9),
        IslandHeroLandmark::FracturedProcession => Vec3::new(5.9, 2.1, 5.9),
        IslandHeroLandmark::VeiledPortal => Vec3::new(5.9, 4.0, 5.9),
        IslandHeroLandmark::CloudGateChoir => Vec3::new(7.4, 3.1, 7.4),
        IslandHeroLandmark::FlightTrainingCircle => Vec3::new(6.3, 1.8, 6.3),
        IslandHeroLandmark::PetalArcade => Vec3::new(6.4, 2.2, 6.4),
        IslandHeroLandmark::LightningMonolith => Vec3::new(5.9, 4.6, 5.9),
        IslandHeroLandmark::ArborCrown => Vec3::new(5.9, 2.9, 5.9),
        IslandHeroLandmark::FogCairn => Vec3::new(5.9, 2.9, 5.9),
        IslandHeroLandmark::RootedThreshold => Vec3::new(5.9, 3.5, 5.9),
        IslandHeroLandmark::WindbreakRibs => Vec3::new(5.9, 3.4, 5.9),
        IslandHeroLandmark::MoonGardenShrine => Vec3::new(5.9, 3.3, 5.9),
        IslandHeroLandmark::CatchSailTerrace => Vec3::new(6.5, 3.7, 5.9),
        IslandHeroLandmark::ThermalOrrery => Vec3::new(6.2, 5.0, 6.1),
        IslandHeroLandmark::CrownletSeat => Vec3::new(5.9, 3.1, 5.9),
        IslandHeroLandmark::SkyhookAqueduct => Vec3::new(5.9, 1.8, 5.9),
        IslandHeroLandmark::StratosLookout => Vec3::new(5.9, 5.1, 5.9),
        IslandHeroLandmark::CascadeTemple => Vec3::new(5.9, 3.4, 5.9),
        IslandHeroLandmark::PilgrimGate => Vec3::new(5.9, 4.3, 5.9),
        IslandHeroLandmark::RoostSpire => Vec3::new(5.9, 4.9, 5.9),
        IslandHeroLandmark::SkyforgeAnvil => Vec3::new(5.9, 3.2, 5.9),
        IslandHeroLandmark::WaystoneGallery => Vec3::new(5.9, 2.4, 5.9),
        IslandHeroLandmark::ChimeColonnade => Vec3::new(8.6, 3.5, 8.6),
        IslandHeroLandmark::BluevaultSanctuary => Vec3::new(5.9, 4.5, 5.9),
        IslandHeroLandmark::SwitchbackBastion => Vec3::new(5.9, 2.6, 5.9),
        IslandHeroLandmark::SolarOrrery => Vec3::new(9.1, 9.2, 7.7),
        IslandHeroLandmark::CloudbreakPortal => Vec3::new(5.9, 5.0, 5.9),
        IslandHeroLandmark::SkyCitadel => Vec3::new(6.0, 4.9, 6.0),
        IslandHeroLandmark::HorizonLens => Vec3::new(5.9, 5.4, 5.9),
        IslandHeroLandmark::ZenithSanctum => Vec3::new(6.8, 5.1, 6.9),
    };
    local * scale
}

fn hero_semantic_sample_local_positions(kind: IslandHeroLandmark) -> [Vec3; 4] {
    *HERO_SEMANTIC_SAMPLE_CACHE[kind as usize]
        .get_or_init(|| derive_hero_semantic_sample_local_positions(kind))
}

fn derive_hero_semantic_sample_local_positions(kind: IslandHeroLandmark) -> [Vec3; 4] {
    let mesh = hero_landmark_mesh(kind, 1.0, 0x5eed_cafe);
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(positions)) => positions,
        _ => panic!("{kind:?} hero mesh must publish Float32x3 positions"),
    };
    let indices = match mesh.indices() {
        Some(Indices::U16(indices)) => indices
            .iter()
            .map(|index| usize::from(*index))
            .collect::<Vec<_>>(),
        Some(Indices::U32(indices)) => indices
            .iter()
            .map(|index| *index as usize)
            .collect::<Vec<_>>(),
        None => (0..positions.len()).collect::<Vec<_>>(),
    };
    let candidates = indices
        .chunks_exact(3)
        .filter_map(|triangle| {
            let a = Vec3::from_array(positions[triangle[0]]);
            let b = Vec3::from_array(positions[triangle[1]]);
            let c = Vec3::from_array(positions[triangle[2]]);
            let centroid = (a + b + c) / 3.0;
            ((b - a).cross(c - a).length_squared() > 0.000_001
                && centroid.is_finite()
                && centroid.y > 0.08)
                .then_some(centroid)
        })
        .collect::<Vec<_>>();
    assert!(
        candidates.len() >= 4,
        "{kind:?} hero mesh must expose at least four semantic triangles"
    );

    let (min, max) = candidates.iter().copied().fold(
        (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY)),
        |(min, max), candidate| (min.min(candidate), max.max(candidate)),
    );
    let center = (min + max) * 0.5;
    let extent = (max - min).max(Vec3::splat(0.001));
    let targets = [
        center + Vec3::new(-extent.x * 0.28, extent.y * 0.14, -extent.z * 0.18),
        center + Vec3::new(extent.x * 0.28, extent.y * 0.14, -extent.z * 0.18),
        center + Vec3::new(-extent.x * 0.18, extent.y * 0.22, extent.z * 0.28),
        center + Vec3::new(extent.x * 0.18, extent.y * 0.22, extent.z * 0.28),
    ];
    let mut selected = [Vec3::ZERO; 4];
    let mut used = vec![false; candidates.len()];

    for (slot, target) in selected.iter_mut().zip(targets) {
        let candidate_index = candidates
            .iter()
            .enumerate()
            .filter(|(index, _)| !used[*index])
            .min_by(|(_, left), (_, right)| {
                ((*left - target) / extent)
                    .length_squared()
                    .total_cmp(&((*right - target) / extent).length_squared())
            })
            .map(|(index, _)| index)
            .expect("hero semantic sample selection should retain an unused triangle");
        used[candidate_index] = true;
        *slot = candidates[candidate_index];
    }

    selected
}

fn hero_landmark_mesh(kind: IslandHeroLandmark, scale: f32, seed: u32) -> Mesh {
    let mut mesh = HeroMesh::default();
    match kind {
        IslandHeroLandmark::BeaconCourt => {
            append_court(&mut mesh, scale, 7, 5.2, 0.72, 4.8, true);
        }
        IslandHeroLandmark::CairnCauseway => {
            append_procession(&mut mesh, scale, 11, 6.2, 0.24, 4, false);
        }
        IslandHeroLandmark::BloomAmphitheater => {
            append_amphitheater(&mut mesh, scale, 3, 7, 0.86, true);
        }
        IslandHeroLandmark::CrownObservatory => {
            append_observatory(&mut mesh, scale, 5.8, 2, 8, 2.8);
        }
        IslandHeroLandmark::AeolianHarp => {
            append_instrument(&mut mesh, scale, 9, 6.6, 5.2, false);
        }
        IslandHeroLandmark::CopperArcade => {
            append_arcade(&mut mesh, scale, 5, 3.2, 4.2, 1.3);
        }
        IslandHeroLandmark::SolarGrove => {
            append_grove(&mut mesh, scale, 8, 4.5, 2, true);
        }
        IslandHeroLandmark::RefugeCanopy => {
            append_canopy(&mut mesh, scale, 6, 4.3, 5.8, 2);
        }
        IslandHeroLandmark::StormTotemField => {
            append_totem_field(&mut mesh, scale, 9, 6.2, 3.8);
        }
        IslandHeroLandmark::OrchardPergola => {
            append_canopy(&mut mesh, scale, 8, 3.8, 7.0, 4);
        }
        IslandHeroLandmark::NeedleHalo => {
            append_lens(&mut mesh, scale, 7.8, 3.5, 2, true);
        }
        IslandHeroLandmark::SapphireShrine => {
            append_temple(&mut mesh, scale, 3, 6, 4.6, false);
        }
        IslandHeroLandmark::FracturedProcession => {
            append_procession(&mut mesh, scale, 13, 7.4, 0.42, 6, true);
        }
        IslandHeroLandmark::VeiledPortal => {
            append_portal(&mut mesh, scale, 5.8, 7.2, 2, 5);
        }
        IslandHeroLandmark::CloudGateChoir => {
            append_choir(&mut mesh, scale, 11, 7.0, 5.2, true);
        }
        IslandHeroLandmark::FlightTrainingCircle => {
            append_training_circle(&mut mesh, scale, 10, 5.8, 4);
        }
        IslandHeroLandmark::PetalArcade => {
            append_petaled_arcade(&mut mesh, scale, 7, 4.7, 6);
        }
        IslandHeroLandmark::LightningMonolith => {
            append_monolith(&mut mesh, scale, 8.8, 1.5, 5, true);
        }
        IslandHeroLandmark::ArborCrown => {
            append_grove(&mut mesh, scale, 10, 5.4, 3, false);
        }
        IslandHeroLandmark::FogCairn => {
            append_cairn(&mut mesh, scale, 8, 5.0, 3);
        }
        IslandHeroLandmark::RootedThreshold => {
            append_portal(&mut mesh, scale, 6.4, 6.2, 3, 7);
        }
        IslandHeroLandmark::WindbreakRibs => {
            append_windbreak(&mut mesh, scale, 11, 6.4, 8.0);
        }
        IslandHeroLandmark::MoonGardenShrine => {
            append_temple(&mut mesh, scale, 4, 8, 5.2, true);
        }
        IslandHeroLandmark::CatchSailTerrace => {
            append_sail(&mut mesh, scale, 7, 7.0, 6.6);
        }
        IslandHeroLandmark::ThermalOrrery => {
            append_orrery(&mut mesh, scale, 3, 6.2, 2.2, 6);
        }
        IslandHeroLandmark::CrownletSeat => {
            append_seat(&mut mesh, scale, 7, 4.8, 5);
        }
        IslandHeroLandmark::SkyhookAqueduct => {
            append_aqueduct(&mut mesh, scale, 5, 5.8, 2.7, true);
        }
        IslandHeroLandmark::StratosLookout => {
            append_lookout(&mut mesh, scale, 8.0, 4.8, 10, 3);
        }
        IslandHeroLandmark::CascadeTemple => {
            append_temple(&mut mesh, scale, 5, 8, 6.2, false);
        }
        IslandHeroLandmark::PilgrimGate => {
            append_portal(&mut mesh, scale, 7.2, 7.8, 4, 8);
        }
        IslandHeroLandmark::RoostSpire => {
            append_spire(&mut mesh, scale, 9.2, 6, 4, true);
        }
        IslandHeroLandmark::SkyforgeAnvil => {
            append_anvil(&mut mesh, scale, 5.4, 7.2, 4);
        }
        IslandHeroLandmark::WaystoneGallery => {
            append_gallery(&mut mesh, scale, 12, 8.0, 4.8);
        }
        IslandHeroLandmark::ChimeColonnade => {
            append_choir(&mut mesh, scale, 13, 8.2, 5.8, false);
        }
        IslandHeroLandmark::BluevaultSanctuary => {
            append_sanctuary(&mut mesh, scale, 10, 6.8, 3);
        }
        IslandHeroLandmark::SwitchbackBastion => {
            append_bastion(&mut mesh, scale, 5, 8.4, 4.8, true);
        }
        IslandHeroLandmark::SolarOrrery => {
            append_orrery(&mut mesh, scale, 4, 7.4, 2.8, 9);
        }
        IslandHeroLandmark::CloudbreakPortal => {
            append_portal(&mut mesh, scale, 8.4, 9.2, 5, 10);
        }
        IslandHeroLandmark::SkyCitadel => {
            append_citadel(&mut mesh, scale, 7, 7.8, 5.8, 3);
        }
        IslandHeroLandmark::HorizonLens => {
            append_lens(&mut mesh, scale, 9.4, 4.8, 4, false);
        }
        IslandHeroLandmark::ZenithSanctum => {
            append_sanctuary(&mut mesh, scale, 14, 8.4, 5);
        }
    }

    append_signature_stones(&mut mesh, scale, kind as usize + 1, seed);
    mesh.build()
}

fn append_court(
    mesh: &mut HeroMesh,
    scale: f32,
    pillars: usize,
    radius: f32,
    dais_height: f32,
    beacon_height: f32,
    crown: bool,
) {
    mesh.column(
        Vec3::ZERO,
        radius * 0.44 * scale,
        radius * 0.48 * scale,
        dais_height * scale,
        16,
    );
    mesh.spire(
        Vec3::Y * dais_height * scale,
        0.62 * scale,
        beacon_height * scale,
        8,
    );
    mesh.radial_columns(
        pillars,
        radius * 0.66 * scale,
        dais_height * scale,
        0.34 * scale,
        2.5 * scale,
    );
    mesh.ring_blocks(
        radius * 0.68 * scale,
        (dais_height + 2.55) * scale,
        0.22 * scale,
        0.24 * scale,
        pillars * 2,
        std::f32::consts::TAU,
    );
    if crown {
        mesh.ring_blocks(
            radius * 0.28 * scale,
            (dais_height + beacon_height * 0.74) * scale,
            0.20 * scale,
            0.18 * scale,
            10,
            std::f32::consts::TAU,
        );
    }
}

fn append_procession(
    mesh: &mut HeroMesh,
    scale: f32,
    steps: usize,
    length: f32,
    rise: f32,
    cairns: usize,
    fractured: bool,
) {
    for step in 0..steps {
        let progress = step as f32 / steps.saturating_sub(1).max(1) as f32;
        let lateral = if fractured && step % 3 == 1 {
            0.55 * scale
        } else {
            0.0
        };
        mesh.box_yaw(
            Vec3::new(
                lateral,
                (0.16 + progress * rise) * scale,
                (progress - 0.5) * length * scale,
            ),
            Vec3::new(1.25, 0.16, length / steps as f32 * 0.44) * scale,
            if fractured {
                (step as f32 * 0.17).sin() * 0.16
            } else {
                0.0
            },
        );
    }
    for index in 0..cairns {
        let progress = (index as f32 + 0.5) / cairns as f32;
        let side = if index % 2 == 0 { -1.0 } else { 1.0 };
        mesh.stacked_cairn(
            Vec3::new(side * 1.75 * scale, 0.0, (progress - 0.5) * length * scale),
            4 + index % 3,
            0.42 * scale,
        );
    }
    mesh.arch(
        Vec3::new(0.0, 0.0, length * 0.48 * scale),
        2.8 * scale,
        3.6 * scale,
        0.28 * scale,
    );
}

fn append_amphitheater(
    mesh: &mut HeroMesh,
    scale: f32,
    tiers: usize,
    petals: usize,
    tier_height: f32,
    centerpiece: bool,
) {
    for tier in 0..tiers {
        let radius = (2.4 + tier as f32 * 1.15) * scale;
        mesh.ring_blocks(
            radius,
            (0.26 + tier as f32 * tier_height) * scale,
            0.38 * scale,
            0.24 * scale,
            14 + tier * 4,
            std::f32::consts::PI * 1.62,
        );
    }
    mesh.radial_columns(petals, 2.15 * scale, 0.0, 0.22 * scale, 1.4 * scale);
    if centerpiece {
        mesh.column(Vec3::ZERO, 0.92 * scale, 1.15 * scale, 0.58 * scale, 14);
        mesh.spire(Vec3::Y * 0.58 * scale, 0.42 * scale, 2.6 * scale, 7);
    }
}

fn append_observatory(
    mesh: &mut HeroMesh,
    scale: f32,
    tower_height: f32,
    rings: usize,
    ribs: usize,
    lens_radius: f32,
) {
    mesh.column(
        Vec3::ZERO,
        1.25 * scale,
        1.05 * scale,
        tower_height * scale,
        12,
    );
    mesh.radial_columns(ribs, 2.25 * scale, 0.0, 0.18 * scale, 2.1 * scale);
    for ring in 0..rings {
        mesh.ring_blocks(
            (1.8 + ring as f32 * 0.65) * scale,
            (tower_height * (0.58 + ring as f32 * 0.14)) * scale,
            0.20 * scale,
            0.20 * scale,
            16 + ring * 4,
            std::f32::consts::TAU,
        );
    }
    mesh.vertical_ring_blocks(
        Vec3::Y * tower_height * 0.82 * scale,
        lens_radius * scale,
        0.22 * scale,
        18,
        true,
    );
}

fn append_instrument(
    mesh: &mut HeroMesh,
    scale: f32,
    strings: usize,
    height: f32,
    width: f32,
    chimes: bool,
) {
    let half_width = width * 0.5 * scale;
    mesh.column(
        Vec3::new(-half_width, 0.0, 0.0),
        0.42 * scale,
        0.32 * scale,
        height * scale,
        8,
    );
    mesh.column(
        Vec3::new(half_width, 0.0, 0.0),
        0.42 * scale,
        0.32 * scale,
        height * scale,
        8,
    );
    mesh.box_yaw(
        Vec3::new(0.0, height * scale, 0.0),
        Vec3::new(half_width + 0.35 * scale, 0.24 * scale, 0.32 * scale),
        0.0,
    );
    for string in 0..strings {
        let progress = (string + 1) as f32 / (strings + 1) as f32;
        let x = (-half_width + progress * width * scale) * 0.94;
        let string_height = height * (0.58 + (progress * std::f32::consts::PI).sin() * 0.34);
        let radius = if chimes { 0.10 } else { 0.055 };
        mesh.column(
            Vec3::new(x, (height - string_height) * scale, 0.0),
            radius * scale,
            radius * 0.76 * scale,
            string_height * scale,
            6,
        );
    }
    mesh.ring_blocks(
        half_width * 0.92,
        height * 0.36 * scale,
        0.18 * scale,
        0.18 * scale,
        strings + 3,
        std::f32::consts::PI,
    );
}

fn append_arcade(
    mesh: &mut HeroMesh,
    scale: f32,
    arches: usize,
    spacing: f32,
    height: f32,
    depth: f32,
) {
    for arch in 0..arches {
        let z = (arch as f32 - arches.saturating_sub(1) as f32 * 0.5) * spacing * scale;
        mesh.arch(
            Vec3::new(0.0, 0.0, z),
            3.5 * scale,
            height * scale,
            0.42 * scale,
        );
        if arch + 1 < arches {
            mesh.box_yaw(
                Vec3::new(0.0, height * 0.48 * scale, z + spacing * 0.5 * scale),
                Vec3::new(2.1, height * 0.48, depth * 0.18) * scale,
                0.0,
            );
        }
    }
}

fn append_grove(
    mesh: &mut HeroMesh,
    scale: f32,
    trunks: usize,
    radius: f32,
    canopy_rings: usize,
    sun_spire: bool,
) {
    for trunk in 0..trunks {
        let angle = trunk as f32 / trunks as f32 * std::f32::consts::TAU;
        let trunk_radius = radius * (0.62 + (trunk % 3) as f32 * 0.08) * scale;
        let height = (3.0 + (trunk % 4) as f32 * 0.42) * scale;
        let center = Vec3::new(angle.cos() * trunk_radius, 0.0, angle.sin() * trunk_radius);
        mesh.column(center, 0.30 * scale, 0.20 * scale, height, 7);
        mesh.spire(center + Vec3::Y * height, 0.46 * scale, 1.1 * scale, 6);
    }
    for ring in 0..canopy_rings {
        mesh.ring_blocks(
            (radius * (0.58 + ring as f32 * 0.16)) * scale,
            (3.5 + ring as f32 * 0.48) * scale,
            0.22 * scale,
            0.18 * scale,
            14 + ring * 4,
            std::f32::consts::TAU,
        );
    }
    if sun_spire {
        mesh.spire(Vec3::ZERO, 0.72 * scale, 5.8 * scale, 9);
    }
}

fn append_canopy(
    mesh: &mut HeroMesh,
    scale: f32,
    supports: usize,
    height: f32,
    radius: f32,
    roof_rings: usize,
) {
    mesh.radial_columns(
        supports,
        radius * 0.74 * scale,
        0.0,
        0.32 * scale,
        height * scale,
    );
    for ring in 0..roof_rings {
        mesh.ring_blocks(
            radius * (0.40 + ring as f32 * 0.22) * scale,
            (height + ring as f32 * 0.22) * scale,
            0.28 * scale,
            0.18 * scale,
            supports * 2 + ring * 3,
            std::f32::consts::TAU,
        );
    }
    for spoke in 0..supports {
        let angle = spoke as f32 / supports as f32 * std::f32::consts::TAU;
        mesh.box_yaw(
            Vec3::new(
                angle.cos() * radius * 0.35 * scale,
                (height + 0.18) * scale,
                angle.sin() * radius * 0.35 * scale,
            ),
            Vec3::new(radius * 0.38, 0.16, 0.18) * scale,
            angle,
        );
    }
}

fn append_totem_field(
    mesh: &mut HeroMesh,
    scale: f32,
    totems: usize,
    max_height: f32,
    radius: f32,
) {
    for totem in 0..totems {
        let angle = totem as f32 / totems as f32 * std::f32::consts::TAU;
        let radial = radius * (0.42 + (totem % 3) as f32 * 0.18) * scale;
        let height = max_height * (0.48 + (totem % 5) as f32 * 0.11) * scale;
        let center = Vec3::new(angle.cos() * radial, 0.0, angle.sin() * radial);
        mesh.spire(
            center,
            (0.42 + (totem % 2) as f32 * 0.12) * scale,
            height,
            6 + totem % 3,
        );
        mesh.ring_blocks(
            radial.max(0.8 * scale),
            height * 0.62,
            0.12 * scale,
            0.12 * scale,
            6,
            std::f32::consts::FRAC_PI_2,
        );
    }
}

fn append_lens(
    mesh: &mut HeroMesh,
    scale: f32,
    height: f32,
    radius: f32,
    rings: usize,
    needle: bool,
) {
    mesh.column(Vec3::ZERO, 1.05 * scale, 0.92 * scale, 1.1 * scale, 12);
    for ring in 0..rings {
        let ring_radius = radius * (1.0 - ring as f32 * 0.16).max(0.42) * scale;
        let ring_thickness = (0.24 - ring as f32 * 0.025).max(0.12) * scale;
        let ring_center_y =
            ((height * 0.54 + ring as f32 * 0.20) * scale).max(ring_radius + ring_thickness * 1.5);
        mesh.vertical_ring_blocks(
            Vec3::Y * ring_center_y,
            ring_radius,
            ring_thickness,
            HERO_RING_SEGMENTS + ring * 4,
            ring % 2 == 0,
        );
    }
    if needle {
        mesh.spire(Vec3::ZERO, 0.52 * scale, height * scale, 9);
    } else {
        mesh.column(
            Vec3::Y * (height * 0.48 - radius * 0.34) * scale,
            0.48 * scale,
            0.28 * scale,
            radius * 0.68 * scale,
            10,
        );
    }
}

fn append_temple(
    mesh: &mut HeroMesh,
    scale: f32,
    tiers: usize,
    columns: usize,
    height: f32,
    moon_ring: bool,
) {
    for tier in 0..tiers {
        mesh.column(
            Vec3::ZERO,
            (3.8 - tier as f32 * 0.46).max(1.4) * scale,
            (4.0 - tier as f32 * 0.44).max(1.5) * scale,
            (0.34 + tier as f32 * 0.06) * scale,
            16,
        );
    }
    mesh.radial_columns(
        columns,
        2.8 * scale,
        0.0,
        0.34 * scale,
        height * 0.64 * scale,
    );
    mesh.ring_blocks(
        2.9 * scale,
        height * 0.66 * scale,
        0.32 * scale,
        0.30 * scale,
        columns * 2,
        std::f32::consts::TAU,
    );
    mesh.spire(
        Vec3::Y * height * 0.65 * scale,
        0.76 * scale,
        height * 0.38 * scale,
        8,
    );
    if moon_ring {
        mesh.vertical_ring_blocks(
            Vec3::Y * height * 0.82 * scale,
            1.75 * scale,
            0.18 * scale,
            16,
            false,
        );
    }
}

fn append_portal(
    mesh: &mut HeroMesh,
    scale: f32,
    width: f32,
    height: f32,
    nested: usize,
    roots: usize,
) {
    for portal in 0..nested {
        let inset = portal as f32 * 0.48 * scale;
        mesh.arch(
            Vec3::new(
                0.0,
                0.0,
                (portal as f32 - nested as f32 * 0.5) * 0.38 * scale,
            ),
            width * scale - inset,
            height * scale - inset * 0.55,
            (0.46 - portal as f32 * 0.035).max(0.24) * scale,
        );
    }
    for root in 0..roots {
        let angle = root as f32 / roots as f32 * std::f32::consts::TAU;
        mesh.box_yaw(
            Vec3::new(
                angle.cos() * width * 0.42 * scale,
                0.14 * scale,
                angle.sin() * width * 0.34 * scale,
            ),
            Vec3::new(width * 0.24, 0.14, 0.18) * scale,
            angle,
        );
    }
}

fn append_choir(
    mesh: &mut HeroMesh,
    scale: f32,
    voices: usize,
    radius: f32,
    height: f32,
    gate: bool,
) {
    for voice in 0..voices {
        let progress = voice as f32 / voices.saturating_sub(1).max(1) as f32;
        let angle = -1.15 + progress * 2.30;
        let column_height = height * (0.62 + (progress * std::f32::consts::PI).sin() * 0.38);
        let center = Vec3::new(
            angle.sin() * radius * scale,
            0.0,
            angle.cos() * radius * scale,
        );
        mesh.column(center, 0.25 * scale, 0.18 * scale, column_height * scale, 7);
        mesh.spire(
            center + Vec3::Y * column_height * scale,
            0.28 * scale,
            0.72 * scale,
            6,
        );
    }
    mesh.ring_blocks(
        radius * scale,
        height * 0.72 * scale,
        0.18 * scale,
        0.18 * scale,
        voices + 5,
        2.45,
    );
    if gate {
        mesh.arch(Vec3::ZERO, 4.2 * scale, height * 0.82 * scale, 0.38 * scale);
    }
}

fn append_training_circle(
    mesh: &mut HeroMesh,
    scale: f32,
    pylons: usize,
    radius: f32,
    gates: usize,
) {
    mesh.ring_blocks(
        radius * scale,
        0.22 * scale,
        0.34 * scale,
        0.20 * scale,
        HERO_RING_SEGMENTS + pylons,
        std::f32::consts::TAU,
    );
    mesh.radial_columns(
        pylons,
        radius * 0.88 * scale,
        0.0,
        0.20 * scale,
        2.0 * scale,
    );
    for gate in 0..gates {
        let angle = gate as f32 / gates as f32 * std::f32::consts::TAU;
        let center = Vec3::new(
            angle.cos() * radius * 0.56 * scale,
            0.0,
            angle.sin() * radius * 0.56 * scale,
        );
        mesh.arch_oriented(center, 2.2 * scale, 3.0 * scale, 0.24 * scale, -angle);
    }
}

fn append_petaled_arcade(
    mesh: &mut HeroMesh,
    scale: f32,
    petals: usize,
    radius: f32,
    inner_columns: usize,
) {
    for petal in 0..petals {
        let angle = petal as f32 / petals as f32 * std::f32::consts::TAU;
        let center = Vec3::new(
            angle.cos() * radius * scale,
            0.0,
            angle.sin() * radius * scale,
        );
        mesh.arch_oriented(center, 2.6 * scale, 3.8 * scale, 0.28 * scale, -angle);
    }
    mesh.radial_columns(
        inner_columns,
        radius * 0.48 * scale,
        0.0,
        0.22 * scale,
        2.6 * scale,
    );
    mesh.ring_blocks(
        radius * 0.76 * scale,
        3.7 * scale,
        0.18 * scale,
        0.16 * scale,
        petals * 3,
        std::f32::consts::TAU,
    );
}

fn append_monolith(
    mesh: &mut HeroMesh,
    scale: f32,
    height: f32,
    base_radius: f32,
    fractures: usize,
    halo: bool,
) {
    mesh.spire(Vec3::ZERO, base_radius * scale, height * scale, 7);
    for fracture in 0..fractures {
        let y = height * (0.18 + fracture as f32 / fractures.max(1) as f32 * 0.62) * scale;
        let angle = fracture as f32 * 1.73;
        mesh.box_yaw(
            Vec3::new(angle.cos() * 0.42 * scale, y, angle.sin() * 0.42 * scale),
            Vec3::new(base_radius * 0.78, 0.10, 0.16) * scale,
            angle,
        );
    }
    if halo {
        mesh.ring_blocks(
            2.5 * scale,
            height * 0.68 * scale,
            0.18 * scale,
            0.16 * scale,
            14,
            std::f32::consts::TAU,
        );
    }
}

fn append_cairn(mesh: &mut HeroMesh, scale: f32, stones: usize, height: f32, satellites: usize) {
    mesh.stacked_cairn(Vec3::ZERO, stones, 1.05 * scale);
    for satellite in 0..satellites {
        let angle = satellite as f32 / satellites as f32 * std::f32::consts::TAU;
        mesh.stacked_cairn(
            Vec3::new(angle.cos() * 3.4 * scale, 0.0, angle.sin() * 3.4 * scale),
            4 + satellite,
            0.52 * scale,
        );
    }
    mesh.vertical_ring_blocks(
        Vec3::Y * height * 0.72 * scale,
        1.6 * scale,
        0.15 * scale,
        13,
        true,
    );
}

fn append_windbreak(mesh: &mut HeroMesh, scale: f32, ribs: usize, height: f32, width: f32) {
    for rib in 0..ribs {
        let progress = rib as f32 / ribs.saturating_sub(1).max(1) as f32;
        let x = (progress - 0.5) * width * scale;
        let rib_height = height * (0.54 + (progress * std::f32::consts::PI).sin() * 0.46);
        mesh.spire(
            Vec3::new(x, 0.0, (progress * 5.0).sin() * 0.45 * scale),
            0.34 * scale,
            rib_height * scale,
            6,
        );
    }
    for brace in 0..3 {
        mesh.box_yaw(
            Vec3::new(0.0, height * (0.30 + brace as f32 * 0.18) * scale, 0.0),
            Vec3::new(width * 0.48, 0.12, 0.18) * scale,
            0.0,
        );
    }
}

fn append_sail(mesh: &mut HeroMesh, scale: f32, ribs: usize, height: f32, width: f32) {
    mesh.column(Vec3::ZERO, 0.52 * scale, 0.34 * scale, height * scale, 9);
    for rib in 0..ribs {
        let progress = (rib + 1) as f32 / (ribs + 1) as f32;
        let y = progress * height * 0.82 * scale;
        let reach = width * (1.0 - (progress - 0.42).abs() * 1.35).max(0.18) * scale;
        mesh.box_yaw(
            Vec3::new(reach * 0.46, y, 0.0),
            Vec3::new(reach * 0.50, 0.12 * scale, 0.14 * scale),
            0.0,
        );
        mesh.spire(
            Vec3::new(reach, y - 0.18 * scale, 0.0),
            0.18 * scale,
            0.56 * scale,
            6,
        );
    }
    mesh.ring_blocks(
        width * 0.58 * scale,
        height * 0.18 * scale,
        0.22 * scale,
        0.18 * scale,
        12,
        std::f32::consts::PI,
    );
}

fn append_orrery(
    mesh: &mut HeroMesh,
    scale: f32,
    rings: usize,
    height: f32,
    core_radius: f32,
    satellites: usize,
) {
    mesh.column(
        Vec3::ZERO,
        0.82 * scale,
        0.66 * scale,
        height * 0.54 * scale,
        10,
    );
    mesh.column(
        Vec3::Y * height * 0.52 * scale,
        core_radius * scale,
        core_radius * 0.94 * scale,
        core_radius * 1.5 * scale,
        14,
    );
    for ring in 0..rings {
        let radius = core_radius * (1.55 + ring as f32 * 0.52) * scale;
        if ring % 2 == 0 {
            mesh.ring_blocks(
                radius,
                height * (0.62 + ring as f32 * 0.06) * scale,
                0.16 * scale,
                0.14 * scale,
                16 + ring * 4,
                std::f32::consts::TAU,
            );
        } else {
            mesh.vertical_ring_blocks(
                Vec3::Y * (height * 0.68 * scale).max(radius + 0.30 * scale),
                radius,
                0.16 * scale,
                18 + ring * 4,
                ring % 3 == 0,
            );
        }
    }
    mesh.radial_columns(
        satellites,
        core_radius * 2.65 * scale,
        height * 0.68 * scale,
        0.20 * scale,
        0.72 * scale,
    );
}

fn append_seat(mesh: &mut HeroMesh, scale: f32, steps: usize, height: f32, crown_points: usize) {
    for step in 0..steps {
        let progress = step as f32 / steps as f32;
        mesh.box_yaw(
            Vec3::new(
                0.0,
                (step as f32 + 0.5) * 0.34 * scale,
                progress * 2.8 * scale,
            ),
            Vec3::new((3.8 - progress * 1.6) * scale, 0.17 * scale, 0.54 * scale),
            0.0,
        );
    }
    mesh.box_yaw(
        Vec3::new(0.0, height * 0.46 * scale, 2.8 * scale),
        Vec3::new(2.2, height * 0.46, 0.42) * scale,
        0.0,
    );
    for point in 0..crown_points {
        let x = (point as f32 - crown_points.saturating_sub(1) as f32 * 0.5) * 0.9 * scale;
        mesh.spire(
            Vec3::new(x, height * 0.92 * scale, 2.8 * scale),
            0.30 * scale,
            (1.0 + (point % 2) as f32 * 0.45) * scale,
            6,
        );
    }
}

fn append_aqueduct(
    mesh: &mut HeroMesh,
    scale: f32,
    arches: usize,
    length: f32,
    height: f32,
    hooked: bool,
) {
    let spacing = length / arches as f32;
    for arch in 0..arches {
        let x = (arch as f32 - arches.saturating_sub(1) as f32 * 0.5) * spacing * scale;
        mesh.arch_oriented(
            Vec3::new(x, 0.0, 0.0),
            spacing * 0.86 * scale,
            height * scale,
            0.28 * scale,
            std::f32::consts::FRAC_PI_2,
        );
    }
    mesh.box_yaw(
        Vec3::new(0.0, height * scale, 0.0),
        Vec3::new(length * 0.56, 0.28, 0.44) * scale,
        0.0,
    );
    if hooked {
        mesh.ring_blocks(
            length * 0.54 * scale,
            (height + 0.25) * scale,
            0.24 * scale,
            0.24 * scale,
            12,
            std::f32::consts::FRAC_PI_2,
        );
    }
}

fn append_lookout(
    mesh: &mut HeroMesh,
    scale: f32,
    height: f32,
    balcony_radius: f32,
    balcony_segments: usize,
    fins: usize,
) {
    mesh.column(Vec3::ZERO, 1.28 * scale, 0.88 * scale, height * scale, 10);
    mesh.ring_blocks(
        balcony_radius * scale,
        height * 0.72 * scale,
        0.42 * scale,
        0.28 * scale,
        balcony_segments,
        std::f32::consts::TAU,
    );
    mesh.radial_columns(
        balcony_segments / 2,
        balcony_radius * scale,
        height * 0.56 * scale,
        0.16 * scale,
        height * 0.18 * scale,
    );
    for fin in 0..fins {
        let angle = fin as f32 / fins as f32 * std::f32::consts::TAU;
        mesh.spire(
            Vec3::new(
                angle.cos() * 1.15 * scale,
                height * scale,
                angle.sin() * 1.15 * scale,
            ),
            0.34 * scale,
            1.8 * scale,
            6,
        );
    }
}

fn append_spire(
    mesh: &mut HeroMesh,
    scale: f32,
    height: f32,
    perches: usize,
    crown_points: usize,
    ringed: bool,
) {
    mesh.spire(Vec3::ZERO, 1.38 * scale, height * scale, 8);
    for perch in 0..perches {
        let angle = perch as f32 / perches as f32 * std::f32::consts::TAU;
        let y = height * (0.24 + perch as f32 / perches as f32 * 0.52) * scale;
        let reach = (2.2 + (perch % 2) as f32 * 0.7) * scale;
        mesh.box_yaw(
            Vec3::new(angle.cos() * reach * 0.46, y, angle.sin() * reach * 0.46),
            Vec3::new(reach * 0.5, 0.14 * scale, 0.18 * scale),
            angle,
        );
    }
    mesh.radial_columns(
        crown_points,
        1.35 * scale,
        height * 0.88 * scale,
        0.18 * scale,
        1.2 * scale,
    );
    if ringed {
        mesh.ring_blocks(
            2.8 * scale,
            height * 0.66 * scale,
            0.17 * scale,
            0.15 * scale,
            15,
            std::f32::consts::TAU,
        );
    }
}

fn append_anvil(mesh: &mut HeroMesh, scale: f32, height: f32, width: f32, vents: usize) {
    mesh.column(
        Vec3::ZERO,
        1.45 * scale,
        1.05 * scale,
        height * 0.64 * scale,
        8,
    );
    mesh.box_yaw(
        Vec3::new(0.0, height * 0.68 * scale, 0.0),
        Vec3::new(width * 0.34, 0.62, 1.45) * scale,
        0.0,
    );
    mesh.box_yaw(
        Vec3::new(width * 0.38 * scale, height * 0.74 * scale, 0.0),
        Vec3::new(width * 0.26, 0.32, 0.92) * scale,
        0.0,
    );
    mesh.spire(
        Vec3::new(width * 0.68 * scale, height * 0.68 * scale, 0.0),
        0.72 * scale,
        width * 0.34 * scale,
        6,
    );
    for vent in 0..vents {
        let angle = vent as f32 / vents as f32 * std::f32::consts::TAU;
        mesh.spire(
            Vec3::new(angle.cos() * 2.4 * scale, 0.0, angle.sin() * 2.4 * scale),
            0.34 * scale,
            (2.2 + vent as f32 * 0.28) * scale,
            6,
        );
    }
}

fn append_gallery(mesh: &mut HeroMesh, scale: f32, stones: usize, length: f32, height: f32) {
    for stone in 0..stones {
        let progress = stone as f32 / stones.saturating_sub(1).max(1) as f32;
        let x = (progress - 0.5) * length * scale;
        let z = if stone % 2 == 0 {
            -1.8 * scale
        } else {
            1.8 * scale
        };
        let stone_height = height * (0.58 + (stone % 4) as f32 * 0.12) * scale;
        mesh.spire(
            Vec3::new(x, 0.0, z),
            (0.46 + (stone % 3) as f32 * 0.10) * scale,
            stone_height,
            6 + stone % 3,
        );
    }
    mesh.box_yaw(
        Vec3::new(0.0, 0.24 * scale, 0.0),
        Vec3::new(length * 0.54, 0.24, 1.15) * scale,
        0.0,
    );
    mesh.ring_blocks(
        length * 0.44 * scale,
        height * 0.76 * scale,
        0.16 * scale,
        0.15 * scale,
        18,
        std::f32::consts::PI,
    );
}

fn append_sanctuary(
    mesh: &mut HeroMesh,
    scale: f32,
    columns: usize,
    radius: f32,
    roof_rings: usize,
) {
    mesh.radial_columns(
        columns,
        radius * 0.66 * scale,
        0.0,
        0.36 * scale,
        4.8 * scale,
    );
    for ring in 0..roof_rings {
        mesh.ring_blocks(
            radius * (0.74 - ring as f32 * 0.12).max(0.28) * scale,
            (4.8 + ring as f32 * 0.58) * scale,
            0.32 * scale,
            0.22 * scale,
            columns + ring * 4,
            std::f32::consts::TAU,
        );
    }
    mesh.column(Vec3::ZERO, 1.6 * scale, 1.35 * scale, 1.0 * scale, 14);
    mesh.spire(
        Vec3::Y * (4.8 + roof_rings as f32 * 0.55) * scale,
        0.72 * scale,
        2.2 * scale,
        8,
    );
}

fn append_bastion(
    mesh: &mut HeroMesh,
    scale: f32,
    towers: usize,
    length: f32,
    height: f32,
    switchback: bool,
) {
    for tower in 0..towers {
        let progress = tower as f32 / towers.saturating_sub(1).max(1) as f32;
        let x = (progress - 0.5) * length * scale;
        let z = if switchback && tower % 2 == 0 {
            -1.6 * scale
        } else {
            1.6 * scale
        };
        mesh.column(
            Vec3::new(x, 0.0, z),
            0.86 * scale,
            0.68 * scale,
            height * (0.76 + progress * 0.24) * scale,
            8,
        );
    }
    for wall in 0..towers.saturating_sub(1) {
        let progress = (wall as f32 + 0.5) / towers.saturating_sub(1).max(1) as f32;
        let x = (progress - 0.5) * length * scale;
        mesh.box_yaw(
            Vec3::new(x, height * 0.34 * scale, 0.0),
            Vec3::new(length / towers as f32 * 0.60, height * 0.34, 0.38) * scale,
            if switchback && wall % 2 == 0 {
                0.28
            } else {
                -0.28
            },
        );
    }
}

fn append_citadel(
    mesh: &mut HeroMesh,
    scale: f32,
    towers: usize,
    radius: f32,
    height: f32,
    levels: usize,
) {
    mesh.radial_columns(
        towers,
        radius * 0.64 * scale,
        0.0,
        0.82 * scale,
        height * scale,
    );
    for level in 0..levels {
        mesh.ring_blocks(
            radius * (0.68 - level as f32 * 0.10) * scale,
            height * (0.34 + level as f32 * 0.24) * scale,
            0.40 * scale,
            0.32 * scale,
            towers * 2 + level * 4,
            std::f32::consts::TAU,
        );
    }
    mesh.column(
        Vec3::ZERO,
        2.0 * scale,
        1.35 * scale,
        height * 1.18 * scale,
        10,
    );
    mesh.spire(
        Vec3::Y * height * 1.16 * scale,
        0.82 * scale,
        2.6 * scale,
        8,
    );
}

fn append_signature_stones(mesh: &mut HeroMesh, scale: f32, count: usize, seed: u32) {
    let radius = (5.1 + (seed & 3) as f32 * 0.18) * scale;
    for stone in 0..count {
        let angle = stone as f32 / count as f32 * std::f32::consts::TAU
            + ((seed.rotate_left((stone % 31) as u32) & 0xff) as f32 / 255.0 - 0.5) * 0.06;
        let height = (0.22 + (stone % 4) as f32 * 0.055) * scale;
        mesh.spire(
            Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius),
            0.10 * scale,
            height,
            5,
        );
    }
}

#[derive(Default)]
struct HeroMesh {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl HeroMesh {
    fn box_yaw(&mut self, center: Vec3, half_extents: Vec3, yaw: f32) {
        self.box_rotated(center, half_extents, Quat::from_rotation_y(yaw));
    }

    fn box_rotated(&mut self, center: Vec3, half_extents: Vec3, rotation: Quat) {
        let faces = [
            (
                Vec3::X,
                [
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(1.0, -1.0, 1.0),
                    Vec3::new(1.0, 1.0, 1.0),
                    Vec3::new(1.0, 1.0, -1.0),
                ],
            ),
            (
                Vec3::NEG_X,
                [
                    Vec3::new(-1.0, -1.0, 1.0),
                    Vec3::new(-1.0, -1.0, -1.0),
                    Vec3::new(-1.0, 1.0, -1.0),
                    Vec3::new(-1.0, 1.0, 1.0),
                ],
            ),
            (
                Vec3::Y,
                [
                    Vec3::new(-1.0, 1.0, -1.0),
                    Vec3::new(1.0, 1.0, -1.0),
                    Vec3::new(1.0, 1.0, 1.0),
                    Vec3::new(-1.0, 1.0, 1.0),
                ],
            ),
            (
                Vec3::NEG_Y,
                [
                    Vec3::new(-1.0, -1.0, 1.0),
                    Vec3::new(1.0, -1.0, 1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(-1.0, -1.0, -1.0),
                ],
            ),
            (
                Vec3::Z,
                [
                    Vec3::new(1.0, -1.0, 1.0),
                    Vec3::new(-1.0, -1.0, 1.0),
                    Vec3::new(-1.0, 1.0, 1.0),
                    Vec3::new(1.0, 1.0, 1.0),
                ],
            ),
            (
                Vec3::NEG_Z,
                [
                    Vec3::new(-1.0, -1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(1.0, 1.0, -1.0),
                    Vec3::new(-1.0, 1.0, -1.0),
                ],
            ),
        ];

        for (normal, corners) in faces {
            let start = self.positions.len() as u32;
            for corner in corners {
                let local = corner * half_extents;
                self.positions.push((center + rotation * local).to_array());
                self.normals.push((rotation * normal).to_array());
            }
            self.uvs
                .extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            self.indices
                .extend([start, start + 1, start + 2, start, start + 2, start + 3]);
        }
    }

    fn column(
        &mut self,
        base_center: Vec3,
        bottom_radius: f32,
        top_radius: f32,
        height: f32,
        segments: usize,
    ) {
        let segments = segments.max(3);
        let side_start = self.positions.len() as u32;
        let slope = (bottom_radius - top_radius) / height.max(0.001);
        for level in 0..=1 {
            let radius = if level == 0 {
                bottom_radius
            } else {
                top_radius
            };
            let y = base_center.y + height * level as f32;
            for segment in 0..segments {
                let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
                let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
                self.positions.push(
                    (Vec3::new(base_center.x, y, base_center.z) + radial * radius).to_array(),
                );
                self.normals
                    .push(Vec3::new(radial.x, slope, radial.z).normalize().to_array());
                self.uvs
                    .push([segment as f32 / segments as f32, level as f32]);
            }
        }
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            let bottom_a = side_start + segment as u32;
            let bottom_b = side_start + next as u32;
            let top_a = side_start + segments as u32 + segment as u32;
            let top_b = side_start + segments as u32 + next as u32;
            self.indices
                .extend([bottom_a, top_a, bottom_b, bottom_b, top_a, top_b]);
        }

        self.cap(
            Vec3::new(base_center.x, base_center.y, base_center.z),
            bottom_radius,
            segments,
            false,
        );
        self.cap(
            Vec3::new(base_center.x, base_center.y + height, base_center.z),
            top_radius,
            segments,
            true,
        );
    }

    fn cap(&mut self, center: Vec3, radius: f32, segments: usize, top: bool) {
        let start = self.positions.len() as u32;
        let normal = if top { Vec3::Y } else { Vec3::NEG_Y };
        self.positions.push(center.to_array());
        self.normals.push(normal.to_array());
        self.uvs.push([0.5, 0.5]);
        for segment in 0..segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            self.positions
                .push((center + Vec3::new(angle.cos(), 0.0, angle.sin()) * radius).to_array());
            self.normals.push(normal.to_array());
            self.uvs
                .push([angle.cos() * 0.5 + 0.5, angle.sin() * 0.5 + 0.5]);
        }
        for segment in 0..segments {
            let a = start + 1 + segment as u32;
            let b = start + 1 + ((segment + 1) % segments) as u32;
            if top {
                self.indices.extend([start, a, b]);
            } else {
                self.indices.extend([start, b, a]);
            }
        }
    }

    fn spire(&mut self, base_center: Vec3, radius: f32, height: f32, segments: usize) {
        self.column(base_center, radius, radius * 0.62, height * 0.72, segments);
        self.column(
            base_center + Vec3::Y * height * 0.72,
            radius * 0.62,
            0.04,
            height * 0.28,
            segments,
        );
    }

    fn radial_columns(
        &mut self,
        count: usize,
        radius: f32,
        base_y: f32,
        column_radius: f32,
        height: f32,
    ) {
        for column in 0..count {
            let angle = column as f32 / count as f32 * std::f32::consts::TAU;
            self.column(
                Vec3::new(angle.cos() * radius, base_y, angle.sin() * radius),
                column_radius,
                column_radius * 0.82,
                height,
                HERO_COLUMN_SEGMENTS,
            );
        }
    }

    fn ring_blocks(
        &mut self,
        radius: f32,
        center_y: f32,
        radial_half: f32,
        vertical_half: f32,
        segments: usize,
        arc: f32,
    ) {
        let segments = segments.max(3);
        for segment in 0..segments {
            let progress = if arc >= std::f32::consts::TAU - 0.01 {
                segment as f32 / segments as f32
            } else {
                segment as f32 / segments.saturating_sub(1).max(1) as f32
            };
            let angle = progress * arc - arc * 0.5;
            let tangent_half = (radius * arc / segments as f32 * 0.58).max(radial_half * 0.8);
            self.box_yaw(
                Vec3::new(angle.cos() * radius, center_y, angle.sin() * radius),
                Vec3::new(tangent_half, vertical_half, radial_half),
                angle + std::f32::consts::FRAC_PI_2,
            );
        }
    }

    fn vertical_ring_blocks(
        &mut self,
        center: Vec3,
        radius: f32,
        thickness: f32,
        segments: usize,
        x_plane: bool,
    ) {
        let segments = segments.max(6);
        for segment in 0..segments {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let radial = Vec2::new(angle.cos(), angle.sin()) * radius;
            let position = if x_plane {
                center + Vec3::new(radial.x, radial.y, 0.0)
            } else {
                center + Vec3::new(0.0, radial.y, radial.x)
            };
            let tangent_half =
                (radius * std::f32::consts::TAU / segments as f32 * 0.56).max(thickness * 0.8);
            let (half_extents, rotation) = if x_plane {
                (
                    Vec3::new(tangent_half, thickness, thickness),
                    Quat::from_rotation_z(angle + std::f32::consts::FRAC_PI_2),
                )
            } else {
                (
                    Vec3::new(thickness, tangent_half, thickness),
                    Quat::from_rotation_x(-angle),
                )
            };
            self.box_rotated(position, half_extents, rotation);
        }
    }

    fn arch(&mut self, center: Vec3, width: f32, height: f32, thickness: f32) {
        self.arch_oriented(center, width, height, thickness, 0.0);
    }

    fn arch_oriented(&mut self, center: Vec3, width: f32, height: f32, thickness: f32, yaw: f32) {
        let transform = |point: Vec3| center + Quat::from_rotation_y(yaw) * point;
        self.column(
            transform(Vec3::new(-width * 0.5, 0.0, 0.0)),
            thickness,
            thickness * 0.88,
            height * 0.64,
            8,
        );
        self.column(
            transform(Vec3::new(width * 0.5, 0.0, 0.0)),
            thickness,
            thickness * 0.88,
            height * 0.64,
            8,
        );
        let arch_segments: usize = 9;
        for segment in 0..arch_segments {
            let progress = segment as f32 / arch_segments.saturating_sub(1) as f32;
            let angle = progress * std::f32::consts::PI;
            let local = Vec3::new(
                angle.cos() * width * 0.5,
                height * 0.62 + angle.sin() * height * 0.38,
                0.0,
            );
            self.box_rotated(
                transform(local),
                Vec3::new(
                    width * 0.5 / arch_segments as f32 * 1.34,
                    thickness,
                    thickness,
                ),
                Quat::from_rotation_y(yaw)
                    * Quat::from_rotation_z(-angle + std::f32::consts::FRAC_PI_2),
            );
        }
    }

    fn stacked_cairn(&mut self, center: Vec3, stones: usize, base_radius: f32) {
        let mut y = center.y;
        for stone in 0..stones {
            let progress = stone as f32 / stones.max(1) as f32;
            let radius = base_radius * (1.0 - progress * 0.62);
            let height = radius * (0.62 + (stone % 2) as f32 * 0.12);
            self.column(
                Vec3::new(
                    center.x + (stone as f32 * 1.71).sin() * radius * 0.12,
                    y,
                    center.z + (stone as f32 * 1.31).cos() * radius * 0.10,
                ),
                radius,
                radius * 0.82,
                height,
                7,
            );
            y += height * 0.88;
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

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::world::{SkyRoute, island_art_directions};
    use std::collections::HashSet;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values,
            _ => panic!("hero mesh should expose Float32x3 positions"),
        }
    }

    fn projected_range(positions: &[[f32; 3]], axis: Vec3) -> f32 {
        let (min, max) = positions.iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(min, max), position| {
                let projection = Vec3::from_array(*position).dot(axis);
                (min.min(projection), max.max(projection))
            },
        );
        max - min
    }

    #[test]
    fn every_authored_island_has_one_matching_hero_landmark() {
        let route = SkyRoute::default();
        let profiles = island_art_directions();
        assert_eq!(route.islands().len(), profiles.len());

        for (island_index, (island, profile)) in
            route.islands().iter().zip(profiles.iter()).enumerate()
        {
            let spec = island_hero_landmark_spec(island_index, *island)
                .unwrap_or_else(|| panic!("{} should have a hero", island.name));
            assert_eq!(spec.kind, profile.hero_landmark);
            assert_eq!(spec.label, profile.hero_landmark.label());
            assert!(spec.translation.is_finite());
            assert!(spec.visual_half_extents.min_element() > 0.0);
            assert!(spec.collision_half_extents.is_none());
        }
    }

    #[test]
    fn hero_meshes_are_substantial_finite_and_geometrically_distinct() {
        let route = SkyRoute::default();
        let mut fingerprints = HashSet::new();

        for (island_index, island) in route.islands().iter().enumerate() {
            let spec = island_hero_landmark_spec(island_index, *island).expect("hero spec");
            let mesh = spec.build_mesh();
            let positions = positions(&mesh);
            assert!(
                positions.len() >= 250,
                "{} hero mesh is too sparse: {} vertices",
                island.name,
                positions.len()
            );
            assert!(
                positions.iter().flatten().copied().all(f32::is_finite),
                "{} hero mesh has non-finite geometry",
                island.name
            );
            let min_y = positions
                .iter()
                .map(|position| position[1])
                .fold(f32::INFINITY, f32::min);
            let max_y = positions
                .iter()
                .map(|position| position[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(min_y >= -0.001, "{} hero sinks below its base", island.name);
            assert!(
                max_y - min_y >= 2.0,
                "{} hero lacks vertical silhouette",
                island.name
            );

            let fingerprint = (
                positions.len(),
                (max_y * 100.0).round() as i32,
                positions
                    .iter()
                    .map(|position| position[0].abs() + position[2].abs())
                    .sum::<f32>()
                    .round() as i32,
            );
            assert!(
                fingerprints.insert(fingerprint),
                "{} duplicates another hero geometry fingerprint",
                island.name
            );
        }
        assert_eq!(fingerprints.len(), 41);
    }

    #[test]
    fn every_hero_visual_bound_contains_its_generated_xyz_geometry() {
        let route = SkyRoute::default();
        let mut kinds = HashSet::new();
        let mut failures = Vec::new();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let spec = island_hero_landmark_spec(island_index, island).expect("hero spec");
            assert!(
                kinds.insert(spec.kind),
                "{} repeats a hero kind",
                island.name
            );
            let mesh = spec.build_mesh();
            let mut required = Vec3::ZERO;
            let mut min_y = f32::INFINITY;
            for position in positions(&mesh) {
                let position = Vec3::from_array(*position);
                required.x = required.x.max(position.x.abs());
                required.y = required.y.max(position.y * 0.5);
                required.z = required.z.max(position.z.abs());
                min_y = min_y.min(position.y);
            }

            if min_y < -0.001
                || !required
                    .cmple(spec.visual_half_extents + Vec3::splat(0.001))
                    .all()
            {
                failures.push(format!(
                    "{} ({:?}) requires local {:?}, publishes local {:?}, min_y={min_y:.3}",
                    island.name,
                    spec.kind,
                    required / spec.scale,
                    spec.visual_half_extents / spec.scale,
                ));
            }
        }

        assert_eq!(kinds.len(), 41);
        assert!(
            failures.is_empty(),
            "hero visual bounds do not enclose generated geometry:\n{}",
            failures.join("\n")
        );
    }

    #[test]
    fn hero_semantic_samples_are_mesh_derived_finite_and_spatially_distinct() {
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let spec = island_hero_landmark_spec(island_index, island).expect("hero spec");
            let samples = spec.semantic_sample_positions();
            assert!(
                samples
                    .into_iter()
                    .all(|sample| sample.is_finite() && sample.y > spec.translation.y),
                "{} hero semantic samples must sit on finite above-ground geometry",
                island.name
            );
            for left in 0..samples.len() {
                for right in left + 1..samples.len() {
                    assert!(
                        samples[left].distance(samples[right]) > spec.scale * 0.05,
                        "{} hero semantic samples must use distinct mesh triangles",
                        island.name
                    );
                }
            }
        }
    }

    #[test]
    fn yz_vertical_ring_blocks_follow_the_ring_tangent() {
        let segments = 12;
        let mut mesh = HeroMesh::default();
        mesh.vertical_ring_blocks(Vec3::ZERO, 4.0, 0.2, segments, false);

        for (segment, block_positions) in mesh.positions.chunks_exact(24).enumerate() {
            let angle = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let tangent = Vec3::new(0.0, angle.cos(), -angle.sin());
            let radial = Vec3::new(0.0, angle.sin(), angle.cos());
            let tangent_range = projected_range(block_positions, tangent);
            let radial_range = projected_range(block_positions, radial);
            let x_range = projected_range(block_positions, Vec3::X);

            assert!(
                tangent_range > radial_range * 2.0 && tangent_range > x_range * 2.0,
                "segment {segment} should be longest along its YZ tangent; \
                 tangent={tangent_range:.3}, radial={radial_range:.3}, x={x_range:.3}"
            );
        }
    }
}
