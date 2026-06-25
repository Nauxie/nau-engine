use crate::asset_pipeline::{
    MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT, MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
    MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT, MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
    MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
    MIN_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT, MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_VISUAL_ANIMATION_GRAPH_COUNT, MIN_VISUAL_ANIMATION_PLAYER_COUNT,
};

use super::super::EvalAccumulator;
use super::derived::SummaryDerivedMetrics;
use crate::eval::{
    scenarios::{AIR_CONTROL_RESPONSE, CAMERA_STRAFE_STABILITY, EvalScenario},
    summary::EvalCheck,
    thresholds::*,
};

pub(super) fn build_checks(
    acc: &EvalAccumulator,
    scenario: EvalScenario,
    derived: &SummaryDerivedMetrics,
) -> Vec<EvalCheck> {
    let thresholds = scenario.thresholds;
    let mut checks = vec![
        EvalCheck::at_least(
            "sample_count",
            acc.sample_count as f32,
            thresholds.min_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "horizontal_distance",
            derived.horizontal_distance_m,
            thresholds.min_horizontal_distance_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_altitude",
            acc.max_altitude_m,
            thresholds.min_max_altitude_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_speed",
            acc.max_speed_mps,
            thresholds.min_max_speed_mps,
            "m/s",
        ),
        EvalCheck::at_least(
            "gliding_samples",
            acc.gliding_samples as f32,
            thresholds.min_gliding_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "grounded_samples",
            acc.grounded_samples as f32,
            thresholds.min_grounded_samples as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "grounded_visual_foot_gap",
            acc.max_grounded_visual_foot_gap_m,
            MAX_GROUNDED_VISUAL_FOOT_GAP_M,
            "m",
        ),
        EvalCheck::at_least(
            "lifted_samples",
            acc.lifted_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "sky_island_count",
            acc.max_sky_island_count as f32,
            thresholds.min_sky_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "active_island_count",
            acc.max_active_island_count as f32,
            thresholds.min_active_island_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "active_chunk_count",
            acc.max_active_chunk_count as f32,
            thresholds.max_active_chunk_count as f32,
            "chunks",
        ),
        EvalCheck::at_least(
            "near_lod_island_count",
            acc.max_near_lod_islands as f32,
            thresholds.min_near_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "mid_lod_island_count",
            acc.max_mid_lod_islands as f32,
            thresholds.min_mid_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "far_lod_island_count",
            acc.max_far_lod_islands as f32,
            thresholds.min_far_lod_island_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "visible_island_terrain_count",
            acc.max_visible_island_terrain_count as f32,
            thresholds.max_visible_island_terrain_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "hidden_island_terrain_count",
            acc.max_hidden_island_terrain_count as f32,
            thresholds.min_hidden_island_terrain_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "visible_island_impostor_count",
            acc.max_visible_island_impostor_count as f32,
            thresholds.min_visible_island_impostor_count as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "visible_island_detail_count",
            acc.max_visible_island_detail_count as f32,
            thresholds.max_visible_island_detail_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "hidden_island_detail_count",
            acc.max_hidden_island_detail_count as f32,
            thresholds.min_hidden_island_detail_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "visible_route_beacon_count",
            acc.max_visible_route_beacon_count as f32,
            thresholds.min_visible_route_beacon_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "weather_cloud_count",
            acc.max_weather_cloud_count as f32,
            thresholds.min_weather_cloud_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "environment_motion_visual_count",
            acc.max_environment_motion_visual_count as f32,
            thresholds.min_environment_motion_visual_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "environment_motion_offset",
            acc.max_environment_motion_offset_m,
            thresholds.min_environment_motion_offset_m,
            "m",
        ),
        EvalCheck::at_least(
            "island_terrain_surface_count",
            acc.min_island_terrain_surface_count as f32,
            thresholds.min_island_terrain_surface_count as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "island_terrain_mesh_vertices",
            acc.min_island_terrain_mesh_vertices as f32,
            thresholds.min_island_terrain_mesh_vertices as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "island_terrain_color_bands",
            acc.min_island_terrain_color_bands as f32,
            thresholds.min_island_terrain_color_bands as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_terrain_material_weight_bands",
            acc.min_island_terrain_material_weight_bands as f32,
            MIN_ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_terrain_material_channels",
            acc.min_island_terrain_material_channels as f32,
            MIN_ISLAND_TERRAIN_MATERIAL_CHANNELS as f32,
            "channels",
        ),
        EvalCheck::at_least(
            "island_terrain_material_regions",
            acc.min_island_terrain_material_regions as f32,
            MIN_ISLAND_TERRAIN_MATERIAL_REGIONS as f32,
            "regions",
        ),
        EvalCheck::at_least(
            "island_terrain_texture_detail_bands",
            acc.min_island_terrain_texture_detail_bands as f32,
            MIN_ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_terrain_relief_range",
            acc.min_island_terrain_relief_range_m,
            thresholds.min_island_terrain_relief_range_m,
            "m",
        ),
        EvalCheck::at_least(
            "island_cliff_color_bands",
            acc.min_island_cliff_color_bands as f32,
            thresholds.min_island_cliff_color_bands as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "island_impostor_mesh_vertices",
            acc.min_island_impostor_mesh_vertices as f32,
            MIN_ISLAND_IMPOSTOR_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "island_impostor_color_bands",
            acc.min_island_impostor_color_bands as f32,
            MIN_ISLAND_IMPOSTOR_COLOR_BANDS as f32,
            "bands",
        ),
        EvalCheck::at_least(
            "procedural_island_body_count",
            acc.min_procedural_island_body_count as f32,
            thresholds.min_procedural_island_body_count as f32,
            "islands",
        ),
        EvalCheck::at_most(
            "primitive_island_body_count",
            acc.max_primitive_island_body_count as f32,
            thresholds.max_primitive_island_body_count as f32,
            "islands",
        ),
        EvalCheck::at_least(
            "island_body_silhouette_segments",
            acc.min_island_body_silhouette_segments as f32,
            thresholds.min_island_body_silhouette_segments as f32,
            "segments",
        ),
        EvalCheck::at_least(
            "island_body_mesh_vertices",
            acc.min_island_body_mesh_vertices as f32,
            MIN_ISLAND_BODY_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "generated_ground_cover_patch_count",
            acc.min_generated_ground_cover_patch_count as f32,
            MIN_GENERATED_GROUND_COVER_PATCH_COUNT as f32,
            "patches",
        ),
        EvalCheck::at_least(
            "ground_cover_blade_count",
            acc.min_ground_cover_blade_count as f32,
            MIN_GROUND_COVER_BLADE_COUNT as f32,
            "blades",
        ),
        EvalCheck::at_least(
            "ground_cover_mesh_vertices",
            acc.min_ground_cover_mesh_vertices as f32,
            MIN_GROUND_COVER_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "generated_tree_trunk_count",
            acc.min_generated_tree_trunk_count as f32,
            MIN_GENERATED_TREE_TRUNK_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "generated_tree_canopy_count",
            acc.min_generated_tree_canopy_count as f32,
            MIN_GENERATED_TREE_CANOPY_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "tree_trunk_mesh_vertices",
            acc.min_tree_trunk_mesh_vertices as f32,
            MIN_TREE_TRUNK_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "tree_canopy_mesh_vertices",
            acc.min_tree_canopy_mesh_vertices as f32,
            MIN_TREE_CANOPY_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "detail_biome_palette_count",
            acc.min_detail_biome_palette_count as f32,
            MIN_DETAIL_BIOME_PALETTE_COUNT as f32,
            "palettes",
        ),
        EvalCheck::at_least(
            "generated_rock_count",
            acc.min_generated_rock_count as f32,
            MIN_GENERATED_ROCK_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "rock_mesh_vertices",
            acc.min_rock_mesh_vertices as f32,
            MIN_ROCK_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_least(
            "generated_weather_cloud_count",
            acc.min_generated_weather_cloud_count as f32,
            MIN_GENERATED_WEATHER_CLOUD_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "generated_weather_cloud_bank_count",
            acc.min_generated_weather_cloud_bank_count as f32,
            MIN_GENERATED_WEATHER_CLOUD_BANK_COUNT as f32,
            "meshes",
        ),
        EvalCheck::at_least(
            "weather_cloud_bank_depth",
            acc.min_weather_cloud_bank_depth_m,
            MIN_WEATHER_CLOUD_BANK_DEPTH_M,
            "m",
        ),
        EvalCheck::at_least(
            "weather_cloud_lobe_count",
            acc.min_weather_cloud_lobe_count as f32,
            MIN_WEATHER_CLOUD_LOBE_COUNT as f32,
            "lobes",
        ),
        EvalCheck::at_least(
            "weather_cloud_bank_lobe_count",
            acc.min_max_weather_cloud_lobe_count as f32,
            MIN_MAX_WEATHER_CLOUD_LOBE_COUNT as f32,
            "lobes",
        ),
        EvalCheck::at_least(
            "weather_cloud_mesh_vertices",
            acc.min_weather_cloud_mesh_vertices as f32,
            MIN_WEATHER_CLOUD_MESH_VERTICES as f32,
            "vertices",
        ),
        EvalCheck::at_most(
            "resident_island_visual_count",
            acc.max_resident_island_visual_count as f32,
            thresholds.max_resident_island_visual_count as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "stream_visibility_changes_per_frame",
            acc.max_stream_visibility_changes_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_least(
            "hidden_island_visual_count",
            acc.max_hidden_island_visual_count as f32,
            (thresholds.min_hidden_island_terrain_count + thresholds.min_hidden_island_detail_count)
                as f32,
            "entities",
        ),
        EvalCheck::at_most(
            "resident_island_visual_fraction",
            acc.max_resident_island_visual_fraction,
            MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
            "ratio",
        ),
        EvalCheck::at_most(
            "stream_spawned_visuals_per_frame",
            acc.max_stream_spawned_visuals_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_most(
            "stream_despawned_visuals_per_frame",
            acc.max_stream_despawned_visuals_per_frame as f32,
            thresholds.max_stream_visibility_changes_per_frame as f32,
            "entities/frame",
        ),
        EvalCheck::at_least(
            "entity_count",
            acc.max_entity_count as f32,
            thresholds.min_entity_count as f32,
            "entities",
        ),
        EvalCheck::at_least(
            "objective_total_count",
            acc.max_objective_total_count as f32,
            thresholds.min_objective_total_count as f32,
            "objectives",
        ),
        EvalCheck::at_least(
            "completed_objective_count",
            acc.max_completed_objective_count as f32,
            thresholds.min_completed_objective_count as f32,
            "objectives",
        ),
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
        EvalCheck::at_most(
            "max_camera_distance",
            acc.max_camera_distance_m,
            thresholds.max_camera_distance_m,
            "m",
        ),
        EvalCheck::at_least(
            "min_camera_surface_clearance",
            acc.min_camera_surface_clearance_m,
            thresholds.min_camera_surface_clearance_m,
            "m",
        ),
        EvalCheck::at_most(
            "max_camera_player_angle",
            acc.max_camera_player_angle_degrees,
            thresholds.max_camera_player_angle_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_camera_step_distance",
            acc.max_camera_step_distance_m,
            thresholds.max_camera_step_distance_m,
            "m",
        ),
        EvalCheck::at_most(
            "max_camera_rotation_delta",
            acc.max_camera_rotation_delta_degrees,
            thresholds.max_camera_rotation_delta_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_camera_orbit_alignment",
            acc.max_camera_orbit_alignment_degrees,
            thresholds.max_camera_orbit_alignment_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "max_abs_camera_view_yaw",
            acc.max_abs_camera_view_yaw_degrees,
            thresholds.max_abs_camera_view_yaw_degrees,
            "deg",
        ),
        EvalCheck::at_least(
            "max_camera_obstruction_adjustment",
            acc.max_camera_obstruction_adjustment_m,
            thresholds.min_camera_obstruction_adjustment_m,
            "m",
        ),
        EvalCheck::at_least(
            "max_abs_camera_yaw_offset",
            acc.max_abs_camera_yaw_offset_degrees,
            thresholds.min_abs_camera_yaw_degrees,
            "deg",
        ),
        EvalCheck::at_most(
            "min_camera_pitch_offset",
            acc.min_camera_pitch_offset_degrees,
            thresholds.min_camera_pitch_offset_degrees,
            "deg",
        ),
        EvalCheck::at_least(
            "max_camera_pitch_offset",
            acc.max_camera_pitch_offset_degrees,
            thresholds.max_camera_pitch_offset_degrees,
            "deg",
        ),
    ];
    if thresholds.min_lifted_samples > 0 {
        checks.push(EvalCheck::at_least(
            "readable_lift_samples",
            acc.readable_lift_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_most(
            "unreadable_lift_samples",
            acc.unreadable_lift_samples as f32,
            0.0,
            "samples",
        ));
    }
    if thresholds.require_target_landing {
        checks.push(EvalCheck::at_most(
            "final_target_distance",
            derived.final_target_distance_m,
            thresholds.max_final_target_distance_m,
            "m",
        ));
        checks.push(EvalCheck::at_least(
            "target_landing_samples",
            acc.target_landing_samples as f32,
            thresholds.min_target_landing_samples as f32,
            "samples",
        ));
    }
    if scenario.name == AIR_CONTROL_RESPONSE {
        checks.push(EvalCheck::at_most(
            "air_control_lateral_response_latency",
            derived.lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_lateral_response",
            acc.max_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_right_lateral_response_latency",
            derived.right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_right_lateral_response",
            acc.max_right_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_left_lateral_response_latency",
            derived.left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_left_lateral_response",
            acc.max_left_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_backward_lateral_response_latency",
            derived.backward_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_backward_lateral_response",
            acc.max_backward_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_backward_right_lateral_response_latency",
            derived.backward_right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_backward_right_lateral_response",
            acc.max_backward_right_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_backward_right_rear_response",
            acc.max_backward_right_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_backward_left_lateral_response_latency",
            derived.backward_left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_backward_left_lateral_response",
            acc.max_backward_left_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_backward_left_rear_response",
            acc.max_backward_left_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_air_brake_speed_drop",
            acc.max_air_brake_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_air_brake_planar_speed_drop",
            acc.max_air_brake_planar_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_post_brake_forward_alignment",
            acc.max_post_brake_forward_alignment_mps,
            AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_desired_heading_alignment",
            acc.max_desired_heading_alignment_mps,
            AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_avg_body_heading_error",
            derived.avg_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_p95_body_heading_error",
            derived.p95_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_max_body_heading_error",
            acc.max_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_max_body_yaw_error_step",
            acc.max_body_yaw_error_step_degrees,
            AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_body_yaw_oscillation_count",
            acc.body_yaw_oscillation_count as f32,
            AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
            "sign changes",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_right_body_bank_response",
            acc.max_right_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_least(
            "air_control_left_body_bank_response",
            acc.max_left_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_max_body_roll_step",
            acc.max_body_roll_step_degrees,
            AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_camera_orbit_yaw_offset",
            acc.max_abs_camera_yaw_offset_degrees,
            AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_camera_rotation_delta",
            acc.max_camera_rotation_delta_degrees,
            AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_camera_view_yaw_drift",
            acc.max_camera_view_yaw_drift_degrees,
            AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_avg_camera_follow_direction_error",
            derived.avg_camera_follow_direction_error_degrees,
            AIR_CONTROL_MAX_AVG_CAMERA_FOLLOW_ERROR_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_p95_camera_follow_direction_error",
            derived.p95_camera_follow_direction_error_degrees,
            AIR_CONTROL_MAX_P95_CAMERA_FOLLOW_ERROR_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "air_control_camera_world_yaw_drift",
            acc.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ));
    }
    if scenario.name == CAMERA_STRAFE_STABILITY {
        checks.push(EvalCheck::at_least(
            "camera_strafe_right_lateral_response",
            acc.max_right_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "camera_strafe_left_lateral_response",
            acc.max_left_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_most(
            "camera_strafe_view_yaw_drift",
            acc.max_camera_view_yaw_drift_degrees,
            CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "camera_strafe_world_yaw_drift",
            acc.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ));
    }
    checks
}
