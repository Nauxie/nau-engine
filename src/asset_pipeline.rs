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

pub const PLAYER_ANIMATION_CLIP_NAMES: &[&str] = &[
    "Idle_Loop",
    "Jog_Fwd_Loop",
    "Launch_Start",
    "Glide_Loop",
    "Air_Brake",
    "Land",
];

pub const VISUAL_ASSET_SPECS: [VisualAssetSpec; 9] = [
    VisualAssetSpec {
        kind: VisualAssetKind::PlayerCharacter,
        label: "player character rig",
        gltf_scene_path: "models/player/player.gltf",
        animation_clip_names: PLAYER_ANIMATION_CLIP_NAMES,
        residency: VisualAssetResidency::Always,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::Glider,
        label: "player glider",
        gltf_scene_path: "models/player/glider.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::Always,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandTerrain,
        label: "island terrain kit",
        gltf_scene_path: "models/world/island_terrain.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::StreamWindow,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandFoliage,
        label: "island foliage kit",
        gltf_scene_path: "models/world/foliage.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::NearLod,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandRock,
        label: "island rock kit",
        gltf_scene_path: "models/world/rocks.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::StreamWindow,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::IslandWater,
        label: "pond and water kit",
        gltf_scene_path: "models/world/water.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::NearLod,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::RouteMarker,
        label: "route marker kit",
        gltf_scene_path: "models/world/route_markers.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::Always,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::WeatherLayer,
        label: "weather cloud layer kit",
        gltf_scene_path: "models/world/weather_layers.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::Weather,
    },
    VisualAssetSpec {
        kind: VisualAssetKind::DistantImpostor,
        label: "sky island distant impostor kit",
        gltf_scene_path: "models/world/island_impostors.gltf",
        animation_clip_names: &[],
        residency: VisualAssetResidency::FarLod,
    },
];
pub const VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SPECS.len();
pub const GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SPECS.len();
pub const STREAMING_VISUAL_ASSET_SLOT_COUNT: usize = 6;
pub const ALWAYS_VISUAL_ASSET_SLOT_COUNT: usize = 3;
pub const STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT: usize = 2;
pub const NEAR_LOD_VISUAL_ASSET_SLOT_COUNT: usize = 2;
pub const FAR_LOD_VISUAL_ASSET_SLOT_COUNT: usize = 1;
pub const WEATHER_VISUAL_ASSET_SLOT_COUNT: usize = 1;
pub const DECLARED_VISUAL_ANIMATION_CLIP_COUNT: usize = PLAYER_ANIMATION_CLIP_NAMES.len();
pub const MIN_READY_VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_LOADED_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT: usize = ALWAYS_VISUAL_ASSET_SLOT_COUNT;
pub const MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT: usize =
    STREAMING_VISUAL_ASSET_SLOT_COUNT;
pub const MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MIN_READY_VISUAL_ASSET_SCENE_COUNT: usize = VISUAL_ASSET_SLOT_COUNT;
pub const MAX_MISSING_VISUAL_ASSET_SLOT_COUNT: usize =
    VISUAL_ASSET_SLOT_COUNT - MIN_READY_VISUAL_ASSET_SLOT_COUNT;
pub const MIN_READY_VISUAL_ANIMATION_CLIP_COUNT: usize = DECLARED_VISUAL_ANIMATION_CLIP_COUNT;
pub const MIN_VISUAL_ANIMATION_PLAYER_COUNT: usize = 1;
pub const MIN_VISUAL_ANIMATION_GRAPH_COUNT: usize = 1;
pub const MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT: usize = 0;
pub const DEFAULT_VISUAL_ASSET_LOAD_POLICY: VisualAssetLoadPolicy = VisualAssetLoadPolicy {
    max_admitted_scene_count: VISUAL_ASSET_SLOT_COUNT,
    max_streaming_admitted_scene_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
};

