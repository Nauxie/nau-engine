use crate::{
    asset_pipeline::{
        DECLARED_VISUAL_ANIMATION_CLIP_COUNT, GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
        STREAMING_VISUAL_ASSET_SLOT_COUNT, VISUAL_ASSET_SLOT_COUNT,
    },
    camera::CameraInput,
    movement::FlightInput,
};
use bevy::prelude::*;

use super::{
    EvalThresholds, MIN_ISLAND_CLIFF_COLOR_BANDS, MIN_ISLAND_TERRAIN_COLOR_BANDS,
    MIN_ISLAND_TERRAIN_MESH_VERTICES, MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
    MIN_ISLAND_TERRAIN_SURFACE_COUNT,
};

pub const BASELINE_ROUTE: &str = "baseline_route";
pub const ISLAND_LAUNCH_TO_LANDING: &str = "island_launch_to_landing";
pub const GROUND_TAXI_CONTROL: &str = "ground_taxi_control";
pub const UPDRAFT_ROUTE: &str = "updraft_route";
pub const CAMERA_MOUSE_CONTROL: &str = "camera_mouse_control";
pub const CAMERA_YAW_STABILITY: &str = "camera_yaw_stability";
pub const CAMERA_TURN_STABILITY: &str = "camera_turn_stability";
pub const CAMERA_STRAFE_STABILITY: &str = "camera_strafe_stability";
pub const AIR_CONTROL_RESPONSE: &str = "air_control_response";
pub const LONG_GLIDE_VISIBILITY: &str = "long_glide_visibility";
pub const BRANCH_RECOVERY_ROUTE: &str = "branch_recovery_route";
pub const SCENARIO_NAMES: &[&str] = &[
    BASELINE_ROUTE,
    ISLAND_LAUNCH_TO_LANDING,
    GROUND_TAXI_CONTROL,
    UPDRAFT_ROUTE,
    BRANCH_RECOVERY_ROUTE,
    CAMERA_MOUSE_CONTROL,
    CAMERA_YAW_STABILITY,
    CAMERA_TURN_STABILITY,
    CAMERA_STRAFE_STABILITY,
    AIR_CONTROL_RESPONSE,
    LONG_GLIDE_VISIBILITY,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalCheckpoint {
    pub frame: u32,
    pub name: &'static str,
}

const BASELINE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 90,
        name: "launch_clear",
    },
    EvalCheckpoint {
        frame: 260,
        name: "glide_midroute",
    },
];
const ISLAND_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 120,
        name: "outbound_glide",
    },
    EvalCheckpoint {
        frame: 320,
        name: "landing_approach",
    },
];
const GROUND_TAXI_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 60,
        name: "ground_turn",
    },
    EvalCheckpoint {
        frame: 150,
        name: "reverse_check",
    },
];
const UPDRAFT_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 150,
        name: "updraft_entry",
    },
    EvalCheckpoint {
        frame: 280,
        name: "high_glide",
    },
];
const BRANCH_RECOVERY_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 180,
        name: "branch_choice",
    },
    EvalCheckpoint {
        frame: 500,
        name: "recovery_approach",
    },
    EvalCheckpoint {
        frame: 690,
        name: "branch_landing",
    },
];
const CAMERA_MOUSE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 5,
        name: "launch_obstruction",
    },
    EvalCheckpoint {
        frame: 50,
        name: "yaw_check",
    },
    EvalCheckpoint {
        frame: 120,
        name: "pitch_check",
    },
    EvalCheckpoint {
        frame: 180,
        name: "settled_view",
    },
];
const CAMERA_YAW_STABILITY_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 30,
        name: "small_yaw_input",
    },
    EvalCheckpoint {
        frame: 180,
        name: "yaw_settle",
    },
    EvalCheckpoint {
        frame: 260,
        name: "drift_check",
    },
];
const CAMERA_TURN_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 90,
        name: "first_turn",
    },
    EvalCheckpoint {
        frame: 180,
        name: "counter_turn",
    },
    EvalCheckpoint {
        frame: 300,
        name: "air_brake",
    },
];
const CAMERA_STRAFE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 60,
        name: "right_strafe",
    },
    EvalCheckpoint {
        frame: 150,
        name: "left_strafe",
    },
    EvalCheckpoint {
        frame: 230,
        name: "settled_strafe",
    },
];
const AIR_CONTROL_RESPONSE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 90,
        name: "diagonal_air_steer",
    },
    EvalCheckpoint {
        frame: 165,
        name: "right_air_steer",
    },
    EvalCheckpoint {
        frame: 245,
        name: "left_air_recovery",
    },
    EvalCheckpoint {
        frame: 335,
        name: "air_brake_recovery",
    },
];
const LONG_GLIDE_CHECKPOINTS: &[EvalCheckpoint] = &[
    EvalCheckpoint {
        frame: 180,
        name: "far_route_entry",
    },
    EvalCheckpoint {
        frame: 420,
        name: "archipelago_midroute",
    },
    EvalCheckpoint {
        frame: 640,
        name: "distant_islands",
    },
];

