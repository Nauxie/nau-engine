use crate::movement::FlightMode;
use bevy::prelude::Vec2;

#[path = "accumulator/summary_report.rs"]
mod summary_report;

use super::{sample::EvalSample, thresholds::*};

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
}
