use crate::asset_pipeline::{
    ALWAYS_VISUAL_ASSET_SLOT_COUNT, DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
    FAR_LOD_VISUAL_ASSET_SLOT_COUNT, GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
    NEAR_LOD_VISUAL_ASSET_SLOT_COUNT, STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
    STREAMING_VISUAL_ASSET_SLOT_COUNT, VISUAL_ASSET_SLOT_COUNT, VISUAL_ASSET_SPECS,
    VisualAssetKind, WEATHER_VISUAL_ASSET_SLOT_COUNT, visual_asset_pipeline_metrics,
};

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