#[derive(Clone, Copy, Debug)]
pub struct EvalScenario {
    pub name: &'static str,
    pub fixed_dt: f32,
    pub frame_count: u32,
    pub sample_stride: u32,
    pub target_island_name: Option<&'static str>,
    pub checkpoints: &'static [EvalCheckpoint],
    pub thresholds: EvalThresholds,
}

impl EvalScenario {
    pub fn duration_secs(self) -> f32 {
        self.frame_count as f32 * self.fixed_dt
    }

    pub fn should_sample(self, frame: u32) -> bool {
        frame == 0 || frame >= self.frame_count || frame.is_multiple_of(self.sample_stride)
    }

    pub fn checkpoint_at(self, frame: u32) -> Option<EvalCheckpoint> {
        self.checkpoints
            .iter()
            .copied()
            .find(|checkpoint| checkpoint.frame == frame)
    }
}
pub fn scenario_named(name: &str) -> Option<EvalScenario> {
    match name {
        BASELINE_ROUTE | "baseline" => Some(baseline_route()),
        ISLAND_LAUNCH_TO_LANDING | "island" => Some(island_launch_to_landing()),
        GROUND_TAXI_CONTROL | "ground_taxi" | "taxi" => Some(ground_taxi_control()),
        UPDRAFT_ROUTE | "updraft" => Some(updraft_route()),
        BRANCH_RECOVERY_ROUTE | "branch_recovery" | "recovery_route" => {
            Some(branch_recovery_route())
        }
        CAMERA_MOUSE_CONTROL | "camera_mouse" | "mouse_camera" => Some(camera_mouse_control()),
        CAMERA_YAW_STABILITY | "camera_yaw" | "yaw_stability" => Some(camera_yaw_stability()),
        CAMERA_TURN_STABILITY | "camera_turn" | "turn_stability" => Some(camera_turn_stability()),
        CAMERA_STRAFE_STABILITY | "camera_strafe" | "strafe_stability" => {
            Some(camera_strafe_stability())
        }
        AIR_CONTROL_RESPONSE | "air_control" | "air_response" => Some(air_control_response()),
        LONG_GLIDE_VISIBILITY | "long_glide" | "glide_visibility" => Some(long_glide_visibility()),
        _ => None,
    }
}

