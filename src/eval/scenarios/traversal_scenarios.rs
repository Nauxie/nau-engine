use crate::{
    asset_pipeline::{
        DECLARED_VISUAL_ANIMATION_CLIP_COUNT, GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
        STREAMING_VISUAL_ASSET_SLOT_COUNT, VISUAL_ASSET_SLOT_COUNT,
    },
    eval::thresholds::{
        EvalThresholds, MAX_ENTITY_COUNT, MAX_RESIDENT_ISLAND_VISUAL_COUNT,
        MAX_VISIBLE_ISLAND_DETAIL_COUNT, MIN_ISLAND_CLIFF_COLOR_BANDS,
        MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT, MIN_ISLAND_TERRAIN_COLOR_BANDS,
        MIN_ISLAND_TERRAIN_MESH_VERTICES, MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
        MIN_ISLAND_TERRAIN_SURFACE_COUNT, MIN_SKY_ISLAND_COUNT,
    },
};

use super::{
    BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_OBSTRUCTION_RESET_STRESS, EvalCheckpoint,
    EvalScenario, GREAT_SKY_PLATEAU_ROUTE, HIGH_ISLAND_JUMP_CAMERA, ISLAND_LAUNCH_TO_LANDING,
    LONG_GLIDE_VISIBILITY, PLATEAU_ARRIVAL_CAMERA, RETURN_DESCENT_ROUTE, UNDERBRIDGE_UNDER_ROUTE,
    UPDRAFT_ROUTE,
    checkpoints::{
        BASELINE_CHECKPOINTS, BRANCH_RECOVERY_CHECKPOINTS, GREAT_SKY_PLATEAU_CHECKPOINTS,
        ISLAND_CHECKPOINTS, LONG_GLIDE_CHECKPOINTS, UNDERBRIDGE_UNDER_ROUTE_CHECKPOINTS,
        UPDRAFT_CHECKPOINTS,
    },
};

const MAX_CAMERA_FOLLOW_DISTANCE_M: f32 = 16.5;
const MAX_CAMERA_PLAYER_ANGLE_DEGREES: f32 = 3.0;
const MAX_CAMERA_STEP_DISTANCE_M: f32 = 1.15;
const MAX_CAMERA_ROTATION_DELTA_DEGREES: f32 = 1.5;
const MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES: f32 = 5.0;
const CAMERA_STRESS_MAX_ENTITY_COUNT: usize = 4_600;

const PLATEAU_ARRIVAL_CAMERA_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 20,
        name: "plateau_spire_setup",
    },
    EvalCheckpoint {
        frame: 90,
        name: "plateau_spire_camera_obstruction",
    },
    EvalCheckpoint {
        frame: 240,
        name: "plateau_rim_camera_recenter",
    },
];

const CAMERA_OBSTRUCTION_RESET_STRESS_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 45,
        name: "obstruction_orbit_entry",
    },
    EvalCheckpoint {
        frame: 145,
        name: "obstruction_pre_reset_memory",
    },
    EvalCheckpoint {
        frame: 170,
        name: "obstruction_reset_recenter",
    },
    EvalCheckpoint {
        frame: 300,
        name: "post_reset_orbit_scrape",
    },
];

const HIGH_ISLAND_JUMP_CAMERA_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 30,
        name: "high_rim_walkoff",
    },
    EvalCheckpoint {
        frame: 120,
        name: "high_fall_camera_follow",
    },
    EvalCheckpoint {
        frame: 240,
        name: "high_fall_recovery_window",
    },
];

const RETURN_DESCENT_ROUTE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 120,
        name: "return_descent_handrail",
    },
    EvalCheckpoint {
        frame: 420,
        name: "crown_gate_tease",
    },
    EvalCheckpoint {
        frame: 720,
        name: "upper_crown_descent_view",
    },
    EvalCheckpoint {
        frame: 900,
        name: "return_descent_final_approach",
    },
];