pub fn visual_asset_load_admission_plan(
    specs: &[VisualAssetSpec],
    mut asset_exists: impl FnMut(&VisualAssetSpec) -> bool,
    policy: VisualAssetLoadPolicy,
) -> Vec<VisualAssetLoadAdmission> {
    let mut admissions = vec![VisualAssetLoadAdmission::Missing; specs.len()];
    let mut admitted_always_count = 0;
    let mut streaming_candidates = Vec::new();

    for (index, spec) in specs.iter().enumerate() {
        if !asset_exists(spec) {
            continue;
        }

        if spec.residency == VisualAssetResidency::Always {
            admissions[index] = VisualAssetLoadAdmission::Admitted;
            admitted_always_count += 1;
        } else {
            streaming_candidates.push((index, visual_asset_load_priority(*spec)));
        }
    }

    let remaining_total_budget = policy
        .max_admitted_scene_count
        .saturating_sub(admitted_always_count);
    let streaming_budget = policy
        .max_streaming_admitted_scene_count
        .min(remaining_total_budget);

    streaming_candidates.sort_by_key(|(index, priority)| (*priority, *index));
    for (rank, (index, _)) in streaming_candidates.into_iter().enumerate() {
        admissions[index] = if rank < streaming_budget {
            VisualAssetLoadAdmission::Admitted
        } else {
            VisualAssetLoadAdmission::Deferred
        };
    }

    admissions
}

fn visual_asset_load_priority(spec: VisualAssetSpec) -> u8 {
    match spec.residency {
        VisualAssetResidency::Always => 0,
        VisualAssetResidency::StreamWindow => 1,
        VisualAssetResidency::NearLod => 2,
        VisualAssetResidency::Weather => 3,
        VisualAssetResidency::FarLod => 4,
    }
}

pub fn visual_asset_pipeline_metrics(
    specs: &[VisualAssetSpec],
    mut asset_exists: impl FnMut(&str) -> bool,
) -> VisualAssetPipelineMetrics {
    visual_asset_pipeline_metrics_with_load_states(specs, |spec| {
        VisualAssetLoadState::from_asset_exists(asset_exists(spec.gltf_scene_path))
    })
}

pub fn visual_asset_pipeline_metrics_with_load_states(
    specs: &[VisualAssetSpec],
    mut asset_load_state: impl FnMut(&VisualAssetSpec) -> VisualAssetLoadState,
) -> VisualAssetPipelineMetrics {
    visual_asset_pipeline_metrics_with_runtime_states(
        specs,
        |spec| asset_load_state(spec),
        |_| VisualAssetSceneState::NotSpawned,
    )
}

pub fn visual_asset_pipeline_metrics_with_runtime_states(
    specs: &[VisualAssetSpec],
    mut asset_load_state: impl FnMut(&VisualAssetSpec) -> VisualAssetLoadState,
    mut scene_state: impl FnMut(&VisualAssetSpec) -> VisualAssetSceneState,
) -> VisualAssetPipelineMetrics {
    visual_asset_pipeline_metrics_with_animation_states(
        specs,
        |spec| asset_load_state(spec),
        |spec| scene_state(spec),
        |_| VisualAssetAnimationState::default(),
    )
}

pub fn visual_asset_pipeline_metrics_with_animation_states(
    specs: &[VisualAssetSpec],
    mut asset_load_state: impl FnMut(&VisualAssetSpec) -> VisualAssetLoadState,
    mut scene_state: impl FnMut(&VisualAssetSpec) -> VisualAssetSceneState,
    mut animation_state: impl FnMut(&VisualAssetSpec) -> VisualAssetAnimationState,
) -> VisualAssetPipelineMetrics {
    visual_asset_pipeline_metrics_with_preload_states(
        specs,
        |spec| asset_load_state(spec),
        |_, load_state| VisualAssetPreloadState::from_load_state(load_state),
        |spec| scene_state(spec),
        |spec| animation_state(spec),
    )
}

