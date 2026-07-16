use super::super::random_unit;
use super::landmarks::{IslandWaterVisualKind, IslandWaterVisualMesh};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use nau_engine::world::SkyIsland;

const MAX_WATER_DETAILS_PER_ISLAND: usize = 12;
const WATER_DETAIL_MIN_VERTICES: usize = 60;
const LILY_PAD_SEGMENTS: usize = 10;
const SHORE_REED_COUNT: usize = 24;
const CHANNEL_REED_COUNT: usize = 18;
const CHANNEL_COBBLE_COUNT: usize = 16;
const WATERFALL_LIP_ROCK_COUNT: usize = 8;
const WATERFALL_BUTTRESS_ROCKS_PER_SIDE: usize = 4;
const WATERFALL_CASCADE_LEDGE_COUNT: usize = 5;
const PLUNGE_POOL_RIPPLE_COUNT: usize = 5;
const PLUNGE_POOL_RIPPLE_SEGMENTS: usize = 36;
const PLUNGE_POOL_CREST_COUNT: usize = 8;
const STEPPING_STONE_COUNT: usize = 7;
const STONE_LATITUDE_SEGMENTS: usize = 3;
const STONE_LONGITUDE_SEGMENTS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum WaterDetailKind {
    LilyPadColony,
    ShoreReedArc,
    RiverbankCobbles,
    WaterfallLipRocks,
    PlungePoolRipples,
    MossySteppingStones,
}