pub(super) fn baseline_route() -> EvalScenario {
    EvalScenario {
        name: BASELINE_ROUTE,
        fixed_dt: 1.0 / 60.0,
        frame_count: 440,
        sample_stride: 1,
        target_island_name: None,
        checkpoints: BASELINE_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 20,
            min_horizontal_distance_m: 80.0,
            min_max_altitude_m: 18.0,
            min_max_speed_mps: 20.0,
            min_gliding_samples: 20,
            min_grounded_samples: 0,
            min_lifted_samples: 0,
            min_sky_island_count: MIN_SKY_ISLAND_COUNT,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: MAX_VISIBLE_ISLAND_DETAIL_COUNT,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_terrain_archetype_count: MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: MIN_SKY_ISLAND_COUNT,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: MAX_RESIDENT_ISLAND_VISUAL_COUNT,
            max_stream_visibility_changes_per_frame: 32,
            max_entity_count: MAX_ENTITY_COUNT,
            max_camera_distance_m: MAX_CAMERA_FOLLOW_DISTANCE_M,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: MAX_CAMERA_PLAYER_ANGLE_DEGREES,
            max_camera_step_distance_m: MAX_CAMERA_STEP_DISTANCE_M,
            max_camera_rotation_delta_degrees: MAX_CAMERA_ROTATION_DELTA_DEGREES,
            max_camera_orbit_alignment_degrees: MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES,
            max_abs_camera_view_yaw_degrees: 12.0,
            min_camera_obstruction_adjustment_m: 0.0,
            min_camera_obstructed_distance_m: 0.0,
            max_camera_obstruction_snap_count: 0,
            min_abs_camera_yaw_degrees: 0.0,
            min_camera_pitch_offset_degrees: 0.0,
            max_camera_pitch_offset_degrees: 0.0,
            min_objective_total_count: 2,
            min_completed_objective_count: 0,
            min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
            min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
            min_declared_animation_clip_count: DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            max_failed_visual_asset_scene_count: 0,
            min_power_up_count: 3,
            min_collected_power_up_count: 0,
            min_power_up_effect_samples: 0,
            require_target_landing: false,
            max_final_target_distance_m: 40.0,
            min_target_landing_samples: 0,
        },
    }
}

pub(super) fn island_launch_to_landing() -> EvalScenario {
    EvalScenario {
        name: ISLAND_LAUNCH_TO_LANDING,
        fixed_dt: 1.0 / 60.0,
        frame_count: 585,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: ISLAND_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 50,
            min_horizontal_distance_m: 220.0,
            min_max_altitude_m: 52.0,
            min_max_speed_mps: 30.0,
            min_gliding_samples: 45,
            min_grounded_samples: 1,
            min_lifted_samples: 8,
            min_sky_island_count: MIN_SKY_ISLAND_COUNT,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: MAX_VISIBLE_ISLAND_DETAIL_COUNT,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_terrain_archetype_count: MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: MIN_SKY_ISLAND_COUNT,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: MAX_RESIDENT_ISLAND_VISUAL_COUNT,
            max_stream_visibility_changes_per_frame: 32,
            max_entity_count: MAX_ENTITY_COUNT,
            max_camera_distance_m: MAX_CAMERA_FOLLOW_DISTANCE_M,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: MAX_CAMERA_PLAYER_ANGLE_DEGREES,
            max_camera_step_distance_m: MAX_CAMERA_STEP_DISTANCE_M,
            max_camera_rotation_delta_degrees: MAX_CAMERA_ROTATION_DELTA_DEGREES,
            max_camera_orbit_alignment_degrees: MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES,
            max_abs_camera_view_yaw_degrees: 12.0,
            min_camera_obstruction_adjustment_m: 0.0,
            min_camera_obstructed_distance_m: 0.0,
            max_camera_obstruction_snap_count: 0,
            min_abs_camera_yaw_degrees: 0.0,
            min_camera_pitch_offset_degrees: 0.0,
            max_camera_pitch_offset_degrees: 0.0,
            min_objective_total_count: 2,
            min_completed_objective_count: 0,
            min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
            min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
            min_declared_animation_clip_count: DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            max_failed_visual_asset_scene_count: 0,
            min_power_up_count: 3,
            min_collected_power_up_count: 0,
            min_power_up_effect_samples: 0,
            require_target_landing: true,
            max_final_target_distance_m: 26.0,
            min_target_landing_samples: 1,
        },
    }
}

