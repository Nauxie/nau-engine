use crate::asset_pipeline::{
    DECLARED_VISUAL_ANIMATION_CLIP_COUNT, PLAYER_ANIMATION_CLIP_NAMES, VISUAL_ASSET_SPECS,
    VisualAssetAnimationState, VisualAssetKind, VisualAssetLoadState, VisualAssetPreloadState,
    VisualAssetSceneState, visual_asset_pipeline_metrics,
    visual_asset_pipeline_metrics_with_animation_states,
    visual_asset_pipeline_metrics_with_load_states,
    visual_asset_pipeline_metrics_with_preload_states,
    visual_asset_pipeline_metrics_with_runtime_states,
};

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
fn asset_metrics_track_bevy_load_state_buckets() {
    let metrics = visual_asset_pipeline_metrics_with_load_states(&VISUAL_ASSET_SPECS, |spec| {
        match spec.kind {
            VisualAssetKind::PlayerCharacter => VisualAssetLoadState::Loading,
            VisualAssetKind::Glider => VisualAssetLoadState::Loaded,
            VisualAssetKind::IslandRock => VisualAssetLoadState::Deferred,
            VisualAssetKind::DistantImpostor => VisualAssetLoadState::Failed,
            _ => VisualAssetLoadState::Missing,
        }
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