pub fn visual_asset_pipeline_metrics_with_preload_states(
    specs: &[VisualAssetSpec],
    mut asset_load_state: impl FnMut(&VisualAssetSpec) -> VisualAssetLoadState,
    mut preload_state: impl FnMut(&VisualAssetSpec, VisualAssetLoadState) -> VisualAssetPreloadState,
    mut scene_state: impl FnMut(&VisualAssetSpec) -> VisualAssetSceneState,
    mut animation_state: impl FnMut(&VisualAssetSpec) -> VisualAssetAnimationState,
) -> VisualAssetPipelineMetrics {
    let mut metrics = VisualAssetPipelineMetrics::default();

    for spec in specs {
        metrics.slot_count += 1;
        if !spec.gltf_scene_path.is_empty() {
            metrics.gltf_scene_slot_count += 1;
        }
        metrics.declared_animation_clip_count += spec.animation_clip_names.len();
        match spec.residency {
            VisualAssetResidency::Always => metrics.always_slot_count += 1,
            VisualAssetResidency::StreamWindow => {
                metrics.stream_window_slot_count += 1;
                metrics.streaming_slot_count += 1;
            }
            VisualAssetResidency::NearLod => {
                metrics.near_lod_slot_count += 1;
                metrics.streaming_slot_count += 1;
            }
            VisualAssetResidency::FarLod => {
                metrics.far_lod_slot_count += 1;
                metrics.streaming_slot_count += 1;
            }
            VisualAssetResidency::Weather => {
                metrics.weather_slot_count += 1;
                metrics.streaming_slot_count += 1;
            }
        }

        let animation_state = animation_state(spec);
        metrics.ready_animation_clip_count += animation_state
            .ready_clip_count
            .min(spec.animation_clip_names.len());
        metrics.animation_player_count += usize::from(animation_state.animation_player_linked);
        metrics.animation_graph_count += usize::from(animation_state.animation_graph_ready);

        let asset_load_state = asset_load_state(spec);
        let preload_state = preload_state(spec, asset_load_state);
        if preload_state.dependencies_loaded {
            metrics.dependency_loaded_scene_count += 1;
        }
        if asset_load_state == VisualAssetLoadState::Loaded && preload_state.dependencies_loaded {
            metrics.preload_ready_scene_count += 1;
            if spec.residency == VisualAssetResidency::Always {
                metrics.always_preload_ready_slot_count += 1;
            } else if spec.residency.is_stream_managed() {
                metrics.streaming_preload_ready_slot_count += 1;
            }
        }

        match asset_load_state {
            VisualAssetLoadState::Missing => {
                metrics.placeholder_slot_count += 1;
                metrics.missing_slot_count += 1;
            }
            VisualAssetLoadState::Deferred => {
                metrics.placeholder_slot_count += 1;
                metrics.deferred_scene_count += 1;
            }
            VisualAssetLoadState::Queued => {
                metrics.queued_scene_count += 1;
            }
            VisualAssetLoadState::Loading => {
                metrics.queued_scene_count += 1;
                metrics.loading_scene_count += 1;
            }
            VisualAssetLoadState::Loaded => {
                metrics.ready_slot_count += 1;
                metrics.queued_scene_count += 1;
                metrics.loaded_scene_count += 1;
            }
            VisualAssetLoadState::Failed => {
                metrics.placeholder_slot_count += 1;
                metrics.queued_scene_count += 1;
                metrics.failed_scene_count += 1;
            }
        }

        match scene_state(spec) {
            VisualAssetSceneState::NotSpawned => {}
            VisualAssetSceneState::Spawned => metrics.spawned_scene_count += 1,
            VisualAssetSceneState::Ready => {
                metrics.spawned_scene_count += 1;
                metrics.ready_scene_count += 1;
            }
        }
    }

    metrics
}

#[cfg(test)]
mod tests {
    use crate::asset_pipeline::{
        ALWAYS_VISUAL_ASSET_SLOT_COUNT, DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
        DEFAULT_VISUAL_ASSET_LOAD_POLICY, FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
        NEAR_LOD_VISUAL_ASSET_SLOT_COUNT, STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
        WEATHER_VISUAL_ASSET_SLOT_COUNT,
    };

    use super::*;

