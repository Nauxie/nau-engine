use super::{
    super::{island_playable_normalized_offset, island_visual_surface_position, random_unit},
    shared::{append_double_sided_detail_card, append_ellipsoid_lobe},
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::{
    IslandFloraIdentity, IslandScaleClass, IslandTraversalPurpose, SkyIsland,
    authored_island_art_direction, authored_island_composition,
};

const GOLDEN_ANGLE_RADIANS: f32 = 2.399_963_1;
const FLORA_SURFACE_OFFSET_M: f32 = 0.045;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FloraVisualKind {
    FernGrove,
    FlowerThicket,
    ReedBed,
    WindShrub,
    BroadleafPatch,
    MushroomRing,
}

impl FloraVisualKind {
    #[cfg(test)]
    const ALL: [Self; 6] = [
        Self::FernGrove,
        Self::FlowerThicket,
        Self::ReedBed,
        Self::WindShrub,
        Self::BroadleafPatch,
        Self::MushroomRing,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::FernGrove => "fern_grove",
            Self::FlowerThicket => "flower_thicket",
            Self::ReedBed => "reed_bed",
            Self::WindShrub => "wind_shrub",
            Self::BroadleafPatch => "broadleaf_patch",
            Self::MushroomRing => "mushroom_ring",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::FernGrove => "fern grove",
            Self::FlowerThicket => "flower thicket",
            Self::ReedBed => "reed bed",
            Self::WindShrub => "wind-shaped shrub",
            Self::BroadleafPatch => "broadleaf patch",
            Self::MushroomRing => "mushroom ring",
        }
    }

    fn material(self) -> FloraMaterialRole {
        match self {
            Self::FernGrove | Self::BroadleafPatch | Self::MushroomRing => {
                FloraMaterialRole::GroundCover
            }
            Self::ReedBed | Self::WindShrub => FloraMaterialRole::Foliage,
            Self::FlowerThicket => FloraMaterialRole::Flower,
        }
    }

    fn index(self) -> usize {
        match self {
            Self::FernGrove => 0,
            Self::FlowerThicket => 1,
            Self::ReedBed => 2,
            Self::WindShrub => 3,
            Self::BroadleafPatch => 4,
            Self::MushroomRing => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloraMaterialRole {
    Foliage,
    GroundCover,
    Flower,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct IslandFloraVisualSpec {
    pub(crate) kind: FloraVisualKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) material: FloraMaterialRole,
    pub(crate) wind_phase: f32,
    pub(crate) wind_motion_scale: f32,
    radius_m: f32,
    height_m: f32,
    plant_count: usize,
    seed: u32,
}

impl IslandFloraVisualSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        flora_cluster_mesh(
            self.kind,
            self.radius_m,
            self.height_m,
            self.plant_count,
            self.seed,
        )
    }

    pub(crate) fn semantic_sample_world_position(self) -> Vec3 {
        self.translation
            + Quat::from_rotation_y(self.rotation_y)
                * flora_semantic_sample_local_position(
                    self.kind,
                    self.radius_m,
                    self.height_m,
                    self.plant_count,
                    self.seed,
                )
    }
}

pub(crate) fn island_flora_visual_specs(
    island_index: usize,
    island: SkyIsland,
) -> Vec<IslandFloraVisualSpec> {
    let Some(art_direction) = authored_island_art_direction(island.name) else {
        return Vec::new();
    };
    let composition = authored_island_composition(island.name);
    let cluster_count = usize::from(art_direction.flora_count);
    let base_seed =
        mixed_seed(flora_seed(island_index, island, 0x6d2b_79f5) ^ art_direction.signature_seed);
    let authored_anchor = Vec2::from_array(art_direction.flora_anchor);

    art_direction
        .flora_kinds
        .into_iter()
        .take(cluster_count)
        .map(flora_visual_kind)
        .enumerate()
        .map(|(cluster_index, kind)| {
            let sample = cluster_index as u32;
            let seed = mixed_seed(
                base_seed
                    ^ sample.wrapping_mul(0x9e37_79b9)
                    ^ (kind.index() as u32).wrapping_mul(0x85eb_ca6b),
            );
            let (radius_m, height_m, plant_count) =
                flora_cluster_dimensions(kind, island.world_tags.scale_class, island);
            let normalized_offset = flora_cluster_offset(
                island,
                composition.map(|value| value.traversal_purpose),
                authored_anchor,
                cluster_index,
                cluster_count,
                seed,
                radius_m,
            );
            let translation = island_visual_surface_position(island, normalized_offset)
                + Vec3::Y * FLORA_SURFACE_OFFSET_M;
            let rotation_y =
                random_unit(seed, sample, 131) * std::f32::consts::TAU - std::f32::consts::PI;
            let scale_motion = match island.world_tags.scale_class {
                IslandScaleClass::Tiny | IslandScaleClass::Small => 0.90,
                IslandScaleClass::Medium | IslandScaleClass::Large => 1.0,
                IslandScaleClass::Vast => 1.08,
                IslandScaleClass::HugePlateau => 1.15,
            };

            IslandFloraVisualSpec {
                kind,
                label: kind.visual_name(),
                translation,
                rotation_y,
                material: kind.material(),
                wind_phase: random_unit(seed, sample, 149) * std::f32::consts::TAU,
                wind_motion_scale: kind.wind_response()
                    * scale_motion
                    * (0.88 + random_unit(seed, sample, 157) * 0.24),
                radius_m,
                height_m,
                plant_count,
                seed,
            }
        })
        .collect()
}

impl FloraVisualKind {
    fn wind_response(self) -> f32 {
        match self {
            Self::FernGrove => 1.18,
            Self::FlowerThicket => 0.92,
            Self::ReedBed => 1.36,
            Self::WindShrub => 1.08,
            Self::BroadleafPatch => 1.16,
            Self::MushroomRing => 0.28,
        }
    }
}

fn flora_visual_kind(identity: IslandFloraIdentity) -> FloraVisualKind {
    match identity {
        IslandFloraIdentity::FernGrove => FloraVisualKind::FernGrove,
        IslandFloraIdentity::FlowerThicket => FloraVisualKind::FlowerThicket,
        IslandFloraIdentity::ReedBed => FloraVisualKind::ReedBed,
        IslandFloraIdentity::WindShrub => FloraVisualKind::WindShrub,
        IslandFloraIdentity::BroadleafPatch => FloraVisualKind::BroadleafPatch,
        IslandFloraIdentity::MushroomRing => FloraVisualKind::MushroomRing,
    }
}

fn flora_cluster_dimensions(
    kind: FloraVisualKind,
    scale: IslandScaleClass,
    island: SkyIsland,
) -> (f32, f32, usize) {
    let (base_radius, base_height, base_plants): (f32, f32, usize) = match scale {
        IslandScaleClass::Tiny => (1.45, 1.20, 8),
        IslandScaleClass::Small => (1.85, 1.45, 10),
        IslandScaleClass::Medium => (2.45, 1.75, 12),
        IslandScaleClass::Large => (3.15, 2.05, 14),
        IslandScaleClass::Vast => (4.25, 2.45, 16),
        IslandScaleClass::HugePlateau => (12.0, 4.20, 32),
    };
    let (radius_scale, height_scale, plant_adjustment): (f32, f32, usize) = match kind {
        FloraVisualKind::FernGrove => (1.00, 0.88, 0),
        FloraVisualKind::FlowerThicket => (0.92, 0.94, 2),
        FloraVisualKind::ReedBed => (1.08, 1.28, 4),
        FloraVisualKind::WindShrub => (1.00, 1.10, 0),
        FloraVisualKind::BroadleafPatch => (1.06, 0.96, 0),
        FloraVisualKind::MushroomRing => (0.88, 0.72, 0),
    };
    let footprint_cap = island.half_extents.min_element() * 0.17;
    let thickness_scale = (island.thickness / 14.0).sqrt().clamp(0.82, 1.28);

    (
        (base_radius * radius_scale).min(footprint_cap).max(1.15),
        (base_height * height_scale * thickness_scale).clamp(0.82, 4.80),
        base_plants + plant_adjustment,
    )
}

fn flora_semantic_sample_local_position(
    kind: FloraVisualKind,
    radius: f32,
    height: f32,
    plant_count: usize,
    seed: u32,
) -> Vec3 {
    match kind {
        FloraVisualKind::FernGrove => {
            let base = clustered_point(seed, 0, plant_count, radius * 0.86, 0.08, 311);
            let plant_height = height * (0.72 + random_unit(seed, 0, 313) * 0.34);
            let phase = random_unit(seed, 0, 317) * 0.54;
            let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
            let frond_scale = 0.82 + random_unit(seed, 0, 331) * 0.30;
            let t = 0.31;
            base + Vec3::Y * plant_height * (0.22 + t * 0.20)
                + outward * plant_height * (0.10 + t * 0.22) * frond_scale
        }
        FloraVisualKind::FlowerThicket => {
            let base = clustered_point(seed, 0, plant_count, radius * 0.88, 0.03, 401);
            let stem_height = height * (0.66 + random_unit(seed, 0, 409) * 0.36);
            let lean_phase = random_unit(seed, 0, 419) * std::f32::consts::TAU;
            let lean = Vec3::new(lean_phase.cos(), 0.0, lean_phase.sin())
                * stem_height
                * (0.025 + random_unit(seed, 0, 421) * 0.035);
            let flower_center = base + Vec3::Y * stem_height + lean;
            let head_radius = stem_height * (0.13 + random_unit(seed, 0, 431) * 0.045);
            flower_center + Vec3::Y * head_radius * 0.05
        }
        FloraVisualKind::ReedBed => {
            let reed_count = plant_count * 2;
            let wind_phase = random_unit(seed, 0, 503) * std::f32::consts::TAU;
            let wind_direction = Vec3::new(wind_phase.cos(), 0.0, wind_phase.sin());
            let mut base = clustered_point(seed, 0, reed_count, radius * 0.92, 0.02, 509);
            base.z *= 0.72;
            let reed_height = height * (0.58 + random_unit(seed, 0, 521) * 0.46);
            let lean = wind_direction * reed_height * (0.035 + random_unit(seed, 0, 523) * 0.045);
            base + Vec3::Y * reed_height * 0.925 + lean
        }
        FloraVisualKind::WindShrub => {
            let shrub_count = (plant_count / 2).max(5);
            let wind_phase = random_unit(seed, 0, 601) * std::f32::consts::TAU;
            let wind_direction = Vec3::new(wind_phase.cos(), 0.0, wind_phase.sin());
            let base = clustered_point(seed, 0, shrub_count, radius * 0.84, 0.04, 607);
            let shrub_height = height * (0.68 + random_unit(seed, 0, 613) * 0.34);
            base + Vec3::Y * shrub_height * 0.64 + wind_direction * shrub_height * 0.15
        }
        FloraVisualKind::BroadleafPatch => {
            let base = clustered_point(seed, 0, plant_count, radius * 0.90, 0.02, 701);
            let plant_height = height * (0.62 + random_unit(seed, 0, 709) * 0.38);
            base + Vec3::Y * plant_height * 0.66
        }
        FloraVisualKind::MushroomRing => {
            let ring_phase = random_unit(seed, 0, 801) * std::f32::consts::TAU;
            let phase = ring_phase + (random_unit(seed, 0, 809) - 0.5) * 0.18;
            let ring_radius = radius * (0.56 + random_unit(seed, 0, 811) * 0.27);
            let stem_height = height * (0.42 + random_unit(seed, 0, 821) * 0.42);
            Vec3::new(
                phase.cos() * ring_radius,
                stem_height,
                phase.sin() * ring_radius,
            )
        }
    }
}

fn flora_cluster_offset(
    island: SkyIsland,
    traversal_purpose: Option<IslandTraversalPurpose>,
    authored_anchor: Vec2,
    cluster_index: usize,
    cluster_count: usize,
    seed: u32,
    cluster_radius_m: f32,
) -> Vec2 {
    let lane_critical = island.is_target
        || matches!(
            traversal_purpose,
            Some(
                IslandTraversalPurpose::LaunchHub
                    | IslandTraversalPurpose::LandingTarget
                    | IslandTraversalPurpose::PlateauApproach
                    | IslandTraversalPurpose::PlateauHub
            )
        );
    let direction = if authored_anchor.length_squared() > 0.0001 {
        authored_anchor.normalize()
    } else {
        let angle = random_unit(seed, 0, 211) * std::f32::consts::TAU;
        Vec2::new(angle.cos(), angle.sin())
    };
    let tangent = Vec2::new(-direction.y, direction.x);
    let centered_index = cluster_index as f32 - cluster_count.saturating_sub(1) as f32 * 0.5;
    let mut candidate = authored_anchor
        + tangent * centered_index * 0.15
        + direction * ((random_unit(seed, cluster_index as u32, 223) - 0.5) * 0.055)
        + tangent * ((random_unit(seed, cluster_index as u32, 227) - 0.5) * 0.035);

    if lane_critical && candidate.x > -0.38 && candidate.x < 0.38 && candidate.y.abs() < 0.25 {
        let lane_side = if authored_anchor.y.abs() > 0.01 {
            authored_anchor.y.signum()
        } else if random_unit(seed, cluster_index as u32, 229) > 0.5 {
            1.0
        } else {
            -1.0
        };
        candidate.y = lane_side * (0.28 + cluster_index as f32 * 0.025);
    }

    inset_playable_offset(island, candidate, cluster_radius_m)
}

fn inset_playable_offset(island: SkyIsland, candidate: Vec2, cluster_radius_m: f32) -> Vec2 {
    if candidate.length_squared() <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let direction = candidate.normalize();
    let angle = direction.y.atan2(direction.x);
    let footprint_padding = cluster_radius_m / island.half_extents.min_element().max(1.0);
    let max_center_radius =
        (island.playable_silhouette_scale(angle) * 0.92 - footprint_padding).max(0.12);
    let inset = direction * candidate.length().min(max_center_radius);
    island_playable_normalized_offset(island, inset)
}

fn flora_cluster_mesh(
    kind: FloraVisualKind,
    radius_m: f32,
    height_m: f32,
    plant_count: usize,
    seed: u32,
) -> Mesh {
    let mut mesh = FloraMeshBuffers::default();
    match kind {
        FloraVisualKind::FernGrove => {
            append_fern_grove(&mut mesh, radius_m, height_m, plant_count, seed)
        }
        FloraVisualKind::FlowerThicket => {
            append_flower_thicket(&mut mesh, radius_m, height_m, plant_count, seed)
        }
        FloraVisualKind::ReedBed => {
            append_reed_bed(&mut mesh, radius_m, height_m, plant_count, seed)
        }
        FloraVisualKind::WindShrub => {
            append_wind_shrubs(&mut mesh, radius_m, height_m, plant_count, seed)
        }
        FloraVisualKind::BroadleafPatch => {
            append_broadleaf_patch(&mut mesh, radius_m, height_m, plant_count, seed)
        }
        FloraVisualKind::MushroomRing => {
            append_mushroom_ring(&mut mesh, radius_m, height_m, plant_count, seed)
        }
    }
    mesh.finish()
}

#[derive(Default)]
struct FloraMeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl FloraMeshBuffers {
    fn card(&mut self, center: Vec3, tangent: Vec3, up: Vec3, half_width: f32, half_height: f32) {
        append_double_sided_detail_card(
            &mut self.positions,
            &mut self.normals,
            &mut self.uvs,
            &mut self.indices,
            center,
            tangent,
            up,
            half_width,
            half_height,
        );
    }

    fn lobe(
        &mut self,
        center: Vec3,
        radii: Vec3,
        latitude_segments: usize,
        longitude_segments: usize,
        seed: u32,
        noise_strength: f32,
    ) {
        append_ellipsoid_lobe(
            &mut self.positions,
            &mut self.normals,
            &mut self.uvs,
            &mut self.indices,
            center,
            radii,
            latitude_segments,
            longitude_segments,
            seed,
            noise_strength,
        );
    }

    fn stem(&mut self, start: Vec3, end: Vec3, base_radius: f32, tip_radius: f32, segments: usize) {
        append_tapered_stem(
            &mut self.positions,
            &mut self.normals,
            &mut self.uvs,
            &mut self.indices,
            start,
            end,
            base_radius,
            tip_radius,
            segments,
        );
    }

    fn finish(self) -> Mesh {
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

fn append_fern_grove(
    mesh: &mut FloraMeshBuffers,
    radius: f32,
    height: f32,
    plant_count: usize,
    seed: u32,
) {
    for plant in 0..plant_count {
        let sample = plant as u32;
        let base = clustered_point(seed, plant, plant_count, radius * 0.86, 0.08, 311);
        let plant_height = height * (0.72 + random_unit(seed, sample, 313) * 0.34);
        mesh.stem(
            base,
            base + Vec3::Y * plant_height * 0.54,
            (plant_height * 0.020).clamp(0.025, 0.075),
            (plant_height * 0.006).clamp(0.009, 0.025),
            5,
        );

        for frond in 0..4 {
            let phase = frond as f32 / 4.0 * std::f32::consts::TAU
                + random_unit(seed, sample * 7 + frond, 317) * 0.54;
            let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
            let tangent = Vec3::new(-outward.z, 0.0, outward.x);
            let frond_scale = 0.82 + random_unit(seed, sample * 7 + frond, 331) * 0.30;

            for segment in 0..2 {
                let t = (segment as f32 + 0.62) / 2.0;
                let frond_up =
                    (Vec3::Y * (0.78 - t * 0.12) + outward * (0.62 + t * 0.18)).normalize();
                mesh.card(
                    base + Vec3::Y * plant_height * (0.22 + t * 0.20)
                        + outward * plant_height * (0.10 + t * 0.22) * frond_scale,
                    tangent,
                    frond_up,
                    plant_height * (0.095 - t * 0.018) * frond_scale,
                    plant_height * (0.19 - t * 0.030) * frond_scale,
                );
            }
        }
    }
}

fn append_flower_thicket(
    mesh: &mut FloraMeshBuffers,
    radius: f32,
    height: f32,
    plant_count: usize,
    seed: u32,
) {
    for plant in 0..plant_count {
        let sample = plant as u32;
        let base = clustered_point(seed, plant, plant_count, radius * 0.88, 0.03, 401);
        let stem_height = height * (0.66 + random_unit(seed, sample, 409) * 0.36);
        let lean_phase = random_unit(seed, sample, 419) * std::f32::consts::TAU;
        let lean = Vec3::new(lean_phase.cos(), 0.0, lean_phase.sin())
            * stem_height
            * (0.025 + random_unit(seed, sample, 421) * 0.035);
        let flower_center = base + Vec3::Y * stem_height + lean;
        mesh.stem(
            base,
            flower_center,
            (stem_height * 0.018).clamp(0.022, 0.070),
            (stem_height * 0.008).clamp(0.010, 0.030),
            5,
        );

        for leaf in 0..2 {
            let phase = lean_phase + leaf as f32 * std::f32::consts::PI + 0.45;
            let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
            mesh.card(
                base + Vec3::Y * stem_height * (0.34 + leaf as f32 * 0.20)
                    + outward * stem_height * 0.08,
                Vec3::new(-outward.z, 0.0, outward.x),
                (Vec3::Y * 0.72 + outward * 0.54).normalize(),
                stem_height * 0.075,
                stem_height * 0.18,
            );
        }

        let head_radius = stem_height * (0.13 + random_unit(seed, sample, 431) * 0.045);
        for petal in 0..5 {
            let phase =
                petal as f32 / 5.0 * std::f32::consts::TAU + random_unit(seed, sample, 433) * 0.24;
            let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
            mesh.card(
                flower_center + outward * head_radius * 0.54 + Vec3::Y * head_radius * 0.04,
                Vec3::new(-outward.z, 0.0, outward.x),
                (outward + Vec3::Y * 0.18).normalize(),
                head_radius * 0.34,
                head_radius * 0.62,
            );
        }
        mesh.lobe(
            flower_center + Vec3::Y * head_radius * 0.05,
            Vec3::new(head_radius * 0.28, head_radius * 0.20, head_radius * 0.28),
            3,
            5,
            seed.wrapping_add(sample * 17 + 439),
            0.10,
        );
    }
}

fn append_reed_bed(
    mesh: &mut FloraMeshBuffers,
    radius: f32,
    height: f32,
    plant_count: usize,
    seed: u32,
) {
    let reed_count = plant_count * 2;
    let wind_phase = random_unit(seed, 0, 503) * std::f32::consts::TAU;
    let wind_direction = Vec3::new(wind_phase.cos(), 0.0, wind_phase.sin());

    for reed in 0..reed_count {
        let sample = reed as u32;
        let mut base = clustered_point(seed, reed, reed_count, radius * 0.92, 0.02, 509);
        base.z *= 0.72;
        let reed_height = height * (0.58 + random_unit(seed, sample, 521) * 0.46);
        let lean = wind_direction * reed_height * (0.035 + random_unit(seed, sample, 523) * 0.045);
        let tip = base + Vec3::Y * reed_height + lean;
        mesh.stem(
            base,
            tip,
            (reed_height * 0.014).clamp(0.018, 0.055),
            (reed_height * 0.006).clamp(0.008, 0.022),
            4,
        );

        for leaf in 0..2 {
            let leaf_phase =
                wind_phase + std::f32::consts::FRAC_PI_2 + leaf as f32 * std::f32::consts::PI;
            let outward = Vec3::new(leaf_phase.cos(), 0.0, leaf_phase.sin());
            mesh.card(
                base + Vec3::Y * reed_height * (0.24 + leaf as f32 * 0.18)
                    + outward * reed_height * 0.035,
                Vec3::new(-outward.z, 0.0, outward.x),
                (Vec3::Y * 0.92 + outward * 0.28).normalize(),
                reed_height * 0.026,
                reed_height * (0.18 - leaf as f32 * 0.025),
            );
        }

        if reed.is_multiple_of(3) {
            mesh.lobe(
                tip - Vec3::Y * reed_height * 0.075,
                Vec3::new(
                    reed_height * 0.026,
                    reed_height * 0.090,
                    reed_height * 0.026,
                ),
                3,
                6,
                seed.wrapping_add(sample * 19 + 541),
                0.08,
            );
        }
    }
}

fn append_wind_shrubs(
    mesh: &mut FloraMeshBuffers,
    radius: f32,
    height: f32,
    plant_count: usize,
    seed: u32,
) {
    let shrub_count = (plant_count / 2).max(5);
    let wind_phase = random_unit(seed, 0, 601) * std::f32::consts::TAU;
    let wind_direction = Vec3::new(wind_phase.cos(), 0.0, wind_phase.sin());

    for shrub in 0..shrub_count {
        let sample = shrub as u32;
        let base = clustered_point(seed, shrub, shrub_count, radius * 0.84, 0.04, 607);
        let shrub_height = height * (0.68 + random_unit(seed, sample, 613) * 0.34);

        for branch in 0..4 {
            let branch_sample = sample * 11 + branch;
            let phase = wind_phase
                + (branch as f32 - 1.5) * 0.72
                + (random_unit(seed, branch_sample, 617) - 0.5) * 0.34;
            let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
            let branch_height =
                shrub_height * (0.58 + random_unit(seed, branch_sample, 619) * 0.34);
            let end = base
                + Vec3::Y * branch_height
                + outward * shrub_height * (0.18 + branch as f32 * 0.025)
                + wind_direction * shrub_height * 0.16;
            mesh.stem(
                base,
                end,
                (shrub_height * 0.026).clamp(0.032, 0.095),
                (shrub_height * 0.008).clamp(0.010, 0.030),
                5,
            );

            let tangent = Vec3::new(-outward.z, 0.0, outward.x);
            for leaf in 0usize..4 {
                let t = 0.28 + leaf as f32 * 0.18;
                let side = if leaf.is_multiple_of(2) { 1.0 } else { -1.0 };
                mesh.card(
                    base.lerp(end, t) + tangent * side * shrub_height * 0.055,
                    tangent,
                    (Vec3::Y * 0.68 + outward * 0.64 + wind_direction * 0.18).normalize(),
                    shrub_height * (0.070 - leaf as f32 * 0.006),
                    shrub_height * (0.13 - leaf as f32 * 0.008),
                );
            }
        }

        mesh.lobe(
            base + Vec3::Y * shrub_height * 0.64 + wind_direction * shrub_height * 0.15,
            Vec3::new(
                shrub_height * 0.24,
                shrub_height * 0.22,
                shrub_height * 0.18,
            ),
            4,
            6,
            seed.wrapping_add(sample * 23 + 631),
            0.20,
        );
    }
}

fn append_broadleaf_patch(
    mesh: &mut FloraMeshBuffers,
    radius: f32,
    height: f32,
    plant_count: usize,
    seed: u32,
) {
    for plant in 0..plant_count {
        let sample = plant as u32;
        let base = clustered_point(seed, plant, plant_count, radius * 0.90, 0.02, 701);
        let plant_height = height * (0.62 + random_unit(seed, sample, 709) * 0.38);
        let crown = base + Vec3::Y * plant_height * 0.66;
        mesh.stem(
            base,
            crown,
            (plant_height * 0.024).clamp(0.028, 0.085),
            (plant_height * 0.008).clamp(0.010, 0.028),
            5,
        );

        for leaf in 0..6 {
            let phase =
                leaf as f32 / 6.0 * std::f32::consts::TAU + random_unit(seed, sample, 719) * 0.38;
            let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
            let upper = leaf >= 3;
            let leaf_center = base
                + Vec3::Y * plant_height * if upper { 0.60 } else { 0.43 }
                + outward
                    * plant_height
                    * if upper { 0.15 } else { 0.20 }
                    * (0.88 + random_unit(seed, sample * 7 + leaf, 727) * 0.24);
            mesh.card(
                leaf_center,
                Vec3::new(-outward.z, 0.0, outward.x),
                (outward * 0.78 + Vec3::Y * if upper { 0.52 } else { 0.34 }).normalize(),
                plant_height * if upper { 0.11 } else { 0.13 },
                plant_height * if upper { 0.21 } else { 0.24 },
            );
        }

        if plant.is_multiple_of(2) {
            mesh.lobe(
                crown,
                Vec3::new(
                    plant_height * 0.17,
                    plant_height * 0.13,
                    plant_height * 0.15,
                ),
                3,
                6,
                seed.wrapping_add(sample * 29 + 733),
                0.16,
            );
        }
    }
}

fn append_mushroom_ring(
    mesh: &mut FloraMeshBuffers,
    radius: f32,
    height: f32,
    plant_count: usize,
    seed: u32,
) {
    let ring_phase = random_unit(seed, 0, 801) * std::f32::consts::TAU;
    for mushroom in 0..plant_count {
        let sample = mushroom as u32;
        let phase = ring_phase
            + mushroom as f32 / plant_count.max(1) as f32 * std::f32::consts::TAU
            + (random_unit(seed, sample, 809) - 0.5) * 0.18;
        let ring_radius = radius * (0.56 + random_unit(seed, sample, 811) * 0.27);
        let base = Vec3::new(phase.cos() * ring_radius, 0.0, phase.sin() * ring_radius);
        let stem_height = height * (0.42 + random_unit(seed, sample, 821) * 0.42);
        let cap_radius = stem_height * (0.20 + random_unit(seed, sample, 823) * 0.08);
        let cap_center = base + Vec3::Y * stem_height;

        mesh.stem(base, cap_center, cap_radius * 0.25, cap_radius * 0.16, 5);
        mesh.lobe(
            cap_center,
            Vec3::new(cap_radius, cap_radius * 0.40, cap_radius * 0.92),
            4,
            8,
            seed.wrapping_add(sample * 31 + 827),
            0.14,
        );

        for gill in 0..4 {
            let gill_phase = phase + gill as f32 / 4.0 * std::f32::consts::TAU;
            let outward = Vec3::new(gill_phase.cos(), 0.0, gill_phase.sin());
            mesh.card(
                cap_center - Vec3::Y * cap_radius * 0.20 + outward * cap_radius * 0.25,
                Vec3::new(-outward.z, 0.0, outward.x),
                outward,
                cap_radius * 0.10,
                cap_radius * 0.38,
            );
        }
    }
}

fn clustered_point(
    seed: u32,
    index: usize,
    count: usize,
    radius: f32,
    inner_radius_fraction: f32,
    salt: u32,
) -> Vec3 {
    let sample = index as u32;
    let phase = random_unit(seed, 0, salt) * std::f32::consts::TAU;
    let angle = phase
        + index as f32 * GOLDEN_ANGLE_RADIANS
        + (random_unit(seed, sample, salt + 2) - 0.5) * 0.32;
    let sequence_radius = ((index as f32 + 0.5) / count.max(1) as f32).sqrt();
    let radius_mix = sequence_radius * 0.46 + random_unit(seed, sample, salt + 4).sqrt() * 0.54;
    let distance = radius * (inner_radius_fraction + (1.0 - inner_radius_fraction) * radius_mix);
    Vec3::new(angle.cos() * distance, 0.0, angle.sin() * distance)
}

#[allow(clippy::too_many_arguments)]
fn append_tapered_stem(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    start: Vec3,
    end: Vec3,
    base_radius: f32,
    tip_radius: f32,
    radial_segments: usize,
) {
    let axis = (end - start).normalize_or_zero();
    if axis.length_squared() <= 0.0001 || radial_segments < 3 {
        return;
    }
    let side_seed = if axis.dot(Vec3::Y).abs() > 0.92 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let side = axis.cross(side_seed).normalize();
    let bitangent = side.cross(axis).normalize();
    let first = positions.len() as u32;

    for (ring, (center, radius)) in [(start, base_radius), (end, tip_radius)]
        .into_iter()
        .enumerate()
    {
        for segment in 0..radial_segments {
            let phase = segment as f32 / radial_segments as f32 * std::f32::consts::TAU;
            let radial = side * phase.cos() + bitangent * phase.sin();
            positions.push((center + radial * radius).to_array());
            normals.push(radial.to_array());
            uvs.push([segment as f32 / radial_segments as f32, ring as f32]);
        }
    }

    for segment in 0..radial_segments {
        let a = first + segment as u32;
        let b = first + ((segment + 1) % radial_segments) as u32;
        let c = first + radial_segments as u32 + segment as u32;
        let d = first + radial_segments as u32 + ((segment + 1) % radial_segments) as u32;
        indices.extend([a, c, b, b, c, d]);
    }
}

fn flora_seed(island_index: usize, island: SkyIsland, salt: u32) -> u32 {
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
            _ => panic!("flora mesh should expose Float32x3 positions"),
        }
    }

    fn u32_indices(mesh: &Mesh) -> &[u32] {
        match mesh.indices() {
            Some(Indices::U32(values)) => values,
            _ => panic!("flora mesh should expose U32 indices"),
        }
    }

    fn axis_range(values: &[[f32; 3]], axis: usize) -> f32 {
        let (min, max) = values.iter().fold(
            (f32::INFINITY, f32::NEG_INFINITY),
            |(min, max), position| (min.min(position[axis]), max.max(position[axis])),
        );
        max - min
    }

    fn normalized_offset(island: SkyIsland, position: Vec3) -> Vec2 {
        Vec2::new(
            (position.x - island.center.x) / island.half_extents.x,
            (position.z - island.center.z) / island.half_extents.y,
        )
    }

    #[test]
    fn flora_specs_and_meshes_are_deterministic() {
        let route = SkyRoute::default();
        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let first = island_flora_visual_specs(island_index, island);
            let second = island_flora_visual_specs(island_index, island);

            assert_eq!(first, second, "{} should be deterministic", island.name);
            for (first_spec, second_spec) in first.into_iter().zip(second) {
                let first_mesh = first_spec.build_mesh();
                let second_mesh = second_spec.build_mesh();
                assert_eq!(positions(&first_mesh), positions(&second_mesh));
                assert_eq!(u32_indices(&first_mesh), u32_indices(&second_mesh));
            }
        }
    }

    #[test]
    fn semantic_samples_track_generated_flora_geometry() {
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_flora_visual_specs(island_index, island) {
                let mesh = spec.build_mesh();
                let local_sample = Quat::from_rotation_y(spec.rotation_y).inverse()
                    * (spec.semantic_sample_world_position() - spec.translation);
                let nearest_vertex_distance = positions(&mesh)
                    .iter()
                    .map(|position| Vec3::from_array(*position).distance(local_sample))
                    .fold(f32::INFINITY, f32::min);
                let maximum_distance = (spec.height_m * 0.35).max(0.20);

                assert!(
                    nearest_vertex_distance <= maximum_distance,
                    "{} {} semantic sample should remain anchored to generated geometry; \
                     nearest={nearest_vertex_distance:.3}m maximum={maximum_distance:.3}m",
                    island.name,
                    spec.kind.label()
                );
            }
        }
    }

    #[test]
    fn route_flora_covers_every_visual_family_and_every_island() {
        let route = SkyRoute::default();
        let mut kind_counts = [0usize; FloraVisualKind::ALL.len()];

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let specs = island_flora_visual_specs(island_index, island);
            assert!(!specs.is_empty(), "{} should receive flora", island.name);
            for spec in specs {
                kind_counts[spec.kind.index()] += 1;
                assert_eq!(spec.label, spec.kind.visual_name());
                assert!(!spec.kind.label().is_empty());
                assert!(
                    spec.build_mesh().count_vertices() >= 60,
                    "{} {} should clear the runtime landmark vertex floor",
                    island.name,
                    spec.kind.label()
                );
            }
        }

        for kind in FloraVisualKind::ALL {
            assert!(
                kind_counts[kind.index()] > 0,
                "{} should appear on the authored route",
                kind.label()
            );
        }
    }

    #[test]
    fn every_authored_flora_profile_is_realized_exactly() {
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let profile = authored_island_art_direction(island.name)
                .expect("every route island should have authored art direction");
            let specs = island_flora_visual_specs(island_index, island);
            assert_eq!(
                specs.len(),
                usize::from(profile.flora_count),
                "unexpected count for {}",
                island.name
            );
            assert!(!specs.is_empty(), "{} needs authored ecology", island.name);
            let expected_kinds = profile
                .flora_kinds
                .into_iter()
                .take(usize::from(profile.flora_count))
                .map(flora_visual_kind)
                .collect::<Vec<_>>();
            assert_eq!(
                specs.iter().map(|spec| spec.kind).collect::<Vec<_>>(),
                expected_kinds,
                "{} should realize its authored flora kinds in order",
                island.name
            );

            let anchor = Vec2::from_array(profile.flora_anchor).normalize();
            let mean_offset = specs
                .iter()
                .map(|spec| normalized_offset(island, spec.translation))
                .sum::<Vec2>()
                / specs.len() as f32;
            assert!(
                mean_offset.normalize().dot(anchor) > 0.70,
                "{} flora should remain driven by its authored anchor",
                island.name
            );
        }
    }

    #[test]
    fn flora_origins_and_cluster_footprints_stay_playable_and_clear_major_lanes() {
        let route = SkyRoute::default();

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            let composition = authored_island_composition(island.name);
            let lane_critical = island.is_target
                || matches!(
                    composition.map(|value| value.traversal_purpose),
                    Some(
                        IslandTraversalPurpose::LaunchHub
                            | IslandTraversalPurpose::LandingTarget
                            | IslandTraversalPurpose::PlateauApproach
                            | IslandTraversalPurpose::PlateauHub
                    )
                );

            for spec in island_flora_visual_specs(island_index, island) {
                let offset = normalized_offset(island, spec.translation);
                let angle = offset.y.atan2(offset.x);
                let playable_radius = island.playable_silhouette_scale(angle);
                let footprint_padding = spec.radius_m / island.half_extents.min_element().max(1.0);
                assert!(
                    offset.length() + footprint_padding <= playable_radius * 0.93 + 0.000_1,
                    "{} {} should remain inside the playable footprint",
                    island.name,
                    spec.kind.label()
                );
                assert!(
                    (spec.translation.y
                        - island.mesh_top_y_at(spec.translation)
                        - FLORA_SURFACE_OFFSET_M)
                        .abs()
                        < 0.001
                );

                if lane_critical {
                    assert!(
                        !(offset.x > -0.38 && offset.x < 0.38 && offset.y.abs() < 0.25),
                        "{} {} should not block its central route lane",
                        island.name,
                        spec.kind.label()
                    );
                }
            }
        }
    }

    #[test]
    fn every_flora_family_builds_a_dense_varied_cluster_mesh() {
        let route = SkyRoute::default();
        let mut representative_specs = [None; FloraVisualKind::ALL.len()];

        for (island_index, island) in route.islands().iter().copied().enumerate() {
            for spec in island_flora_visual_specs(island_index, island) {
                representative_specs[spec.kind.index()].get_or_insert(spec);
            }
        }

        let mut aspect_ratios = Vec::new();
        let mut vertex_counts = Vec::new();
        for kind in FloraVisualKind::ALL {
            let spec = representative_specs[kind.index()]
                .expect("each flora family should have a route representative");
            let mesh = spec.build_mesh();
            let mesh_positions = positions(&mesh);
            let x_span = axis_range(mesh_positions, 0);
            let y_span = axis_range(mesh_positions, 1);
            let z_span = axis_range(mesh_positions, 2);
            let horizontal_span = x_span.max(z_span);

            assert!(
                (300..=3_000).contains(&mesh.count_vertices()),
                "{} should batch many detailed plants at a bounded cost; vertices={}",
                kind.label(),
                mesh.count_vertices()
            );
            assert!(
                horizontal_span > 1.5,
                "{} should read as a cluster",
                kind.label()
            );
            assert!(
                y_span > 0.55,
                "{} should have vertical silhouette",
                kind.label()
            );
            assert!(
                mesh_positions
                    .iter()
                    .flatten()
                    .all(|value| value.is_finite())
            );
            assert!(!u32_indices(&mesh).is_empty());
            assert!(u32_indices(&mesh).len().is_multiple_of(3));

            aspect_ratios.push(y_span / horizontal_span.max(0.001));
            vertex_counts.push(mesh.count_vertices());
        }

        let min_aspect = aspect_ratios.iter().copied().fold(f32::INFINITY, f32::min);
        let max_aspect = aspect_ratios
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        vertex_counts.sort_unstable();
        vertex_counts.dedup();
        assert!(
            max_aspect - min_aspect > 0.20,
            "flora families should not collapse to one silhouette"
        );
        assert!(
            vertex_counts.len() >= 4,
            "flora families should use meaningfully different geometry"
        );
    }
}
