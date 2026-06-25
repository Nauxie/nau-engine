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
    movement::FlightMode,
};
use bevy::prelude::*;

mod sample;
mod scenarios;
mod summary;
mod thresholds;
pub use sample::{EvalMovementMetrics, EvalObjectiveProgress, EvalSample};
pub use scenarios::{
    AIR_CONTROL_RESPONSE, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_MOUSE_CONTROL,
    CAMERA_STRAFE_STABILITY, CAMERA_TURN_STABILITY, CAMERA_YAW_STABILITY, EvalCheckpoint,
    EvalScenario, GROUND_TAXI_CONTROL, ISLAND_LAUNCH_TO_LANDING, LONG_GLIDE_VISIBILITY,
    SCENARIO_NAMES, UPDRAFT_ROUTE, scenario_named, scripted_camera_input, scripted_input,
};
pub use summary::{EvalArtifacts, EvalCheck, EvalMetricsSummary, EvalSummary};
use thresholds::*;
pub use thresholds::{EvalThresholds, MAX_RESIDENT_ISLAND_VISUAL_FRACTION};
#[derive(Default, Clone, Debug)]
pub struct EvalAccumulator {
    first_sample: Option<EvalSample>,
    final_sample: Option<EvalSample>,
    frame_times_ms: Vec<f32>,
    sample_count: u32,
    max_altitude_m: f32,
    min_altitude_m: f32,
    max_grounded_visual_foot_gap_m: f32,
    max_speed_mps: f32,
    max_camera_distance_m: f32,
    min_camera_surface_clearance_m: f32,
    max_camera_player_angle_degrees: f32,
    max_camera_step_distance_m: f32,
    max_camera_rotation_delta_degrees: f32,
    max_camera_orbit_alignment_degrees: f32,
    camera_follow_direction_error_sum_degrees: f32,
    camera_follow_direction_error_samples: u32,
    camera_follow_direction_error_values_degrees: Vec<f32>,
    max_camera_follow_direction_error_degrees: f32,
    max_abs_camera_view_yaw_degrees: f32,
    first_camera_view_yaw_degrees: Option<f32>,
    max_camera_view_yaw_drift_degrees: f32,
    first_camera_world_yaw_degrees: Option<f32>,
    max_camera_world_yaw_drift_degrees: f32,
    max_camera_obstruction_adjustment_m: f32,
    max_camera_obstruction_hits: usize,
    desired_body_heading_error_sum_degrees: f32,
    desired_body_heading_samples: u32,
    desired_body_heading_error_values_degrees: Vec<f32>,
    max_desired_body_heading_error_degrees: f32,
    previous_desired_body_yaw_error_degrees: Option<f32>,
    max_body_yaw_error_step_degrees: f32,
    previous_body_yaw_error_sign: Option<f32>,
    body_yaw_oscillation_count: u32,
    previous_body_roll_degrees: Option<f32>,
    max_body_roll_step_degrees: f32,
    max_right_body_bank_degrees: f32,
    max_left_body_bank_degrees: f32,
    max_desired_heading_alignment_mps: f32,
    max_lateral_response_mps: f32,
    first_lateral_input_time_secs: Option<f32>,
    first_lateral_response_time_secs: Option<f32>,
    max_right_lateral_response_mps: f32,
    first_right_lateral_input_time_secs: Option<f32>,
    first_right_lateral_response_time_secs: Option<f32>,
    max_left_lateral_response_mps: f32,
    first_left_lateral_input_time_secs: Option<f32>,
    first_left_lateral_response_time_secs: Option<f32>,
    max_backward_lateral_response_mps: f32,
    first_backward_lateral_input_time_secs: Option<f32>,
    first_backward_lateral_response_time_secs: Option<f32>,
    max_backward_right_lateral_response_mps: f32,
    max_backward_right_rear_response_mps: f32,
    first_backward_right_lateral_input_time_secs: Option<f32>,
    first_backward_right_lateral_response_time_secs: Option<f32>,
    max_backward_left_lateral_response_mps: f32,
    max_backward_left_rear_response_mps: f32,
    first_backward_left_lateral_input_time_secs: Option<f32>,
    first_backward_left_lateral_response_time_secs: Option<f32>,
    backward_air_control_start_speed_mps: Option<f32>,
    min_backward_air_control_speed_mps: Option<f32>,
    backward_air_control_start_planar_speed_mps: Option<f32>,
    min_backward_air_control_planar_speed_mps: Option<f32>,
    max_air_brake_speed_drop_mps: f32,
    max_air_brake_planar_speed_drop_mps: f32,
    max_post_brake_forward_alignment_mps: f32,
    min_target_distance_m: f32,
    min_camera_pitch_degrees: f32,
    max_camera_pitch_degrees: f32,
    max_abs_camera_yaw_offset_degrees: f32,
    min_camera_pitch_offset_degrees: f32,
    max_camera_pitch_offset_degrees: f32,
    max_visible_wind_fields: usize,
    max_active_lift_fields: usize,
    max_readable_lift_fields: usize,
    max_sky_island_count: usize,
    max_active_chunk_count: usize,
    max_active_island_count: usize,
    max_near_lod_islands: usize,
    max_mid_lod_islands: usize,
    max_far_lod_islands: usize,
    max_visible_island_terrain_count: usize,
    max_hidden_island_terrain_count: usize,
    max_visible_island_impostor_count: usize,
    max_hidden_island_impostor_count: usize,
    max_visible_island_detail_count: usize,
    max_hidden_island_detail_count: usize,
    max_visible_route_beacon_count: usize,
    max_weather_cloud_count: usize,
    max_environment_motion_visual_count: usize,
    max_environment_motion_offset_m: f32,
    min_island_terrain_surface_count: usize,
    min_island_terrain_mesh_vertices: usize,
    min_island_terrain_color_bands: usize,
    min_island_terrain_material_weight_bands: usize,
    min_island_terrain_material_channels: usize,
    min_island_terrain_material_regions: usize,
    min_island_terrain_texture_detail_bands: usize,
    min_island_terrain_relief_range_m: f32,
    min_island_cliff_color_bands: usize,
    min_island_impostor_mesh_vertices: usize,
    min_island_impostor_color_bands: usize,
    min_procedural_island_body_count: usize,
    max_primitive_island_body_count: usize,
    min_island_body_silhouette_segments: usize,
    max_avg_island_body_silhouette_segments: f32,
    min_island_body_mesh_vertices: usize,
    max_island_body_mesh_vertices: usize,
    min_generated_ground_cover_patch_count: usize,
    min_ground_cover_blade_count: usize,
    min_ground_cover_mesh_vertices: usize,
    min_generated_tree_trunk_count: usize,
    min_generated_tree_canopy_count: usize,
    min_tree_trunk_mesh_vertices: usize,
    min_tree_canopy_mesh_vertices: usize,
    min_detail_biome_palette_count: usize,
    min_generated_rock_count: usize,
    min_rock_mesh_vertices: usize,
    min_generated_weather_cloud_count: usize,
    min_generated_weather_cloud_bank_count: usize,
    min_weather_cloud_bank_depth_m: f32,
    min_weather_cloud_lobe_count: usize,
    min_max_weather_cloud_lobe_count: usize,
    min_weather_cloud_mesh_vertices: usize,
    max_resident_island_visual_count: usize,
    max_stream_visibility_changes_per_frame: usize,
    total_stream_visibility_changes: usize,
    max_catalog_island_visual_count: usize,
    max_hidden_island_visual_count: usize,
    max_resident_island_visual_fraction: f32,
    max_stream_spawned_visuals_per_frame: usize,
    max_stream_despawned_visuals_per_frame: usize,
    total_stream_spawned_visuals: usize,
    total_stream_despawned_visuals: usize,
    max_entity_count: usize,
    max_objective_total_count: usize,
    max_completed_objective_count: usize,
    min_objective_distance_m: f32,
    objective_complete_samples: u32,
    max_visual_asset_slot_count: usize,
    max_gltf_scene_asset_slot_count: usize,
    max_ready_visual_asset_slot_count: usize,
    max_placeholder_visual_asset_slot_count: usize,
    max_streaming_visual_asset_slot_count: usize,
    max_missing_visual_asset_slot_count: usize,
    max_deferred_visual_asset_scene_count: usize,
    max_queued_visual_asset_scene_count: usize,
    max_loading_visual_asset_scene_count: usize,
    max_loaded_visual_asset_scene_count: usize,
    max_dependency_loaded_visual_asset_scene_count: usize,
    max_preload_ready_visual_asset_scene_count: usize,
    max_failed_visual_asset_scene_count: usize,
    max_spawned_visual_asset_scene_count: usize,
    max_ready_visual_asset_scene_count: usize,
    max_visible_authored_world_fixture_count: usize,
    max_always_visual_asset_slot_count: usize,
    max_stream_window_visual_asset_slot_count: usize,
    max_near_lod_visual_asset_slot_count: usize,
    max_far_lod_visual_asset_slot_count: usize,
    max_weather_visual_asset_slot_count: usize,
    max_always_preload_ready_visual_asset_slot_count: usize,
    max_streaming_preload_ready_visual_asset_slot_count: usize,
    max_declared_animation_clip_count: usize,
    max_ready_animation_clip_count: usize,
    max_animation_player_count: usize,
    max_animation_graph_count: usize,
    max_power_up_count: usize,
    min_visible_power_up_count: usize,
    max_collected_power_up_count: usize,
    power_up_effect_samples: u32,
    total_power_up_activations: usize,
    target_landing_samples: u32,
    lifted_samples: u32,
    readable_lift_samples: u32,
    unreadable_lift_samples: u32,
    gliding_samples: u32,
    launching_samples: u32,
    grounded_samples: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct EvalFrameTimeStats {
    avg_ms: f32,
    p95_ms: f32,
    p99_ms: f32,
    max_ms: f32,
}

impl EvalFrameTimeStats {
    fn from_samples(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(f32::total_cmp);

        let sum: f32 = sorted.iter().sum();
        Self {
            avg_ms: sum / sorted.len() as f32,
            p95_ms: percentile(&sorted, 0.95),
            p99_ms: percentile(&sorted, 0.99),
            max_ms: *sorted.last().unwrap_or(&0.0),
        }
    }
}

fn response_latency_secs(
    input_time_secs: Option<f32>,
    response_time_secs: Option<f32>,
    scenario: EvalScenario,
) -> f32 {
    match (input_time_secs, response_time_secs) {
        (Some(input_time), Some(response_time)) => (response_time - input_time).max(0.0),
        (Some(_), None) => scenario.duration_secs(),
        _ => 0.0,
    }
}

fn backward_diagonal_rear_response_mps(sample: &EvalSample) -> Option<f32> {
    if sample.movement_input_forward_axis >= 0.0
        || sample.movement_input_lateral_axis.abs() <= f32::EPSILON
        || !sample.desired_heading_alignment_mps.is_finite()
        || !sample.lateral_response_mps.is_finite()
    {
        return None;
    }

    Some(
        sample.desired_heading_alignment_mps * std::f32::consts::SQRT_2
            - sample.lateral_response_mps,
    )
}

impl EvalAccumulator {
    pub fn observe_frame_time_ms(&mut self, frame_time_ms: f32) {
        if frame_time_ms.is_finite() && frame_time_ms >= 0.0 {
            self.frame_times_ms.push(frame_time_ms);
        }
    }