pub fn scripted_input(scenario: EvalScenario, frame: u32) -> FlightInput {
    let t = frame as f32 * scenario.fixed_dt;
    if matches!(scenario.name, CAMERA_MOUSE_CONTROL | CAMERA_YAW_STABILITY) {
        return FlightInput::default();
    }
    if scenario.name == CAMERA_STRAFE_STABILITY {
        return FlightInput {
            right: (0.15..=1.65).contains(&t),
            left: (1.75..=3.1).contains(&t),
            ..default()
        };
    }
    if scenario.name == AIR_CONTROL_RESPONSE {
        let dive = (5.75..=6.0).contains(&t);
        return FlightInput {
            forward: (0.05..=1.55).contains(&t) || (6.1..=6.45).contains(&t),
            right: (1.0..=2.45).contains(&t) || (4.0..=4.55).contains(&t),
            left: (2.65..=3.75).contains(&t) || (4.75..=5.3).contains(&t),
            backward: (4.0..=5.65).contains(&t),
            glide: t >= 0.45 && !dive,
            dive,
            launch: frame == 1,
        };
    }
    if scenario.name == GROUND_TAXI_CONTROL {
        return FlightInput {
            forward: (0.05..=1.95).contains(&t),
            right: (0.75..=1.65).contains(&t),
            backward: (2.2..=2.35).contains(&t),
            ..default()
        };
    }
    if scenario.name == UPDRAFT_ROUTE {
        return FlightInput {
            forward: t >= 0.05,
            right: (1.2..=2.2).contains(&t),
            left: (4.7..=5.0).contains(&t),
            glide: t >= 0.45,
            launch: frame == 1,
            ..default()
        };
    }
    if scenario.name == BRANCH_RECOVERY_ROUTE {
        let dive = (8.45..=10.9).contains(&t);
        return FlightInput {
            forward: (0.05..=10.25).contains(&t),
            backward: (10.35..=11.35).contains(&t),
            right: (1.2..=2.2).contains(&t) || (9.1..=10.0).contains(&t),
            left: (4.7..=5.0).contains(&t) || (10.45..=10.75).contains(&t),
            glide: t >= 0.45 && !dive,
            dive,
            launch: frame == 1,
        };
    }
    if scenario.name == LONG_GLIDE_VISIBILITY {
        return FlightInput {
            forward: t >= 0.05,
            right: (1.1..=2.25).contains(&t),
            left: (2.35..=2.8).contains(&t),
            glide: t >= 0.45,
            launch: frame == 1,
            ..default()
        };
    }
    if scenario.name == CAMERA_TURN_STABILITY {
        return FlightInput {
            forward: (0.05..=1.6).contains(&t),
            backward: (3.9..=5.1).contains(&t),
            left: (1.05..=1.65).contains(&t) || (2.2..=2.75).contains(&t),
            right: (1.65..=2.2).contains(&t) || (2.75..=3.35).contains(&t),
            glide: t >= 0.45,
            launch: frame == 1,
            ..default()
        };
    }

    let dive = match scenario.name {
        ISLAND_LAUNCH_TO_LANDING => (5.8..=6.7).contains(&t),
        _ => (6.2..=7.0).contains(&t),
    };
    let left = (3.1..=4.2).contains(&t);
    let right = (5.1..=5.35).contains(&t);

    let forward = if scenario.name == ISLAND_LAUNCH_TO_LANDING {
        (0.05..=7.05).contains(&t)
    } else {
        t >= 0.05
    };

    FlightInput {
        forward,
        left,
        right,
        glide: t >= 0.45 && !dive,
        dive,
        launch: frame == 1,
        ..default()
    }
}

pub fn scripted_camera_input(scenario: EvalScenario, frame: u32) -> CameraInput {
    let t = frame as f32 * scenario.fixed_dt;

    let mouse_delta = match scenario.name {
        CAMERA_MOUSE_CONTROL if (0.2..=0.7).contains(&t) => Vec2::new(5.0, 0.0),
        CAMERA_MOUSE_CONTROL if (0.9..=1.3).contains(&t) => Vec2::new(0.0, -5.0),
        CAMERA_MOUSE_CONTROL if (1.5..=2.1).contains(&t) => Vec2::new(0.0, 8.0),
        CAMERA_MOUSE_CONTROL if (2.2..=2.55).contains(&t) => Vec2::new(0.0, -8.0),
        CAMERA_YAW_STABILITY if (0.2..=0.45).contains(&t) => Vec2::new(3.0, 0.0),
        _ => Vec2::ZERO,
    };

    CameraInput { mouse_delta }
}