    #[test]
    fn asset_specs_cover_streamed_world_and_player_slots() {
        let metrics = visual_asset_pipeline_metrics(&VISUAL_ASSET_SPECS, |_| false);

        assert_eq!(metrics.slot_count, VISUAL_ASSET_SPECS.len());
        assert!(metrics.gltf_scene_slot_count >= 8);
        assert!(metrics.streaming_slot_count >= 5);
        assert_eq!(metrics.ready_slot_count, 0);
        assert_eq!(metrics.placeholder_slot_count, VISUAL_ASSET_SPECS.len());
        assert_eq!(metrics.missing_slot_count, VISUAL_ASSET_SPECS.len());
        assert_eq!(metrics.queued_scene_count, 0);
        assert_eq!(metrics.slot_count, VISUAL_ASSET_SLOT_COUNT);
        assert_eq!(
            metrics.gltf_scene_slot_count,
            GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT
        );
        assert_eq!(
            metrics.streaming_slot_count,
            STREAMING_VISUAL_ASSET_SLOT_COUNT
        );
        assert_eq!(metrics.always_slot_count, ALWAYS_VISUAL_ASSET_SLOT_COUNT);
        assert_eq!(
            metrics.stream_window_slot_count,
            STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT
        );
        assert_eq!(
            metrics.near_lod_slot_count,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT
        );
        assert_eq!(metrics.far_lod_slot_count, FAR_LOD_VISUAL_ASSET_SLOT_COUNT);
        assert_eq!(metrics.weather_slot_count, WEATHER_VISUAL_ASSET_SLOT_COUNT);
        assert_eq!(
            metrics.declared_animation_clip_count,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT
        );
        assert_eq!(metrics.ready_animation_clip_count, 0);
        assert_eq!(metrics.animation_player_count, 0);
        assert_eq!(metrics.animation_graph_count, 0);
        assert_eq!(metrics.dependency_loaded_scene_count, 0);
        assert_eq!(metrics.preload_ready_scene_count, 0);
        assert_eq!(metrics.always_preload_ready_slot_count, 0);
        assert_eq!(metrics.streaming_preload_ready_slot_count, 0);
        assert!(
            VISUAL_ASSET_SPECS
                .iter()
                .any(|spec| spec.kind == VisualAssetKind::PlayerCharacter)
        );
        assert!(
            VISUAL_ASSET_SPECS
                .iter()
                .any(|spec| spec.kind == VisualAssetKind::DistantImpostor)
        );
        assert!(
            VISUAL_ASSET_SPECS
                .iter()
                .any(|spec| spec.kind == VisualAssetKind::Glider
                    && spec.gltf_scene_path == "models/player/glider.gltf")
        );
        assert!(
            VISUAL_ASSET_SPECS
                .iter()
                .any(|spec| spec.kind == VisualAssetKind::IslandTerrain
                    && spec.gltf_scene_path == "models/world/island_terrain.gltf")
        );
        assert!(
            VISUAL_ASSET_SPECS
                .iter()
                .any(|spec| spec.kind == VisualAssetKind::IslandFoliage
                    && spec.gltf_scene_path == "models/world/foliage.gltf")
        );
        assert!(
            VISUAL_ASSET_SPECS
                .iter()
                .any(|spec| spec.kind == VisualAssetKind::IslandWater
                    && spec.gltf_scene_path == "models/world/water.gltf")
        );
        assert!(
            VISUAL_ASSET_SPECS
                .iter()
                .any(|spec| spec.kind == VisualAssetKind::RouteMarker
                    && spec.gltf_scene_path == "models/world/route_markers.gltf")
        );
    }

    #[test]
    fn asset_metrics_count_queued_and_placeholder_slots() {
        let metrics = visual_asset_pipeline_metrics(&VISUAL_ASSET_SPECS, |path| {
            path == "models/player/player.gltf" || path == "models/world/foliage.gltf"
        });

        assert_eq!(metrics.ready_slot_count, 0);
        assert_eq!(metrics.queued_scene_count, 2);
        assert_eq!(metrics.missing_slot_count, VISUAL_ASSET_SPECS.len() - 2);
        assert_eq!(metrics.placeholder_slot_count, VISUAL_ASSET_SPECS.len() - 2);
    }

