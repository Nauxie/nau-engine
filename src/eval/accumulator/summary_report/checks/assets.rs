use crate::{
    asset_pipeline::{
        MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT, MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
        MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT, MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
        MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
        MIN_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT, MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_VISUAL_ANIMATION_GRAPH_COUNT, MIN_VISUAL_ANIMATION_PLAYER_COUNT,
    },
    eval::{
        summary::EvalCheck,
        thresholds::{EvalThresholds, *},
    },
};

use super::super::super::EvalAccumulator;

pub(super) fn append_asset_checks(
    checks: &mut Vec<EvalCheck>,
    acc: &EvalAccumulator,
    thresholds: &EvalThresholds,
) {
    checks.extend([
        EvalCheck::at_least(
            "visual_asset_slot_count",
            acc.max_visual_asset_slot_count as f32,
            thresholds.min_visual_asset_slot_count as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "gltf_scene_asset_slot_count",
            acc.max_gltf_scene_asset_slot_count as f32,
            thresholds.min_gltf_scene_asset_slot_count as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "ready_visual_asset_slot_count",
            acc.max_ready_visual_asset_slot_count as f32,
            MIN_READY_VISUAL_ASSET_SLOT_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_most(
            "missing_visual_asset_slot_count",
            acc.max_missing_visual_asset_slot_count as f32,
            MAX_MISSING_VISUAL_ASSET_SLOT_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_most(
            "deferred_visual_asset_scene_count",
            acc.max_deferred_visual_asset_scene_count as f32,
            MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "streaming_visual_asset_slot_count",
            acc.max_streaming_visual_asset_slot_count as f32,
            thresholds.min_streaming_visual_asset_slot_count as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "loaded_visual_asset_scene_count",
            acc.max_loaded_visual_asset_scene_count as f32,
            MIN_LOADED_VISUAL_ASSET_SCENE_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "dependency_loaded_visual_asset_scene_count",
            acc.max_dependency_loaded_visual_asset_scene_count as f32,
            MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "preload_ready_visual_asset_scene_count",
            acc.max_preload_ready_visual_asset_scene_count as f32,
            MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "always_preload_ready_visual_asset_slot_count",
            acc.max_always_preload_ready_visual_asset_slot_count as f32,
            MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "streaming_preload_ready_visual_asset_slot_count",
            acc.max_streaming_preload_ready_visual_asset_slot_count as f32,
            MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "spawned_visual_asset_scene_count",
            acc.max_spawned_visual_asset_scene_count as f32,
            MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "ready_visual_asset_scene_count",
            acc.max_ready_visual_asset_scene_count as f32,
            MIN_READY_VISUAL_ASSET_SCENE_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "visible_authored_world_fixture_count",
            acc.max_visible_authored_world_fixture_count as f32,
            MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "declared_animation_clip_count",
            acc.max_declared_animation_clip_count as f32,
            thresholds.min_declared_animation_clip_count as f32,
            "clips",
        ),
        EvalCheck::at_least(
            "ready_animation_clip_count",
            acc.max_ready_animation_clip_count as f32,
            MIN_READY_VISUAL_ANIMATION_CLIP_COUNT as f32,
            "clips",
        ),
        EvalCheck::at_least(
            "animation_player_count",
            acc.max_animation_player_count as f32,
            MIN_VISUAL_ANIMATION_PLAYER_COUNT as f32,
            "players",
        ),
        EvalCheck::at_least(
            "animation_graph_count",
            acc.max_animation_graph_count as f32,
            MIN_VISUAL_ANIMATION_GRAPH_COUNT as f32,
            "graphs",
        ),
        EvalCheck::at_most(
            "failed_visual_asset_scene_count",
            acc.max_failed_visual_asset_scene_count as f32,
            thresholds.max_failed_visual_asset_scene_count as f32,
            "assets",
        ),
        EvalCheck::at_least(
            "power_up_count",
            acc.max_power_up_count as f32,
            thresholds.min_power_up_count as f32,
            "power-ups",
        ),
        EvalCheck::at_least(
            "collected_power_up_count",
            acc.max_collected_power_up_count as f32,
            thresholds.min_collected_power_up_count as f32,
            "power-ups",
        ),
        EvalCheck::at_least(
            "power_up_effect_samples",
            acc.power_up_effect_samples as f32,
            thresholds.min_power_up_effect_samples as f32,
            "samples",
        ),
    ]);

    if thresholds.min_collected_power_up_count > 0 {
        checks.push(EvalCheck::at_most(
            "collected_power_up_count_ceiling",
            acc.max_collected_power_up_count as f32,
            thresholds.min_collected_power_up_count as f32,
            "power-ups",
        ));
    }
}