    pub fn observe(&mut self, sample: EvalSample) {
        if self.first_sample.is_none() {
            self.first_sample = Some(sample.clone());
            self.min_altitude_m = sample.altitude_m;
            self.min_camera_surface_clearance_m = sample.camera_surface_clearance_m;
            self.min_target_distance_m = sample.target_distance_m;
            self.min_objective_distance_m = sample.objective.current_distance_m;
            self.min_camera_pitch_degrees = sample.camera_pitch_degrees;
            self.max_camera_pitch_degrees = sample.camera_pitch_degrees;
            self.min_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
            self.max_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
            self.min_visible_power_up_count = sample.visible_power_up_count;
            self.min_island_terrain_surface_count = sample.island_terrain_surface_count;
            self.min_island_terrain_mesh_vertices = sample.min_island_terrain_mesh_vertices;
            self.min_island_terrain_color_bands = sample.min_island_terrain_color_bands;
            self.min_island_terrain_material_weight_bands =
                sample.min_island_terrain_material_weight_bands;
            self.min_island_terrain_material_channels = sample.min_island_terrain_material_channels;
            self.min_island_terrain_material_regions = sample.min_island_terrain_material_regions;
            self.min_island_terrain_texture_detail_bands =
                sample.min_island_terrain_texture_detail_bands;
            self.min_island_terrain_relief_range_m = sample.min_island_terrain_relief_range_m;
            self.min_island_cliff_color_bands = sample.min_island_cliff_color_bands;
            self.min_island_impostor_mesh_vertices = sample.min_island_impostor_mesh_vertices;
            self.min_island_impostor_color_bands = sample.min_island_impostor_color_bands;
            self.min_procedural_island_body_count = sample.procedural_island_body_count;
            self.min_island_body_silhouette_segments = sample.min_island_body_silhouette_segments;
            self.min_island_body_mesh_vertices = sample.min_island_body_mesh_vertices;
            self.min_generated_ground_cover_patch_count = sample.generated_ground_cover_patch_count;
            self.min_ground_cover_blade_count = sample.min_ground_cover_blade_count;
            self.min_ground_cover_mesh_vertices = sample.min_ground_cover_mesh_vertices;
            self.min_generated_tree_trunk_count = sample.generated_tree_trunk_count;
            self.min_generated_tree_canopy_count = sample.generated_tree_canopy_count;
            self.min_tree_trunk_mesh_vertices = sample.min_tree_trunk_mesh_vertices;
            self.min_tree_canopy_mesh_vertices = sample.min_tree_canopy_mesh_vertices;
            self.min_detail_biome_palette_count = sample.detail_biome_palette_count;
            self.min_generated_rock_count = sample.generated_rock_count;
            self.min_rock_mesh_vertices = sample.min_rock_mesh_vertices;
            self.min_generated_weather_cloud_count = sample.generated_weather_cloud_count;
            self.min_generated_weather_cloud_bank_count = sample.generated_weather_cloud_bank_count;
            self.min_weather_cloud_bank_depth_m = sample.min_weather_cloud_bank_depth_m;
            self.min_weather_cloud_lobe_count = sample.min_weather_cloud_lobe_count;
            self.min_max_weather_cloud_lobe_count = sample.max_weather_cloud_lobe_count;
            self.min_weather_cloud_mesh_vertices = sample.min_weather_cloud_mesh_vertices;
        }

        self.sample_count += 1;
        self.max_altitude_m = self.max_altitude_m.max(sample.altitude_m);
        self.min_altitude_m = self.min_altitude_m.min(sample.altitude_m);
        if sample.mode == FlightMode::Grounded.label() && sample.visual_foot_gap_m.is_finite() {
            self.max_grounded_visual_foot_gap_m = self
                .max_grounded_visual_foot_gap_m
                .max(sample.visual_foot_gap_m.abs());
        }
        self.max_speed_mps = self.max_speed_mps.max(sample.speed_mps);
        self.max_camera_distance_m = self.max_camera_distance_m.max(sample.camera_distance_m);
        self.min_camera_surface_clearance_m = self
            .min_camera_surface_clearance_m
            .min(sample.camera_surface_clearance_m);
        self.max_camera_player_angle_degrees = self
            .max_camera_player_angle_degrees
            .max(sample.camera_player_angle_degrees);
        self.max_camera_step_distance_m = self
            .max_camera_step_distance_m
            .max(sample.camera_step_distance_m);
        self.max_camera_rotation_delta_degrees = self
            .max_camera_rotation_delta_degrees
            .max(sample.camera_rotation_delta_degrees);
        self.max_camera_orbit_alignment_degrees = self
            .max_camera_orbit_alignment_degrees
            .max(sample.camera_orbit_alignment_degrees);
        if sample.camera_follow_direction_error_degrees.is_finite() {
            self.camera_follow_direction_error_sum_degrees +=
                sample.camera_follow_direction_error_degrees;
            self.camera_follow_direction_error_samples += 1;
            self.camera_follow_direction_error_values_degrees
                .push(sample.camera_follow_direction_error_degrees);
            self.max_camera_follow_direction_error_degrees = self
                .max_camera_follow_direction_error_degrees
                .max(sample.camera_follow_direction_error_degrees);
        }
        self.max_abs_camera_view_yaw_degrees = self
            .max_abs_camera_view_yaw_degrees
            .max(sample.camera_view_yaw_degrees.abs());
        if sample.camera_view_yaw_degrees.is_finite() {
            let first_yaw = self
                .first_camera_view_yaw_degrees
                .get_or_insert(sample.camera_view_yaw_degrees);
            self.max_camera_view_yaw_drift_degrees = self
                .max_camera_view_yaw_drift_degrees
                .max((sample.camera_view_yaw_degrees - *first_yaw).abs());
        }
        if sample.camera_world_yaw_degrees.is_finite() {
            let first_world_yaw = self
                .first_camera_world_yaw_degrees
                .get_or_insert(sample.camera_world_yaw_degrees);
            self.max_camera_world_yaw_drift_degrees = self
                .max_camera_world_yaw_drift_degrees
                .max((sample.camera_world_yaw_degrees - *first_world_yaw).abs());
        }
        self.max_camera_obstruction_adjustment_m = self
            .max_camera_obstruction_adjustment_m
            .max(sample.camera_obstruction_adjustment_m);
        self.max_camera_obstruction_hits = self
            .max_camera_obstruction_hits
            .max(sample.camera_obstruction_hits);
        if sample.desired_body_yaw_error_degrees.is_finite() {
            let heading_error = sample.desired_body_heading_error_degrees;
            self.desired_body_heading_error_sum_degrees += heading_error;
            self.desired_body_heading_samples += 1;
            self.desired_body_heading_error_values_degrees
                .push(heading_error);
            self.max_desired_body_heading_error_degrees = self
                .max_desired_body_heading_error_degrees
                .max(heading_error);
            if let Some(previous_error) = self.previous_desired_body_yaw_error_degrees {
                self.max_body_yaw_error_step_degrees = self
                    .max_body_yaw_error_step_degrees
                    .max((sample.desired_body_yaw_error_degrees - previous_error).abs());
            }
            self.previous_desired_body_yaw_error_degrees =
                Some(sample.desired_body_yaw_error_degrees);

            if sample.desired_body_yaw_error_degrees.abs()
                >= AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES
            {
                let sign = sample.desired_body_yaw_error_degrees.signum();
                if self
                    .previous_body_yaw_error_sign
                    .is_some_and(|previous| previous != sign)
                {
                    self.body_yaw_oscillation_count += 1;
                }
                self.previous_body_yaw_error_sign = Some(sign);
            }
        }
        if !sample.body_roll_degrees.is_finite() || sample.mode == FlightMode::Grounded.label() {
            self.previous_body_roll_degrees = None;
        } else {
            if let Some(previous_roll) = self.previous_body_roll_degrees {
                self.max_body_roll_step_degrees = self
                    .max_body_roll_step_degrees
                    .max((sample.body_roll_degrees - previous_roll).abs());
            }
            self.previous_body_roll_degrees = Some(sample.body_roll_degrees);

            match sample.movement_input_lateral_axis.signum() {
                sign if sign > 0.0 => {
                    self.max_right_body_bank_degrees = self
                        .max_right_body_bank_degrees
                        .max((-sample.body_roll_degrees).max(0.0));
                }
                sign if sign < 0.0 => {
                    self.max_left_body_bank_degrees = self
                        .max_left_body_bank_degrees
                        .max(sample.body_roll_degrees.max(0.0));
                }
                _ => {}
            }
        }
        if sample.desired_heading_alignment_mps.is_finite() {
            self.max_desired_heading_alignment_mps = self
                .max_desired_heading_alignment_mps
                .max(sample.desired_heading_alignment_mps);
        }
        let lateral_axis_active =
            sample.lateral_input_active || sample.movement_input_lateral_axis.abs() > f32::EPSILON;
        if lateral_axis_active {
            if self.first_lateral_input_time_secs.is_none() {
                self.first_lateral_input_time_secs = Some(sample.time_secs);
            }
            self.max_lateral_response_mps = self
                .max_lateral_response_mps
                .max(sample.lateral_response_mps);
            if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                && self.first_lateral_response_time_secs.is_none()
            {
                self.first_lateral_response_time_secs = Some(sample.time_secs);
            }
            match sample.movement_input_lateral_axis.signum() {
                sign if sign > 0.0 => {
                    if self.first_right_lateral_input_time_secs.is_none() {
                        self.first_right_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_right_lateral_response_mps = self
                        .max_right_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                        && self.first_right_lateral_response_time_secs.is_none()
                    {
                        self.first_right_lateral_response_time_secs = Some(sample.time_secs);
                    }
                    if sample.movement_input_forward_axis < 0.0 {
                        if self.first_backward_right_lateral_input_time_secs.is_none() {
                            self.first_backward_right_lateral_input_time_secs =
                                Some(sample.time_secs);
                        }
                        self.max_backward_right_lateral_response_mps = self
                            .max_backward_right_lateral_response_mps
                            .max(sample.lateral_response_mps);
                        if let Some(rear_response_mps) =
                            backward_diagonal_rear_response_mps(&sample)
                        {
                            self.max_backward_right_rear_response_mps = self
                                .max_backward_right_rear_response_mps
                                .max(rear_response_mps);
                        }
                        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                            && self
                                .first_backward_right_lateral_response_time_secs
                                .is_none()
                        {
                            self.first_backward_right_lateral_response_time_secs =
                                Some(sample.time_secs);
                        }
                    }
                }
                sign if sign < 0.0 => {
                    if self.first_left_lateral_input_time_secs.is_none() {
                        self.first_left_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_left_lateral_response_mps = self
                        .max_left_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                        && self.first_left_lateral_response_time_secs.is_none()
                    {
                        self.first_left_lateral_response_time_secs = Some(sample.time_secs);
                    }
                    if sample.movement_input_forward_axis < 0.0 {
                        if self.first_backward_left_lateral_input_time_secs.is_none() {
                            self.first_backward_left_lateral_input_time_secs =
                                Some(sample.time_secs);
                        }
                        self.max_backward_left_lateral_response_mps = self
                            .max_backward_left_lateral_response_mps
                            .max(sample.lateral_response_mps);
                        if let Some(rear_response_mps) =
                            backward_diagonal_rear_response_mps(&sample)
                        {
                            self.max_backward_left_rear_response_mps = self
                                .max_backward_left_rear_response_mps
                                .max(rear_response_mps);
                        }
                        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                            && self
                                .first_backward_left_lateral_response_time_secs
                                .is_none()
                        {
                            self.first_backward_left_lateral_response_time_secs =
                                Some(sample.time_secs);
                        }
                    }
                }
                _ => {}
            }
            if sample.movement_input_forward_axis < 0.0 {
                if self.first_backward_lateral_input_time_secs.is_none() {
                    self.first_backward_lateral_input_time_secs = Some(sample.time_secs);
                }
                self.max_backward_lateral_response_mps = self
                    .max_backward_lateral_response_mps
                    .max(sample.lateral_response_mps);
                if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
                    && self.first_backward_lateral_response_time_secs.is_none()
                {
                    self.first_backward_lateral_response_time_secs = Some(sample.time_secs);
                }
            }
        }
        if sample.movement_input_forward_axis < 0.0 && sample.mode != FlightMode::Grounded.label() {
            let planar_speed = Vec2::new(sample.velocity[0], sample.velocity[2]).length();
            if self.backward_air_control_start_speed_mps.is_none() {
                self.backward_air_control_start_speed_mps = Some(sample.speed_mps);
            }
            if self.backward_air_control_start_planar_speed_mps.is_none() {
                self.backward_air_control_start_planar_speed_mps = Some(planar_speed);
            }
            let min_speed = self
                .min_backward_air_control_speed_mps
                .map_or(sample.speed_mps, |speed| speed.min(sample.speed_mps));
            self.min_backward_air_control_speed_mps = Some(min_speed);
            let min_planar_speed = self
                .min_backward_air_control_planar_speed_mps
                .map_or(planar_speed, |speed| speed.min(planar_speed));
            self.min_backward_air_control_planar_speed_mps = Some(min_planar_speed);
            if let Some(start_speed) = self.backward_air_control_start_speed_mps {
                self.max_air_brake_speed_drop_mps = self
                    .max_air_brake_speed_drop_mps
                    .max(start_speed - min_speed);
            }
            if let Some(start_planar_speed) = self.backward_air_control_start_planar_speed_mps {
                self.max_air_brake_planar_speed_drop_mps = self
                    .max_air_brake_planar_speed_drop_mps
                    .max(start_planar_speed - min_planar_speed);
            }
        } else if self.backward_air_control_start_speed_mps.is_some()
            && sample.movement_input_forward_axis > 0.0
            && sample.desired_heading_alignment_mps.is_finite()
        {
            self.max_post_brake_forward_alignment_mps = self
                .max_post_brake_forward_alignment_mps
                .max(sample.desired_heading_alignment_mps);
        }
        self.min_target_distance_m = self.min_target_distance_m.min(sample.target_distance_m);
        self.min_camera_pitch_degrees = self
            .min_camera_pitch_degrees
            .min(sample.camera_pitch_degrees);
        self.max_camera_pitch_degrees = self
            .max_camera_pitch_degrees
            .max(sample.camera_pitch_degrees);
        self.max_abs_camera_yaw_offset_degrees = self
            .max_abs_camera_yaw_offset_degrees
            .max(sample.camera_yaw_offset_degrees.abs());
        self.min_camera_pitch_offset_degrees = self
            .min_camera_pitch_offset_degrees
            .min(sample.camera_pitch_offset_degrees);
        self.max_camera_pitch_offset_degrees = self
            .max_camera_pitch_offset_degrees
            .max(sample.camera_pitch_offset_degrees);
        self.max_visible_wind_fields = self.max_visible_wind_fields.max(sample.visible_wind_fields);
        self.max_active_lift_fields = self.max_active_lift_fields.max(sample.active_lift_fields);
        self.max_readable_lift_fields = self
            .max_readable_lift_fields
            .max(sample.readable_lift_fields);
        self.max_sky_island_count = self.max_sky_island_count.max(sample.sky_island_count);
        self.max_active_chunk_count = self.max_active_chunk_count.max(sample.active_chunk_count);
        self.max_active_island_count = self.max_active_island_count.max(sample.active_island_count);
        self.max_near_lod_islands = self.max_near_lod_islands.max(sample.near_lod_islands);
        self.max_mid_lod_islands = self.max_mid_lod_islands.max(sample.mid_lod_islands);
        self.max_far_lod_islands = self.max_far_lod_islands.max(sample.far_lod_islands);
        self.max_visible_island_terrain_count = self
            .max_visible_island_terrain_count
            .max(sample.visible_island_terrain_count);
        self.max_hidden_island_terrain_count = self
            .max_hidden_island_terrain_count
            .max(sample.hidden_island_terrain_count);
        self.max_visible_island_impostor_count = self
            .max_visible_island_impostor_count
            .max(sample.visible_island_impostor_count);
        self.max_hidden_island_impostor_count = self
            .max_hidden_island_impostor_count
            .max(sample.hidden_island_impostor_count);
        self.max_visible_island_detail_count = self
            .max_visible_island_detail_count
            .max(sample.visible_island_detail_count);
        self.max_hidden_island_detail_count = self
            .max_hidden_island_detail_count
            .max(sample.hidden_island_detail_count);
        self.max_visible_route_beacon_count = self
            .max_visible_route_beacon_count
            .max(sample.visible_route_beacon_count);
        self.max_weather_cloud_count = self.max_weather_cloud_count.max(sample.weather_cloud_count);
        self.max_environment_motion_visual_count = self
            .max_environment_motion_visual_count
            .max(sample.environment_motion_visual_count);
        self.max_environment_motion_offset_m = self
            .max_environment_motion_offset_m
            .max(sample.max_environment_motion_offset_m);
        self.min_island_terrain_surface_count = self
            .min_island_terrain_surface_count
            .min(sample.island_terrain_surface_count);
        self.min_island_terrain_mesh_vertices = self
            .min_island_terrain_mesh_vertices
            .min(sample.min_island_terrain_mesh_vertices);
        self.min_island_terrain_color_bands = self
            .min_island_terrain_color_bands
            .min(sample.min_island_terrain_color_bands);
        self.min_island_terrain_material_weight_bands = self
            .min_island_terrain_material_weight_bands
            .min(sample.min_island_terrain_material_weight_bands);
        self.min_island_terrain_material_channels = self
            .min_island_terrain_material_channels
            .min(sample.min_island_terrain_material_channels);
        self.min_island_terrain_material_regions = self
            .min_island_terrain_material_regions
            .min(sample.min_island_terrain_material_regions);
        self.min_island_terrain_texture_detail_bands = self
            .min_island_terrain_texture_detail_bands
            .min(sample.min_island_terrain_texture_detail_bands);
        self.min_island_terrain_relief_range_m = self
            .min_island_terrain_relief_range_m
            .min(sample.min_island_terrain_relief_range_m);
        self.min_island_cliff_color_bands = self
            .min_island_cliff_color_bands
            .min(sample.min_island_cliff_color_bands);
        self.min_island_impostor_mesh_vertices = self
            .min_island_impostor_mesh_vertices
            .min(sample.min_island_impostor_mesh_vertices);
        self.min_island_impostor_color_bands = self
            .min_island_impostor_color_bands
            .min(sample.min_island_impostor_color_bands);
        self.min_procedural_island_body_count = self
            .min_procedural_island_body_count
            .min(sample.procedural_island_body_count);
        self.max_primitive_island_body_count = self
            .max_primitive_island_body_count
            .max(sample.primitive_island_body_count);
        self.min_island_body_silhouette_segments = self
            .min_island_body_silhouette_segments
            .min(sample.min_island_body_silhouette_segments);
        self.max_avg_island_body_silhouette_segments = self
            .max_avg_island_body_silhouette_segments
            .max(sample.avg_island_body_silhouette_segments);
        self.min_island_body_mesh_vertices = self
            .min_island_body_mesh_vertices
            .min(sample.min_island_body_mesh_vertices);
        self.max_island_body_mesh_vertices = self
            .max_island_body_mesh_vertices
            .max(sample.max_island_body_mesh_vertices);
        self.min_generated_ground_cover_patch_count = self
            .min_generated_ground_cover_patch_count
            .min(sample.generated_ground_cover_patch_count);
        self.min_ground_cover_blade_count = self
            .min_ground_cover_blade_count
            .min(sample.min_ground_cover_blade_count);
        self.min_ground_cover_mesh_vertices = self
            .min_ground_cover_mesh_vertices
            .min(sample.min_ground_cover_mesh_vertices);
        self.min_generated_tree_trunk_count = self
            .min_generated_tree_trunk_count
            .min(sample.generated_tree_trunk_count);
        self.min_generated_tree_canopy_count = self
            .min_generated_tree_canopy_count
            .min(sample.generated_tree_canopy_count);
        self.min_tree_trunk_mesh_vertices = self
            .min_tree_trunk_mesh_vertices
            .min(sample.min_tree_trunk_mesh_vertices);
        self.min_tree_canopy_mesh_vertices = self
            .min_tree_canopy_mesh_vertices
            .min(sample.min_tree_canopy_mesh_vertices);
        self.min_detail_biome_palette_count = self
            .min_detail_biome_palette_count
            .min(sample.detail_biome_palette_count);
        self.min_generated_rock_count = self
            .min_generated_rock_count
            .min(sample.generated_rock_count);
        self.min_rock_mesh_vertices = self
            .min_rock_mesh_vertices
            .min(sample.min_rock_mesh_vertices);
        self.min_generated_weather_cloud_count = self
            .min_generated_weather_cloud_count
            .min(sample.generated_weather_cloud_count);
        self.min_generated_weather_cloud_bank_count = self
            .min_generated_weather_cloud_bank_count
            .min(sample.generated_weather_cloud_bank_count);
        self.min_weather_cloud_bank_depth_m = self
            .min_weather_cloud_bank_depth_m
            .min(sample.min_weather_cloud_bank_depth_m);
        self.min_weather_cloud_lobe_count = self
            .min_weather_cloud_lobe_count
            .min(sample.min_weather_cloud_lobe_count);
        self.min_max_weather_cloud_lobe_count = self
            .min_max_weather_cloud_lobe_count
            .min(sample.max_weather_cloud_lobe_count);
        self.min_weather_cloud_mesh_vertices = self
            .min_weather_cloud_mesh_vertices
            .min(sample.min_weather_cloud_mesh_vertices);
        self.max_resident_island_visual_count = self
            .max_resident_island_visual_count
            .max(sample.resident_island_visual_count);
        self.max_stream_visibility_changes_per_frame = self
            .max_stream_visibility_changes_per_frame
            .max(sample.max_stream_visibility_changes_per_frame);
        self.total_stream_visibility_changes = self
            .total_stream_visibility_changes
            .max(sample.total_stream_visibility_changes);
        self.max_catalog_island_visual_count = self
            .max_catalog_island_visual_count
            .max(sample.catalog_island_visual_count);
        self.max_hidden_island_visual_count = self
            .max_hidden_island_visual_count
            .max(sample.hidden_island_visual_count);
        self.max_resident_island_visual_fraction = self
            .max_resident_island_visual_fraction
            .max(sample.resident_island_visual_fraction);
        self.max_stream_spawned_visuals_per_frame = self
            .max_stream_spawned_visuals_per_frame
            .max(sample.max_stream_spawned_visuals_per_frame);
        self.max_stream_despawned_visuals_per_frame = self
            .max_stream_despawned_visuals_per_frame
            .max(sample.max_stream_despawned_visuals_per_frame);
        self.total_stream_spawned_visuals = self
            .total_stream_spawned_visuals
            .max(sample.total_stream_spawned_visuals);
        self.total_stream_despawned_visuals = self
            .total_stream_despawned_visuals
            .max(sample.total_stream_despawned_visuals);
        self.max_entity_count = self.max_entity_count.max(sample.entity_count);
        self.max_objective_total_count = self
            .max_objective_total_count
            .max(sample.objective.total_count);
        self.max_completed_objective_count = self
            .max_completed_objective_count
            .max(sample.objective.completed_count);
        self.min_objective_distance_m = self
            .min_objective_distance_m
            .min(sample.objective.current_distance_m);
        if sample.objective.complete {
            self.objective_complete_samples += 1;
        }
        self.max_visual_asset_slot_count = self
            .max_visual_asset_slot_count
            .max(sample.visual_asset_slot_count);
        self.max_gltf_scene_asset_slot_count = self
            .max_gltf_scene_asset_slot_count
            .max(sample.gltf_scene_asset_slot_count);
        self.max_ready_visual_asset_slot_count = self
            .max_ready_visual_asset_slot_count
            .max(sample.ready_visual_asset_slot_count);
        self.max_placeholder_visual_asset_slot_count = self
            .max_placeholder_visual_asset_slot_count
            .max(sample.placeholder_visual_asset_slot_count);
        self.max_streaming_visual_asset_slot_count = self
            .max_streaming_visual_asset_slot_count
            .max(sample.streaming_visual_asset_slot_count);
        self.max_missing_visual_asset_slot_count = self
            .max_missing_visual_asset_slot_count
            .max(sample.missing_visual_asset_slot_count);
        self.max_deferred_visual_asset_scene_count = self
            .max_deferred_visual_asset_scene_count
            .max(sample.deferred_visual_asset_scene_count);
        self.max_queued_visual_asset_scene_count = self
            .max_queued_visual_asset_scene_count
            .max(sample.queued_visual_asset_scene_count);
        self.max_loading_visual_asset_scene_count = self
            .max_loading_visual_asset_scene_count
            .max(sample.loading_visual_asset_scene_count);
        self.max_loaded_visual_asset_scene_count = self
            .max_loaded_visual_asset_scene_count
            .max(sample.loaded_visual_asset_scene_count);
        self.max_dependency_loaded_visual_asset_scene_count = self
            .max_dependency_loaded_visual_asset_scene_count
            .max(sample.dependency_loaded_visual_asset_scene_count);
        self.max_preload_ready_visual_asset_scene_count = self
            .max_preload_ready_visual_asset_scene_count
            .max(sample.preload_ready_visual_asset_scene_count);
        self.max_failed_visual_asset_scene_count = self
            .max_failed_visual_asset_scene_count
            .max(sample.failed_visual_asset_scene_count);
        self.max_spawned_visual_asset_scene_count = self
            .max_spawned_visual_asset_scene_count
            .max(sample.spawned_visual_asset_scene_count);
        self.max_ready_visual_asset_scene_count = self
            .max_ready_visual_asset_scene_count
            .max(sample.ready_visual_asset_scene_count);
        self.max_visible_authored_world_fixture_count = self
            .max_visible_authored_world_fixture_count
            .max(sample.visible_authored_world_fixture_count);
        self.max_always_visual_asset_slot_count = self
            .max_always_visual_asset_slot_count
            .max(sample.always_visual_asset_slot_count);
        self.max_stream_window_visual_asset_slot_count = self
            .max_stream_window_visual_asset_slot_count
            .max(sample.stream_window_visual_asset_slot_count);
        self.max_near_lod_visual_asset_slot_count = self
            .max_near_lod_visual_asset_slot_count
            .max(sample.near_lod_visual_asset_slot_count);
        self.max_far_lod_visual_asset_slot_count = self
            .max_far_lod_visual_asset_slot_count
            .max(sample.far_lod_visual_asset_slot_count);
        self.max_weather_visual_asset_slot_count = self
            .max_weather_visual_asset_slot_count
            .max(sample.weather_visual_asset_slot_count);
        self.max_always_preload_ready_visual_asset_slot_count = self
            .max_always_preload_ready_visual_asset_slot_count
            .max(sample.always_preload_ready_visual_asset_slot_count);
        self.max_streaming_preload_ready_visual_asset_slot_count = self
            .max_streaming_preload_ready_visual_asset_slot_count
            .max(sample.streaming_preload_ready_visual_asset_slot_count);
        self.max_declared_animation_clip_count = self
            .max_declared_animation_clip_count
            .max(sample.declared_animation_clip_count);
        self.max_ready_animation_clip_count = self
            .max_ready_animation_clip_count
            .max(sample.ready_animation_clip_count);
        self.max_animation_player_count = self
            .max_animation_player_count
            .max(sample.animation_player_count);
        self.max_animation_graph_count = self
            .max_animation_graph_count
            .max(sample.animation_graph_count);
        self.max_power_up_count = self.max_power_up_count.max(sample.power_up_count);
        self.min_visible_power_up_count = self
            .min_visible_power_up_count
            .min(sample.visible_power_up_count);
        self.max_collected_power_up_count = self
            .max_collected_power_up_count
            .max(sample.collected_power_up_count);
        self.total_power_up_activations = self
            .total_power_up_activations
            .max(sample.total_power_up_activations);
        if sample.active_power_up_effects > 0 {
            self.power_up_effect_samples += 1;
        }
        if sample.on_landing_target {
            self.target_landing_samples += 1;
        }
        if sample.active_lift_fields > 0 {
            self.lifted_samples += 1;
            if sample.readable_lift_fields > 0 {
                self.readable_lift_samples += 1;
            } else {
                self.unreadable_lift_samples += 1;
            }
        }

        match sample.mode {
            "gliding" => self.gliding_samples += 1,
            "launching" => self.launching_samples += 1,
            "grounded" => self.grounded_samples += 1,
            _ => {}
        }

        self.final_sample = Some(sample);
    }

    pub fn summary(&self, scenario: EvalScenario, artifacts: EvalArtifacts) -> EvalSummary {
        let horizontal_distance_m = match (&self.first_sample, &self.final_sample) {
            (Some(first), Some(final_sample)) => {
                horizontal_distance(first.position, final_sample.position)
            }
            _ => 0.0,
        };
        let thresholds = scenario.thresholds;
        let final_target_distance_m = self
            .final_sample
            .as_ref()
            .map_or(0.0, |sample| sample.target_distance_m);
        let final_objective_completed_count = self
            .final_sample
            .as_ref()
            .map_or(0, |sample| sample.objective.completed_count);
        let final_objective_distance_m = self
            .final_sample
            .as_ref()
            .map_or(0.0, |sample| sample.objective.current_distance_m);
        let frame_time_stats = EvalFrameTimeStats::from_samples(&self.frame_times_ms);
        let avg_desired_body_heading_error_degrees = if self.desired_body_heading_samples == 0 {
            0.0
        } else {
            self.desired_body_heading_error_sum_degrees / self.desired_body_heading_samples as f32
        };
        let mut desired_body_heading_error_values_degrees =
            self.desired_body_heading_error_values_degrees.clone();
        desired_body_heading_error_values_degrees.sort_by(f32::total_cmp);
        let p95_desired_body_heading_error_degrees =
            percentile(&desired_body_heading_error_values_degrees, 0.95);
        let avg_camera_follow_direction_error_degrees =
            if self.camera_follow_direction_error_samples == 0 {
                0.0
            } else {
                self.camera_follow_direction_error_sum_degrees
                    / self.camera_follow_direction_error_samples as f32
            };
        let mut camera_follow_direction_error_values_degrees =
            self.camera_follow_direction_error_values_degrees.clone();
        camera_follow_direction_error_values_degrees.sort_by(f32::total_cmp);
        let p95_camera_follow_direction_error_degrees =
            percentile(&camera_follow_direction_error_values_degrees, 0.95);
        let lateral_response_latency_secs = response_latency_secs(
            self.first_lateral_input_time_secs,
            self.first_lateral_response_time_secs,
            scenario,
        );
        let right_lateral_response_latency_secs = response_latency_secs(
            self.first_right_lateral_input_time_secs,
            self.first_right_lateral_response_time_secs,
            scenario,
        );
        let left_lateral_response_latency_secs = response_latency_secs(
            self.first_left_lateral_input_time_secs,
            self.first_left_lateral_response_time_secs,
            scenario,
        );
        let backward_lateral_response_latency_secs = response_latency_secs(
            self.first_backward_lateral_input_time_secs,
            self.first_backward_lateral_response_time_secs,
            scenario,
        );
        let backward_right_lateral_response_latency_secs = response_latency_secs(
            self.first_backward_right_lateral_input_time_secs,
            self.first_backward_right_lateral_response_time_secs,
            scenario,
        );
        let backward_left_lateral_response_latency_secs = response_latency_secs(
            self.first_backward_left_lateral_input_time_secs,
            self.first_backward_left_lateral_response_time_secs,
            scenario,
        );
        let mut checks = vec![
            EvalCheck::at_least(
                "sample_count",
                self.sample_count as f32,
                thresholds.min_samples as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "horizontal_distance",
                horizontal_distance_m,
                thresholds.min_horizontal_distance_m,
                "m",
            ),
            EvalCheck::at_least(
                "max_altitude",
                self.max_altitude_m,
                thresholds.min_max_altitude_m,
                "m",
            ),
            EvalCheck::at_least(
                "max_speed",
                self.max_speed_mps,
                thresholds.min_max_speed_mps,
                "m/s",
            ),
            EvalCheck::at_least(
                "gliding_samples",
                self.gliding_samples as f32,
                thresholds.min_gliding_samples as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "grounded_samples",
                self.grounded_samples as f32,
                thresholds.min_grounded_samples as f32,
                "samples",
            ),
            EvalCheck::at_most(
                "grounded_visual_foot_gap",
                self.max_grounded_visual_foot_gap_m,
                MAX_GROUNDED_VISUAL_FOOT_GAP_M,
                "m",
            ),
            EvalCheck::at_least(
                "lifted_samples",
                self.lifted_samples as f32,
                thresholds.min_lifted_samples as f32,
                "samples",
            ),
            EvalCheck::at_least(
                "sky_island_count",
                self.max_sky_island_count as f32,
                thresholds.min_sky_island_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "active_island_count",
                self.max_active_island_count as f32,
                thresholds.min_active_island_count as f32,
                "islands",
            ),
            EvalCheck::at_most(
                "active_chunk_count",
                self.max_active_chunk_count as f32,
                thresholds.max_active_chunk_count as f32,
                "chunks",
            ),
            EvalCheck::at_least(
                "near_lod_island_count",
                self.max_near_lod_islands as f32,
                thresholds.min_near_lod_island_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "mid_lod_island_count",
                self.max_mid_lod_islands as f32,
                thresholds.min_mid_lod_island_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "far_lod_island_count",
                self.max_far_lod_islands as f32,
                thresholds.min_far_lod_island_count as f32,
                "islands",
            ),
            EvalCheck::at_most(
                "visible_island_terrain_count",
                self.max_visible_island_terrain_count as f32,
                thresholds.max_visible_island_terrain_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "hidden_island_terrain_count",
                self.max_hidden_island_terrain_count as f32,
                thresholds.min_hidden_island_terrain_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "visible_island_impostor_count",
                self.max_visible_island_impostor_count as f32,
                thresholds.min_visible_island_impostor_count as f32,
                "entities",
            ),
            EvalCheck::at_most(
                "visible_island_detail_count",
                self.max_visible_island_detail_count as f32,
                thresholds.max_visible_island_detail_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "hidden_island_detail_count",
                self.max_hidden_island_detail_count as f32,
                thresholds.min_hidden_island_detail_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "visible_route_beacon_count",
                self.max_visible_route_beacon_count as f32,
                thresholds.min_visible_route_beacon_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "weather_cloud_count",
                self.max_weather_cloud_count as f32,
                thresholds.min_weather_cloud_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "environment_motion_visual_count",
                self.max_environment_motion_visual_count as f32,
                thresholds.min_environment_motion_visual_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "environment_motion_offset",
                self.max_environment_motion_offset_m,
                thresholds.min_environment_motion_offset_m,
                "m",
            ),
            EvalCheck::at_least(
                "island_terrain_surface_count",
                self.min_island_terrain_surface_count as f32,
                thresholds.min_island_terrain_surface_count as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "island_terrain_mesh_vertices",
                self.min_island_terrain_mesh_vertices as f32,
                thresholds.min_island_terrain_mesh_vertices as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "island_terrain_color_bands",
                self.min_island_terrain_color_bands as f32,
                thresholds.min_island_terrain_color_bands as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_terrain_material_weight_bands",
                self.min_island_terrain_material_weight_bands as f32,
                MIN_ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_terrain_material_channels",
                self.min_island_terrain_material_channels as f32,
                MIN_ISLAND_TERRAIN_MATERIAL_CHANNELS as f32,
                "channels",
            ),
            EvalCheck::at_least(
                "island_terrain_material_regions",
                self.min_island_terrain_material_regions as f32,
                MIN_ISLAND_TERRAIN_MATERIAL_REGIONS as f32,
                "regions",
            ),
            EvalCheck::at_least(
                "island_terrain_texture_detail_bands",
                self.min_island_terrain_texture_detail_bands as f32,
                MIN_ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_terrain_relief_range",
                self.min_island_terrain_relief_range_m,
                thresholds.min_island_terrain_relief_range_m,
                "m",
            ),
            EvalCheck::at_least(
                "island_cliff_color_bands",
                self.min_island_cliff_color_bands as f32,
                thresholds.min_island_cliff_color_bands as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "island_impostor_mesh_vertices",
                self.min_island_impostor_mesh_vertices as f32,
                MIN_ISLAND_IMPOSTOR_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "island_impostor_color_bands",
                self.min_island_impostor_color_bands as f32,
                MIN_ISLAND_IMPOSTOR_COLOR_BANDS as f32,
                "bands",
            ),
            EvalCheck::at_least(
                "procedural_island_body_count",
                self.min_procedural_island_body_count as f32,
                thresholds.min_procedural_island_body_count as f32,
                "islands",
            ),
            EvalCheck::at_most(
                "primitive_island_body_count",
                self.max_primitive_island_body_count as f32,
                thresholds.max_primitive_island_body_count as f32,
                "islands",
            ),
            EvalCheck::at_least(
                "island_body_silhouette_segments",
                self.min_island_body_silhouette_segments as f32,
                thresholds.min_island_body_silhouette_segments as f32,
                "segments",
            ),
            EvalCheck::at_least(
                "island_body_mesh_vertices",
                self.min_island_body_mesh_vertices as f32,
                MIN_ISLAND_BODY_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "generated_ground_cover_patch_count",
                self.min_generated_ground_cover_patch_count as f32,
                MIN_GENERATED_GROUND_COVER_PATCH_COUNT as f32,
                "patches",
            ),
            EvalCheck::at_least(
                "ground_cover_blade_count",
                self.min_ground_cover_blade_count as f32,
                MIN_GROUND_COVER_BLADE_COUNT as f32,
                "blades",
            ),
            EvalCheck::at_least(
                "ground_cover_mesh_vertices",
                self.min_ground_cover_mesh_vertices as f32,
                MIN_GROUND_COVER_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "generated_tree_trunk_count",
                self.min_generated_tree_trunk_count as f32,
                MIN_GENERATED_TREE_TRUNK_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "generated_tree_canopy_count",
                self.min_generated_tree_canopy_count as f32,
                MIN_GENERATED_TREE_CANOPY_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "tree_trunk_mesh_vertices",
                self.min_tree_trunk_mesh_vertices as f32,
                MIN_TREE_TRUNK_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "tree_canopy_mesh_vertices",
                self.min_tree_canopy_mesh_vertices as f32,
                MIN_TREE_CANOPY_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "detail_biome_palette_count",
                self.min_detail_biome_palette_count as f32,
                MIN_DETAIL_BIOME_PALETTE_COUNT as f32,
                "palettes",
            ),
            EvalCheck::at_least(
                "generated_rock_count",
                self.min_generated_rock_count as f32,
                MIN_GENERATED_ROCK_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "rock_mesh_vertices",
                self.min_rock_mesh_vertices as f32,
                MIN_ROCK_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_least(
                "generated_weather_cloud_count",
                self.min_generated_weather_cloud_count as f32,
                MIN_GENERATED_WEATHER_CLOUD_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "generated_weather_cloud_bank_count",
                self.min_generated_weather_cloud_bank_count as f32,
                MIN_GENERATED_WEATHER_CLOUD_BANK_COUNT as f32,
                "meshes",
            ),
            EvalCheck::at_least(
                "weather_cloud_bank_depth",
                self.min_weather_cloud_bank_depth_m,
                MIN_WEATHER_CLOUD_BANK_DEPTH_M,
                "m",
            ),
            EvalCheck::at_least(
                "weather_cloud_lobe_count",
                self.min_weather_cloud_lobe_count as f32,
                MIN_WEATHER_CLOUD_LOBE_COUNT as f32,
                "lobes",
            ),
            EvalCheck::at_least(
                "weather_cloud_bank_lobe_count",
                self.min_max_weather_cloud_lobe_count as f32,
                MIN_MAX_WEATHER_CLOUD_LOBE_COUNT as f32,
                "lobes",
            ),
            EvalCheck::at_least(
                "weather_cloud_mesh_vertices",
                self.min_weather_cloud_mesh_vertices as f32,
                MIN_WEATHER_CLOUD_MESH_VERTICES as f32,
                "vertices",
            ),
            EvalCheck::at_most(
                "resident_island_visual_count",
                self.max_resident_island_visual_count as f32,
                thresholds.max_resident_island_visual_count as f32,
                "entities",
            ),
            EvalCheck::at_most(
                "stream_visibility_changes_per_frame",
                self.max_stream_visibility_changes_per_frame as f32,
                thresholds.max_stream_visibility_changes_per_frame as f32,
                "entities/frame",
            ),
            EvalCheck::at_least(
                "hidden_island_visual_count",
                self.max_hidden_island_visual_count as f32,
                (thresholds.min_hidden_island_terrain_count
                    + thresholds.min_hidden_island_detail_count) as f32,
                "entities",
            ),
            EvalCheck::at_most(
                "resident_island_visual_fraction",
                self.max_resident_island_visual_fraction,
                MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
                "ratio",
            ),
            EvalCheck::at_most(
                "stream_spawned_visuals_per_frame",
                self.max_stream_spawned_visuals_per_frame as f32,
                thresholds.max_stream_visibility_changes_per_frame as f32,
                "entities/frame",
            ),
            EvalCheck::at_most(
                "stream_despawned_visuals_per_frame",
                self.max_stream_despawned_visuals_per_frame as f32,
                thresholds.max_stream_visibility_changes_per_frame as f32,
                "entities/frame",
            ),
            EvalCheck::at_least(
                "entity_count",
                self.max_entity_count as f32,
                thresholds.min_entity_count as f32,
                "entities",
            ),
            EvalCheck::at_least(
                "objective_total_count",
                self.max_objective_total_count as f32,
                thresholds.min_objective_total_count as f32,
                "objectives",
            ),
            EvalCheck::at_least(
                "completed_objective_count",
                self.max_completed_objective_count as f32,
                thresholds.min_completed_objective_count as f32,
                "objectives",
            ),
            EvalCheck::at_least(
                "visual_asset_slot_count",
                self.max_visual_asset_slot_count as f32,
                thresholds.min_visual_asset_slot_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "gltf_scene_asset_slot_count",
                self.max_gltf_scene_asset_slot_count as f32,
                thresholds.min_gltf_scene_asset_slot_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "ready_visual_asset_slot_count",
                self.max_ready_visual_asset_slot_count as f32,
                MIN_READY_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_most(
                "missing_visual_asset_slot_count",
                self.max_missing_visual_asset_slot_count as f32,
                MAX_MISSING_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_most(
                "deferred_visual_asset_scene_count",
                self.max_deferred_visual_asset_scene_count as f32,
                MAX_DEFERRED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "streaming_visual_asset_slot_count",
                self.max_streaming_visual_asset_slot_count as f32,
                thresholds.min_streaming_visual_asset_slot_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "loaded_visual_asset_scene_count",
                self.max_loaded_visual_asset_scene_count as f32,
                MIN_LOADED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "dependency_loaded_visual_asset_scene_count",
                self.max_dependency_loaded_visual_asset_scene_count as f32,
                MIN_DEPENDENCY_LOADED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "preload_ready_visual_asset_scene_count",
                self.max_preload_ready_visual_asset_scene_count as f32,
                MIN_PRELOAD_READY_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "always_preload_ready_visual_asset_slot_count",
                self.max_always_preload_ready_visual_asset_slot_count as f32,
                MIN_ALWAYS_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "streaming_preload_ready_visual_asset_slot_count",
                self.max_streaming_preload_ready_visual_asset_slot_count as f32,
                MIN_STREAMING_PRELOAD_READY_VISUAL_ASSET_SLOT_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "spawned_visual_asset_scene_count",
                self.max_spawned_visual_asset_scene_count as f32,
                MIN_SPAWNED_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "ready_visual_asset_scene_count",
                self.max_ready_visual_asset_scene_count as f32,
                MIN_READY_VISUAL_ASSET_SCENE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "visible_authored_world_fixture_count",
                self.max_visible_authored_world_fixture_count as f32,
                MIN_VISIBLE_AUTHORED_WORLD_FIXTURE_COUNT as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "declared_animation_clip_count",
                self.max_declared_animation_clip_count as f32,
                thresholds.min_declared_animation_clip_count as f32,
                "clips",
            ),
            EvalCheck::at_least(
                "ready_animation_clip_count",
                self.max_ready_animation_clip_count as f32,
                MIN_READY_VISUAL_ANIMATION_CLIP_COUNT as f32,
                "clips",
            ),
            EvalCheck::at_least(
                "animation_player_count",
                self.max_animation_player_count as f32,
                MIN_VISUAL_ANIMATION_PLAYER_COUNT as f32,
                "players",
            ),
            EvalCheck::at_least(
                "animation_graph_count",
                self.max_animation_graph_count as f32,
                MIN_VISUAL_ANIMATION_GRAPH_COUNT as f32,
                "graphs",
            ),
            EvalCheck::at_most(
                "failed_visual_asset_scene_count",
                self.max_failed_visual_asset_scene_count as f32,
                thresholds.max_failed_visual_asset_scene_count as f32,
                "assets",
            ),
            EvalCheck::at_least(
                "power_up_count",
                self.max_power_up_count as f32,
                thresholds.min_power_up_count as f32,
                "power-ups",
            ),
            EvalCheck::at_least(
                "collected_power_up_count",
                self.max_collected_power_up_count as f32,
                thresholds.min_collected_power_up_count as f32,
                "power-ups",
            ),
            EvalCheck::at_least(
                "power_up_effect_samples",
                self.power_up_effect_samples as f32,
                thresholds.min_power_up_effect_samples as f32,
                "samples",
            ),
            EvalCheck::at_most(
                "max_camera_distance",
                self.max_camera_distance_m,
                thresholds.max_camera_distance_m,
                "m",
            ),
            EvalCheck::at_least(
                "min_camera_surface_clearance",
                self.min_camera_surface_clearance_m,
                thresholds.min_camera_surface_clearance_m,
                "m",
            ),
            EvalCheck::at_most(
                "max_camera_player_angle",
                self.max_camera_player_angle_degrees,
                thresholds.max_camera_player_angle_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "max_camera_step_distance",
                self.max_camera_step_distance_m,
                thresholds.max_camera_step_distance_m,
                "m",
            ),
            EvalCheck::at_most(
                "max_camera_rotation_delta",
                self.max_camera_rotation_delta_degrees,
                thresholds.max_camera_rotation_delta_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "max_camera_orbit_alignment",
                self.max_camera_orbit_alignment_degrees,
                thresholds.max_camera_orbit_alignment_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "max_abs_camera_view_yaw",
                self.max_abs_camera_view_yaw_degrees,
                thresholds.max_abs_camera_view_yaw_degrees,
                "deg",
            ),
            EvalCheck::at_least(
                "max_camera_obstruction_adjustment",
                self.max_camera_obstruction_adjustment_m,
                thresholds.min_camera_obstruction_adjustment_m,
                "m",
            ),
            EvalCheck::at_least(
                "max_abs_camera_yaw_offset",
                self.max_abs_camera_yaw_offset_degrees,
                thresholds.min_abs_camera_yaw_degrees,
                "deg",
            ),
            EvalCheck::at_most(
                "min_camera_pitch_offset",
                self.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            ),
            EvalCheck::at_least(
                "max_camera_pitch_offset",
                self.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            ),
        ];
        if thresholds.min_lifted_samples > 0 {
            checks.push(EvalCheck::at_least(
                "readable_lift_samples",
                self.readable_lift_samples as f32,
                thresholds.min_lifted_samples as f32,
                "samples",
            ));
            checks.push(EvalCheck::at_most(
                "unreadable_lift_samples",
                self.unreadable_lift_samples as f32,
                0.0,
                "samples",
            ));
        }
        if thresholds.require_target_landing {
            checks.push(EvalCheck::at_most(
                "final_target_distance",
                final_target_distance_m,
                thresholds.max_final_target_distance_m,
                "m",
            ));
            checks.push(EvalCheck::at_least(
                "target_landing_samples",
                self.target_landing_samples as f32,
                thresholds.min_target_landing_samples as f32,
                "samples",
            ));
        }
        if scenario.name == AIR_CONTROL_RESPONSE {
            checks.push(EvalCheck::at_most(
                "air_control_lateral_response_latency",
                lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_lateral_response",
                self.max_lateral_response_mps,
                AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_right_lateral_response_latency",
                right_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_right_lateral_response",
                self.max_right_lateral_response_mps,
                AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_left_lateral_response_latency",
                left_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_left_lateral_response",
                self.max_left_lateral_response_mps,
                AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_backward_lateral_response_latency",
                backward_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_lateral_response",
                self.max_backward_lateral_response_mps,
                AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_backward_right_lateral_response_latency",
                backward_right_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_right_lateral_response",
                self.max_backward_right_lateral_response_mps,
                AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_right_rear_response",
                self.max_backward_right_rear_response_mps,
                AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_backward_left_lateral_response_latency",
                backward_left_lateral_response_latency_secs,
                AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                "s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_left_lateral_response",
                self.max_backward_left_lateral_response_mps,
                AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_backward_left_rear_response",
                self.max_backward_left_rear_response_mps,
                AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_air_brake_speed_drop",
                self.max_air_brake_speed_drop_mps,
                AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_air_brake_planar_speed_drop",
                self.max_air_brake_planar_speed_drop_mps,
                AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_post_brake_forward_alignment",
                self.max_post_brake_forward_alignment_mps,
                AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_desired_heading_alignment",
                self.max_desired_heading_alignment_mps,
                AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_avg_body_heading_error",
                avg_desired_body_heading_error_degrees,
                AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_p95_body_heading_error",
                p95_desired_body_heading_error_degrees,
                AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_max_body_heading_error",
                self.max_desired_body_heading_error_degrees,
                AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_max_body_yaw_error_step",
                self.max_body_yaw_error_step_degrees,
                AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_body_yaw_oscillation_count",
                self.body_yaw_oscillation_count as f32,
                AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
                "sign changes",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_right_body_bank_response",
                self.max_right_body_bank_degrees,
                AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_least(
                "air_control_left_body_bank_response",
                self.max_left_body_bank_degrees,
                AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_max_body_roll_step",
                self.max_body_roll_step_degrees,
                AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_orbit_yaw_offset",
                self.max_abs_camera_yaw_offset_degrees,
                AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_rotation_delta",
                self.max_camera_rotation_delta_degrees,
                AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_view_yaw_drift",
                self.max_camera_view_yaw_drift_degrees,
                AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_avg_camera_follow_direction_error",
                avg_camera_follow_direction_error_degrees,
                AIR_CONTROL_MAX_AVG_CAMERA_FOLLOW_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_p95_camera_follow_direction_error",
                p95_camera_follow_direction_error_degrees,
                AIR_CONTROL_MAX_P95_CAMERA_FOLLOW_ERROR_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "air_control_camera_world_yaw_drift",
                self.max_camera_world_yaw_drift_degrees,
                MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                "deg",
            ));
        }
        if scenario.name == CAMERA_STRAFE_STABILITY {
            checks.push(EvalCheck::at_least(
                "camera_strafe_right_lateral_response",
                self.max_right_lateral_response_mps,
                CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_least(
                "camera_strafe_left_lateral_response",
                self.max_left_lateral_response_mps,
                CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                "m/s",
            ));
            checks.push(EvalCheck::at_most(
                "camera_strafe_view_yaw_drift",
                self.max_camera_view_yaw_drift_degrees,
                CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
                "deg",
            ));
            checks.push(EvalCheck::at_most(
                "camera_strafe_world_yaw_drift",
                self.max_camera_world_yaw_drift_degrees,
                MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                "deg",
            ));
        }
        let passed = checks.iter().all(|check| check.passed);

        EvalSummary {
            scenario_name: scenario.name,
            target_island_name: scenario.target_island_name,
            passed,
            frame_count: scenario.frame_count,
            duration_secs: scenario.duration_secs(),
            thresholds,
            metrics: EvalMetricsSummary {
                sample_count: self.sample_count,
                avg_frame_time_ms: frame_time_stats.avg_ms,
                p95_frame_time_ms: frame_time_stats.p95_ms,
                p99_frame_time_ms: frame_time_stats.p99_ms,
                max_frame_time_ms: frame_time_stats.max_ms,
                horizontal_distance_m,
                max_altitude_m: self.max_altitude_m,
                min_altitude_m: self.min_altitude_m,
                max_grounded_visual_foot_gap_m: self.max_grounded_visual_foot_gap_m,
                max_speed_mps: self.max_speed_mps,
                max_camera_distance_m: self.max_camera_distance_m,
                min_camera_surface_clearance_m: self.min_camera_surface_clearance_m,
                max_camera_player_angle_degrees: self.max_camera_player_angle_degrees,
                max_camera_step_distance_m: self.max_camera_step_distance_m,
                max_camera_rotation_delta_degrees: self.max_camera_rotation_delta_degrees,
                max_camera_orbit_alignment_degrees: self.max_camera_orbit_alignment_degrees,
                avg_camera_follow_direction_error_degrees,
                p95_camera_follow_direction_error_degrees,
                max_camera_follow_direction_error_degrees: self
                    .max_camera_follow_direction_error_degrees,
                max_abs_camera_view_yaw_degrees: self.max_abs_camera_view_yaw_degrees,
                max_camera_view_yaw_drift_degrees: self.max_camera_view_yaw_drift_degrees,
                max_camera_world_yaw_drift_degrees: self.max_camera_world_yaw_drift_degrees,
                max_camera_obstruction_adjustment_m: self.max_camera_obstruction_adjustment_m,
                max_camera_obstruction_hits: self.max_camera_obstruction_hits,
                avg_desired_body_heading_error_degrees,
                p95_desired_body_heading_error_degrees,
                max_desired_body_heading_error_degrees: self.max_desired_body_heading_error_degrees,
                max_body_yaw_error_step_degrees: self.max_body_yaw_error_step_degrees,
                body_yaw_oscillation_count: self.body_yaw_oscillation_count,
                max_body_roll_step_degrees: self.max_body_roll_step_degrees,
                max_right_body_bank_degrees: self.max_right_body_bank_degrees,
                max_left_body_bank_degrees: self.max_left_body_bank_degrees,
                max_desired_heading_alignment_mps: self.max_desired_heading_alignment_mps,
                max_lateral_response_mps: self.max_lateral_response_mps,
                lateral_response_latency_secs,
                max_right_lateral_response_mps: self.max_right_lateral_response_mps,
                right_lateral_response_latency_secs,
                max_left_lateral_response_mps: self.max_left_lateral_response_mps,
                left_lateral_response_latency_secs,
                max_backward_lateral_response_mps: self.max_backward_lateral_response_mps,
                backward_lateral_response_latency_secs,
                max_backward_right_lateral_response_mps: self
                    .max_backward_right_lateral_response_mps,
                backward_right_lateral_response_latency_secs,
                max_backward_right_rear_response_mps: self.max_backward_right_rear_response_mps,
                max_backward_left_lateral_response_mps: self.max_backward_left_lateral_response_mps,
                backward_left_lateral_response_latency_secs,
                max_backward_left_rear_response_mps: self.max_backward_left_rear_response_mps,
                max_air_brake_speed_drop_mps: self.max_air_brake_speed_drop_mps,
                max_air_brake_planar_speed_drop_mps: self.max_air_brake_planar_speed_drop_mps,
                max_post_brake_forward_alignment_mps: self.max_post_brake_forward_alignment_mps,
                min_target_distance_m: self.min_target_distance_m,
                final_target_distance_m,
                min_camera_pitch_degrees: self.min_camera_pitch_degrees,
                max_camera_pitch_degrees: self.max_camera_pitch_degrees,
                max_abs_camera_yaw_offset_degrees: self.max_abs_camera_yaw_offset_degrees,
                min_camera_pitch_offset_degrees: self.min_camera_pitch_offset_degrees,
                max_camera_pitch_offset_degrees: self.max_camera_pitch_offset_degrees,
                max_visible_wind_fields: self.max_visible_wind_fields,
                max_active_lift_fields: self.max_active_lift_fields,
                max_readable_lift_fields: self.max_readable_lift_fields,
                max_sky_island_count: self.max_sky_island_count,
                max_active_chunk_count: self.max_active_chunk_count,
                max_active_island_count: self.max_active_island_count,
                max_near_lod_islands: self.max_near_lod_islands,
                max_mid_lod_islands: self.max_mid_lod_islands,
                max_far_lod_islands: self.max_far_lod_islands,
                max_visible_island_terrain_count: self.max_visible_island_terrain_count,
                max_hidden_island_terrain_count: self.max_hidden_island_terrain_count,
                max_visible_island_impostor_count: self.max_visible_island_impostor_count,
                max_hidden_island_impostor_count: self.max_hidden_island_impostor_count,
                max_visible_island_detail_count: self.max_visible_island_detail_count,
                max_hidden_island_detail_count: self.max_hidden_island_detail_count,
                max_visible_route_beacon_count: self.max_visible_route_beacon_count,
                max_weather_cloud_count: self.max_weather_cloud_count,
                max_environment_motion_visual_count: self.max_environment_motion_visual_count,
                max_environment_motion_offset_m: self.max_environment_motion_offset_m,
                min_island_terrain_surface_count: self.min_island_terrain_surface_count,
                min_island_terrain_mesh_vertices: self.min_island_terrain_mesh_vertices,
                min_island_terrain_color_bands: self.min_island_terrain_color_bands,
                min_island_terrain_material_weight_bands: self
                    .min_island_terrain_material_weight_bands,
                min_island_terrain_material_channels: self.min_island_terrain_material_channels,
                min_island_terrain_material_regions: self.min_island_terrain_material_regions,
                min_island_terrain_texture_detail_bands: self
                    .min_island_terrain_texture_detail_bands,
                min_island_terrain_relief_range_m: self.min_island_terrain_relief_range_m,
                min_island_cliff_color_bands: self.min_island_cliff_color_bands,
                min_island_impostor_mesh_vertices: self.min_island_impostor_mesh_vertices,
                min_island_impostor_color_bands: self.min_island_impostor_color_bands,
                min_procedural_island_body_count: self.min_procedural_island_body_count,
                max_primitive_island_body_count: self.max_primitive_island_body_count,
                min_island_body_silhouette_segments: self.min_island_body_silhouette_segments,
                max_avg_island_body_silhouette_segments: self
                    .max_avg_island_body_silhouette_segments,
                min_island_body_mesh_vertices: self.min_island_body_mesh_vertices,
                max_island_body_mesh_vertices: self.max_island_body_mesh_vertices,
                min_generated_ground_cover_patch_count: self.min_generated_ground_cover_patch_count,
                min_ground_cover_blade_count: self.min_ground_cover_blade_count,
                min_ground_cover_mesh_vertices: self.min_ground_cover_mesh_vertices,
                min_generated_tree_trunk_count: self.min_generated_tree_trunk_count,
                min_generated_tree_canopy_count: self.min_generated_tree_canopy_count,
                min_tree_trunk_mesh_vertices: self.min_tree_trunk_mesh_vertices,
                min_tree_canopy_mesh_vertices: self.min_tree_canopy_mesh_vertices,
                min_detail_biome_palette_count: self.min_detail_biome_palette_count,
                min_generated_rock_count: self.min_generated_rock_count,
                min_rock_mesh_vertices: self.min_rock_mesh_vertices,
                min_generated_weather_cloud_count: self.min_generated_weather_cloud_count,
                min_generated_weather_cloud_bank_count: self.min_generated_weather_cloud_bank_count,
                min_weather_cloud_bank_depth_m: self.min_weather_cloud_bank_depth_m,
                min_weather_cloud_lobe_count: self.min_weather_cloud_lobe_count,
                min_max_weather_cloud_lobe_count: self.min_max_weather_cloud_lobe_count,
                min_weather_cloud_mesh_vertices: self.min_weather_cloud_mesh_vertices,
                max_resident_island_visual_count: self.max_resident_island_visual_count,
                max_stream_visibility_changes_per_frame: self
                    .max_stream_visibility_changes_per_frame,
                total_stream_visibility_changes: self.total_stream_visibility_changes,
                max_catalog_island_visual_count: self.max_catalog_island_visual_count,
                max_hidden_island_visual_count: self.max_hidden_island_visual_count,
                max_resident_island_visual_fraction: self.max_resident_island_visual_fraction,
                max_stream_spawned_visuals_per_frame: self.max_stream_spawned_visuals_per_frame,
                max_stream_despawned_visuals_per_frame: self.max_stream_despawned_visuals_per_frame,
                total_stream_spawned_visuals: self.total_stream_spawned_visuals,
                total_stream_despawned_visuals: self.total_stream_despawned_visuals,
                max_entity_count: self.max_entity_count,
                objective_total_count: self.max_objective_total_count,
                max_completed_objective_count: self.max_completed_objective_count,
                final_objective_completed_count,
                min_objective_distance_m: self.min_objective_distance_m,
                final_objective_distance_m,
                objective_complete_samples: self.objective_complete_samples,
                max_visual_asset_slot_count: self.max_visual_asset_slot_count,
                max_gltf_scene_asset_slot_count: self.max_gltf_scene_asset_slot_count,
                max_ready_visual_asset_slot_count: self.max_ready_visual_asset_slot_count,
                max_placeholder_visual_asset_slot_count: self
                    .max_placeholder_visual_asset_slot_count,
                max_streaming_visual_asset_slot_count: self.max_streaming_visual_asset_slot_count,
                max_missing_visual_asset_slot_count: self.max_missing_visual_asset_slot_count,
                max_deferred_visual_asset_scene_count: self.max_deferred_visual_asset_scene_count,
                max_queued_visual_asset_scene_count: self.max_queued_visual_asset_scene_count,
                max_loading_visual_asset_scene_count: self.max_loading_visual_asset_scene_count,
                max_loaded_visual_asset_scene_count: self.max_loaded_visual_asset_scene_count,
                max_dependency_loaded_visual_asset_scene_count: self
                    .max_dependency_loaded_visual_asset_scene_count,
                max_preload_ready_visual_asset_scene_count: self
                    .max_preload_ready_visual_asset_scene_count,
                max_failed_visual_asset_scene_count: self.max_failed_visual_asset_scene_count,
                max_spawned_visual_asset_scene_count: self.max_spawned_visual_asset_scene_count,
                max_ready_visual_asset_scene_count: self.max_ready_visual_asset_scene_count,
                max_visible_authored_world_fixture_count: self
                    .max_visible_authored_world_fixture_count,
                max_always_visual_asset_slot_count: self.max_always_visual_asset_slot_count,
                max_stream_window_visual_asset_slot_count: self
                    .max_stream_window_visual_asset_slot_count,
                max_near_lod_visual_asset_slot_count: self.max_near_lod_visual_asset_slot_count,
                max_far_lod_visual_asset_slot_count: self.max_far_lod_visual_asset_slot_count,
                max_weather_visual_asset_slot_count: self.max_weather_visual_asset_slot_count,
                max_always_preload_ready_visual_asset_slot_count: self
                    .max_always_preload_ready_visual_asset_slot_count,
                max_streaming_preload_ready_visual_asset_slot_count: self
                    .max_streaming_preload_ready_visual_asset_slot_count,
                max_declared_animation_clip_count: self.max_declared_animation_clip_count,
                max_ready_animation_clip_count: self.max_ready_animation_clip_count,
                max_animation_player_count: self.max_animation_player_count,
                max_animation_graph_count: self.max_animation_graph_count,
                max_power_up_count: self.max_power_up_count,
                min_visible_power_up_count: self.min_visible_power_up_count,
                max_collected_power_up_count: self.max_collected_power_up_count,
                power_up_effect_samples: self.power_up_effect_samples,
                total_power_up_activations: self.total_power_up_activations,
                target_landing_samples: self.target_landing_samples,
                lifted_samples: self.lifted_samples,
                readable_lift_samples: self.readable_lift_samples,
                unreadable_lift_samples: self.unreadable_lift_samples,
                gliding_samples: self.gliding_samples,
                launching_samples: self.launching_samples,
                grounded_samples: self.grounded_samples,
            },
            checks,
            artifacts,
            final_sample: self.final_sample.clone(),
        }
    }
}

fn vec3_array(value: Vec3) -> [f32; 3] {
    [value.x, value.y, value.z]
}

fn horizontal_distance(start: [f32; 3], end: [f32; 3]) -> f32 {
    let dx = end[0] - start[0];
    let dz = end[2] - start[2];
    (dx * dx + dz * dz).sqrt()
}

fn percentile(sorted_values: &[f32], percentile: f32) -> f32 {
    if sorted_values.is_empty() {
        return 0.0;
    }

    let index = ((sorted_values.len() as f32 * percentile).ceil() as usize)
        .saturating_sub(1)
        .min(sorted_values.len() - 1);
    sorted_values[index]
}

fn json_array3(values: [f32; 3]) -> String {
    format!(
        "[{},{},{}]",
        json_number(values[0]),
        json_number(values[1]),
        json_number(values[2])
    )
}

fn json_number(value: f32) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "null".to_string()
    }
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests;