    #[test]
    fn default_asset_load_policy_admits_current_manifest() {
        let admissions = visual_asset_load_admission_plan(
            &VISUAL_ASSET_SPECS,
            |_| true,
            DEFAULT_VISUAL_ASSET_LOAD_POLICY,
        );

        assert_eq!(admissions.len(), VISUAL_ASSET_SPECS.len());
        assert!(admissions.iter().all(|admission| admission.is_admitted()));
        assert_eq!(
            admissions
                .iter()
                .filter(|admission| admission.load_state() == VisualAssetLoadState::Deferred)
                .count(),
            0
        );
    }

    #[test]
    fn asset_load_policy_defers_lower_priority_streamed_assets() {
        let specs = [
            VisualAssetSpec {
                kind: VisualAssetKind::PlayerCharacter,
                label: "always player",
                gltf_scene_path: "player.gltf",
                animation_clip_names: &[],
                residency: VisualAssetResidency::Always,
            },
            VisualAssetSpec {
                kind: VisualAssetKind::Glider,
                label: "always glider",
                gltf_scene_path: "glider.gltf",
                animation_clip_names: &[],
                residency: VisualAssetResidency::Always,
            },
            VisualAssetSpec {
                kind: VisualAssetKind::IslandTerrain,
                label: "stream terrain",
                gltf_scene_path: "terrain.gltf",
                animation_clip_names: &[],
                residency: VisualAssetResidency::StreamWindow,
            },
            VisualAssetSpec {
                kind: VisualAssetKind::IslandRock,
                label: "stream rock",
                gltf_scene_path: "rock.gltf",
                animation_clip_names: &[],
                residency: VisualAssetResidency::StreamWindow,
            },
            VisualAssetSpec {
                kind: VisualAssetKind::IslandFoliage,
                label: "near foliage",
                gltf_scene_path: "foliage.gltf",
                animation_clip_names: &[],
                residency: VisualAssetResidency::NearLod,
            },
            VisualAssetSpec {
                kind: VisualAssetKind::WeatherLayer,
                label: "weather",
                gltf_scene_path: "weather.gltf",
                animation_clip_names: &[],
                residency: VisualAssetResidency::Weather,
            },
            VisualAssetSpec {
                kind: VisualAssetKind::DistantImpostor,
                label: "far impostor",
                gltf_scene_path: "far.gltf",
                animation_clip_names: &[],
                residency: VisualAssetResidency::FarLod,
            },
        ];

        let admissions = visual_asset_load_admission_plan(
            &specs,
            |_| true,
            VisualAssetLoadPolicy {
                max_admitted_scene_count: 4,
                max_streaming_admitted_scene_count: 2,
            },
        );

        assert_eq!(
            admissions,
            vec![
                VisualAssetLoadAdmission::Admitted,
                VisualAssetLoadAdmission::Admitted,
                VisualAssetLoadAdmission::Admitted,
                VisualAssetLoadAdmission::Admitted,
                VisualAssetLoadAdmission::Deferred,
                VisualAssetLoadAdmission::Deferred,
                VisualAssetLoadAdmission::Deferred,
            ]
        );
    }

    #[test]
    fn asset_metrics_track_bevy_load_state_buckets() {
        let metrics =
            visual_asset_pipeline_metrics_with_load_states(&VISUAL_ASSET_SPECS, |spec| match spec
                .kind
            {
                VisualAssetKind::PlayerCharacter => VisualAssetLoadState::Loading,
                VisualAssetKind::Glider => VisualAssetLoadState::Loaded,
                VisualAssetKind::IslandRock => VisualAssetLoadState::Deferred,
                VisualAssetKind::DistantImpostor => VisualAssetLoadState::Failed,
                _ => VisualAssetLoadState::Missing,
            });

        assert_eq!(metrics.ready_slot_count, 1);
        assert_eq!(metrics.placeholder_slot_count, VISUAL_ASSET_SPECS.len() - 2);
        assert_eq!(metrics.queued_scene_count, 3);
        assert_eq!(metrics.loading_scene_count, 1);
        assert_eq!(metrics.loaded_scene_count, 1);
        assert_eq!(metrics.dependency_loaded_scene_count, 1);
        assert_eq!(metrics.preload_ready_scene_count, 1);
        assert_eq!(metrics.deferred_scene_count, 1);
        assert_eq!(metrics.failed_scene_count, 1);
    }

