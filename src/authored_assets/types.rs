use bevy::gltf::Gltf;
use bevy::prelude::*;
use nau_engine::asset_pipeline::{
    VisualAssetKind, VisualAssetLoadAdmission, VisualAssetPipelineMetrics, VisualAssetSpec,
};

#[derive(Resource, Debug)]
pub(crate) struct VisualAssetRegistry {
    pub(crate) slots: Vec<VisualAssetSlot>,
}

#[derive(Debug)]
pub(crate) struct VisualAssetSlot {
    pub(crate) spec: VisualAssetSpec,
    pub(crate) load_admission: VisualAssetLoadAdmission,
    pub(crate) gltf_handle: Option<Handle<Gltf>>,
    pub(crate) scene_handle: Option<Handle<Scene>>,
    pub(crate) scene_entity: Option<Entity>,
    pub(crate) scene_ready: bool,
    pub(crate) animation_player_entity: Option<Entity>,
    pub(crate) ready_animation_clip_count: usize,
    pub(crate) animation_graph_ready: bool,
}

#[derive(Clone)]
pub(crate) struct PendingAuthoredAnimationLink {
    pub(crate) kind: VisualAssetKind,
    pub(crate) spec: VisualAssetSpec,
    pub(crate) scene_entity: Entity,
    pub(crate) gltf_handle: Handle<Gltf>,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct VisualAssetDiagnostics {
    pub(crate) metrics: VisualAssetPipelineMetrics,
    pub(crate) visible_world_fixture_count: usize,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct AuthoredVisualScene {
    pub(crate) kind: VisualAssetKind,
    pub(crate) role: AuthoredVisualSceneRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AuthoredVisualSceneRole {
    PlayerRuntime,
    GliderRuntime,
    WorldFixture,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct VisibleAuthoredWorldFixture {
    pub(crate) kind: VisualAssetKind,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct GeneratedPlayerPlaceholder;