pub(super) fn updraft_route() -> EvalScenario {
    EvalScenario {
        name: UPDRAFT_ROUTE,
        fixed_dt: 1.0 / 60.0,
        frame_count: 360,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: UPDRAFT_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 60,
            min_horizontal_distance_m: 150.0,
            min_max_altitude_m: 90.0,
            min_max_speed_mps: 35.0,
            min_gliding_samples: 45,
            min_grounded_samples: 1,
            min_lifted_samples: 4,
            min_sky_island_count: MIN_SKY_ISLAND_COUNT,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: MAX_VISIBLE_ISLAND_DETAIL_COUNT,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_terrain_archetype_count: MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: MIN_SKY_ISLAND_COUNT,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: MAX_RESIDENT_ISLAND_VISUAL_COUNT,
            max_stream_visibility_changes_per_frame: 35,
            max_entity_count: MAX_ENTITY_COUNT,
            max_camera_distance_m: MAX_CAMERA_FOLLOW_DISTANCE_M,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: MAX_CAMERA_PLAYER_ANGLE_DEGREES,
            max_camera_step_distance_m: MAX_CAMERA_STEP_DISTANCE_M,
            max_camera_rotation_delta_degrees: MAX_CAMERA_ROTATION_DELTA_DEGREES,
            max_camera_orbit_alignment_degrees: MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES,
            max_abs_camera_view_yaw_degrees: 12.0,
            min_camera_obstruction_adjustment_m: 0.0,
            min_camera_obstructed_distance_m: 0.0,
            max_camera_obstruction_snap_count: 0,
            min_abs_camera_yaw_degrees: 0.0,
            min_camera_pitch_offset_degrees: 0.0,
            max_camera_pitch_offset_degrees: 0.0,
            min_objective_total_count: 2,
            min_completed_objective_count: 1,
            min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
            min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
            min_declared_animation_clip_count: DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            max_failed_visual_asset_scene_count: 0,
            min_power_up_count: 3,
            min_collected_power_up_count: 0,
            min_power_up_effect_samples: 0,
            require_target_landing: false,
            max_final_target_distance_m: 180.0,
            min_target_landing_samples: 0,
        },
    }
}

pub(super) fn branch_recovery_route() -> EvalScenario {
    EvalScenario {
        name: BRANCH_RECOVERY_ROUTE,
        fixed_dt: 1.0 / 60.0,
        frame_count: 840,
        sample_stride: 5,
        target_island_name: Some("sunlit terrace"),
        checkpoints: BRANCH_RECOVERY_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 80,
            min_horizontal_distance_m: 390.0,
            min_max_altitude_m: 100.0,
            min_max_speed_mps: 45.0,
            min_gliding_samples: 55,
            min_grounded_samples: 2,
            min_lifted_samples: 4,
            min_sky_island_count: MIN_SKY_ISLAND_COUNT,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 60,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: MAX_VISIBLE_ISLAND_DETAIL_COUNT,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 14,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_terrain_archetype_count: MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: MIN_SKY_ISLAND_COUNT,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: MAX_RESIDENT_ISLAND_VISUAL_COUNT,
            max_stream_visibility_changes_per_frame: 40,
            max_entity_count: MAX_ENTITY_COUNT,
            max_camera_distance_m: MAX_CAMERA_FOLLOW_DISTANCE_M,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: MAX_CAMERA_PLAYER_ANGLE_DEGREES,
            max_camera_step_distance_m: MAX_CAMERA_STEP_DISTANCE_M,
            max_camera_rotation_delta_degrees: MAX_CAMERA_ROTATION_DELTA_DEGREES,
            max_camera_orbit_alignment_degrees: MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES,
            max_abs_camera_view_yaw_degrees: 14.0,
            min_camera_obstruction_adjustment_m: 0.0,
            min_camera_obstructed_distance_m: 0.0,
            max_camera_obstruction_snap_count: 0,
            min_abs_camera_yaw_degrees: 0.0,
            min_camera_pitch_offset_degrees: 0.0,
            max_camera_pitch_offset_degrees: 0.0,
            min_objective_total_count: 3,
            min_completed_objective_count: 3,
            min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
            min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
            min_declared_animation_clip_count: DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            max_failed_visual_asset_scene_count: 0,
            min_power_up_count: 3,
            min_collected_power_up_count: 0,
            min_power_up_effect_samples: 0,
            require_target_landing: true,
            max_final_target_distance_m: 18.0,
            min_target_landing_samples: 2,
        },
    }
}