    #[test]
    fn asset_metrics_track_recursive_dependency_preload_readiness() {
        let metrics = visual_asset_pipeline_metrics_with_preload_states(
            &VISUAL_ASSET_SPECS,
            |spec| match spec.kind {
                VisualAssetKind::PlayerCharacter
                | VisualAssetKind::Glider
                | VisualAssetKind::IslandTerrain => VisualAssetLoadState::Loaded,
                _ => VisualAssetLoadState::Missing,
            },
            |spec, _| match spec.kind {
                VisualAssetKind::Glider | VisualAssetKind::IslandTerrain => {
                    VisualAssetPreloadState::from_dependencies_loaded(true)
                }
                _ => VisualAssetPreloadState::from_dependencies_loaded(false),
            },
            |_| VisualAssetSceneState::NotSpawned,
            |_| VisualAssetAnimationState::default(),
        );

        assert_eq!(metrics.loaded_scene_count, 3);
        assert_eq!(metrics.dependency_loaded_scene_count, 2);
        assert_eq!(metrics.preload_ready_scene_count, 2);
        assert_eq!(metrics.always_preload_ready_slot_count, 1);
        assert_eq!(metrics.streaming_preload_ready_slot_count, 1);
    }

    #[test]
    fn asset_metrics_track_spawned_and_ready_scene_instances() {
        let metrics = visual_asset_pipeline_metrics_with_runtime_states(
            &VISUAL_ASSET_SPECS,
            |spec| match spec.kind {
                VisualAssetKind::PlayerCharacter
                | VisualAssetKind::Glider
                | VisualAssetKind::IslandTerrain => VisualAssetLoadState::Loaded,
                _ => VisualAssetLoadState::Missing,
            },
            |spec| match spec.kind {
                VisualAssetKind::PlayerCharacter => VisualAssetSceneState::Ready,
                VisualAssetKind::Glider => VisualAssetSceneState::Spawned,
                _ => VisualAssetSceneState::NotSpawned,
            },
        );

        assert_eq!(metrics.ready_slot_count, 3);
        assert_eq!(metrics.preload_ready_scene_count, 3);
        assert_eq!(metrics.spawned_scene_count, 2);
        assert_eq!(metrics.ready_scene_count, 1);
    }

    #[test]
    fn asset_metrics_track_animation_graph_readiness() {
        let metrics = visual_asset_pipeline_metrics_with_animation_states(
            &VISUAL_ASSET_SPECS,
            |spec| match spec.kind {
                VisualAssetKind::PlayerCharacter => VisualAssetLoadState::Loaded,
                _ => VisualAssetLoadState::Missing,
            },
            |spec| match spec.kind {
                VisualAssetKind::PlayerCharacter => VisualAssetSceneState::Ready,
                _ => VisualAssetSceneState::NotSpawned,
            },
            |spec| match spec.kind {
                VisualAssetKind::PlayerCharacter => VisualAssetAnimationState {
                    ready_clip_count: PLAYER_ANIMATION_CLIP_NAMES.len(),
                    animation_player_linked: true,
                    animation_graph_ready: true,
                },
                _ => VisualAssetAnimationState::default(),
            },
        );

        assert_eq!(
            metrics.declared_animation_clip_count,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT
        );
        assert_eq!(
            metrics.ready_animation_clip_count,
            DECLARED_VISUAL_ANIMATION_CLIP_COUNT
        );
        assert_eq!(metrics.animation_player_count, 1);
        assert_eq!(metrics.animation_graph_count, 1);
    }
}
