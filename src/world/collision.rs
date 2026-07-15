use crate::movement::{FlightController, FlightState};
use bevy::prelude::*;

use super::{ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT, SkyIsland, TERRAIN_MAX_RISE_M};

const PLAYER_COLLISION_RADIUS_M: f32 = 0.42;
const PLAYER_COLLISION_HEIGHT_M: f32 = 1.85;
const MAX_COLLISION_CORRECTION_PER_STEP_M: f32 = 0.5;
const BASE_COLLISION_SKIN_M: f32 = 0.002;
const TERRAIN_SIDE_COLLISION_RADIUS_M: f32 = 0.24;
const TERRAIN_SIDE_SURFACE_CLEARANCE_M: f32 = 0.55;
const TERRAIN_SIDE_COLLISION_BAND_M: f32 = TERRAIN_SIDE_COLLISION_RADIUS_M + 0.85;
const TERRAIN_CONTACT_PROBE_OUTSET_M: f32 = 0.12;
const TERRAIN_CONTACT_PROBE_DEPTH_M: f32 = TERRAIN_SIDE_SURFACE_CLEARANCE_M + 0.18;
pub const TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND: usize = 32;
pub const TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND: usize = ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT;
pub const TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND: usize = 4;
const TERRAIN_SIDE_CONTOUR_DISTANCE_SAMPLES: usize = ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT * 8;
const TERRAIN_COLLISION_TRUTH_TOP_EDGE_OUTSET_M: f32 = 0.35;
const TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_OFFSETS_M: [f32; 6] =
    [-0.70, -0.35, -0.12, 0.0, 0.12, 0.35];
const TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_VERTICAL_OFFSETS_M: [f32; 3] = [0.04, -0.18, -0.42];
const TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_OUTSETS_M: [f32; 3] = [0.06, 0.18, 0.32];
const TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_VERTICAL_OFFSETS_M: [f32; 3] = [-0.22, 0.0, 0.22];
pub const TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_PROBES_PER_CONTOUR_SAMPLE: usize = 18;
pub const TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_PROBES_PER_CONTOUR_SAMPLE: usize = 9;
const TERRAIN_COLLISION_TRUTH_NEAR_CLIFF_OUTSET_M: f32 = 0.08;
const TERRAIN_COLLISION_TRUTH_NEAR_CLIFF_DEPTH_M: f32 = TERRAIN_SIDE_SURFACE_CLEARANCE_M + 0.18;
const TERRAIN_COLLISION_TRUTH_FAR_FIELD_DEPTH_M: f32 = TERRAIN_SIDE_SURFACE_CLEARANCE_M + 0.55;
const TERRAIN_COLLISION_TRUTH_FAR_FIELD_OUTSET_M: f32 = TERRAIN_SIDE_COLLISION_BAND_M + 0.55;
const TERRAIN_COLLISION_TRUTH_FAR_FIELD_MIN_CLEARANCE_M: f32 = TERRAIN_SIDE_COLLISION_BAND_M + 0.50;
const TERRAIN_COLLISION_TRUTH_FAR_FIELD_OUTSET_STEP_M: f32 = 0.5;
const TERRAIN_COLLISION_TRUTH_FAR_FIELD_MAX_OUTSET_M: f32 = TERRAIN_SIDE_COLLISION_BAND_M + 8.0;
pub const TERRAIN_COLLISION_TRUTH_MAX_NEAR_CLIFF_PUSH_M: f32 = 0.24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldCollisionProxyKind {
    TerrainRim,
    TerrainBody,
    Tree,
    Rock,
    Landmark,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct WorldCollisionProxy {
    pub center: Vec3,
    pub half_extents: Vec3,
    pub kind: WorldCollisionProxyKind,
    terrain_side_clip: Option<SkyIsland>,
}

impl WorldCollisionProxy {
    pub fn new(center: Vec3, half_extents: Vec3, kind: WorldCollisionProxyKind) -> Self {
        Self {
            center,
            half_extents: half_extents.abs(),
            kind,
            terrain_side_clip: None,
        }
    }

    fn with_terrain_side_clip(mut self, island: SkyIsland) -> Self {
        self.terrain_side_clip = Some(island);
        self
    }
}

pub fn terrain_rim_collision_proxies(
    island: SkyIsland,
) -> [WorldCollisionProxy; TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND] {
    std::array::from_fn(|segment| terrain_rim_collision_proxy(island, segment))
}

pub fn terrain_body_collision_proxies(
    island: SkyIsland,
) -> [WorldCollisionProxy; TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND] {
    std::array::from_fn(|segment| terrain_body_collision_proxy(island, segment))
}

pub fn terrain_collision_contact_probe_position(
    island: SkyIsland,
    kind: WorldCollisionProxyKind,
    preferred_outward: Vec2,
) -> Option<Vec3> {
    if !matches!(
        kind,
        WorldCollisionProxyKind::TerrainRim | WorldCollisionProxyKind::TerrainBody
    ) {
        return None;
    }

    let preferred_outward = preferred_outward.normalize_or_zero();
    let proxies = terrain_body_collision_proxies(island)
        .into_iter()
        .chain(terrain_rim_collision_proxies(island))
        .collect::<Vec<_>>();
    let mut best_probe = None;
    let mut best_alignment = f32::NEG_INFINITY;

    for sample in 0..TERRAIN_SIDE_CONTOUR_DISTANCE_SAMPLES {
        let angle =
            sample as f32 / TERRAIN_SIDE_CONTOUR_DISTANCE_SAMPLES as f32 * std::f32::consts::TAU;
        let contour = island.footprint_contour_point(angle, false);
        let Some(outward) = playable_contour_outward_normal(island, angle) else {
            continue;
        };
        let horizontal = contour + outward * TERRAIN_CONTACT_PROBE_OUTSET_M;
        let mut probe = Vec3::new(horizontal.x, 0.0, horizontal.y);
        probe.y = island.terrain_surface_y_at(probe) - TERRAIN_CONTACT_PROBE_DEPTH_M;
        let resolution = resolve_world_collisions(
            FlightState::new(probe, Vec3::ZERO, FlightController::default()),
            proxies.iter().copied(),
        );
        let matches_kind = match kind {
            WorldCollisionProxyKind::TerrainRim => {
                resolution.terrain_rim_hit_count > 0 && resolution.terrain_body_hit_count == 0
            }
            WorldCollisionProxyKind::TerrainBody => resolution.terrain_body_hit_count > 0,
            WorldCollisionProxyKind::Tree
            | WorldCollisionProxyKind::Rock
            | WorldCollisionProxyKind::Landmark => false,
        };
        let alignment = outward.dot(preferred_outward);
        if matches_kind && alignment > best_alignment {
            best_probe = Some(probe);
            best_alignment = alignment;
        }
    }

    best_probe
}

fn terrain_rim_collision_proxy(island: SkyIsland, segment: usize) -> WorldCollisionProxy {
    let half_depth = 4.0;
    let half_height = (island.thickness * 0.5).max(1.2);
    let center_y = island.floor_y() - half_height + 0.08;
    let angle0 =
        segment as f32 / TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND as f32 * std::f32::consts::TAU;
    let angle1 = (segment + 1) as f32 / TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND as f32
        * std::f32::consts::TAU;
    let start = island.footprint_contour_point(angle0, false);
    let end = island.footprint_contour_point(angle1, false);
    let midpoint = (start + end) * 0.5;
    let island_center = Vec2::new(island.center.x, island.center.z);
    let outward = (midpoint - island_center).normalize_or_zero();
    let center = midpoint + outward * half_depth;
    let chord = end - start;
    let horizontal_padding =
        Vec2::new(outward.x.abs(), outward.y.abs()) * half_depth + Vec2::splat(3.0);
    let half_extents = Vec3::new(
        (chord.x.abs() * 0.5 + horizontal_padding.x)
            .max(2.0)
            .max(island.half_extents.x),
        half_height,
        (chord.y.abs() * 0.5 + horizontal_padding.y)
            .max(2.0)
            .max(island.half_extents.y),
    );

    WorldCollisionProxy::new(
        Vec3::new(center.x, center_y, center.y),
        half_extents,
        WorldCollisionProxyKind::TerrainRim,
    )
    .with_terrain_side_clip(island)
}

fn terrain_body_collision_proxy(island: SkyIsland, segment: usize) -> WorldCollisionProxy {
    let half_depth = (island.half_extents.min_element() * 0.15).clamp(2.4, 5.2);
    let half_height = (island.thickness * 0.5).max(1.2);
    let top_y = island.floor_y() + TERRAIN_MAX_RISE_M + 0.04;
    let center_y = top_y - half_height;
    let angle =
        segment as f32 / TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND as f32 * std::f32::consts::TAU;
    let contour = island.footprint_contour_point(angle, false);
    let island_center = Vec2::new(island.center.x, island.center.z);
    let outward = (contour - island_center).normalize_or_zero();
    let tangent = Vec2::new(-outward.y, outward.x);
    let center = contour - outward * 1.1;
    let tangent_span =
        (island.half_extents.x * tangent.x.abs() + island.half_extents.y * tangent.y.abs()) * 0.68;
    let horizontal_padding = Vec2::new(outward.x.abs(), outward.y.abs()) * half_depth
        + Vec2::new(tangent.x.abs(), tangent.y.abs()) * tangent_span
        + Vec2::splat(0.36);
    let half_extents = Vec3::new(
        horizontal_padding.x.max(0.8),
        half_height,
        horizontal_padding.y.max(0.8),
    );

    WorldCollisionProxy::new(
        Vec3::new(center.x, center_y, center.y),
        half_extents,
        WorldCollisionProxyKind::TerrainBody,
    )
    .with_terrain_side_clip(island)
}

#[derive(Clone, Copy, Debug)]
pub struct WorldCollisionResolution {
    pub state: FlightState,
    pub hit_count: usize,
    pub terrain_rim_hit_count: usize,
    pub terrain_body_hit_count: usize,
    pub correction_distance_m: f32,
    pub max_push_m: f32,
    pub max_terrain_rim_push_m: f32,
    pub max_terrain_body_push_m: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TerrainCollisionTruthReport {
    pub island_count: usize,
    pub contour_sample_count: usize,
    pub top_edge_probe_count: usize,
    pub top_edge_air_barrier_count: usize,
    pub edge_traverse_probe_count: usize,
    pub edge_traverse_barrier_count: usize,
    pub walkoff_shoulder_probe_count: usize,
    pub walkoff_shoulder_barrier_count: usize,
    pub far_field_probe_count: usize,
    pub far_field_hit_count: usize,
    pub near_cliff_probe_count: usize,
    pub near_cliff_miss_count: usize,
    pub excessive_near_cliff_push_count: usize,
    pub max_top_edge_push_m: f32,
    pub max_edge_traverse_push_m: f32,
    pub max_walkoff_shoulder_push_m: f32,
    pub max_far_field_push_m: f32,
    pub max_near_cliff_push_m: f32,
}

pub fn terrain_collision_truth_report(islands: &[SkyIsland]) -> TerrainCollisionTruthReport {
    debug_assert_eq!(
        TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_PROBES_PER_CONTOUR_SAMPLE,
        TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_OUTSETS_M.len()
            * TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_VERTICAL_OFFSETS_M.len()
    );
    debug_assert_eq!(
        TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_PROBES_PER_CONTOUR_SAMPLE,
        TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_OFFSETS_M.len()
            * TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_VERTICAL_OFFSETS_M.len()
    );
    let mut report = TerrainCollisionTruthReport {
        island_count: islands.len(),
        contour_sample_count: TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND,
        ..default()
    };

    for island in islands.iter().copied() {
        let proxies = terrain_body_collision_proxies(island)
            .into_iter()
            .chain(terrain_rim_collision_proxies(island))
            .collect::<Vec<_>>();

        for sample in 0..TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND {
            let angle = sample as f32 / TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND as f32
                * std::f32::consts::TAU;
            let contour = island.footprint_contour_point(angle, false);
            let island_center = Vec2::new(island.center.x, island.center.z);
            let outward = playable_contour_outward_normal(island, angle)
                .unwrap_or_else(|| (contour - island_center).normalize_or_zero());
            if outward == Vec2::ZERO {
                continue;
            }

            let top_edge_position = terrain_truth_probe_position(
                island,
                contour,
                outward,
                TERRAIN_COLLISION_TRUTH_TOP_EDGE_OUTSET_M,
                0.02,
            );
            let top_edge_resolution = resolve_world_collisions(
                FlightState::new(
                    top_edge_position,
                    Vec3::new(outward.x, 0.0, outward.y),
                    FlightController::default(),
                ),
                proxies.iter().copied(),
            );
            report.top_edge_probe_count += 1;
            report.max_top_edge_push_m = report
                .max_top_edge_push_m
                .max(top_edge_resolution.max_push_m);
            if top_edge_resolution.terrain_rim_hit_count
                + top_edge_resolution.terrain_body_hit_count
                > 0
            {
                report.top_edge_air_barrier_count += 1;
            }

            for traverse_offset_m in TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_OFFSETS_M {
                for vertical_offset_m in TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_VERTICAL_OFFSETS_M {
                    let traverse_position = terrain_truth_probe_position(
                        island,
                        contour,
                        outward,
                        traverse_offset_m,
                        vertical_offset_m,
                    );
                    let traverse_resolution = resolve_world_collisions(
                        FlightState::new(
                            traverse_position,
                            Vec3::new(outward.x * 3.0, 0.0, outward.y * 3.0),
                            FlightController::default(),
                        ),
                        proxies.iter().copied(),
                    );
                    report.edge_traverse_probe_count += 1;
                    report.max_edge_traverse_push_m = report
                        .max_edge_traverse_push_m
                        .max(traverse_resolution.max_push_m);
                    if traverse_resolution.terrain_rim_hit_count
                        + traverse_resolution.terrain_body_hit_count
                        > 0
                    {
                        report.edge_traverse_barrier_count += 1;
                    }
                }
            }

            for walkoff_outset_m in TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_OUTSETS_M {
                for vertical_offset_m in TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_VERTICAL_OFFSETS_M
                {
                    let walkoff_position = terrain_truth_probe_position(
                        island,
                        contour,
                        outward,
                        walkoff_outset_m,
                        vertical_offset_m,
                    );
                    let walkoff_resolution = resolve_world_collisions(
                        FlightState::new(
                            walkoff_position,
                            Vec3::new(outward.x, 0.0, outward.y),
                            FlightController::default(),
                        ),
                        proxies.iter().copied(),
                    );
                    report.walkoff_shoulder_probe_count += 1;
                    report.max_walkoff_shoulder_push_m = report
                        .max_walkoff_shoulder_push_m
                        .max(walkoff_resolution.max_push_m);
                    if walkoff_resolution.terrain_rim_hit_count
                        + walkoff_resolution.terrain_body_hit_count
                        > 0
                    {
                        report.walkoff_shoulder_barrier_count += 1;
                    }
                }
            }

            let far_field_position = terrain_truth_far_field_probe_position(
                island,
                contour,
                outward,
                -TERRAIN_COLLISION_TRUTH_FAR_FIELD_DEPTH_M,
            );
            let far_field_resolution = resolve_world_collisions(
                FlightState::new(
                    far_field_position,
                    Vec3::new(-outward.x, 0.0, -outward.y),
                    FlightController::default(),
                ),
                proxies.iter().copied(),
            );
            report.far_field_probe_count += 1;
            report.max_far_field_push_m = report
                .max_far_field_push_m
                .max(far_field_resolution.max_push_m);
            if far_field_resolution.terrain_rim_hit_count
                + far_field_resolution.terrain_body_hit_count
                > 0
            {
                report.far_field_hit_count += 1;
            }

            let near_cliff_position = terrain_truth_near_cliff_probe_position(
                island,
                contour,
                outward,
                -TERRAIN_COLLISION_TRUTH_NEAR_CLIFF_DEPTH_M,
            );
            let near_cliff_resolution = resolve_world_collisions(
                FlightState::new(
                    near_cliff_position,
                    Vec3::new(-outward.x * 4.0, 0.0, -outward.y * 4.0),
                    FlightController::default(),
                ),
                proxies.iter().copied(),
            );
            report.near_cliff_probe_count += 1;
            report.max_near_cliff_push_m = report
                .max_near_cliff_push_m
                .max(near_cliff_resolution.max_push_m);
            if near_cliff_resolution.terrain_rim_hit_count
                + near_cliff_resolution.terrain_body_hit_count
                == 0
            {
                report.near_cliff_miss_count += 1;
            }
            if near_cliff_resolution.max_push_m > TERRAIN_COLLISION_TRUTH_MAX_NEAR_CLIFF_PUSH_M {
                report.excessive_near_cliff_push_count += 1;
            }
        }
    }

    report
}

fn terrain_truth_far_field_probe_position(
    island: SkyIsland,
    contour: Vec2,
    outward: Vec2,
    vertical_offset_m: f32,
) -> Vec3 {
    let mut outset = TERRAIN_COLLISION_TRUTH_FAR_FIELD_OUTSET_M;
    loop {
        let position =
            terrain_truth_probe_position(island, contour, outward, outset, vertical_offset_m);
        if distance_to_playable_contour_m(island, position)
            > TERRAIN_COLLISION_TRUTH_FAR_FIELD_MIN_CLEARANCE_M
            || outset >= TERRAIN_COLLISION_TRUTH_FAR_FIELD_MAX_OUTSET_M
        {
            return position;
        }
        outset += TERRAIN_COLLISION_TRUTH_FAR_FIELD_OUTSET_STEP_M;
    }
}

fn terrain_truth_near_cliff_probe_position(
    island: SkyIsland,
    contour: Vec2,
    outward: Vec2,
    vertical_offset_m: f32,
) -> Vec3 {
    let mut position = terrain_truth_probe_position(
        island,
        contour,
        outward,
        TERRAIN_COLLISION_TRUTH_NEAR_CLIFF_OUTSET_M,
        vertical_offset_m,
    );

    for _ in 0..12 {
        let Some(nearest_projection) = playable_contour_projection(island, position) else {
            return position;
        };
        position = terrain_truth_probe_position(
            island,
            nearest_projection.point,
            nearest_projection.outward,
            TERRAIN_COLLISION_TRUTH_NEAR_CLIFF_OUTSET_M,
            vertical_offset_m,
        );
    }

    position
}

fn terrain_truth_probe_position(
    island: SkyIsland,
    contour: Vec2,
    outward: Vec2,
    outward_offset_m: f32,
    vertical_offset_m: f32,
) -> Vec3 {
    let horizontal = contour + outward * outward_offset_m;
    let mut position = Vec3::new(horizontal.x, 0.0, horizontal.y);
    position.y = island.terrain_surface_y_at(position) + vertical_offset_m;
    position
}

pub fn resolve_world_collisions(
    mut state: FlightState,
    proxies: impl IntoIterator<Item = WorldCollisionProxy>,
) -> WorldCollisionResolution {
    let mut proxies = proxies.into_iter().collect::<Vec<_>>();
    proxies.sort_by_key(|proxy| collision_resolution_priority(proxy.kind));

    let mut hit_count = 0;
    let mut terrain_rim_hit_count = 0;
    let mut terrain_body_hit_count = 0;
    let mut correction_distance_m = 0.0_f32;
    let mut max_push_m = 0.0_f32;
    let mut max_terrain_rim_push_m = 0.0_f32;
    let mut max_terrain_body_push_m = 0.0_f32;
    let mut resolved_terrain_side_islands = Vec::new();
    let mut remaining_correction_m = MAX_COLLISION_CORRECTION_PER_STEP_M;

    for proxy in proxies {
        if remaining_correction_m <= f32::EPSILON {
            break;
        }
        if skips_landing_recovery_collision(proxy.kind, state.controller.landing_recovery_timer) {
            continue;
        }
        let terrain_side_island_name = proxy.terrain_side_clip.and_then(|island| {
            matches!(
                proxy.kind,
                WorldCollisionProxyKind::TerrainRim | WorldCollisionProxyKind::TerrainBody
            )
            .then_some(island.name)
        });
        if terrain_side_island_name
            .is_some_and(|island_name| resolved_terrain_side_islands.contains(&island_name))
        {
            continue;
        }
        let Some((normal, push_m)) = player_proxy_push_out(state.position, proxy) else {
            continue;
        };
        let push_m = push_m.min(remaining_correction_m);

        state.position += normal * push_m;
        remaining_correction_m -= push_m;
        correction_distance_m += push_m;
        let inward_speed = state.velocity.dot(normal);
        if inward_speed < 0.0 {
            state.velocity -= normal * inward_speed;
        }
        hit_count += 1;
        max_push_m = max_push_m.max(push_m);
        if let Some(island_name) = terrain_side_island_name {
            resolved_terrain_side_islands.push(island_name);
        }
        if proxy.kind == WorldCollisionProxyKind::TerrainRim {
            terrain_rim_hit_count += 1;
            max_terrain_rim_push_m = max_terrain_rim_push_m.max(push_m);
        } else if proxy.kind == WorldCollisionProxyKind::TerrainBody {
            terrain_body_hit_count += 1;
            max_terrain_body_push_m = max_terrain_body_push_m.max(push_m);
        }
    }

    WorldCollisionResolution {
        state,
        hit_count,
        terrain_rim_hit_count,
        terrain_body_hit_count,
        correction_distance_m,
        max_push_m,
        max_terrain_rim_push_m,
        max_terrain_body_push_m,
    }
}

fn collision_resolution_priority(kind: WorldCollisionProxyKind) -> u8 {
    match kind {
        WorldCollisionProxyKind::TerrainBody
        | WorldCollisionProxyKind::Tree
        | WorldCollisionProxyKind::Rock
        | WorldCollisionProxyKind::Landmark => 0,
        WorldCollisionProxyKind::TerrainRim => 1,
    }
}

fn skips_landing_recovery_collision(
    kind: WorldCollisionProxyKind,
    landing_recovery_timer: f32,
) -> bool {
    landing_recovery_timer > 0.0
        && matches!(
            kind,
            WorldCollisionProxyKind::TerrainRim | WorldCollisionProxyKind::TerrainBody
        )
}

fn player_proxy_push_out(position: Vec3, proxy: WorldCollisionProxy) -> Option<(Vec3, f32)> {
    let player_min_y = position.y;
    let player_max_y = position.y + PLAYER_COLLISION_HEIGHT_M;
    let proxy_min = proxy.center - proxy.half_extents;
    let proxy_max = proxy.center + proxy.half_extents;
    if player_max_y < proxy_min.y || player_min_y > proxy_max.y {
        return None;
    }
    if skips_terrain_side_collision(position, proxy) {
        return None;
    }

    let min_x = proxy_min.x - PLAYER_COLLISION_RADIUS_M;
    let max_x = proxy_max.x + PLAYER_COLLISION_RADIUS_M;
    let min_z = proxy_min.z - PLAYER_COLLISION_RADIUS_M;
    let max_z = proxy_max.z + PLAYER_COLLISION_RADIUS_M;
    if position.x < min_x || position.x > max_x || position.z < min_z || position.z > max_z {
        return None;
    }
    if proxy.terrain_side_clip.is_some() {
        return terrain_side_push_out(position, proxy);
    }

    let exits = [
        (Vec3::NEG_X, position.x - min_x),
        (Vec3::X, max_x - position.x),
        (Vec3::NEG_Z, position.z - min_z),
        (Vec3::Z, max_z - position.z),
    ];
    exits
        .into_iter()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(normal, distance)| (normal, distance.max(0.0) + collision_skin_m(proxy.kind)))
}

fn collision_skin_m(kind: WorldCollisionProxyKind) -> f32 {
    match kind {
        WorldCollisionProxyKind::TerrainRim | WorldCollisionProxyKind::TerrainBody => {
            BASE_COLLISION_SKIN_M * 2.0
        }
        WorldCollisionProxyKind::Tree | WorldCollisionProxyKind::Rock => BASE_COLLISION_SKIN_M,
        WorldCollisionProxyKind::Landmark => BASE_COLLISION_SKIN_M * 1.5,
    }
}

fn skips_terrain_side_collision(position: Vec3, proxy: WorldCollisionProxy) -> bool {
    if !matches!(
        proxy.kind,
        WorldCollisionProxyKind::TerrainRim | WorldCollisionProxyKind::TerrainBody
    ) {
        return false;
    }

    let Some(island) = proxy.terrain_side_clip else {
        return false;
    };

    let surface_y = island.terrain_surface_y_at(position);
    if position.y >= surface_y - TERRAIN_SIDE_SURFACE_CLEARANCE_M {
        return true;
    }

    distance_to_playable_contour_m(island, position) > TERRAIN_SIDE_COLLISION_BAND_M
}

fn terrain_side_push_out(position: Vec3, proxy: WorldCollisionProxy) -> Option<(Vec3, f32)> {
    let island = proxy.terrain_side_clip?;
    let projection = playable_contour_projection(island, position)?;
    let signed_offset_from_contour = signed_offset_from_projection(position, projection);
    if !signed_offset_from_contour.is_finite() {
        return None;
    }

    let penetration = TERRAIN_SIDE_COLLISION_RADIUS_M - signed_offset_from_contour;
    if penetration <= 0.0 {
        return None;
    }

    Some((
        Vec3::new(projection.outward.x, 0.0, projection.outward.y),
        penetration + collision_skin_m(proxy.kind),
    ))
}

fn signed_offset_from_projection(position: Vec3, projection: PlayableContourProjection) -> f32 {
    let horizontal_position = Vec2::new(position.x, position.z);
    (horizontal_position - projection.point).dot(projection.outward)
}

fn distance_to_playable_contour_m(island: SkyIsland, position: Vec3) -> f32 {
    playable_contour_projection(island, position)
        .map(|projection| projection.distance_m)
        .unwrap_or(f32::INFINITY)
}

#[derive(Clone, Copy, Debug)]
struct PlayableContourProjection {
    point: Vec2,
    outward: Vec2,
    distance_m: f32,
}

fn playable_contour_projection(
    island: SkyIsland,
    position: Vec3,
) -> Option<PlayableContourProjection> {
    let horizontal_position = Vec2::new(position.x, position.z);
    let dx = (position.x - island.center.x) / island.half_extents.x.max(0.001);
    let dz = (position.z - island.center.z) / island.half_extents.y.max(0.001);
    let radial_angle = dz.atan2(dx);
    let mut closest = island.footprint_contour_point(radial_angle, false);
    let mut closest_angle = radial_angle;
    let mut closest_distance_sq = horizontal_position.distance_squared(closest);

    for sample in 0..TERRAIN_SIDE_CONTOUR_DISTANCE_SAMPLES {
        let angle =
            sample as f32 / TERRAIN_SIDE_CONTOUR_DISTANCE_SAMPLES as f32 * std::f32::consts::TAU;
        let contour = island.footprint_contour_point(angle, false);
        let distance_sq = horizontal_position.distance_squared(contour);
        if distance_sq < closest_distance_sq {
            closest = contour;
            closest_angle = angle;
            closest_distance_sq = distance_sq;
        }
    }

    let outward = playable_contour_outward_normal(island, closest_angle)?;

    Some(PlayableContourProjection {
        point: closest,
        outward,
        distance_m: closest_distance_sq.sqrt(),
    })
}

fn playable_contour_outward_normal(island: SkyIsland, angle: f32) -> Option<Vec2> {
    let delta = std::f32::consts::TAU / TERRAIN_SIDE_CONTOUR_DISTANCE_SAMPLES as f32;
    let previous = island.footprint_contour_point(angle - delta, false);
    let next = island.footprint_contour_point(angle + delta, false);
    let tangent = next - previous;
    let mut outward = Vec2::new(tangent.y, -tangent.x).normalize_or_zero();
    if outward == Vec2::ZERO {
        return None;
    }

    let contour = island.footprint_contour_point(angle, false);
    let radial = (contour - Vec2::new(island.center.x, island.center.z)).normalize_or_zero();
    if outward.dot(radial) < 0.0 {
        outward = -outward;
    }

    Some(outward)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement::FlightController;
    use crate::world::{START_FLOOR_Y, START_POSITION, SkyRoute};
    use std::collections::HashSet;

    #[test]
    fn collision_pushes_player_out_of_tree_proxy() {
        let state = FlightState::new(
            Vec3::new(0.2, 0.0, 0.0),
            Vec3::new(-4.0, 0.0, 0.0),
            FlightController::default(),
        );
        let proxy = WorldCollisionProxy::new(
            Vec3::new(0.0, 0.9, 0.0),
            Vec3::new(0.25, 0.9, 0.25),
            WorldCollisionProxyKind::Tree,
        );

        let resolution = resolve_world_collisions(state, [proxy]);

        assert_eq!(resolution.hit_count, 1);
        assert!(resolution.max_push_m > 0.0);
        assert!(resolution.state.position.x >= 0.25 + PLAYER_COLLISION_RADIUS_M);
        assert_eq!(resolution.state.velocity.x, 0.0);
    }

    #[test]
    fn collision_ignores_proxies_above_player_height() {
        let state = FlightState::new(
            Vec3::ZERO,
            Vec3::new(2.0, 0.0, 0.0),
            FlightController::default(),
        );
        let proxy = WorldCollisionProxy::new(
            Vec3::new(0.0, 4.0, 0.0),
            Vec3::splat(0.5),
            WorldCollisionProxyKind::Landmark,
        );

        let resolution = resolve_world_collisions(state, [proxy]);

        assert_eq!(resolution.hit_count, 0);
        assert_eq!(resolution.state.position, Vec3::ZERO);
        assert_eq!(resolution.state.velocity.x, 2.0);
    }

    #[test]
    fn deep_collision_correction_is_bounded_per_step() {
        let state = FlightState::new(
            Vec3::ZERO,
            Vec3::new(12.0, 0.0, 0.0),
            FlightController::default(),
        );
        let proxy = WorldCollisionProxy::new(
            Vec3::new(0.0, 0.9, 0.0),
            Vec3::new(4.0, 0.9, 4.0),
            WorldCollisionProxyKind::Landmark,
        );

        let resolution = resolve_world_collisions(state, [proxy]);

        assert_eq!(resolution.hit_count, 1);
        assert_eq!(resolution.max_push_m, MAX_COLLISION_CORRECTION_PER_STEP_M);
        assert!(
            resolution.state.position.distance(state.position)
                <= MAX_COLLISION_CORRECTION_PER_STEP_M
        );
    }

    #[test]
    fn terrain_rim_collision_pushes_side_contacts_without_blocking_top_surface() {
        let proxy = WorldCollisionProxy::new(
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::new(10.0, 5.0, 10.0),
            WorldCollisionProxyKind::TerrainRim,
        );
        let top_state = FlightState::new(
            Vec3::new(0.0, 10.2, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            FlightController::default(),
        );
        let top_resolution = resolve_world_collisions(top_state, [proxy]);

        assert_eq!(top_resolution.hit_count, 0);
        assert_eq!(top_resolution.terrain_rim_hit_count, 0);
        assert_eq!(top_resolution.state.position, top_state.position);

        let mut recovery_controller = FlightController::default();
        recovery_controller.record_landing_impact(12.0);
        let landing_recovery_state =
            FlightState::new(Vec3::new(9.8, 10.0, 0.0), Vec3::ZERO, recovery_controller);
        let landing_recovery_resolution = resolve_world_collisions(landing_recovery_state, [proxy]);

        assert_eq!(landing_recovery_resolution.hit_count, 0);
        assert_eq!(landing_recovery_resolution.terrain_rim_hit_count, 0);
        assert_eq!(
            landing_recovery_resolution.state.position,
            landing_recovery_state.position
        );

        let side_state = FlightState::new(
            Vec3::new(10.1, 9.0, 0.0),
            Vec3::new(-3.0, 0.0, 0.0),
            FlightController::default(),
        );
        let side_resolution = resolve_world_collisions(side_state, [proxy]);

        assert_eq!(side_resolution.hit_count, 1);
        assert_eq!(side_resolution.terrain_rim_hit_count, 1);
        assert!(side_resolution.max_terrain_rim_push_m > 0.0);
        assert!(side_resolution.state.position.x >= 10.0 + PLAYER_COLLISION_RADIUS_M);
        assert!(side_resolution.state.velocity.x.abs() < 0.0001);
    }

    #[test]
    fn terrain_body_collision_pushes_cliff_sides_without_blocking_top_surface() {
        let island = SkyIsland::new(
            "launch mesa",
            Vec3::new(0.0, START_FLOOR_Y, 0.0),
            Vec2::new(40.0, 32.0),
            11.0,
            false,
        );
        let proxies = terrain_body_collision_proxies(island);
        let top_state = FlightState::new(
            START_POSITION,
            Vec3::new(2.0, 0.0, 0.0),
            FlightController::default(),
        );
        let top_resolution = resolve_world_collisions(top_state, proxies);

        assert_eq!(top_resolution.hit_count, 0);
        assert_eq!(top_resolution.terrain_body_hit_count, 0);
        assert_eq!(top_resolution.state.position, top_state.position);

        let mut recovery_controller = FlightController::default();
        recovery_controller.record_landing_impact(12.0);
        let landing_recovery_state =
            FlightState::new(Vec3::new(0.0, 28.1, -23.5), Vec3::ZERO, recovery_controller);
        let landing_recovery_resolution = resolve_world_collisions(landing_recovery_state, proxies);

        assert_eq!(landing_recovery_resolution.hit_count, 0);
        assert_eq!(landing_recovery_resolution.terrain_body_hit_count, 0);
        assert_eq!(
            landing_recovery_resolution.state.position,
            landing_recovery_state.position
        );

        let side_proxy = proxies[0];
        let top_edge_position = Vec3::new(
            side_proxy.center.x - side_proxy.half_extents.x - PLAYER_COLLISION_RADIUS_M - 0.2,
            START_FLOOR_Y,
            island.center.z,
        );
        assert!(island.contains_horizontal(top_edge_position));
        let top_edge_state = FlightState::new(
            top_edge_position,
            Vec3::new(3.0, 0.0, 0.0),
            FlightController::default(),
        );
        let top_edge_resolution = resolve_world_collisions(top_edge_state, proxies);

        assert_eq!(top_edge_resolution.hit_count, 0);
        assert_eq!(top_edge_resolution.terrain_body_hit_count, 0);

        let side_contour = island.footprint_contour_point(0.0, false);
        let side_outward =
            playable_contour_outward_normal(island, 0.0).expect("side contour should have normal");
        let side_surface_sample = Vec3::new(
            side_contour.x + PLAYER_COLLISION_RADIUS_M * 0.5,
            island.floor_y(),
            side_contour.y,
        );
        let side_position = Vec3::new(
            side_surface_sample.x,
            island.terrain_surface_y_at(side_surface_sample)
                - TERRAIN_SIDE_SURFACE_CLEARANCE_M
                - 0.18,
            side_surface_sample.z,
        );
        let side_state = FlightState::new(
            side_position,
            Vec3::new(-3.0, 0.0, 0.0),
            FlightController::default(),
        );
        let side_resolution = resolve_world_collisions(side_state, proxies);

        assert!(side_resolution.hit_count >= 1);
        assert_eq!(side_resolution.terrain_rim_hit_count, 0);
        assert!(side_resolution.terrain_body_hit_count >= 1);
        assert!(side_resolution.max_terrain_body_push_m > 0.0);
        assert!(side_resolution.max_terrain_body_push_m < 0.3);
        let side_normal = Vec3::new(side_outward.x, 0.0, side_outward.y);
        assert!((side_resolution.state.position - side_state.position).dot(side_normal) > 0.0);
        assert!(side_resolution.state.velocity.dot(side_normal) >= -0.0001);
    }

    #[test]
    fn terrain_rim_collision_samples_full_footprint_contour() {
        let island = SkyIsland::new(
            "storm porch",
            Vec3::new(-74.0, START_FLOOR_Y, -548.0),
            Vec2::new(42.0, 28.0),
            15.0,
            false,
        );
        let proxies = terrain_rim_collision_proxies(island);
        let mut occupied_octants = HashSet::new();

        assert_eq!(proxies.len(), TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND);
        for proxy in proxies {
            assert_eq!(proxy.kind, WorldCollisionProxyKind::TerrainRim);
            assert!(proxy.half_extents.x > 0.42);
            assert!(proxy.half_extents.z > 0.42);
            let offset = Vec2::new(
                proxy.center.x - island.center.x,
                proxy.center.z - island.center.z,
            );
            assert!(offset.length() > island.half_extents.min_element() * 0.45);
            let octant = (offset.y.atan2(offset.x).rem_euclid(std::f32::consts::TAU)
                / std::f32::consts::TAU
                * 8.0)
                .round() as i32;
            occupied_octants.insert(octant);
        }

        assert!(occupied_octants.len() >= 7);
    }

    #[test]
    fn terrain_contact_probes_track_authored_rim_and_body_geometry() {
        let route = SkyRoute::default();
        let island = route
            .island_named("launch mesa")
            .expect("launch island should exist");
        let proxies = terrain_body_collision_proxies(island)
            .into_iter()
            .chain(terrain_rim_collision_proxies(island))
            .collect::<Vec<_>>();

        for (kind, preferred_outward) in [
            (WorldCollisionProxyKind::TerrainRim, Vec2::new(1.0, 0.75)),
            (WorldCollisionProxyKind::TerrainBody, Vec2::X),
        ] {
            let probe = terrain_collision_contact_probe_position(island, kind, preferred_outward)
                .expect("authored terrain should expose the requested contact lane");
            let resolution = resolve_world_collisions(
                FlightState::new(probe, Vec3::ZERO, FlightController::default()),
                proxies.iter().copied(),
            );

            match kind {
                WorldCollisionProxyKind::TerrainRim => {
                    assert_eq!(resolution.terrain_rim_hit_count, 1);
                    assert_eq!(resolution.terrain_body_hit_count, 0);
                    assert!(resolution.max_terrain_rim_push_m <= 0.15);
                }
                WorldCollisionProxyKind::TerrainBody => {
                    assert_eq!(resolution.terrain_body_hit_count, 1);
                    assert_eq!(resolution.terrain_rim_hit_count, 0);
                    assert!(resolution.max_terrain_body_push_m <= 0.15);
                }
                WorldCollisionProxyKind::Tree
                | WorldCollisionProxyKind::Rock
                | WorldCollisionProxyKind::Landmark => unreachable!(),
            }
            assert!(resolution.max_push_m >= 0.04);
        }
    }

    #[test]
    fn terrain_side_collision_does_not_create_top_edge_air_barriers() {
        let island = SkyIsland::new(
            "storm porch",
            Vec3::new(-74.0, START_FLOOR_Y, -548.0),
            Vec2::new(42.0, 28.0),
            15.0,
            false,
        );
        let proxies = terrain_body_collision_proxies(island)
            .into_iter()
            .chain(terrain_rim_collision_proxies(island))
            .collect::<Vec<_>>();

        for step in 0..32 {
            let angle = step as f32 / 32.0 * std::f32::consts::TAU;
            let contour = island.footprint_contour_point(angle, false);
            let outward = (contour - Vec2::new(island.center.x, island.center.z)).normalize();
            let position_2d = contour + outward * 0.35;
            let mut position = Vec3::new(position_2d.x, 0.0, position_2d.y);
            position.y = island.terrain_surface_y_at(position) + 0.02;
            let state = FlightState::new(position, Vec3::new(outward.x, 0.0, outward.y), default());

            let resolution = resolve_world_collisions(state, proxies.iter().copied());

            assert_eq!(
                resolution.terrain_rim_hit_count, 0,
                "rim collision should not block walkoff airspace at angle {angle}"
            );
            assert_eq!(
                resolution.terrain_body_hit_count, 0,
                "body collision should not block walkoff airspace at angle {angle}"
            );
            assert_eq!(resolution.state.position, state.position);
            assert_eq!(resolution.state.velocity, state.velocity);
        }
    }

    #[test]
    fn terrain_side_collision_stays_close_to_visible_cliff_shell() {
        let island = SkyIsland::new(
            "launch mesa",
            Vec3::new(0.0, START_FLOOR_Y, 0.0),
            Vec2::new(40.0, 32.0),
            11.0,
            false,
        );
        let proxies = terrain_body_collision_proxies(island)
            .into_iter()
            .chain(terrain_rim_collision_proxies(island))
            .collect::<Vec<_>>();
        let angle = 0.0;
        let contour = island.footprint_contour_point(angle, false);
        let outward = (contour - Vec2::new(island.center.x, island.center.z)).normalize();
        let contour_position = Vec3::new(contour.x, START_FLOOR_Y, contour.y);
        let surface_y = island.terrain_surface_y_at(contour_position);
        let near_position = Vec3::new(
            contour.x + outward.x * 0.08,
            surface_y - 1.1,
            contour.y + outward.y * 0.08,
        );
        let far_position = Vec3::new(
            contour.x + outward.x * (TERRAIN_SIDE_COLLISION_BAND_M + 0.55),
            surface_y - 1.1,
            contour.y + outward.y * (TERRAIN_SIDE_COLLISION_BAND_M + 0.55),
        );

        let near_resolution = resolve_world_collisions(
            FlightState::new(near_position, Vec3::new(-4.0, 0.0, 0.0), default()),
            proxies.iter().copied(),
        );
        let far_resolution = resolve_world_collisions(
            FlightState::new(far_position, Vec3::new(-4.0, 0.0, 0.0), default()),
            proxies.iter().copied(),
        );

        assert!(near_resolution.hit_count > 0);
        assert!(near_resolution.terrain_rim_hit_count + near_resolution.terrain_body_hit_count > 0);
        assert!(near_resolution.max_push_m < 0.4);
        assert_eq!(far_resolution.terrain_rim_hit_count, 0);
        assert_eq!(far_resolution.terrain_body_hit_count, 0);
        assert_eq!(far_resolution.state.position, far_position);
    }

    #[test]
    fn terrain_side_projection_uses_authored_contour_normals() {
        let route = SkyRoute::default();
        let island_names = [
            "storm porch",
            "underbridge cay",
            "cloud gate",
            "great sky plateau",
        ];
        let mut checked_projection_count = 0;

        for island_name in island_names {
            let island = route
                .island_named(island_name)
                .expect("authored island should exist");
            for step in 0..TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND {
                let angle = step as f32 / TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND as f32
                    * std::f32::consts::TAU;
                let contour = island.footprint_contour_point(angle, false);
                let Some(outward) = playable_contour_outward_normal(island, angle) else {
                    continue;
                };

                let outside = Vec3::new(
                    contour.x + outward.x * 0.18,
                    island.floor_y(),
                    contour.y + outward.y * 0.18,
                );
                let inside = Vec3::new(
                    contour.x - outward.x * 0.18,
                    island.floor_y(),
                    contour.y - outward.y * 0.18,
                );
                let outside_projection = playable_contour_projection(island, outside)
                    .expect("outside shoulder should project to contour");
                let inside_projection = playable_contour_projection(island, inside)
                    .expect("inside shoulder should project to contour");

                assert!(
                    signed_offset_from_projection(outside, outside_projection) > 0.05,
                    "{island_name} outside shoulder should remain outside at angle {angle}"
                );
                assert!(
                    signed_offset_from_projection(inside, inside_projection) < -0.05,
                    "{island_name} inside shoulder should remain inside at angle {angle}"
                );
                assert!(
                    outside_projection.outward.dot(outward) > 0.76,
                    "{island_name} projected normal should follow authored contour at angle {angle}"
                );
                checked_projection_count += 1;
            }
        }

        assert!(
            checked_projection_count
                >= island_names.len() * TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND - 2,
            "authored cove contour projection should cover all sampled island shoulders"
        );
    }

    #[test]
    fn route_terrain_collision_allows_grounded_rim_walks() {
        let route = SkyRoute::default();
        let horizontal_insets_m = [0.12_f32, 0.35, 0.70];
        let vertical_offsets_m = [0.02_f32, -0.24, -0.50];
        let mut checked_probe_count = 0;

        for island in route.islands().iter().copied() {
            let proxies = terrain_body_collision_proxies(island)
                .into_iter()
                .chain(terrain_rim_collision_proxies(island))
                .collect::<Vec<_>>();

            for step in 0..TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND {
                let angle = step as f32 / TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND as f32
                    * std::f32::consts::TAU;
                let contour = island.footprint_contour_point(angle, false);
                let island_center = Vec2::new(island.center.x, island.center.z);
                let outward = (contour - island_center).normalize_or_zero();
                if outward == Vec2::ZERO {
                    continue;
                }

                for inset_m in horizontal_insets_m {
                    let horizontal = contour - outward * inset_m;
                    let mut position = Vec3::new(horizontal.x, 0.0, horizontal.y);
                    if !island.contains_horizontal(position) {
                        continue;
                    }

                    let surface_y = island.terrain_surface_y_at(position);
                    for vertical_offset_m in vertical_offsets_m {
                        checked_probe_count += 1;
                        position.y = surface_y + vertical_offset_m;
                        let state = FlightState::new(
                            position,
                            Vec3::new(outward.x * 3.0, 0.0, outward.y * 3.0),
                            default(),
                        );

                        let resolution = resolve_world_collisions(state, proxies.iter().copied());

                        assert_eq!(
                            resolution.terrain_rim_hit_count, 0,
                            "{} rim collision should not block grounded edge walking at angle {angle}, inset {inset_m}, vertical offset {vertical_offset_m}",
                            island.name
                        );
                        assert_eq!(
                            resolution.terrain_body_hit_count, 0,
                            "{} body collision should not block grounded edge walking at angle {angle}, inset {inset_m}, vertical offset {vertical_offset_m}",
                            island.name
                        );
                        assert_eq!(resolution.state.position, state.position);
                        assert_eq!(resolution.state.velocity, state.velocity);
                    }
                }
            }
        }

        assert!(
            checked_probe_count
                >= route.islands().len()
                    * TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND
                    * vertical_offsets_m.len()
                    * 2,
            "rim-walk collision truth should cover most authored island edges"
        );
    }

    #[test]
    fn route_terrain_collision_truth_has_no_invisible_edge_barriers() {
        let route = SkyRoute::default();
        let report = terrain_collision_truth_report(route.islands());
        let expected_probe_count =
            route.islands().len() * TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND;
        let expected_walkoff_probe_count = expected_probe_count
            * TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_PROBES_PER_CONTOUR_SAMPLE;

        assert_eq!(report.island_count, route.islands().len());
        assert_eq!(
            report.contour_sample_count,
            TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND
        );
        assert_eq!(report.top_edge_probe_count, expected_probe_count);
        assert_eq!(
            report.edge_traverse_probe_count,
            expected_probe_count * TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_PROBES_PER_CONTOUR_SAMPLE
        );
        assert_eq!(
            report.walkoff_shoulder_probe_count,
            expected_walkoff_probe_count
        );
        assert_eq!(report.far_field_probe_count, expected_probe_count);
        assert_eq!(report.near_cliff_probe_count, expected_probe_count);
        assert_eq!(report.top_edge_air_barrier_count, 0);
        assert_eq!(report.edge_traverse_barrier_count, 0);
        assert_eq!(report.walkoff_shoulder_barrier_count, 0);
        assert_eq!(report.far_field_hit_count, 0);
        assert_eq!(report.near_cliff_miss_count, 0);
        assert_eq!(report.excessive_near_cliff_push_count, 0);
        assert_eq!(report.max_edge_traverse_push_m, 0.0);
        assert_eq!(report.max_walkoff_shoulder_push_m, 0.0);
        assert!(
            report.max_near_cliff_push_m <= TERRAIN_COLLISION_TRUTH_MAX_NEAR_CLIFF_PUSH_M,
            "{report:?}"
        );
    }
}