pub(super) fn long_glide_visibility() -> EvalScenario {
    EvalScenario {
        name: LONG_GLIDE_VISIBILITY,
        fixed_dt: 1.0 / 60.0,
        frame_count: 720,
        sample_stride: 1,
        target_island_name: None,
        checkpoints: LONG_GLIDE_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 60,
            min_horizontal_distance_m: 430.0,
            min_max_altitude_m: 80.0,
            min_max_speed_mps: 45.0,
            min_gliding_samples: 55,
            min_grounded_samples: 0,
            min_lifted_samples: 0,
            min_sky_island_count: MIN_SKY_ISLAND_COUNT,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 60,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: MAX_VISIBLE_ISLAND_DETAIL_COUNT,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_terrain_archetype_count: MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: MIN_SKY_ISLAND_COUNT,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: MAX_RESIDENT_ISLAND_VISUAL_COUNT,
            max_stream_visibility_changes_per_frame: 32,
            max_entity_count: MAX_ENTITY_COUNT,
            max_camera_distance_m: MAX_CAMERA_FOLLOW_DISTANCE_M,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: MAX_CAMERA_PLAYER_ANGLE_DEGREES,
            max_camera_step_distance_m: MAX_CAMERA_STEP_DISTANCE_M,
            max_camera_rotation_delta_degrees: MAX_CAMERA_ROTATION_DELTA_DEGREES,
            max_camera_orbit_alignment_degrees: MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES,
            max_abs_camera_view_yaw_degrees: 15.0,
            min_camera_obstruction_adjustment_m: 0.0,
            min_camera_obstructed_distance_m: 0.0,
            max_camera_obstruction_snap_count: 0,
            min_abs_camera_yaw_degrees: 0.0,
            min_camera_pitch_offset_degrees: 0.0,
            max_camera_pitch_offset_degrees: 0.0,
            min_objective_total_count: 2,
            min_completed_objective_count: 0,
            min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
            min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
            min_declared_animation_clip_count: DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            max_failed_visual_asset_scene_count: 0,
            min_power_up_count: 3,
            min_collected_power_up_count: 3,
            min_power_up_effect_samples: 3,
            require_target_landing: false,
            max_final_target_distance_m: 520.0,
            min_target_landing_samples: 0,
        },
    }
}

pub(super) fn great_sky_plateau_route() -> EvalScenario {
    let mut scenario = long_glide_visibility();
    scenario.name = GREAT_SKY_PLATEAU_ROUTE;
    scenario.frame_count = 2520;
    scenario.sample_stride = 15;
    scenario.target_island_name = Some("great sky plateau");
    scenario.checkpoints = GREAT_SKY_PLATEAU_CHECKPOINTS;
    scenario.thresholds.min_samples = 120;
    scenario.thresholds.min_horizontal_distance_m = 1150.0;
    scenario.thresholds.min_max_altitude_m = 145.0;
    scenario.thresholds.min_max_speed_mps = 45.0;
    scenario.thresholds.min_gliding_samples = 100;
    scenario.thresholds.min_lifted_samples = 8;
    scenario.thresholds.min_visible_route_beacon_count = 14;
    scenario.thresholds.max_visible_island_terrain_count = 62;
    scenario.thresholds.max_entity_count = MAX_ENTITY_COUNT;
    scenario.thresholds.max_abs_camera_view_yaw_degrees = 15.0;
    scenario.thresholds.min_objective_total_count = 10;
    scenario.thresholds.min_completed_objective_count = 3;
    scenario.thresholds.min_collected_power_up_count = 0;
    scenario.thresholds.min_power_up_effect_samples = 0;
    scenario.thresholds.max_final_target_distance_m = 1450.0;

    scenario
}

