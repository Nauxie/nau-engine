use super::{EvalAccumulator, EvalSample};

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.max_visual_asset_slot_count = accumulator
        .max_visual_asset_slot_count
        .max(sample.visual_asset_slot_count);
    accumulator.max_gltf_scene_asset_slot_count = accumulator
        .max_gltf_scene_asset_slot_count
        .max(sample.gltf_scene_asset_slot_count);
    accumulator.max_ready_visual_asset_slot_count = accumulator
        .max_ready_visual_asset_slot_count
        .max(sample.ready_visual_asset_slot_count);
    accumulator.max_placeholder_visual_asset_slot_count = accumulator
        .max_placeholder_visual_asset_slot_count
        .max(sample.placeholder_visual_asset_slot_count);
    accumulator.max_streaming_visual_asset_slot_count = accumulator
        .max_streaming_visual_asset_slot_count
        .max(sample.streaming_visual_asset_slot_count);
    accumulator.max_missing_visual_asset_slot_count = accumulator
        .max_missing_visual_asset_slot_count
        .max(sample.missing_visual_asset_slot_count);
    accumulator.max_deferred_visual_asset_scene_count = accumulator
        .max_deferred_visual_asset_scene_count
        .max(sample.deferred_visual_asset_scene_count);
    accumulator.max_queued_visual_asset_scene_count = accumulator
        .max_queued_visual_asset_scene_count
        .max(sample.queued_visual_asset_scene_count);
    accumulator.max_loading_visual_asset_scene_count = accumulator
        .max_loading_visual_asset_scene_count
        .max(sample.loading_visual_asset_scene_count);
    accumulator.max_loaded_visual_asset_scene_count = accumulator
        .max_loaded_visual_asset_scene_count
        .max(sample.loaded_visual_asset_scene_count);
    accumulator.max_dependency_loaded_visual_asset_scene_count = accumulator
        .max_dependency_loaded_visual_asset_scene_count
        .max(sample.dependency_loaded_visual_asset_scene_count);
    accumulator.max_preload_ready_visual_asset_scene_count = accumulator
        .max_preload_ready_visual_asset_scene_count
        .max(sample.preload_ready_visual_asset_scene_count);
    accumulator.max_failed_visual_asset_scene_count = accumulator
        .max_failed_visual_asset_scene_count
        .max(sample.failed_visual_asset_scene_count);
    accumulator.max_spawned_visual_asset_scene_count = accumulator
        .max_spawned_visual_asset_scene_count
        .max(sample.spawned_visual_asset_scene_count);
    accumulator.max_ready_visual_asset_scene_count = accumulator
        .max_ready_visual_asset_scene_count
        .max(sample.ready_visual_asset_scene_count);
    accumulator.max_visible_authored_world_fixture_count = accumulator
        .max_visible_authored_world_fixture_count
        .max(sample.visible_authored_world_fixture_count);
    accumulator.max_always_visual_asset_slot_count = accumulator
        .max_always_visual_asset_slot_count
        .max(sample.always_visual_asset_slot_count);
    accumulator.max_stream_window_visual_asset_slot_count = accumulator
        .max_stream_window_visual_asset_slot_count
        .max(sample.stream_window_visual_asset_slot_count);
    accumulator.max_near_lod_visual_asset_slot_count = accumulator
        .max_near_lod_visual_asset_slot_count
        .max(sample.near_lod_visual_asset_slot_count);
    accumulator.max_far_lod_visual_asset_slot_count = accumulator
        .max_far_lod_visual_asset_slot_count
        .max(sample.far_lod_visual_asset_slot_count);
    accumulator.max_weather_visual_asset_slot_count = accumulator
        .max_weather_visual_asset_slot_count
        .max(sample.weather_visual_asset_slot_count);
    accumulator.max_always_preload_ready_visual_asset_slot_count = accumulator
        .max_always_preload_ready_visual_asset_slot_count
        .max(sample.always_preload_ready_visual_asset_slot_count);
    accumulator.max_streaming_preload_ready_visual_asset_slot_count = accumulator
        .max_streaming_preload_ready_visual_asset_slot_count
        .max(sample.streaming_preload_ready_visual_asset_slot_count);
    accumulator.max_declared_animation_clip_count = accumulator
        .max_declared_animation_clip_count
        .max(sample.declared_animation_clip_count);
    accumulator.max_ready_animation_clip_count = accumulator
        .max_ready_animation_clip_count
        .max(sample.ready_animation_clip_count);
    accumulator.max_animation_player_count = accumulator
        .max_animation_player_count
        .max(sample.animation_player_count);
    accumulator.max_animation_graph_count = accumulator
        .max_animation_graph_count
        .max(sample.animation_graph_count);
}
