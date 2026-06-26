use bevy::prelude::*;
use nau_engine::movement::FlightState;

const PLAYER_COLLISION_RADIUS_M: f32 = 0.42;
const PLAYER_COLLISION_HEIGHT_M: f32 = 1.85;
const BASE_COLLISION_SKIN_M: f32 = 0.002;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorldCollisionProxyKind {
    Tree,
    Rock,
    Landmark,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct WorldCollisionProxy {
    pub(crate) center: Vec3,
    pub(crate) half_extents: Vec3,
    pub(crate) kind: WorldCollisionProxyKind,
}

impl WorldCollisionProxy {
    pub(crate) fn new(center: Vec3, half_extents: Vec3, kind: WorldCollisionProxyKind) -> Self {
        Self {
            center,
            half_extents: half_extents.abs(),
            kind,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WorldCollisionResolution {
    pub(crate) state: FlightState,
    pub(crate) hit_count: usize,
    pub(crate) max_push_m: f32,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct WorldCollisionDiagnostics {
    pub(crate) proxy_count: usize,
    pub(crate) resolved_count: usize,
    pub(crate) max_push_m: f32,
}

pub(crate) fn resolve_world_collisions(
    mut state: FlightState,
    proxies: impl IntoIterator<Item = WorldCollisionProxy>,
) -> WorldCollisionResolution {
    let mut hit_count = 0;
    let mut max_push_m = 0.0_f32;

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
    }

    WorldCollisionResolution {
        state,
        hit_count,
        max_push_m,
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
        WorldCollisionProxyKind::Tree | WorldCollisionProxyKind::Rock => BASE_COLLISION_SKIN_M,
        WorldCollisionProxyKind::Landmark => BASE_COLLISION_SKIN_M * 1.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::movement::FlightController;

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
}