pub(super) fn return_descent_route() -> EvalScenario {
    let mut scenario = long_glide_visibility();
    scenario.name = RETURN_DESCENT_ROUTE;
    scenario.frame_count = 1_020;
    scenario.sample_stride = 5;
    scenario.target_island_name = Some("upper crown");
    scenario.checkpoints = RETURN_DESCENT_ROUTE_CHECKPOINTS;
    scenario.thresholds.min_samples = 160;
    scenario.thresholds.min_horizontal_distance_m = 260.0;
    scenario.thresholds.min_max_altitude_m = 820.0;
    scenario.thresholds.min_max_speed_mps = 24.0;
    scenario.thresholds.min_gliding_samples = 100;
    scenario.thresholds.min_grounded_samples = 1;
    scenario.thresholds.min_lifted_samples = 2;
    scenario.thresholds.min_active_island_count = 3;
    scenario.thresholds.min_near_lod_island_count = 1;
    scenario.thresholds.min_visible_route_beacon_count = 5;
    scenario.thresholds.max_visible_island_terrain_count = 62;
    scenario.thresholds.max_entity_count = MAX_ENTITY_COUNT;
    scenario.thresholds.max_abs_camera_view_yaw_degrees = 18.0;
    scenario.thresholds.min_objective_total_count = 11;
    scenario.thresholds.min_completed_objective_count = 0;
    scenario.thresholds.min_collected_power_up_count = 0;
    scenario.thresholds.min_power_up_effect_samples = 0;
    scenario.thresholds.require_target_landing = false;
    scenario.thresholds.max_final_target_distance_m = 720.0;
    scenario.thresholds.min_target_landing_samples = 0;

    scenario
}

pub(super) fn plateau_arrival_camera() -> EvalScenario {
    let mut scenario = long_glide_visibility();
    scenario.name = PLATEAU_ARRIVAL_CAMERA;
    scenario.frame_count = 420;
    scenario.sample_stride = 1;
    scenario.target_island_name = Some("great sky plateau");
    scenario.checkpoints = PLATEAU_ARRIVAL_CAMERA_CHECKPOINTS;
    scenario.thresholds.min_samples = 360;
    scenario.thresholds.min_horizontal_distance_m = 26.0;
    scenario.thresholds.min_max_altitude_m = 660.0;
    scenario.thresholds.min_max_speed_mps = 4.0;
    scenario.thresholds.min_gliding_samples = 0;
    scenario.thresholds.min_grounded_samples = 330;
    scenario.thresholds.min_lifted_samples = 0;
    scenario.thresholds.min_active_island_count = 2;
    scenario.thresholds.min_near_lod_island_count = 1;
    scenario.thresholds.min_mid_lod_island_count = 2;
    scenario.thresholds.max_visible_island_terrain_count = 62;
    scenario.thresholds.min_visible_route_beacon_count = 5;
    scenario.thresholds.max_entity_count = MAX_ENTITY_COUNT;
    scenario.thresholds.max_camera_distance_m = MAX_CAMERA_FOLLOW_DISTANCE_M;
    scenario.thresholds.min_camera_surface_clearance_m = 1.0;
    scenario.thresholds.max_camera_player_angle_degrees = 1.5;
    scenario.thresholds.max_camera_step_distance_m = 0.75;
    scenario.thresholds.max_camera_rotation_delta_degrees = 1.5;
    scenario.thresholds.max_camera_orbit_alignment_degrees = MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES;
    scenario.thresholds.max_abs_camera_view_yaw_degrees = 22.0;
    scenario.thresholds.min_camera_obstruction_adjustment_m = 4.0;
    scenario.thresholds.min_camera_obstructed_distance_m = 5.0;
    scenario.thresholds.max_camera_obstruction_snap_count = 0;
    scenario.thresholds.min_abs_camera_yaw_degrees = 10.0;
    scenario.thresholds.min_objective_total_count = 10;
    scenario.thresholds.min_completed_objective_count = 0;
    scenario.thresholds.min_collected_power_up_count = 0;
    scenario.thresholds.min_power_up_effect_samples = 0;
    scenario.thresholds.require_target_landing = false;
    scenario.thresholds.max_final_target_distance_m = 3_000.0;
    scenario.thresholds.min_target_landing_samples = 0;

    scenario
}

pub(super) fn camera_obstruction_reset_stress() -> EvalScenario {
    let mut scenario = plateau_arrival_camera();
    scenario.name = CAMERA_OBSTRUCTION_RESET_STRESS;
    scenario.frame_count = 390;
    scenario.checkpoints = CAMERA_OBSTRUCTION_RESET_STRESS_CHECKPOINTS;
    scenario.thresholds.min_samples = 340;
    scenario.thresholds.min_horizontal_distance_m = 20.0;
    scenario.thresholds.max_entity_count = CAMERA_STRESS_MAX_ENTITY_COUNT;
    scenario.thresholds.max_camera_step_distance_m = 0.9;
    scenario.thresholds.max_camera_rotation_delta_degrees = MAX_CAMERA_ROTATION_DELTA_DEGREES;
    scenario.thresholds.max_abs_camera_view_yaw_degrees = 28.0;
    scenario.thresholds.min_abs_camera_yaw_degrees = 12.0;
    scenario.thresholds.min_camera_pitch_offset_degrees = -4.5;
    scenario.thresholds.max_camera_pitch_offset_degrees = 4.0;
    scenario.thresholds.max_final_target_distance_m = 3_100.0;

    scenario
}

