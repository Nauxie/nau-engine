use crate::movement::FlightState;
use bevy::prelude::*;

use super::{ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT, SkyIsland, TERRAIN_MAX_RISE_M};

const PLAYER_COLLISION_RADIUS_M: f32 = 0.42;
const PLAYER_COLLISION_HEIGHT_M: f32 = 1.85;
const BASE_COLLISION_SKIN_M: f32 = 0.002;
pub const TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND: usize = ISLAND_FOOTPRINT_CONTOUR_SAMPLE_COUNT;
pub const TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND: usize = 4;

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
}

impl WorldCollisionProxy {
    pub fn new(center: Vec3, half_extents: Vec3, kind: WorldCollisionProxyKind) -> Self {
        Self {
            center,
            half_extents: half_extents.abs(),
            kind,
        }
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

fn terrain_rim_collision_proxy(island: SkyIsland, segment: usize) -> WorldCollisionProxy {
    let half_depth = 0.55;
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
        Vec2::new(outward.x.abs(), outward.y.abs()) * half_depth + Vec2::splat(0.24);
    let half_extents = Vec3::new(
        (chord.x.abs() * 0.5 + horizontal_padding.x).max(0.42),
        half_height,
        (chord.y.abs() * 0.5 + horizontal_padding.y).max(0.42),
    );

    WorldCollisionProxy::new(
        Vec3::new(center.x, center_y, center.y),
        half_extents,
        WorldCollisionProxyKind::TerrainRim,
    )
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
}

#[derive(Clone, Copy, Debug)]
pub struct WorldCollisionResolution {
    pub state: FlightState,
    pub hit_count: usize,
    pub terrain_rim_hit_count: usize,
    pub terrain_body_hit_count: usize,
    pub max_push_m: f32,
    pub max_terrain_rim_push_m: f32,
    pub max_terrain_body_push_m: f32,
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
    let mut max_push_m = 0.0_f32;
    let mut max_terrain_rim_push_m = 0.0_f32;
    let mut max_terrain_body_push_m = 0.0_f32;

    for proxy in proxies {
        if skips_landing_recovery_collision(proxy.kind, state.controller.landing_recovery_timer) {
            continue;
        }
        let Some((normal, push_m)) = player_proxy_push_out(state.position, proxy) else {
            continue;
        };

        state.position += normal * push_m;
        let inward_speed = state.velocity.dot(normal);
        if inward_speed < 0.0 {
            state.velocity -= normal * inward_speed;
        }
        hit_count += 1;
        max_push_m = max_push_m.max(push_m);
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

    let min_x = proxy_min.x - PLAYER_COLLISION_RADIUS_M;
    let max_x = proxy_max.x + PLAYER_COLLISION_RADIUS_M;
    let min_z = proxy_min.z - PLAYER_COLLISION_RADIUS_M;
    let max_z = proxy_max.z + PLAYER_COLLISION_RADIUS_M;
    if position.x < min_x || position.x > max_x || position.z < min_z || position.z > max_z {
        return None;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement::FlightController;
    use crate::world::{START_FLOOR_Y, START_POSITION};
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
        assert_eq!(side_resolution.state.velocity.x, 0.0);
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

        let side_position = Vec3::new(
            side_proxy.center.x - side_proxy.half_extents.x - PLAYER_COLLISION_RADIUS_M * 0.5,
            28.1,
            island.center.z,
        );
        let side_state = FlightState::new(
            side_position,
            Vec3::new(3.0, 0.0, 0.0),
            FlightController::default(),
        );
        let side_resolution = resolve_world_collisions(side_state, proxies);

        assert!(side_resolution.hit_count >= 1);
        assert_eq!(side_resolution.terrain_rim_hit_count, 0);
        assert!(side_resolution.terrain_body_hit_count >= 1);
        assert!(side_resolution.max_terrain_body_push_m > 0.0);
        assert!(side_resolution.state.position.x < side_state.position.x);
        assert_eq!(side_resolution.state.velocity.x, 0.0);
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
}
