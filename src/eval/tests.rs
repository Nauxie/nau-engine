use crate::asset_pipeline::{
    ALWAYS_VISUAL_ASSET_SLOT_COUNT, DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
    FAR_LOD_VISUAL_ASSET_SLOT_COUNT, GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
    MAX_MISSING_VISUAL_ASSET_SLOT_COUNT, MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT, MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
    MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
    MIN_READY_VISUAL_ASSET_SCENE_COUNT, MIN_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT, MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
    MIN_VISUAL_ANIMATION_GRAPH_COUNT, MIN_VISUAL_ANIMATION_PLAYER_COUNT,
    NEAR_LOD_VISUAL_ASSET_SLOT_COUNT, STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
    STREAMING_VISUAL_ASSET_SLOT_COUNT, VISUAL_ASSET_SLOT_COUNT, WEATHER_VISUAL_ASSET_SLOT_COUNT,
};
use crate::camera::CameraInput;
use crate::environment::AERIAL_POWER_UP_ROUTE;
use crate::movement::{FlightInput, FlightMode};
use bevy::prelude::{Vec2, Vec3};

use super::*;

mod accumulator_controls;
mod content_gates;
mod scenario_scripts;

fn air_control_metric_sample(
    scenario: EvalScenario,
    frame: u32,
    velocity: Vec3,
    movement_axis: Vec2,
    lateral_response_mps: f32,
    desired_alignment_mps: f32,
    yaw_error_degrees: f32,
) -> EvalSample {
    let objective = EvalObjectiveProgress::new(0, 2, "near route updraft", 120.0, false);
    let pose_intent_label = if movement_axis.y < 0.0 {
        "air_brake"
    } else if velocity.y < -14.0 {
        "diving"
    } else if movement_axis.x.abs() > f32::EPSILON {
        "air_turn"
    } else {
        "gliding"
    };
    EvalSample::new(
        frame,
        scenario.fixed_dt,
        Vec3::new(frame as f32 * 0.5, 42.0, -(frame as f32) * 0.25),
        velocity,
        FlightMode::Gliding,
        pose_intent_label,
        14.0,
        3.0,
        4.0,
        -18.0,
        0.0,
        0.0,
        0.2,
        1.0,
        0.0,
        0.0,
        0.0,
        0,
        0,
        3,
        0,
        0.0,
        0.0,
        0,
        0,
        1,
        140.0,
        false,
        objective,
        MIN_SKY_ISLAND_COUNT,
        25,
        6,
        2,
        4,
        6,
        24,
        36,
        8,
        4,
        26,
        118,
        16,
        12,
        8,
        0.08,
        160,
        0,
        12,
        12,
        335,
        175,
        0.48,
        0,
        0,
        12,
        12,
        20,
        20,
        130,
        VISUAL_ASSET_SLOT_COUNT,
        GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
        MIN_READY_VISUAL_ASSET_SLOT_COUNT,
        MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
        STREAMING_VISUAL_ASSET_SLOT_COUNT,
        MAX_MISSING_VISUAL_ASSET_SLOT_COUNT,
        MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
        0,
        MIN_LOADED_VISUAL_ASSET_SCENE_COUNT,
        MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT,
        MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT,
        0,
        MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT,
        MIN_READY_VISUAL_ASSET_SCENE_COUNT,
        ALWAYS_VISUAL_ASSET_SLOT_COUNT,
        STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
        NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
        FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
        WEATHER_VISUAL_ASSET_SLOT_COUNT,
        MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT,
        DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
        MIN_READY_VISUAL_ANIMATION_CLIP_COUNT,
        MIN_VISUAL_ANIMATION_PLAYER_COUNT,
        MIN_VISUAL_ANIMATION_GRAPH_COUNT,
        AERIAL_POWER_UP_ROUTE.len(),
        AERIAL_POWER_UP_ROUTE.len(),
        0,
        0,
        0,
    )
    .with_content_metrics(
        MIN_ISLAND_TERRAIN_SURFACE_COUNT,
        2305,
        61,
        0.8,
        MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
        9,
        MIN_SKY_ISLAND_COUNT,
        0,
        96,
        96.0,
        1633,
        1633,
    )
    .with_island_impostor_metrics(146, 24)
    .with_terrain_material_metrics(36, 3, 4, 64)
    .with_generated_visual_shape_metrics(
        MIN_GENERATED_GROUND_COVER_PATCH_COUNT,
        220,
        1100,
        MIN_GENERATED_TREE_TRUNK_COUNT,
        MIN_GENERATED_TREE_CANOPY_COUNT,
        196,
        412,
        MIN_DETAIL_BIOME_PALETTE_COUNT,
        MIN_GENERATED_ROCK_COUNT,
        74,
        MIN_GENERATED_LANDMARK_COUNT,
        MIN_GENERATED_ROUTE_CAIRN_COUNT,
        MIN_GENERATED_LAUNCH_BEACON_COUNT,
        MIN_GENERATED_LANDING_GARDEN_MARKER_COUNT,
        MIN_GENERATED_POND_SURFACE_COUNT,
        39,
        MIN_GENERATED_WEATHER_CLOUD_COUNT,
        MIN_GENERATED_WEATHER_CLOUD_BANK_COUNT,
        6.2,
        9,
        18,
        1458,
        27,
    )
    .with_visible_authored_world_fixture_count(MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT)
    .with_world_collision_metrics(MIN_WORLD_COLLISION_PROXY_COUNT, 0, 0.0)
    .with_terrain_rim_collision_metrics(MIN_TERRAIN_RIM_COLLISION_PROXY_COUNT, 0, 0.0)
    .with_wind_guide_visual_metrics(
        MIN_UPDRAFT_GUIDE_VISUAL_COUNT,
        MIN_UPDRAFT_RIBBON_VISUAL_COUNT,
        MIN_CROSSWIND_GUIDE_VISUAL_COUNT,
        MIN_CROSSWIND_RIBBON_VISUAL_COUNT,
        MIN_UPDRAFT_VISUAL_MOTION_M,
        MIN_UPDRAFT_VISUAL_RISE_M,
        MIN_UPDRAFT_VISUAL_SWIRL_DISPLACEMENT_M,
        MIN_CROSSWIND_VISUAL_MOTION_M,
        MIN_CROSSWIND_GUIDE_FLOW_DISPLACEMENT_M,
        MIN_CROSSWIND_RIBBON_FLOW_DISPLACEMENT_M,
    )
    .with_wind_force_metrics(
        1,
        1,
        1,
        MIN_WIND_FORCE_DELTA_MPS,
        MIN_CROSSWIND_FORCE_DELTA_MPS,
        MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
        MIN_WIND_FORCE_FLOW_SPEED_MPS,
        MIN_WIND_FORCE_VARIATION,
    )
    .with_pose_temporal_metrics(EvalPoseTemporalMetrics {
        visible_pose_part_count: 5,
        max_pose_part_rotation_delta_degrees: 8.0,
        max_pose_part_translation_delta_m: 0.04,
    })
    .with_authored_glider_metrics(AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES, 0.08)
    .with_movement_metrics(EvalMovementMetrics {
        desired_body_yaw_error_degrees: yaw_error_degrees,
        body_travel_heading_error_degrees: yaw_error_degrees.abs(),
        body_roll_degrees: -movement_axis.x.signum() * 12.0,
        desired_heading_alignment_mps: desired_alignment_mps,
        desired_travel_heading_error_degrees: yaw_error_degrees.abs(),
        lateral_response_mps,
        lateral_input_active: movement_axis.x.abs() > f32::EPSILON,
        movement_axis,
    })
}

fn content_metric_sample(
    scenario: EvalScenario,
    frame: u32,
    procedural_body_count: usize,
    primitive_body_count: usize,
    silhouette_segments: usize,
) -> EvalSample {
    air_control_metric_sample(
        scenario,
        frame,
        Vec3::new(12.0, -2.0, -18.0),
        Vec2::new(1.0, 0.0),
        14.0,
        18.0,
        8.0,
    )
    .with_content_metrics(
        MIN_ISLAND_TERRAIN_SURFACE_COUNT,
        2305,
        61,
        0.8,
        MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
        9,
        procedural_body_count,
        primitive_body_count,
        silhouette_segments,
        silhouette_segments as f32,
        1633,
        1633,
    )
    .with_island_impostor_metrics(146, 24)
}

fn named_check<'a>(summary: &'a EvalSummary, name: &str) -> &'a EvalCheck {
    summary
        .checks
        .iter()
        .find(|check| check.name == name)
        .unwrap_or_else(|| panic!("{name} check exists"))
}