pub(super) fn high_island_jump_camera() -> EvalScenario {
    let mut scenario = plateau_arrival_camera();
    scenario.name = HIGH_ISLAND_JUMP_CAMERA;
    scenario.frame_count = 300;
    scenario.checkpoints = HIGH_ISLAND_JUMP_CAMERA_CHECKPOINTS;
    scenario.thresholds.min_samples = 260;
    scenario.thresholds.min_horizontal_distance_m = 12.0;
    scenario.thresholds.min_max_speed_mps = 4.0;
    scenario.thresholds.min_grounded_samples = 8;
    scenario.thresholds.max_entity_count = CAMERA_STRESS_MAX_ENTITY_COUNT;
    scenario.thresholds.max_camera_step_distance_m = MAX_CAMERA_STEP_DISTANCE_M;
    scenario.thresholds.min_camera_obstruction_adjustment_m = 0.0;
    scenario.thresholds.min_camera_obstructed_distance_m = 0.0;
    scenario.thresholds.min_abs_camera_yaw_degrees = 0.0;
    scenario.thresholds.min_camera_pitch_offset_degrees = 0.0;
    scenario.thresholds.max_camera_pitch_offset_degrees = 0.0;
    scenario.thresholds.max_final_target_distance_m = 3_200.0;

    scenario
}

pub(super) fn underbridge_under_route() -> EvalScenario {
    EvalScenario {
        name: UNDERBRIDGE_UNDER_ROUTE,
        fixed_dt: 1.0 / 60.0,
        frame_count: 480,
        sample_stride: 1,
        target_island_name: Some("underbridge cay"),
        checkpoints: UNDERBRIDGE_UNDER_ROUTE_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 80,
            min_horizontal_distance_m: 70.0,
            min_max_altitude_m: 12.0,
            min_max_speed_mps: 18.0,
            min_gliding_samples: 35,
            min_grounded_samples: 0,
            min_lifted_samples: 2,
            min_sky_island_count: MIN_SKY_ISLAND_COUNT,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: MAX_VISIBLE_ISLAND_DETAIL_COUNT,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_terrain_archetype_count: MIN_ISLAND_TERRAIN_ARCHETYPE_COUNT,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: MIN_SKY_ISLAND_COUNT,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: MAX_RESIDENT_ISLAND_VISUAL_COUNT,
            max_stream_visibility_changes_per_frame: 32,
            max_entity_count: MAX_ENTITY_COUNT,
            max_camera_distance_m: MAX_CAMERA_FOLLOW_DISTANCE_M,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: MAX_CAMERA_PLAYER_ANGLE_DEGREES,
            max_camera_step_distance_m: 1.03,
            max_camera_rotation_delta_degrees: MAX_CAMERA_ROTATION_DELTA_DEGREES,
            max_camera_orbit_alignment_degrees: MAX_CAMERA_ORBIT_ALIGNMENT_DEGREES,
            max_abs_camera_view_yaw_degrees: 22.0,
            min_camera_obstruction_adjustment_m: 0.25,
            min_camera_obstructed_distance_m: 1.75,
            max_camera_obstruction_snap_count: 0,
            min_abs_camera_yaw_degrees: 0.0,
            min_camera_pitch_offset_degrees: 0.0,
            max_camera_pitch_offset_degrees: 0.0,
            min_objective_total_count: 2,
            min_completed_objective_count: 0,
            min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
            min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
            min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
            min_declared_animation_clip_count: DECLARED_VISUAL_ANIMATION_CLIP_COUNT,
            max_failed_visual_asset_scene_count: 0,
            min_power_up_count: 3,
            min_collected_power_up_count: 0,
            min_power_up_effect_samples: 0,
            require_target_landing: false,
            max_final_target_distance_m: 95.0,
            min_target_landing_samples: 0,
        },
    }
}
