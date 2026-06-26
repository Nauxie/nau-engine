use super::{
    VisualAssetAnimationState, VisualAssetLoadState, VisualAssetPipelineMetrics,
    VisualAssetPreloadState, VisualAssetResidency, VisualAssetSceneState, VisualAssetSpec,
};

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
