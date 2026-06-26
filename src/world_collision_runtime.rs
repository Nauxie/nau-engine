use bevy::prelude::*;

pub(crate) use nau_engine::world::{
    WorldCollisionProxy, WorldCollisionProxyKind, resolve_world_collisions,
};

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct WorldCollisionDiagnostics {
    pub(crate) proxy_count: usize,
    pub(crate) resolved_count: usize,
    pub(crate) max_push_m: f32,
}