impl WaterDetailKind {
    #[cfg(test)]
    const ALL: [Self; 6] = [
        Self::LilyPadColony,
        Self::ShoreReedArc,
        Self::RiverbankCobbles,
        Self::WaterfallLipRocks,
        Self::PlungePoolRipples,
        Self::MossySteppingStones,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::LilyPadColony => "lily_pad_colony",
            Self::ShoreReedArc => "shore_reed_arc",
            Self::RiverbankCobbles => "riverbank_cobbles",
            Self::WaterfallLipRocks => "waterfall_lip_rocks",
            Self::PlungePoolRipples => "plunge_pool_ripples",
            Self::MossySteppingStones => "mossy_stepping_stones",
        }
    }

    pub(crate) fn visual_name(self) -> &'static str {
        match self {
            Self::LilyPadColony => "lily pad colony",
            Self::ShoreReedArc => "shore reed arc",
            Self::RiverbankCobbles => "riverbank cobbles",
            Self::WaterfallLipRocks => "waterfall lip rocks",
            Self::PlungePoolRipples => "plunge pool ripples",
            Self::MossySteppingStones => "mossy stepping stones",
        }
    }

    fn sort_order(self) -> u8 {
        match self {
            Self::LilyPadColony => 0,
            Self::ShoreReedArc => 1,
            Self::RiverbankCobbles => 2,
            Self::WaterfallLipRocks => 3,
            Self::PlungePoolRipples => 4,
            Self::MossySteppingStones => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum WaterDetailMaterialRole {
    Water,
    Stone,
    Foliage,
    Flower,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct IslandWaterDetailSpec {
    pub(crate) kind: WaterDetailKind,
    pub(crate) label: &'static str,
    pub(crate) translation: Vec3,
    pub(crate) rotation_y: f32,
    pub(crate) material: WaterDetailMaterialRole,
    pub(crate) wind_phase: f32,
    pub(crate) wind_motion_scale: f32,
    pub(crate) collision_half_extents: Option<Vec3>,
    pub(crate) camera_half_extents: Option<Vec3>,
    mesh: WaterDetailMesh,
    seed: u32,
}

impl IslandWaterDetailSpec {
    pub(crate) fn build_mesh(self) -> Mesh {
        let mesh = self.mesh.build(self.seed);
        debug_assert!(
            mesh.count_vertices() >= WATER_DETAIL_MIN_VERTICES,
            "{} must remain above the runtime landmark vertex floor",
            self.kind.visual_name()
        );
        mesh
    }
}

#[derive(Clone, Copy, Debug)]
enum WaterDetailMesh {
    LilyPadColony {
        radius_x: f32,
        radius_z: f32,
        pad_count: usize,
    },
    ShoreReedArc {
        radius_x: f32,
        radius_z: f32,
        reed_count: usize,
    },
    RiverbankReeds {
        length: f32,
        width: f32,
        elevation_drop: f32,
        reed_count: usize,
    },
    RiverbankCobbles {
        length: f32,
        width: f32,
        elevation_drop: f32,
        cobble_count: usize,
    },
    WaterfallLipRocks {
        width: f32,
        depth: f32,
        upper_fall_height: f32,
        rock_count: usize,
    },
    PlungePoolRipples {
        radius_x: f32,
        radius_z: f32,
        ripple_count: usize,
    },
    MossySteppingStones {
        span: f32,
        lateral_wander: f32,
        stone_count: usize,
    },
}

impl WaterDetailMesh {
    fn build(self, seed: u32) -> Mesh {
        match self {
            Self::LilyPadColony {
                radius_x,
                radius_z,
                pad_count,
            } => lily_pad_colony_mesh(radius_x, radius_z, pad_count, seed),
            Self::ShoreReedArc {
                radius_x,
                radius_z,
                reed_count,
            } => shore_reed_arc_mesh(radius_x, radius_z, reed_count, seed),
            Self::RiverbankReeds {
                length,
                width,
                elevation_drop,
                reed_count,
            } => riverbank_reeds_mesh(length, width, elevation_drop, reed_count, seed),
            Self::RiverbankCobbles {
                length,
                width,
                elevation_drop,
                cobble_count,
            } => riverbank_cobbles_mesh(length, width, elevation_drop, cobble_count, seed),
            Self::WaterfallLipRocks {
                width,
                depth,
                upper_fall_height,
                rock_count,
            } => waterfall_lip_rocks_mesh(width, depth, upper_fall_height, rock_count, seed),
            Self::PlungePoolRipples {
                radius_x,
                radius_z,
                ripple_count,
            } => plunge_pool_ripples_mesh(radius_x, radius_z, ripple_count, seed),
            Self::MossySteppingStones {
                span,
                lateral_wander,
                stone_count,
            } => mossy_stepping_stones_mesh(span, lateral_wander, stone_count, seed),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct WaterDetailCandidate {
    priority: u8,
    source_index: usize,
    spec: IslandWaterDetailSpec,
}

pub(crate) fn island_water_detail_specs(
    island_index: usize,
    island: SkyIsland,
    water_visuals: &[super::landmarks::IslandWaterVisualSpec],
) -> Vec<IslandWaterDetailSpec> {
    let island_seed = stable_name_seed(island.name)
        .wrapping_add((island_index as u32).wrapping_mul(1_009))
        .wrapping_add(73_000);
    let mut candidates = Vec::new();

    for (source_index, water) in water_visuals.iter().copied().enumerate() {
        let source_seed = island_seed
            .wrapping_add((source_index as u32).wrapping_mul(2_003))
            .wrapping_add((water.kind as u32).wrapping_mul(307));

        match (water.kind, water.mesh) {
            (
                IslandWaterVisualKind::PondSurface,
                IslandWaterVisualMesh::PondSurface { radius_x, radius_z },
            ) => push_lake_candidates(
                &mut candidates,
                source_index,
                water,
                radius_x,
                radius_z,
                true,
                source_seed,
            ),
            (
                IslandWaterVisualKind::PlateauLakeSurface | IslandWaterVisualKind::RouteLakeSurface,
                IslandWaterVisualMesh::LakeSurface { radius_x, radius_z },
            ) => push_lake_candidates(
                &mut candidates,
                source_index,
                water,
                radius_x,
                radius_z,
                false,
                source_seed,
            ),
            (
                IslandWaterVisualKind::RiverChannel,
                IslandWaterVisualMesh::RiverChannel {
                    length,
                    width,
                    elevation_drop,
                },
            ) => push_channel_candidates(
                &mut candidates,
                source_index,
                water,
                length,
                width,
                elevation_drop,
                source_seed,
            ),
            (
                IslandWaterVisualKind::PlateauWaterfallRibbon
                | IslandWaterVisualKind::RouteWaterfallRibbon,
                IslandWaterVisualMesh::WaterfallRibbon {
                    width,
                    height,
                    depth,
                },
            ) => push_waterfall_candidates(
                &mut candidates,
                source_index,
                water,
                width,
                height,
                depth,
                source_seed,
            ),
            (
                IslandWaterVisualKind::PlateauWaterfallMist
                | IslandWaterVisualKind::RouteWaterfallMist,
                _,
            ) => {}
            _ => {}
        }
    }

    candidates.sort_by_key(|candidate| {
        (
            candidate.priority,
            candidate.source_index,
            candidate.spec.kind.sort_order(),
        )
    });
    candidates
        .into_iter()
        .take(MAX_WATER_DETAILS_PER_ISLAND)
        .map(|candidate| candidate.spec)
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn push_lake_candidates(
    candidates: &mut Vec<WaterDetailCandidate>,
    source_index: usize,
    water: super::landmarks::IslandWaterVisualSpec,
    radius_x: f32,
    radius_z: f32,
    pond: bool,
    seed: u32,
) {
    let radius_x = radius_x.max(1.0);
    let radius_z = radius_z.max(1.0);
    let cluster_radius_x = (radius_x * 0.34).clamp(1.4, 9.0);
    let cluster_radius_z = (radius_z * 0.34).clamp(1.2, 7.0);
    let local_lily_offset = Vec3::new(
        (random_unit(seed, 1, 101) - 0.5) * radius_x * 0.36,
        0.055,
        (random_unit(seed, 2, 103) - 0.5) * radius_z * 0.28,
    );
    let lily_kind = WaterDetailKind::LilyPadColony;
    candidates.push(WaterDetailCandidate {
        priority: 0,
        source_index,
        spec: detail_spec(
            lily_kind,
            local_to_world(water.translation, water.rotation_y, local_lily_offset),
            water.rotation_y + (random_unit(seed, 3, 107) - 0.5) * 0.34,
            if pond {
                WaterDetailMaterialRole::Flower
            } else {
                WaterDetailMaterialRole::Foliage
            },
            water.wind_phase + 0.21,
            (water.wind_motion_scale * 0.42).max(0.25),
            WaterDetailMesh::LilyPadColony {
                radius_x: cluster_radius_x,
                radius_z: cluster_radius_z,
                pad_count: if pond { 8 } else { 11 },
            },
            seed.wrapping_add(11),
        ),
    });

    let reed_kind = WaterDetailKind::ShoreReedArc;
    candidates.push(WaterDetailCandidate {
        priority: 1,
        source_index,
        spec: detail_spec(
            reed_kind,
            water.translation + Vec3::Y * 0.035,
            water.rotation_y,
            WaterDetailMaterialRole::Foliage,
            water.wind_phase + 0.63,
            (water.wind_motion_scale * 0.88).max(0.5),
            WaterDetailMesh::ShoreReedArc {
                radius_x: radius_x * 0.92,
                radius_z: radius_z * 0.92,
                reed_count: if pond {
                    SHORE_REED_COUNT - 6
                } else {
                    SHORE_REED_COUNT
                },
            },
            seed.wrapping_add(23),
        ),
    });

    if !pond {
        let stepping_kind = WaterDetailKind::MossySteppingStones;
        candidates.push(WaterDetailCandidate {
            priority: 2,
            source_index,
            spec: detail_spec(
                stepping_kind,
                water.translation + Vec3::Y * 0.025,
                water.rotation_y + 0.34 + (random_unit(seed, 5, 109) - 0.5) * 0.28,
                WaterDetailMaterialRole::Stone,
                0.0,
                0.0,
                WaterDetailMesh::MossySteppingStones {
                    span: (radius_x * 1.18).clamp(5.0, 25.0),
                    lateral_wander: (radius_z * 0.34).clamp(1.0, 6.5),
                    stone_count: STEPPING_STONE_COUNT,
                },
                seed.wrapping_add(37),
            ),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn push_channel_candidates(
    candidates: &mut Vec<WaterDetailCandidate>,
    source_index: usize,
    water: super::landmarks::IslandWaterVisualSpec,
    length: f32,
    width: f32,
    elevation_drop: f32,
    seed: u32,
) {
    let length = length.max(1.0);
    let width = width.max(0.5);
    let cobble_kind = WaterDetailKind::RiverbankCobbles;
    candidates.push(WaterDetailCandidate {
        priority: 0,
        source_index,
        spec: detail_spec(
            cobble_kind,
            water.translation + Vec3::Y * 0.035,
            water.rotation_y,
            WaterDetailMaterialRole::Stone,
            0.0,
            0.0,
            WaterDetailMesh::RiverbankCobbles {
                length,
                width,
                elevation_drop,
                cobble_count: CHANNEL_COBBLE_COUNT,
            },
            seed.wrapping_add(41),
        ),
    });

    let reed_kind = WaterDetailKind::ShoreReedArc;
    candidates.push(WaterDetailCandidate {
        priority: 2,
        source_index,
        spec: detail_spec(
            reed_kind,
            water.translation + Vec3::Y * 0.025,
            water.rotation_y,
            WaterDetailMaterialRole::Foliage,
            water.wind_phase + 0.47,
            (water.wind_motion_scale * 0.72).max(0.45),
            WaterDetailMesh::RiverbankReeds {
                length,
                width,
                elevation_drop,
                reed_count: CHANNEL_REED_COUNT,
            },
            seed.wrapping_add(53),
        ),
    });
}

#[allow(clippy::too_many_arguments)]
fn push_waterfall_candidates(
    candidates: &mut Vec<WaterDetailCandidate>,
    source_index: usize,
    water: super::landmarks::IslandWaterVisualSpec,
    width: f32,
    height: f32,
    depth: f32,
    seed: u32,
) {
    let width = width.max(2.0);
    let height = height.max(2.0);
    let depth = depth.max(0.2);
    let lip_backset = (width * 0.18).max(2.5);
    let lip_translation = local_to_world(
        water.translation,
        water.rotation_y,
        Vec3::new(0.0, height * 0.5 + width * 0.025, -lip_backset),
    );
    let lip_kind = WaterDetailKind::WaterfallLipRocks;
    candidates.push(WaterDetailCandidate {
        priority: 0,
        source_index,
        spec: detail_spec(
            lip_kind,
            lip_translation,
            water.rotation_y,
            WaterDetailMaterialRole::Stone,
            0.0,
            0.0,
            WaterDetailMesh::WaterfallLipRocks {
                width: width * 1.12,
                depth: (width * 0.28).max(depth * 4.0),
                upper_fall_height: (height * 0.48).max(width * 0.75).min(height * 0.65),
                rock_count: WATERFALL_LIP_ROCK_COUNT,
            },
            seed.wrapping_add(67),
        ),
    });

    let pool_translation = local_to_world(
        water.translation,
        water.rotation_y,
        Vec3::new(0.0, -height * 0.5 + 0.12, (width * 0.14).max(1.0)),
    );
    let ripple_kind = WaterDetailKind::PlungePoolRipples;
    candidates.push(WaterDetailCandidate {
        priority: 1,
        source_index,
        spec: detail_spec(
            ripple_kind,
            pool_translation,
            water.rotation_y,
            WaterDetailMaterialRole::Water,
            water.wind_phase + 0.37,
            (water.wind_motion_scale * 0.58).max(0.65),
            WaterDetailMesh::PlungePoolRipples {
                radius_x: (width * 0.96).max(3.2),
                radius_z: (width * 0.68).max(2.2),
                ripple_count: PLUNGE_POOL_RIPPLE_COUNT,
            },
            seed.wrapping_add(79),
        ),
    });
}

#[allow(clippy::too_many_arguments)]
fn detail_spec(
    kind: WaterDetailKind,
    translation: Vec3,
    rotation_y: f32,
    material: WaterDetailMaterialRole,
    wind_phase: f32,
    wind_motion_scale: f32,
    mesh: WaterDetailMesh,
    seed: u32,
) -> IslandWaterDetailSpec {
    IslandWaterDetailSpec {
        kind,
        label: kind.visual_name(),
        translation,
        rotation_y,
        material,
        wind_phase,
        wind_motion_scale,
        // Every mesh is a spatially separated cluster. A single AABB would block
        // navigable water or route gaps, so integration must not create one.
        collision_half_extents: None,
        camera_half_extents: None,
        mesh,
        seed,
    }
}

fn local_to_world(origin: Vec3, rotation_y: f32, local_offset: Vec3) -> Vec3 {
    origin + Quat::from_rotation_y(rotation_y) * local_offset
}

fn stable_name_seed(name: &str) -> u32 {
    name.bytes().fold(2_166_136_261, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    })
}

fn lily_pad_colony_mesh(radius_x: f32, radius_z: f32, pad_count: usize, seed: u32) -> Mesh {
    let mut mesh = MeshBuffers::default();
    let radius_x = radius_x.max(0.5);
    let radius_z = radius_z.max(0.5);
    let base_pad_radius = radius_x.min(radius_z) * 0.16;

    for pad in 0..pad_count {
        let ring = (0.28 + random_unit(seed, pad as u32, 211) * 0.67).sqrt();
        let angle = random_unit(seed, pad as u32, 223) * std::f32::consts::TAU;
        let center = Vec3::new(
            angle.cos() * radius_x * ring,
            (random_unit(seed, pad as u32, 227) - 0.5) * 0.055,
            angle.sin() * radius_z * ring,
        );
        let pad_radius = base_pad_radius * (0.72 + random_unit(seed, pad as u32, 229) * 0.54);
        let stretch = 0.82 + random_unit(seed, pad as u32, 233) * 0.30;
        let rotation = random_unit(seed, pad as u32, 239) * std::f32::consts::TAU;
        append_lily_pad(
            &mut mesh,
            center,
            Vec2::new(pad_radius, pad_radius * stretch),
            rotation,
            seed.wrapping_add(pad as u32 * 43),
        );

        if pad % 3 == 0 {
            append_lily_blossom(
                &mut mesh,
                center + Vec3::Y * 0.07,
                pad_radius * 0.34,
                seed.wrapping_add(pad as u32 * 59),
            );
        }
    }

    mesh.finish()
}

fn append_lily_pad(mesh: &mut MeshBuffers, center: Vec3, radii: Vec2, rotation: f32, seed: u32) {
    let notch = (random_unit(seed, 1, 251) * LILY_PAD_SEGMENTS as f32) as usize % LILY_PAD_SEGMENTS;
    for segment in 0..LILY_PAD_SEGMENTS {
        if segment == notch {
            continue;
        }
        let angle_a = rotation + segment as f32 / LILY_PAD_SEGMENTS as f32 * std::f32::consts::TAU;
        let angle_b =
            rotation + (segment + 1) as f32 / LILY_PAD_SEGMENTS as f32 * std::f32::consts::TAU;
        let edge_a = center
            + Vec3::new(
                angle_a.cos() * radii.x,
                (angle_a * 3.0 + seed as f32 * 0.013).sin() * radii.min_element() * 0.018,
                angle_a.sin() * radii.y,
            );
        let edge_b = center
            + Vec3::new(
                angle_b.cos() * radii.x,
                (angle_b * 3.0 + seed as f32 * 0.013).sin() * radii.min_element() * 0.018,
                angle_b.sin() * radii.y,
            );
        mesh.append_double_sided_triangle(center, edge_b, edge_a);
    }
}

fn append_lily_blossom(mesh: &mut MeshBuffers, center: Vec3, radius: f32, seed: u32) {
    let petal_count = 5;
    let rotation = random_unit(seed, 0, 263) * std::f32::consts::TAU;
    for petal in 0..petal_count {
        let angle = rotation + petal as f32 / petal_count as f32 * std::f32::consts::TAU;
        let tangent = Vec3::new(-angle.sin(), 0.0, angle.cos());
        let outward = Vec3::new(angle.cos(), 0.0, angle.sin());
        let base = center + outward * radius * 0.10;
        let tip = center + outward * radius + Vec3::Y * radius * 0.82;
        mesh.append_double_sided_triangle(
            base - tangent * radius * 0.34,
            tip,
            base + tangent * radius * 0.34,
        );
    }
}

fn shore_reed_arc_mesh(radius_x: f32, radius_z: f32, reed_count: usize, seed: u32) -> Mesh {
    let mut mesh = MeshBuffers::default();
    let radius_x = radius_x.max(1.0);
    let radius_z = radius_z.max(1.0);
    let base_height = (radius_x.min(radius_z) * 0.13).clamp(1.1, 3.8);
    let arc_start = random_unit(seed, 0, 277) * std::f32::consts::TAU;
    let arc_span = 1.72;

    for reed in 0..reed_count {
        let t = if reed_count <= 1 {
            0.5
        } else {
            reed as f32 / (reed_count - 1) as f32
        };
        let angle =
            arc_start + (t - 0.5) * arc_span + (random_unit(seed, reed as u32, 281) - 0.5) * 0.11;
        let radial_scale = 0.94 + random_unit(seed, reed as u32, 283) * 0.10;
        let base = Vec3::new(
            angle.cos() * radius_x * radial_scale,
            0.0,
            angle.sin() * radius_z * radial_scale,
        );
        let tangent =
            Vec3::new(-angle.sin() * radius_x, 0.0, angle.cos() * radius_z).normalize_or_zero();
        let radial = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize_or_zero();
        let height = base_height * (0.70 + random_unit(seed, reed as u32, 293) * 0.62);
        let half_width = (height * 0.035).clamp(0.035, 0.11);
        append_crossed_reed(
            &mut mesh,
            base,
            tangent,
            radial,
            half_width,
            height,
            random_unit(seed, reed as u32, 307) - 0.5,
        );
    }

    mesh.finish()
}

fn riverbank_reeds_mesh(
    length: f32,
    width: f32,
    elevation_drop: f32,
    reed_count: usize,
    seed: u32,
) -> Mesh {
    let mut mesh = MeshBuffers::default();
    let base_height = (width * 0.58).clamp(0.9, 2.8);

    for reed in 0..reed_count {
        let t = (reed as f32 + 0.25 + random_unit(seed, reed as u32, 311) * 0.5)
            / reed_count.max(1) as f32;
        let side = if reed % 2 == 0 { -1.0 } else { 1.0 };
        let bank_offset = width * (0.60 + random_unit(seed, reed as u32, 313) * 0.22);
        let base = Vec3::new(
            side * bank_offset,
            elevation_drop * (0.5 - t),
            (t - 0.5) * length + (random_unit(seed, reed as u32, 317) - 0.5) * width * 0.65,
        );
        let height = base_height * (0.72 + random_unit(seed, reed as u32, 331) * 0.58);
        append_crossed_reed(
            &mut mesh,
            base,
            Vec3::Z,
            Vec3::X * side,
            (height * 0.038).clamp(0.035, 0.10),
            height,
            random_unit(seed, reed as u32, 337) - 0.5,
        );
    }

    mesh.finish()
}

#[allow(clippy::too_many_arguments)]
fn append_crossed_reed(
    mesh: &mut MeshBuffers,
    base: Vec3,
    tangent: Vec3,
    outward: Vec3,
    half_width: f32,
    height: f32,
    lean: f32,
) {
    let tangent = tangent.normalize_or_zero();
    let outward = outward.normalize_or_zero();
    let tip = base + Vec3::Y * height + outward * lean * height * 0.10;
    let narrow_width = half_width * 0.34;
    mesh.append_double_sided_quad([
        base - tangent * half_width,
        base + tangent * half_width,
        tip + tangent * narrow_width,
        tip - tangent * narrow_width,
    ]);
    mesh.append_double_sided_quad([
        base - outward * half_width,
        base + outward * half_width,
        tip + outward * narrow_width,
        tip - outward * narrow_width,
    ]);
}

fn riverbank_cobbles_mesh(
    length: f32,
    width: f32,
    elevation_drop: f32,
    cobble_count: usize,
    seed: u32,
) -> Mesh {
    let mut mesh = MeshBuffers::default();
    let base_radius = (width * 0.20).clamp(0.28, 1.15);

    for cobble in 0..cobble_count {
        let t = (cobble as f32 + 0.2 + random_unit(seed, cobble as u32, 347) * 0.6)
            / cobble_count.max(1) as f32;
        let side = if cobble % 2 == 0 { -1.0 } else { 1.0 };
        let radius = base_radius * (0.66 + random_unit(seed, cobble as u32, 349) * 0.68);
        let radii = Vec3::new(
            radius * (0.78 + random_unit(seed, cobble as u32, 353) * 0.42),
            radius * (0.38 + random_unit(seed, cobble as u32, 359) * 0.28),
            radius * (0.72 + random_unit(seed, cobble as u32, 367) * 0.50),
        );
        let center = Vec3::new(
            side * width * (0.65 + random_unit(seed, cobble as u32, 373) * 0.22),
            elevation_drop * (0.5 - t) + radii.y * 0.52,
            (t - 0.5) * length + (random_unit(seed, cobble as u32, 379) - 0.5) * width * 0.62,
        );
        mesh.append_ellipsoid(center, radii, seed.wrapping_add(cobble as u32 * 71), 0.22);
    }

    mesh.finish()
}

fn waterfall_lip_rocks_mesh(
    width: f32,
    depth: f32,
    upper_fall_height: f32,
    rock_count: usize,
    seed: u32,
) -> Mesh {
    let mut mesh = MeshBuffers::default();
    let base_radius = (width / rock_count.max(1) as f32 * 0.72).clamp(0.45, 2.2);
    let upper_fall_height = upper_fall_height.max(width * 0.55);

    for rock in 0..rock_count {
        let t = if rock_count <= 1 {
            0.5
        } else {
            rock as f32 / (rock_count - 1) as f32
        };
        let radius = base_radius * (0.74 + random_unit(seed, rock as u32, 383) * 0.58);
        let radii = Vec3::new(
            radius * (0.86 + random_unit(seed, rock as u32, 389) * 0.36),
            radius * (0.72 + random_unit(seed, rock as u32, 397) * 0.48),
            radius * (0.68 + random_unit(seed, rock as u32, 401) * 0.40),
        );
        let center = Vec3::new(
            (t - 0.5) * width + (random_unit(seed, rock as u32, 409) - 0.5) * base_radius * 0.62,
            radii.y * 0.54,
            (random_unit(seed, rock as u32, 419) - 0.5) * depth,
        );
        mesh.append_ellipsoid(center, radii, seed.wrapping_add(rock as u32 * 83), 0.26);
    }

    for side_index in 0..2 {
        let side = if side_index == 0 { -1.0 } else { 1.0 };
        for layer in 0..WATERFALL_BUTTRESS_ROCKS_PER_SIDE {
            let index = (side_index * WATERFALL_BUTTRESS_ROCKS_PER_SIDE + layer) as u32;
            let t = (layer as f32 + 0.35) / WATERFALL_BUTTRESS_ROCKS_PER_SIDE as f32;
            let radius = base_radius * (1.08 - t * 0.24 + random_unit(seed, index, 487) * 0.32);
            let radii = Vec3::new(
                radius * (1.04 + random_unit(seed, index, 491) * 0.36),
                radius * (1.10 + random_unit(seed, index, 499) * 0.55),
                radius * (0.82 + random_unit(seed, index, 503) * 0.40),
            );
            let center = Vec3::new(
                side * (width * (0.49 + t * 0.07)
                    + (random_unit(seed, index, 509) - 0.5) * base_radius),
                -upper_fall_height * (0.08 + t * 0.68),
                depth * (0.08 + t * 0.72) + (random_unit(seed, index, 521) - 0.5) * depth * 0.26,
            );
            mesh.append_ellipsoid(center, radii, seed.wrapping_add(1_000 + index * 101), 0.30);
        }
    }

    for ledge in 0..WATERFALL_CASCADE_LEDGE_COUNT {
        let side = if ledge % 2 == 0 { -1.0 } else { 1.0 };
        let t = (ledge as f32 + 1.0) / (WATERFALL_CASCADE_LEDGE_COUNT + 1) as f32;
        let ledge_radius = base_radius * (0.92 + random_unit(seed, ledge as u32, 523) * 0.38);
        let radii = Vec3::new(
            width * (0.14 + random_unit(seed, ledge as u32, 541) * 0.07),
            ledge_radius * (0.36 + random_unit(seed, ledge as u32, 547) * 0.25),
            depth * (0.38 + random_unit(seed, ledge as u32, 557) * 0.26),
        );
        let center = Vec3::new(
            side * width * (0.31 + random_unit(seed, ledge as u32, 563) * 0.11),
            -upper_fall_height * (0.12 + t * 0.62),
            depth * (0.28 + t * 0.62),
        );
        mesh.append_ellipsoid(
            center,
            radii,
            seed.wrapping_add(2_000 + ledge as u32 * 109),
            0.24,
        );
    }

    mesh.finish()
}

fn plunge_pool_ripples_mesh(radius_x: f32, radius_z: f32, ripple_count: usize, seed: u32) -> Mesh {
    let mut mesh = MeshBuffers::default();
    let radius_x = radius_x.max(0.8);
    let radius_z = radius_z.max(0.6);

    append_plunge_pool_churn(&mut mesh, radius_x, radius_z, seed);

    for ripple in 0..ripple_count {
        let t = (ripple + 1) as f32 / ripple_count.max(1) as f32;
        let ring_width = (radius_x.min(radius_z) * (0.045 + t * 0.012)).clamp(0.06, 0.28);
        let phase = random_unit(seed, ripple as u32, 421) * std::f32::consts::TAU;
        append_ripple_ring(
            &mut mesh,
            radius_x * (0.30 + t * 0.70),
            radius_z * (0.30 + t * 0.70),
            ring_width,
            phase,
            ripple,
            seed,
        );
    }

    mesh.finish()
}

fn append_plunge_pool_churn(mesh: &mut MeshBuffers, radius_x: f32, radius_z: f32, seed: u32) {
    let center = Vec3::new(0.0, 0.055, 0.0);
    for segment in 0..PLUNGE_POOL_RIPPLE_SEGMENTS {
        let angle_a = segment as f32 / PLUNGE_POOL_RIPPLE_SEGMENTS as f32 * std::f32::consts::TAU;
        let angle_b =
            (segment + 1) as f32 / PLUNGE_POOL_RIPPLE_SEGMENTS as f32 * std::f32::consts::TAU;
        let scale_a = 0.58 + (random_unit(seed, segment as u32, 569) - 0.5) * 0.10;
        let scale_b = 0.58 + (random_unit(seed, (segment + 1) as u32, 569) - 0.5) * 0.10;
        let edge_a = Vec3::new(
            angle_a.cos() * radius_x * scale_a,
            0.025 + (angle_a * 4.0 + seed as f32 * 0.013).sin() * 0.055,
            angle_a.sin() * radius_z * scale_a,
        );
        let edge_b = Vec3::new(
            angle_b.cos() * radius_x * scale_b,
            0.025 + (angle_b * 4.0 + seed as f32 * 0.013).sin() * 0.055,
            angle_b.sin() * radius_z * scale_b,
        );
        mesh.append_double_sided_triangle(center, edge_b, edge_a);
    }

    let crest_height = (radius_x.min(radius_z) * 0.15).clamp(0.28, 1.25);
    for crest in 0..PLUNGE_POOL_CREST_COUNT {
        let angle = crest as f32 / PLUNGE_POOL_CREST_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, crest as u32, 571) * 0.24;
        let outward = Vec3::new(angle.cos(), 0.0, angle.sin());
        let tangent = Vec3::new(-angle.sin(), 0.0, angle.cos());
        let base =
            outward * radius_x.min(radius_z) * (0.10 + random_unit(seed, crest as u32, 577) * 0.14);
        let half_width = crest_height * (0.18 + random_unit(seed, crest as u32, 587) * 0.12);
        let tip = base
            + outward * crest_height * 0.24
            + Vec3::Y * crest_height * (0.72 + random_unit(seed, crest as u32, 593) * 0.38);
        mesh.append_double_sided_triangle(
            base - tangent * half_width,
            tip,
            base + tangent * half_width,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn append_ripple_ring(
    mesh: &mut MeshBuffers,
    radius_x: f32,
    radius_z: f32,
    width: f32,
    phase: f32,
    ripple: usize,
    seed: u32,
) {
    for segment in 0..PLUNGE_POOL_RIPPLE_SEGMENTS {
        let angle_a = segment as f32 / PLUNGE_POOL_RIPPLE_SEGMENTS as f32 * std::f32::consts::TAU;
        let angle_b =
            (segment + 1) as f32 / PLUNGE_POOL_RIPPLE_SEGMENTS as f32 * std::f32::consts::TAU;
        let wave_a = (angle_a * 3.0 + phase).sin() * width * 0.18;
        let wave_b = (angle_b * 3.0 + phase).sin() * width * 0.18;
        let jitter_a =
            1.0 + (random_unit(seed, segment as u32 + ripple as u32 * 53, 431) - 0.5) * 0.035;
        let jitter_b =
            1.0 + (random_unit(seed, (segment + 1) as u32 + ripple as u32 * 53, 431) - 0.5) * 0.035;
        let inner_a = Vec3::new(
            angle_a.cos() * (radius_x - width) * jitter_a,
            wave_a,
            angle_a.sin() * (radius_z - width) * jitter_a,
        );
        let outer_a = Vec3::new(
            angle_a.cos() * (radius_x + width) * jitter_a,
            wave_a,
            angle_a.sin() * (radius_z + width) * jitter_a,
        );
        let inner_b = Vec3::new(
            angle_b.cos() * (radius_x - width) * jitter_b,
            wave_b,
            angle_b.sin() * (radius_z - width) * jitter_b,
        );
        let outer_b = Vec3::new(
            angle_b.cos() * (radius_x + width) * jitter_b,
            wave_b,
            angle_b.sin() * (radius_z + width) * jitter_b,
        );
        mesh.append_double_sided_quad([inner_a, outer_a, outer_b, inner_b]);
    }
}

fn mossy_stepping_stones_mesh(
    span: f32,
    lateral_wander: f32,
    stone_count: usize,
    seed: u32,
) -> Mesh {
    let mut mesh = MeshBuffers::default();
    let spacing = span / stone_count.max(1) as f32;
    let base_radius = (spacing * 0.38).clamp(0.55, 1.8);

    for stone in 0..stone_count {
        let t = if stone_count <= 1 {
            0.5
        } else {
            stone as f32 / (stone_count - 1) as f32
        };
        let radius = base_radius * (0.78 + random_unit(seed, stone as u32, 439) * 0.46);
        let radii = Vec3::new(
            radius * (0.90 + random_unit(seed, stone as u32, 443) * 0.30),
            radius * (0.34 + random_unit(seed, stone as u32, 449) * 0.18),
            radius * (0.72 + random_unit(seed, stone as u32, 457) * 0.34),
        );
        let center = Vec3::new(
            (t - 0.5) * span,
            radii.y * 0.48 + random_unit(seed, stone as u32, 461) * 0.08,
            (t * std::f32::consts::TAU * 1.35 + seed as f32 * 0.009).sin() * lateral_wander * 0.42,
        );
        mesh.append_ellipsoid(center, radii, seed.wrapping_add(stone as u32 * 97), 0.18);
    }

    mesh.finish()
}

#[derive(Default)]
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

impl MeshBuffers {
    fn append_double_sided_triangle(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        let normal = (b - a).cross(c - a).normalize_or_zero();
        let start = self.positions.len() as u32;
        for point in [a, b, c] {
            self.positions.push(point.to_array());
            self.normals.push(normal.to_array());
        }
        self.uvs.extend([[0.5, 0.5], [1.0, 0.0], [0.0, 0.0]]);
        for point in [a, b, c] {
            self.positions.push(point.to_array());
            self.normals.push((-normal).to_array());
        }
        self.uvs.extend([[0.5, 0.5], [1.0, 0.0], [0.0, 0.0]]);
        self.indices
            .extend([start, start + 1, start + 2, start + 5, start + 4, start + 3]);
    }

    fn append_double_sided_quad(&mut self, points: [Vec3; 4]) {
        let normal = (points[1] - points[0])
            .cross(points[2] - points[0])
            .normalize_or_zero();
        let start = self.positions.len() as u32;
        for point in points {
            self.positions.push(point.to_array());
            self.normals.push(normal.to_array());
        }
        self.uvs
            .extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        for point in points {
            self.positions.push(point.to_array());
            self.normals.push((-normal).to_array());
        }
        self.uvs
            .extend([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        self.indices.extend([
            start,
            start + 1,
            start + 2,
            start,
            start + 2,
            start + 3,
            start + 6,
            start + 5,
            start + 4,
            start + 7,
            start + 6,
            start + 4,
        ]);
    }

    fn append_ellipsoid(&mut self, center: Vec3, radii: Vec3, seed: u32, noise: f32) {
        let start = self.positions.len() as u32;
        for latitude in 0..=STONE_LATITUDE_SEGMENTS {
            let theta = latitude as f32 / STONE_LATITUDE_SEGMENTS as f32 * std::f32::consts::PI;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            for longitude in 0..=STONE_LONGITUDE_SEGMENTS {
                let phi =
                    longitude as f32 / STONE_LONGITUDE_SEGMENTS as f32 * std::f32::consts::TAU;
                let unit = Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin());
                let variation = 1.0
                    + (random_unit(seed, latitude as u32 * 31 + longitude as u32, 467) - 0.5)
                        * noise;
                let point = center + unit * radii * variation;
                let normal = Vec3::new(
                    unit.x / radii.x.max(0.001),
                    unit.y / radii.y.max(0.001),
                    unit.z / radii.z.max(0.001),
                )
                .normalize_or_zero();
                self.positions.push(point.to_array());
                self.normals.push(normal.to_array());
                self.uvs.push([
                    longitude as f32 / STONE_LONGITUDE_SEGMENTS as f32,
                    latitude as f32 / STONE_LATITUDE_SEGMENTS as f32,
                ]);
            }
        }

        let stride = STONE_LONGITUDE_SEGMENTS + 1;
        for latitude in 0..STONE_LATITUDE_SEGMENTS {
            for longitude in 0..STONE_LONGITUDE_SEGMENTS {
                let a = start + (latitude * stride + longitude) as u32;
                let b = a + 1;
                let c = start + ((latitude + 1) * stride + longitude) as u32;
                let d = c + 1;
                self.indices.extend([a, c, b, b, c, d]);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::super::landmarks::island_water_visual_specs;
    use super::*;
    use bevy::mesh::VertexAttributeValues;
    use nau_engine::world::SkyRoute;
    use std::collections::{HashMap, HashSet};

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values,
            _ => panic!("mesh should expose Float32x3 positions"),
        }
    }

    fn u32_indices(mesh: &Mesh) -> &[u32] {
        match mesh.indices() {
            Some(Indices::U32(values)) => values,
            _ => panic!("mesh should expose U32 indices"),
        }
    }

    fn axis_range(points: &[[f32; 3]], axis: usize) -> f32 {
        let (min, max) = points
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), point| {
                (min.min(point[axis]), max.max(point[axis]))
            });
        max - min
    }

    fn route_detail_specs() -> Vec<IslandWaterDetailSpec> {
        SkyRoute::default()
            .islands()
            .iter()
            .copied()
            .enumerate()
            .flat_map(|(index, island)| {
                let water = island_water_visual_specs(index, island);
                island_water_detail_specs(index, island, &water)
            })
            .collect()
    }

    #[test]
    fn water_detail_specs_and_meshes_are_deterministic() {
        let island = SkyIsland::new(
            "sapphire basin",
            Vec3::new(-18.0, 94.0, -30.0),
            Vec2::new(100.0, 64.0),
            20.0,
            false,
        );
        let water = island_water_visual_specs(7, island);
        let first = island_water_detail_specs(7, island, &water);
        let second = island_water_detail_specs(7, island, &water);

        assert!(!first.is_empty());
        assert_eq!(first.len(), second.len());
        for (first, second) in first.into_iter().zip(second) {
            assert_eq!(first.kind, second.kind);
            assert_eq!(first.label, second.label);
            assert_eq!(first.translation, second.translation);
            assert_eq!(first.rotation_y, second.rotation_y);
            assert_eq!(first.material, second.material);
            assert_eq!(first.wind_phase, second.wind_phase);
            assert_eq!(first.wind_motion_scale, second.wind_motion_scale);
            assert_eq!(first.collision_half_extents, second.collision_half_extents);
            assert_eq!(first.camera_half_extents, second.camera_half_extents);

            let first_mesh = first.build_mesh();
            let second_mesh = second.build_mesh();
            assert_eq!(positions(&first_mesh), positions(&second_mesh));
            assert_eq!(u32_indices(&first_mesh), u32_indices(&second_mesh));
        }
    }

    #[test]
    fn route_water_features_cover_every_detail_kind_and_material_role() {
        let specs = route_detail_specs();
        let kinds = specs.iter().map(|spec| spec.kind).collect::<HashSet<_>>();
        let materials = specs
            .iter()
            .map(|spec| spec.material)
            .collect::<HashSet<_>>();

        for kind in WaterDetailKind::ALL {
            assert!(
                kinds.contains(&kind),
                "route water should exercise {}",
                kind.label()
            );
            assert!(!kind.visual_name().is_empty());
        }
        for material in [
            WaterDetailMaterialRole::Water,
            WaterDetailMaterialRole::Stone,
            WaterDetailMaterialRole::Foliage,
            WaterDetailMaterialRole::Flower,
        ] {
            assert!(materials.contains(&material));
        }
    }

    #[test]
    fn water_detail_count_is_capped_per_island() {
        let route = SkyRoute::default();
        for (index, island) in route.islands().iter().copied().enumerate() {
            let water = island_water_visual_specs(index, island);
            let details = island_water_detail_specs(index, island, &water);
            assert!(
                details.len() <= MAX_WATER_DETAILS_PER_ISLAND,
                "{} generated {} water accents",
                island.name,
                details.len()
            );
        }

        let (plateau_index, plateau) = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .find(|(_, island)| island.is_great_plateau_anchor())
            .expect("route should include the great plateau");
        let water = island_water_visual_specs(plateau_index, plateau);
        let details = island_water_detail_specs(plateau_index, plateau, &water);
        assert_eq!(details.len(), MAX_WATER_DETAILS_PER_ISLAND);
    }

    #[test]
    fn clustered_meshes_have_bounded_complexity_and_readable_spans() {
        let route_specs = route_detail_specs();
        for spec in &route_specs {
            assert!(
                spec.build_mesh().count_vertices() >= WATER_DETAIL_MIN_VERTICES,
                "{} fell below the runtime landmark vertex floor",
                spec.kind.visual_name()
            );
        }

        let mut representative = HashMap::new();
        for spec in route_specs {
            representative.entry(spec.kind).or_insert(spec);
        }

        for kind in WaterDetailKind::ALL {
            let spec = *representative
                .get(&kind)
                .unwrap_or_else(|| panic!("missing representative for {}", kind.visual_name()));
            let mesh = spec.build_mesh();
            let points = positions(&mesh);
            let x_span = axis_range(points, 0);
            let y_span = axis_range(points, 1);
            let z_span = axis_range(points, 2);

            assert!((WATER_DETAIL_MIN_VERTICES..=2_500).contains(&mesh.count_vertices()));
            assert!(!u32_indices(&mesh).is_empty());
            assert!(points.iter().flatten().all(|value| value.is_finite()));

            match kind {
                WaterDetailKind::LilyPadColony => {
                    assert!(x_span > 1.5 && z_span > 1.0 && y_span > 0.12);
                }
                WaterDetailKind::ShoreReedArc => {
                    assert!(x_span.max(z_span) > 3.0 && y_span > 0.8);
                }
                WaterDetailKind::RiverbankCobbles => {
                    assert!(x_span > 1.0 && z_span > 4.0 && y_span > 0.25);
                }
                WaterDetailKind::WaterfallLipRocks => {
                    assert!(x_span > 3.0 && z_span > 2.0 && y_span > 5.0);
                }
                WaterDetailKind::PlungePoolRipples => {
                    assert!(x_span > 6.0 && z_span > 4.0 && (0.2..2.5).contains(&y_span));
                }
                WaterDetailKind::MossySteppingStones => {
                    assert!(x_span > 4.0 && z_span > 0.8 && y_span > 0.25);
                }
            }
        }
    }

    #[test]
    fn details_correspond_to_source_water_features_and_ignore_mist() {
        let route = SkyRoute::default();
        for (index, island) in route.islands().iter().copied().enumerate() {
            let water = island_water_visual_specs(index, island);
            let details = island_water_detail_specs(index, island, &water);
            let lake_count = water
                .iter()
                .filter(|source| {
                    matches!(
                        source.kind,
                        IslandWaterVisualKind::PondSurface
                            | IslandWaterVisualKind::PlateauLakeSurface
                            | IslandWaterVisualKind::RouteLakeSurface
                    )
                })
                .count();
            let channel_count = water
                .iter()
                .filter(|source| source.kind == IslandWaterVisualKind::RiverChannel)
                .count();
            let waterfall_count = water
                .iter()
                .filter(|source| {
                    matches!(
                        source.kind,
                        IslandWaterVisualKind::PlateauWaterfallRibbon
                            | IslandWaterVisualKind::RouteWaterfallRibbon
                    )
                })
                .count();

            assert_eq!(
                details
                    .iter()
                    .filter(|detail| detail.kind == WaterDetailKind::LilyPadColony)
                    .count(),
                lake_count
            );
            assert!(
                details
                    .iter()
                    .filter(|detail| detail.kind == WaterDetailKind::ShoreReedArc)
                    .count()
                    >= lake_count
            );
            assert_eq!(
                details
                    .iter()
                    .filter(|detail| detail.kind == WaterDetailKind::RiverbankCobbles)
                    .count(),
                channel_count
            );
            assert_eq!(
                details
                    .iter()
                    .filter(|detail| detail.kind == WaterDetailKind::WaterfallLipRocks)
                    .count(),
                waterfall_count
            );
            assert_eq!(
                details
                    .iter()
                    .filter(|detail| detail.kind == WaterDetailKind::PlungePoolRipples)
                    .count(),
                waterfall_count
            );
            assert!(details.iter().all(|detail| {
                detail.collision_half_extents.is_none() && detail.camera_half_extents.is_none()
            }));
        }

        let (plateau_index, plateau) = route
            .islands()
            .iter()
            .copied()
            .enumerate()
            .find(|(_, island)| island.is_great_plateau_anchor())
            .expect("route should include the great plateau");
        let water = island_water_visual_specs(plateau_index, plateau);
        let mist = water
            .iter()
            .copied()
            .filter(|source| {
                matches!(
                    source.kind,
                    IslandWaterVisualKind::PlateauWaterfallMist
                        | IslandWaterVisualKind::RouteWaterfallMist
                )
            })
            .collect::<Vec<_>>();
        assert!(!mist.is_empty());
        assert!(island_water_detail_specs(plateau_index, plateau, &mist).is_empty());

        let waterfall = SkyIsland::new(
            "cloudfall meadow",
            Vec3::new(-144.0, 142.0, -1208.0),
            Vec2::new(90.0, 64.0),
            28.0,
            false,
        );
        let water = island_water_visual_specs(29, waterfall);
        let ribbon = water
            .iter()
            .copied()
            .find(|source| source.kind == IslandWaterVisualKind::RouteWaterfallRibbon)
            .expect("waterfall island should expose a ribbon");
        let details = island_water_detail_specs(29, waterfall, &water);
        let lip = details
            .iter()
            .find(|detail| detail.kind == WaterDetailKind::WaterfallLipRocks)
            .expect("waterfall ribbon should receive lip rocks");
        let ripples = details
            .iter()
            .find(|detail| detail.kind == WaterDetailKind::PlungePoolRipples)
            .expect("waterfall ribbon should receive plunge-pool ripples");
        let IslandWaterVisualMesh::WaterfallRibbon { width, height, .. } = ribbon.mesh else {
            panic!("waterfall source should use ribbon dimensions");
        };
        let expected_lip = local_to_world(
            ribbon.translation,
            ribbon.rotation_y,
            Vec3::new(0.0, height * 0.5 + width * 0.025, -(width * 0.18).max(2.5)),
        );
        let expected_pool = local_to_world(
            ribbon.translation,
            ribbon.rotation_y,
            Vec3::new(0.0, -height * 0.5 + 0.12, (width * 0.14).max(1.0)),
        );
        assert!(lip.translation.distance(expected_lip) < 0.001);
        assert!(ripples.translation.distance(expected_pool) < 0.001);
        assert_eq!(lip.rotation_y, ribbon.rotation_y);
        assert_eq!(ripples.rotation_y, ribbon.rotation_y);

        let lip_mesh = (*lip).build_mesh();
        let lip_points = positions(&lip_mesh);
        assert!(axis_range(lip_points, 0) > width);
        assert!(axis_range(lip_points, 1) > height * 0.30);
        assert!(axis_range(lip_points, 2) > width * 0.25);

        let ripple_mesh = (*ripples).build_mesh();
        let ripple_points = positions(&ripple_mesh);
        assert!(axis_range(ripple_points, 0) > width * 1.65);
        assert!(axis_range(ripple_points, 2) > width * 1.10);
    }
}
