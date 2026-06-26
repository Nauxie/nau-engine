mod metrics;
mod policy;
mod specs;
mod types;

#[cfg(test)]
mod tests;

pub use metrics::{
    visual_asset_pipeline_metrics, visual_asset_pipeline_metrics_with_animation_states,
    visual_asset_pipeline_metrics_with_load_states,
    visual_asset_pipeline_metrics_with_preload_states,
    visual_asset_pipeline_metrics_with_runtime_states,
};
pub use policy::visual_asset_load_admission_plan;
pub use specs::{
    ALWAYS_VISUAL_ASSET_SLOT_COUNT, DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
    DEFAULT_VISUAL_ASSET_LOAD_POLICY, FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
    GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT, MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT,
    MAX_MISSING_VISUAL_ASSET_SLOT_COUNT, MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT, MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
    MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
    MIN_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT, MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_VISUAL_ANIMATION_GRAPH_COUNT, MIN_VISUAL_ANIMATION_PLAYER_COUNT,
    NEAR_LOD_VISUAL_ASSET_SLOT_COUNT, PLAYER_ANIMATION_CLIP_NAMES,
    STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT, STREAMING_VISUAL_ASSET_SLOT_COUNT,
    VISUAL_ASSET_SLOT_COUNT, VISUAL_ASSET_SPECS, WEATHER_VISUAL_ASSET_SLOT_COUNT,
};
pub use types::{
    VisualAssetAnimationState, VisualAssetKind, VisualAssetLoadAdmission, VisualAssetLoadPolicy,
    VisualAssetLoadState, VisualAssetPipelineMetrics, VisualAssetPreloadState,
    VisualAssetResidency, VisualAssetSceneState, VisualAssetSpec,
};