fn baseline_route() -> EvalScenario {
    EvalScenario {
        name: BASELINE_ROUTE,
        fixed_dt: 1.0 / 60.0,
        frame_count: 420,
        sample_stride: 10,
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
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 35.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 12.0,
            max_camera_rotation_delta_degrees: 28.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 12.0,
            min_camera_obstruction_adjustment_m: 0.0,
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

fn island_launch_to_landing() -> EvalScenario {
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
            min_lifted_samples: 0,
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 36.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 12.0,
            max_camera_rotation_delta_degrees: 28.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 12.0,
            min_camera_obstruction_adjustment_m: 0.0,
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

fn ground_taxi_control() -> EvalScenario {
    EvalScenario {
        name: GROUND_TAXI_CONTROL,
        fixed_dt: 1.0 / 60.0,
        frame_count: 180,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: GROUND_TAXI_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 30,
            min_horizontal_distance_m: 14.0,
            min_max_altitude_m: 28.0,
            min_max_speed_mps: 8.0,
            min_gliding_samples: 0,
            min_grounded_samples: 28,
            min_lifted_samples: 0,
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 28.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 10.0,
            max_camera_rotation_delta_degrees: 25.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 8.0,
            min_camera_obstruction_adjustment_m: 0.0,
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
            max_final_target_distance_m: 280.0,
            min_target_landing_samples: 0,
        },
    }
}

fn updraft_route() -> EvalScenario {
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
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 36.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 12.0,
            max_camera_rotation_delta_degrees: 28.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 12.0,
            min_camera_obstruction_adjustment_m: 0.0,
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

fn branch_recovery_route() -> EvalScenario {
    EvalScenario {
        name: BRANCH_RECOVERY_ROUTE,
        fixed_dt: 1.0 / 60.0,
        frame_count: 760,
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
            min_sky_island_count: 12,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 14,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 220,
            max_camera_distance_m: 38.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 14.0,
            max_camera_rotation_delta_degrees: 30.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 14.0,
            min_camera_obstruction_adjustment_m: 0.0,
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

fn camera_mouse_control() -> EvalScenario {
    EvalScenario {
        name: CAMERA_MOUSE_CONTROL,
        fixed_dt: 1.0 / 60.0,
        frame_count: 200,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: CAMERA_MOUSE_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 40,
            min_horizontal_distance_m: 0.0,
            min_max_altitude_m: 28.0,
            min_max_speed_mps: 0.0,
            min_gliding_samples: 0,
            min_grounded_samples: 30,
            min_lifted_samples: 0,
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 36.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 14.0,
            max_camera_rotation_delta_degrees: 35.0,
            max_camera_orbit_alignment_degrees: 30.0,
            max_abs_camera_view_yaw_degrees: 45.0,
            min_camera_obstruction_adjustment_m: 1.0,
            min_abs_camera_yaw_degrees: 25.0,
            min_camera_pitch_offset_degrees: -10.0,
            max_camera_pitch_offset_degrees: 10.0,
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
            max_final_target_distance_m: 280.0,
            min_target_landing_samples: 0,
        },
    }
}

fn camera_yaw_stability() -> EvalScenario {
    EvalScenario {
        name: CAMERA_YAW_STABILITY,
        fixed_dt: 1.0 / 60.0,
        frame_count: 300,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: CAMERA_YAW_STABILITY_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 50,
            min_horizontal_distance_m: 0.0,
            min_max_altitude_m: 28.0,
            min_max_speed_mps: 0.0,
            min_gliding_samples: 0,
            min_grounded_samples: 50,
            min_lifted_samples: 0,
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 36.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 14.0,
            max_camera_rotation_delta_degrees: 25.0,
            max_camera_orbit_alignment_degrees: 15.0,
            max_abs_camera_view_yaw_degrees: 25.0,
            min_camera_obstruction_adjustment_m: 0.0,
            min_abs_camera_yaw_degrees: 8.0,
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
            max_final_target_distance_m: 280.0,
            min_target_landing_samples: 0,
        },
    }
}

fn camera_turn_stability() -> EvalScenario {
    EvalScenario {
        name: CAMERA_TURN_STABILITY,
        fixed_dt: 1.0 / 60.0,
        frame_count: 360,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: CAMERA_TURN_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 60,
            min_horizontal_distance_m: 35.0,
            min_max_altitude_m: 42.0,
            min_max_speed_mps: 28.0,
            min_gliding_samples: 40,
            min_grounded_samples: 0,
            min_lifted_samples: 0,
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 36.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 10.0,
            max_camera_rotation_delta_degrees: 25.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 8.0,
            min_camera_obstruction_adjustment_m: 0.0,
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
            max_final_target_distance_m: 280.0,
            min_target_landing_samples: 0,
        },
    }
}

fn camera_strafe_stability() -> EvalScenario {
    EvalScenario {
        name: CAMERA_STRAFE_STABILITY,
        fixed_dt: 1.0 / 60.0,
        frame_count: 260,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: CAMERA_STRAFE_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 45,
            min_horizontal_distance_m: 1.0,
            min_max_altitude_m: 28.0,
            min_max_speed_mps: 8.0,
            min_gliding_samples: 0,
            min_grounded_samples: 45,
            min_lifted_samples: 0,
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 28.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 10.0,
            max_camera_rotation_delta_degrees: 8.0,
            max_camera_orbit_alignment_degrees: 15.0,
            max_abs_camera_view_yaw_degrees: 2.0,
            min_camera_obstruction_adjustment_m: 0.0,
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
            max_final_target_distance_m: 280.0,
            min_target_landing_samples: 0,
        },
    }
}

fn air_control_response() -> EvalScenario {
    EvalScenario {
        name: AIR_CONTROL_RESPONSE,
        fixed_dt: 1.0 / 60.0,
        frame_count: 390,
        sample_stride: 5,
        target_island_name: None,
        checkpoints: AIR_CONTROL_RESPONSE_CHECKPOINTS,
        thresholds: EvalThresholds {
            min_samples: 70,
            min_horizontal_distance_m: 30.0,
            min_max_altitude_m: 38.0,
            min_max_speed_mps: 24.0,
            min_gliding_samples: 45,
            min_grounded_samples: 0,
            min_lifted_samples: 0,
            min_sky_island_count: 10,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 100,
            max_camera_distance_m: 36.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 12.0,
            max_camera_rotation_delta_degrees: 25.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 2.0,
            min_camera_obstruction_adjustment_m: 0.0,
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
            max_final_target_distance_m: 280.0,
            min_target_landing_samples: 0,
        },
    }
}

fn long_glide_visibility() -> EvalScenario {
    EvalScenario {
        name: LONG_GLIDE_VISIBILITY,
        fixed_dt: 1.0 / 60.0,
        frame_count: 720,
        sample_stride: 10,
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
            min_sky_island_count: 12,
            min_active_island_count: 4,
            max_active_chunk_count: 25,
            min_near_lod_island_count: 2,
            min_mid_lod_island_count: 3,
            min_far_lod_island_count: 3,
            max_visible_island_terrain_count: 55,
            min_hidden_island_terrain_count: 5,
            min_visible_island_impostor_count: 2,
            max_visible_island_detail_count: 95,
            min_hidden_island_detail_count: 20,
            min_visible_route_beacon_count: 12,
            min_weather_cloud_count: 12,
            min_environment_motion_visual_count: 6,
            min_environment_motion_offset_m: 0.03,
            min_island_terrain_surface_count: MIN_ISLAND_TERRAIN_SURFACE_COUNT,
            min_island_terrain_mesh_vertices: MIN_ISLAND_TERRAIN_MESH_VERTICES,
            min_island_terrain_color_bands: MIN_ISLAND_TERRAIN_COLOR_BANDS,
            min_island_terrain_relief_range_m: MIN_ISLAND_TERRAIN_RELIEF_RANGE_M,
            min_island_cliff_color_bands: MIN_ISLAND_CLIFF_COLOR_BANDS,
            min_procedural_island_body_count: 12,
            max_primitive_island_body_count: 0,
            min_island_body_silhouette_segments: 96,
            max_resident_island_visual_count: 180,
            max_stream_visibility_changes_per_frame: 32,
            min_entity_count: 220,
            max_camera_distance_m: 38.0,
            min_camera_surface_clearance_m: 1.0,
            max_camera_player_angle_degrees: 18.0,
            max_camera_step_distance_m: 12.0,
            max_camera_rotation_delta_degrees: 28.0,
            max_camera_orbit_alignment_degrees: 45.0,
            max_abs_camera_view_yaw_degrees: 15.0,
            min_camera_obstruction_adjustment_m: 0.0,
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
