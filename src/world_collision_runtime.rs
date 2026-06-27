use bevy::prelude::*;

pub(crate) use nau_engine::world::{
    WorldCollisionProxy, WorldCollisionProxyKind, resolve_world_collisions,
    terrain_rim_collision_proxies,
};

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct WorldCollisionDiagnostics {
    pub(crate) proxy_count: usize,
    pub(crate) terrain_rim_proxy_count: usize,
    pub(crate) solid_proxy_count: usize,
    pub(crate) tree_proxy_count: usize,
    pub(crate) rock_proxy_count: usize,
    pub(crate) landmark_proxy_count: usize,
    pub(crate) resolved_count: usize,
    pub(crate) terrain_rim_resolved_count: usize,
    pub(crate) max_push_m: f32,
    pub(crate) max_terrain_rim_push_m: f32,
}
