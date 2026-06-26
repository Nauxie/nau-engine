#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualAssetKind {
    PlayerCharacter,
    Glider,
    IslandTerrain,
    IslandFoliage,
    IslandRock,
    IslandWater,
    RouteMarker,
    WeatherLayer,
    DistantImpostor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualAssetResidency {
    Always,
    StreamWindow,
    NearLod,
    FarLod,
    Weather,
}

impl VisualAssetResidency {
    pub fn is_stream_managed(self) -> bool {
        matches!(
            self,
            Self::StreamWindow | Self::NearLod | Self::FarLod | Self::Weather
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualAssetSpec {
    pub kind: VisualAssetKind,
    pub label: &'static str,
    pub gltf_scene_path: &'static str,
    pub animation_clip_names: &'static [&'static str],
    pub residency: VisualAssetResidency,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualAssetLoadState {
    Missing,
    Deferred,
    Queued,
    Loading,
    Loaded,
    Failed,
}

impl VisualAssetLoadState {
    pub fn from_asset_exists(asset_exists: bool) -> Self {
        if asset_exists {
            Self::Queued
        } else {
            Self::Missing
        }
    }

    pub fn is_available(self) -> bool {
        matches!(self, Self::Queued | Self::Loading | Self::Loaded)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualAssetLoadPolicy {
    pub max_admitted_scene_count: usize,
    pub max_streaming_admitted_scene_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisualAssetLoadAdmission {
    Missing,
    Deferred,
    Admitted,
}

impl VisualAssetLoadAdmission {
    pub fn is_admitted(self) -> bool {
        matches!(self, Self::Admitted)
    }

    pub fn load_state(self) -> VisualAssetLoadState {
        match self {
            Self::Missing => VisualAssetLoadState::Missing,
            Self::Deferred => VisualAssetLoadState::Deferred,
            Self::Admitted => VisualAssetLoadState::Queued,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct VisualAssetPreloadState {
    pub dependencies_loaded: bool,
}

impl VisualAssetPreloadState {
    pub fn from_load_state(load_state: VisualAssetLoadState) -> Self {
        Self {
            dependencies_loaded: matches!(load_state, VisualAssetLoadState::Loaded),
        }
    }

    pub fn from_dependencies_loaded(dependencies_loaded: bool) -> Self {
        Self {
            dependencies_loaded,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VisualAssetSceneState {
    #[default]
    NotSpawned,
    Spawned,
    Ready,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct VisualAssetAnimationState {
    pub ready_clip_count: usize,
    pub animation_player_linked: bool,
    pub animation_graph_ready: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisualAssetPipelineMetrics {
    pub slot_count: usize,
    pub gltf_scene_slot_count: usize,
    pub ready_slot_count: usize,
    pub placeholder_slot_count: usize,
    pub streaming_slot_count: usize,
    pub missing_slot_count: usize,
    pub deferred_scene_count: usize,
    pub queued_scene_count: usize,
    pub loading_scene_count: usize,
    pub loaded_scene_count: usize,
    pub dependency_loaded_scene_count: usize,
    pub preload_ready_scene_count: usize,
    pub failed_scene_count: usize,
    pub spawned_scene_count: usize,
    pub ready_scene_count: usize,
    pub always_slot_count: usize,
    pub stream_window_slot_count: usize,
    pub near_lod_slot_count: usize,
    pub far_lod_slot_count: usize,
    pub weather_slot_count: usize,
    pub always_preload_ready_slot_count: usize,
    pub streaming_preload_ready_slot_count: usize,
    pub declared_animation_clip_count: usize,
    pub ready_animation_clip_count: usize,
    pub animation_player_count: usize,
    pub animation_graph_count: usize,
}
