use crate::movement::FlightState;
use bevy::prelude::*;

use super::SkyIsland;

const PLAYER_COLLISION_RADIUS_M: f32 = 0.42;
const PLAYER_COLLISION_HEIGHT_M: f32 = 1.85;
const BASE_COLLISION_SKIN_M: f32 = 0.002;
pub const TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldCollisionProxyKind {
    TerrainRim,
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

pub fn terrain_rim_collision_proxies(island: SkyIsland) -> [WorldCollisionProxy; 4] {
    let half_depth = 0.55;
    let half_height = (island.thickness * 0.5).max(1.2);
    let center_y = island.floor_y() - half_height + 0.08;
    let east_extent = island.half_extents.x * island.playable_silhouette_scale(0.0);
    let west_extent =
        island.half_extents.x * island.playable_silhouette_scale(std::f32::consts::PI);
    let north_extent =
        island.half_extents.y * island.playable_silhouette_scale(std::f32::consts::FRAC_PI_2);
    let south_extent =
        island.half_extents.y * island.playable_silhouette_scale(-std::f32::consts::FRAC_PI_2);
    let x_span = island.half_extents.x * 0.72;
    let z_span = island.half_extents.y * 0.72;

    [
        WorldCollisionProxy::new(
            Vec3::new(
                island.center.x + east_extent + half_depth,
                center_y,
                island.center.z,
            ),
            Vec3::new(half_depth, half_height, z_span),
            WorldCollisionProxyKind::TerrainRim,
        ),
        WorldCollisionProxy::new(
            Vec3::new(
                island.center.x - west_extent - half_depth,
                center_y,
                island.center.z,
            ),
            Vec3::new(half_depth, half_height, z_span),
            WorldCollisionProxyKind::TerrainRim,
        ),
        WorldCollisionProxy::new(
            Vec3::new(
                island.center.x,
                center_y,
                island.center.z + north_extent + half_depth,
            ),
            Vec3::new(x_span, half_height, half_depth),
            WorldCollisionProxyKind::TerrainRim,
        ),
        WorldCollisionProxy::new(
            Vec3::new(
                island.center.x,
                center_y,
                island.center.z - south_extent - half_depth,
            ),
            Vec3::new(x_span, half_height, half_depth),
            WorldCollisionProxyKind::TerrainRim,
        ),
    ]
}

#[derive(Clone, Copy, Debug)]
pub struct WorldCollisionResolution {
    pub state: FlightState,
    pub hit_count: usize,
    pub terrain_rim_hit_count: usize,
    pub max_push_m: f32,
    pub max_terrain_rim_push_m: f32,
}

pub fn resolve_world_collisions(
    mut state: FlightState,
    proxies: impl IntoIterator<Item = WorldCollisionProxy>,
) -> WorldCollisionResolution {
    let mut hit_count = 0;
    let mut terrain_rim_hit_count = 0;
    let mut max_push_m = 0.0_f32;
    let mut max_terrain_rim_push_m = 0.0_f32;

    for proxy in proxies {
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
        }
    }

    WorldCollisionResolution {
        state,
        hit_count,
        terrain_rim_hit_count,
        max_push_m,
        max_terrain_rim_push_m,
    }
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
        WorldCollisionProxyKind::TerrainRim => BASE_COLLISION_SKIN_M * 2.0,
        WorldCollisionProxyKind::Tree | WorldCollisionProxyKind::Rock => BASE_COLLISION_SKIN_M,
        WorldCollisionProxyKind::Landmark => BASE_COLLISION_SKIN_M * 1.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement::FlightController;

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
}
