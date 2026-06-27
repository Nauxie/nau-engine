use bevy::prelude::Vec2;
use nau_engine::animation::MIN_KEY_POSE_READABILITY_SCORE;
use nau_engine::eval::{
    CAMERA_STRAFE_STABILITY, EvalScenario, MIN_CROSSWIND_FORCE_DELTA_MPS,
    MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS, MIN_WIND_FORCE_ALIGNED_DELTA_MPS, MIN_WIND_FORCE_DELTA_MPS,
    MIN_WIND_FORCE_FLOW_ALIGNMENT, MIN_WIND_FORCE_VARIATION, MIN_WIND_LOAD_LATERAL_LOAD,
};

use super::super::{
    AIR_CONTROL_RESPONSE_THRESHOLD_MPS, AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES, SimSample,
};
use super::{
    SimMetrics,
    util::{backward_diagonal_rear_response_mps, horizontal_distance},
};

const BODY_YAW_INTENT_AXIS_EPSILON: f32 = 0.05;
const BODY_YAW_INTENT_CHANGE_DOT: f32 = 0.98;

impl SimMetrics {
    pub(crate) fn observe(&mut self, sample: &SimSample, scenario: EvalScenario) {
        self.sample_count += 1;
        self.final_position = sample.position;
        self.horizontal_distance_m = self
            .horizontal_distance_m
            .max(horizontal_distance(self.start_position, sample.position));
        self.max_altitude_m = self.max_altitude_m.max(sample.altitude_m);
        self.min_altitude_m = self.min_altitude_m.min(sample.altitude_m);
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
        self.max_abs_camera_view_yaw_degrees = self
            .max_abs_camera_view_yaw_degrees
            .max(sample.camera_view_yaw_degrees.abs());
        let first_view_yaw = self
            .first_camera_view_yaw_degrees
            .get_or_insert(sample.camera_view_yaw_degrees);
        self.max_camera_view_yaw_drift_degrees = self
            .max_camera_view_yaw_drift_degrees
            .max((sample.camera_view_yaw_degrees - *first_view_yaw).abs());
        let first_world_yaw = self
            .first_camera_world_yaw_degrees
            .get_or_insert(sample.camera_world_yaw_degrees);
        self.max_camera_world_yaw_drift_degrees = self
            .max_camera_world_yaw_drift_degrees
            .max((sample.camera_world_yaw_degrees - *first_world_yaw).abs());
        self.max_camera_obstruction_adjustment_m = self
            .max_camera_obstruction_adjustment_m
            .max(sample.camera_obstruction_adjustment_m);
        self.max_camera_obstruction_hits = self
            .max_camera_obstruction_hits
            .max(sample.camera_obstruction_hits);
        self.max_abs_camera_yaw_offset_degrees = self
            .max_abs_camera_yaw_offset_degrees
            .max(sample.camera_yaw_offset_degrees.abs());
        self.min_camera_pitch_offset_degrees = self
            .min_camera_pitch_offset_degrees
            .min(sample.camera_pitch_offset_degrees);
        self.max_camera_pitch_offset_degrees = self
            .max_camera_pitch_offset_degrees
            .max(sample.camera_pitch_offset_degrees);

        if sample.desired_body_yaw_error_degrees.is_finite() {
            let current_intent_axis = body_yaw_intent_axis(sample);
            let intent_changed =
                body_yaw_intent_changed(self.previous_body_yaw_intent_axis, current_intent_axis);
            if intent_changed {
                self.previous_desired_body_yaw_error_degrees = None;
                self.previous_body_yaw_error_sign = None;
            }
            self.previous_body_yaw_intent_axis = current_intent_axis;

            self.desired_body_heading_error_sum_degrees +=
                sample.desired_body_heading_error_degrees;
            self.desired_body_heading_samples += 1;
            self.desired_body_heading_error_values_degrees
                .push(sample.desired_body_heading_error_degrees);
            self.max_desired_body_heading_error_degrees = self
                .max_desired_body_heading_error_degrees
                .max(sample.desired_body_heading_error_degrees);
            if let Some(previous) = self.previous_desired_body_yaw_error_degrees {
                self.max_body_yaw_error_step_degrees = self
                    .max_body_yaw_error_step_degrees
                    .max((sample.desired_body_yaw_error_degrees - previous).abs());
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
        } else {
            self.previous_desired_body_yaw_error_degrees = None;
            self.previous_body_yaw_intent_axis = None;
            self.previous_body_yaw_error_sign = None;
        }
        if !sample.body_roll_degrees.is_finite() || sample.mode == "grounded" {
            self.previous_body_roll_degrees = None;
        } else {
            if let Some(previous) = self.previous_body_roll_degrees {
                self.max_body_roll_step_degrees = self
                    .max_body_roll_step_degrees
                    .max((sample.body_roll_degrees - previous).abs());
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
            if sample.movement_input_forward_axis > 0.0 {
                self.max_post_brake_forward_alignment_mps = self
                    .max_post_brake_forward_alignment_mps
                    .max(sample.desired_heading_alignment_mps);
            }
        }
        self.observe_lateral_response(sample);
        self.observe_body_travel_heading_alignment(sample);
        self.observe_desired_travel_heading_alignment(sample);
        self.observe_backward_air_control(sample);
        self.max_pose_torso_pitch_degrees = self
            .max_pose_torso_pitch_degrees
            .max(sample.pose_torso_pitch_degrees);
        self.max_pose_arm_spread_degrees = self
            .max_pose_arm_spread_degrees
            .max(sample.pose_arm_spread_degrees);
        self.max_pose_leg_tuck_degrees = self
            .max_pose_leg_tuck_degrees
            .max(sample.pose_leg_tuck_degrees);
        self.max_pose_lateral_lean_degrees = self
            .max_pose_lateral_lean_degrees
            .max(sample.pose_lateral_lean_degrees);
        if sample.movement_input_lateral_axis > 0.25 {
            self.max_right_pose_lateral_lean_degrees = self
                .max_right_pose_lateral_lean_degrees
                .max((-sample.pose_signed_lateral_lean_degrees).max(0.0));
        } else if sample.movement_input_lateral_axis < -0.25 {
            self.max_left_pose_lateral_lean_degrees = self
                .max_left_pose_lateral_lean_degrees
                .max(sample.pose_signed_lateral_lean_degrees.max(0.0));
        }
        match sample.pose_intent_label {
            "grounded_walk" => {
                self.max_grounded_walk_stride_foot_travel_m = self
                    .max_grounded_walk_stride_foot_travel_m
                    .max(sample.pose_grounded_stride_foot_travel_m);
                self.max_grounded_walk_stride_leg_opposition_degrees = self
                    .max_grounded_walk_stride_leg_opposition_degrees
                    .max(sample.pose_grounded_stride_leg_opposition_degrees);
            }
            "grounded_run" => {
                self.max_grounded_run_stride_foot_travel_m = self
                    .max_grounded_run_stride_foot_travel_m
                    .max(sample.pose_grounded_stride_foot_travel_m);
                self.max_grounded_run_stride_leg_opposition_degrees = self
                    .max_grounded_run_stride_leg_opposition_degrees
                    .max(sample.pose_grounded_stride_leg_opposition_degrees);
            }
            _ => {}
        }
        self.max_pose_landing_crouch_m = self
            .max_pose_landing_crouch_m
            .max(sample.pose_landing_crouch_m);
        self.max_pose_landing_foot_forward_m = self
            .max_pose_landing_foot_forward_m
            .max(sample.pose_landing_foot_forward_m);
        if sample.pose_intent_label == "landing_anticipation" {
            self.max_pose_landing_flare_degrees = self
                .max_pose_landing_flare_degrees
                .max(sample.pose_torso_pitch_degrees);
        }
        if sample.pose_intent_label == "landing_recovery" {
            self.max_pose_landing_recovery_flip_degrees = self
                .max_pose_landing_recovery_flip_degrees
                .max(sample.pose_landing_recovery_flip_degrees);
        }
        self.max_pose_wing_airflow_strength = self
            .max_pose_wing_airflow_strength
            .max(sample.pose_wing_airflow_strength);
        self.max_pose_scarf_stream_m = self.max_pose_scarf_stream_m.max(sample.pose_scarf_stream_m);
        self.max_pose_scarf_lateral_sway_m = self
            .max_pose_scarf_lateral_sway_m
            .max(sample.pose_scarf_lateral_sway_m);
        self.max_pose_scarf_tail_flex_degrees = self
            .max_pose_scarf_tail_flex_degrees
            .max(sample.pose_scarf_tail_flex_degrees);

        self.min_target_distance_m = self.min_target_distance_m.min(sample.target_distance_m);
        self.final_target_distance_m = sample.target_distance_m;
        self.objective_total_count = sample.objective.total_count;
        self.max_completed_objective_count = self
            .max_completed_objective_count
            .max(sample.objective.completed_count);
        self.final_objective_completed_count = sample.objective.completed_count;
        self.min_objective_distance_m = self
            .min_objective_distance_m
            .min(sample.objective.current_distance_m);
        self.final_objective_distance_m = sample.objective.current_distance_m;
        if sample.objective.complete {
            self.objective_complete_samples += 1;
        }
        if sample.on_landing_target {
            self.target_landing_samples += 1;
        }
        self.max_sky_island_count = self.max_sky_island_count.max(sample.sky_island_count);
        self.max_dynamic_wind_flow_fields = self
            .max_dynamic_wind_flow_fields
            .max(sample.dynamic_wind_flow_fields);
        self.max_wind_flow_speed_mps = self
            .max_wind_flow_speed_mps
            .max(sample.max_wind_flow_speed_mps);
        self.max_wind_flow_variation = self
            .max_wind_flow_variation
            .max(sample.max_wind_flow_variation);
        self.max_wind_flow_direction_change_degrees = self
            .max_wind_flow_direction_change_degrees
            .max(sample.max_wind_flow_direction_change_degrees);
        if sample.active_wind_force_fields > 0 {
            self.wind_force_samples += 1;
            let meaningful_delta = sample.max_wind_force_delta_mps >= MIN_WIND_FORCE_DELTA_MPS
                || sample.max_crosswind_force_delta_mps >= MIN_CROSSWIND_FORCE_DELTA_MPS
                || sample.max_updraft_swirl_force_delta_mps >= MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS;
            if meaningful_delta && sample.max_wind_force_variation >= MIN_WIND_FORCE_VARIATION {
                self.meaningful_wind_force_samples += 1;
            }
            if sample.max_wind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
                && sample.max_wind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
            {
                self.aligned_wind_force_samples += 1;
            }
        }
        if sample.crosswind_force_fields > 0 {
            self.crosswind_force_samples += 1;
            if sample.max_crosswind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
                && sample.max_crosswind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
            {
                self.aligned_crosswind_force_samples += 1;
            }
        }
        if sample.updraft_swirl_force_fields > 0 {
            self.updraft_swirl_force_samples += 1;
            if sample.max_updraft_swirl_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
                && sample.max_updraft_swirl_force_aligned_delta_mps
                    >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
            {
                self.aligned_updraft_swirl_force_samples += 1;
            }
        }
        if sample.active_wind_force_fields >= 2 {
            self.layered_wind_force_samples += 1;
            self.max_layered_wind_force_fields = self
                .max_layered_wind_force_fields
                .max(sample.active_wind_force_fields);
            self.max_layered_wind_force_delta_mps = self
                .max_layered_wind_force_delta_mps
                .max(sample.max_wind_force_delta_mps);
            self.max_layered_wind_force_flow_alignment = self
                .max_layered_wind_force_flow_alignment
                .max(sample.max_wind_force_flow_alignment);
            self.max_layered_wind_force_aligned_delta_mps = self
                .max_layered_wind_force_aligned_delta_mps
                .max(sample.max_wind_force_aligned_delta_mps);
            if sample.max_wind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
                && sample.max_wind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
            {
                self.aligned_layered_wind_force_samples += 1;
            }
        }
        if sample.active_wind_force_fields >= 2
            && sample.crosswind_force_fields > 0
            && sample.updraft_swirl_force_fields > 0
        {
            self.crosswind_updraft_overlap_samples += 1;
            if sample.max_crosswind_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
                && sample.max_crosswind_force_aligned_delta_mps >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
                && sample.max_updraft_swirl_force_flow_alignment >= MIN_WIND_FORCE_FLOW_ALIGNMENT
                && sample.max_updraft_swirl_force_aligned_delta_mps
                    >= MIN_WIND_FORCE_ALIGNED_DELTA_MPS
            {
                self.aligned_crosswind_updraft_overlap_samples += 1;
            }
        }
        self.max_active_wind_force_fields = self
            .max_active_wind_force_fields
            .max(sample.active_wind_force_fields);
        self.max_crosswind_force_fields = self
            .max_crosswind_force_fields
            .max(sample.crosswind_force_fields);
        self.max_updraft_swirl_force_fields = self
            .max_updraft_swirl_force_fields
            .max(sample.updraft_swirl_force_fields);
        self.max_wind_force_delta_mps = self
            .max_wind_force_delta_mps
            .max(sample.max_wind_force_delta_mps);
        self.max_crosswind_force_delta_mps = self
            .max_crosswind_force_delta_mps
            .max(sample.max_crosswind_force_delta_mps);
        self.max_updraft_swirl_force_delta_mps = self
            .max_updraft_swirl_force_delta_mps
            .max(sample.max_updraft_swirl_force_delta_mps);
        self.max_wind_force_flow_speed_mps = self
            .max_wind_force_flow_speed_mps
            .max(sample.max_wind_force_flow_speed_mps);
        self.max_wind_force_variation = self
            .max_wind_force_variation
            .max(sample.max_wind_force_variation);
        self.max_wind_force_flow_alignment = self
            .max_wind_force_flow_alignment
            .max(sample.max_wind_force_flow_alignment);
        self.max_crosswind_force_flow_alignment = self
            .max_crosswind_force_flow_alignment
            .max(sample.max_crosswind_force_flow_alignment);
        self.max_updraft_swirl_force_flow_alignment = self
            .max_updraft_swirl_force_flow_alignment
            .max(sample.max_updraft_swirl_force_flow_alignment);
        self.max_wind_force_aligned_delta_mps = self
            .max_wind_force_aligned_delta_mps
            .max(sample.max_wind_force_aligned_delta_mps);
        self.max_crosswind_force_aligned_delta_mps = self
            .max_crosswind_force_aligned_delta_mps
            .max(sample.max_crosswind_force_aligned_delta_mps);
        self.max_updraft_swirl_force_aligned_delta_mps = self
            .max_updraft_swirl_force_aligned_delta_mps
            .max(sample.max_updraft_swirl_force_aligned_delta_mps);
        if wind_load_response_sample(sample) {
            self.wind_load_response_samples += 1;
            self.max_wind_load_lateral_load = self
                .max_wind_load_lateral_load
                .max(sample.wind_lateral_load.abs());
            self.max_wind_load_pose_lean_degrees = self
                .max_wind_load_pose_lean_degrees
                .max(sample.pose_lateral_lean_degrees);
            self.max_wind_load_glider_response_degrees = self
                .max_wind_load_glider_response_degrees
                .max(sample.wind_load_glider_response_degrees);
        }
        self.max_paired_visual_lift_fields = self
            .max_paired_visual_lift_fields
            .max(sample.paired_visual_lift_fields);
        self.max_dynamic_lift_fields = self.max_dynamic_lift_fields.max(sample.dynamic_lift_fields);
        if sample.active_lift_fields > 0
            && sample.paired_visual_lift_fields > 0
            && sample.dynamic_lift_fields > 0
            && sample.lift_applied_delta_mps > 0.001
        {
            self.dynamic_lift_samples += 1;
            self.max_lift_applied_delta_mps = self
                .max_lift_applied_delta_mps
                .max(sample.lift_applied_delta_mps);
            self.max_dynamic_lift_multiplier = self
                .max_dynamic_lift_multiplier
                .max(sample.max_lift_multiplier);
            let min_multiplier = self
                .min_dynamic_lift_multiplier
                .map_or(sample.min_lift_multiplier, |current| {
                    current.min(sample.min_lift_multiplier)
                });
            self.min_dynamic_lift_multiplier = Some(min_multiplier);
            self.max_dynamic_lift_multiplier_range = self
                .max_dynamic_lift_multiplier_range
                .max(self.max_dynamic_lift_multiplier - min_multiplier);
        }
        self.max_active_chunk_count = self.max_active_chunk_count.max(sample.active_chunk_count);
        self.max_active_island_count = self.max_active_island_count.max(sample.active_island_count);
        self.max_near_lod_islands = self.max_near_lod_islands.max(sample.near_lod_islands);
        self.max_mid_lod_islands = self.max_mid_lod_islands.max(sample.mid_lod_islands);
        self.max_far_lod_islands = self.max_far_lod_islands.max(sample.far_lod_islands);
        self.max_power_up_count = self.max_power_up_count.max(sample.power_up_count);
        self.min_visible_power_up_count = self
            .min_visible_power_up_count
            .min(sample.visible_power_up_count);
        self.max_collected_power_up_count = self
            .max_collected_power_up_count
            .max(sample.collected_power_up_count);
        if sample.active_power_up_effects > 0 {
            self.power_up_effect_samples += 1;
        }
        self.total_power_up_activations = sample.total_power_up_activations;
        if sample.active_lift_fields > 0 {
            self.lifted_samples += 1;
            if sample.readable_lift_fields > 0 {
                self.readable_lift_samples += 1;
                if sample.dynamic_wind_flow_fields > 0 && sample.max_wind_flow_variation > 0.05 {
                    self.dynamic_readable_lift_samples += 1;
                    self.max_dynamic_readable_wind_flow_variation = self
                        .max_dynamic_readable_wind_flow_variation
                        .max(sample.max_wind_flow_variation);
                    let min_variation = self
                        .min_dynamic_readable_wind_flow_variation
                        .map_or(sample.max_wind_flow_variation, |current| {
                            current.min(sample.max_wind_flow_variation)
                        });
                    self.min_dynamic_readable_wind_flow_variation = Some(min_variation);
                    self.max_wind_flow_variation_range = self
                        .max_wind_flow_variation_range
                        .max(self.max_dynamic_readable_wind_flow_variation - min_variation);
                }
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
        if sample.mode == "gliding" && sample.pose_intent_label == "diving" {
            self.gliding_dive_samples += 1;
        }
        self.observe_pose_intent_counts(sample);

        if scenario.name == CAMERA_STRAFE_STABILITY {
            self.max_camera_obstruction_adjustment_m = 0.0;
        }
    }

    fn observe_pose_intent_counts(&mut self, sample: &SimSample) {
        match sample.pose_intent_label {
            "grounded_walk" => self.pose_grounded_walk_samples += 1,
            "grounded_run" => self.pose_grounded_run_samples += 1,
            _ => {}
        }

        if !key_pose_intent_label(sample.pose_intent_label) {
            return;
        }

        self.max_key_pose_readability_score = self
            .max_key_pose_readability_score
            .max(sample.key_pose_readability_score);
        let min_score = self
            .min_key_pose_readability_score
            .map_or(sample.key_pose_readability_score, |current| {
                current.min(sample.key_pose_readability_score)
            });
        self.min_key_pose_readability_score = Some(min_score);

        if sample.key_pose_transition_grace {
            self.key_pose_transition_grace_samples += 1;
        }

        if sample.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE {
            self.unreadable_key_pose_samples += 1;
            return;
        }

        match sample.pose_intent_label {
            "launching" => self.pose_launching_samples += 1,
            "falling" => self.pose_falling_samples += 1,
            "gliding" => self.pose_gliding_samples += 1,
            "air_turn" => {
                self.pose_air_turn_samples += 1;
                if sample.movement_input_lateral_axis > 0.25 {
                    self.right_pose_air_turn_samples += 1;
                } else if sample.movement_input_lateral_axis < -0.25 {
                    self.left_pose_air_turn_samples += 1;
                }
            }
            "diving" => self.pose_diving_samples += 1,
            "air_brake" => {
                self.pose_air_brake_samples += 1;
                if sample.movement_input_lateral_axis > 0.25 {
                    self.right_pose_air_brake_samples += 1;
                    if sample.movement_input_forward_axis < -0.25 {
                        self.backward_right_pose_air_brake_samples += 1;
                    }
                } else if sample.movement_input_lateral_axis < -0.25 {
                    self.left_pose_air_brake_samples += 1;
                    if sample.movement_input_forward_axis < -0.25 {
                        self.backward_left_pose_air_brake_samples += 1;
                    }
                }
            }
            "landing_anticipation" => self.pose_landing_anticipation_samples += 1,
            "landing_recovery" => self.pose_landing_recovery_samples += 1,
            _ => {}
        }
    }

    fn observe_lateral_response(&mut self, sample: &SimSample) {
        let lateral_axis_active =
            sample.lateral_input_active || sample.movement_input_lateral_axis.abs() > f32::EPSILON;
        if !lateral_axis_active {
            return;
        }
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
                        self.first_backward_right_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_backward_right_lateral_response_mps = self
                        .max_backward_right_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if let Some(rear_response) = backward_diagonal_rear_response_mps(sample) {
                        self.max_backward_right_rear_response_mps =
                            self.max_backward_right_rear_response_mps.max(rear_response);
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
                        self.first_backward_left_lateral_input_time_secs = Some(sample.time_secs);
                    }
                    self.max_backward_left_lateral_response_mps = self
                        .max_backward_left_lateral_response_mps
                        .max(sample.lateral_response_mps);
                    if let Some(rear_response) = backward_diagonal_rear_response_mps(sample) {
                        self.max_backward_left_rear_response_mps =
                            self.max_backward_left_rear_response_mps.max(rear_response);
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

    fn observe_body_travel_heading_alignment(&mut self, sample: &SimSample) {
        if !body_travel_heading_alignment_sample(sample) {
            return;
        }

        let Some(lateral_response_time) = lateral_response_time_for_sample(self, sample) else {
            return;
        };
        if sample.time_secs < lateral_response_time {
            return;
        }

        self.lateral_body_travel_heading_error_values_degrees
            .push(sample.body_travel_heading_error_degrees);
        self.max_lateral_body_travel_heading_error_degrees = self
            .max_lateral_body_travel_heading_error_degrees
            .max(sample.body_travel_heading_error_degrees);
        if sample.movement_input_lateral_axis > 0.0 {
            self.right_lateral_body_travel_heading_samples += 1;
        } else if sample.movement_input_lateral_axis < 0.0 {
            self.left_lateral_body_travel_heading_samples += 1;
        }

        if sample.movement_input_forward_axis >= 0.0 {
            return;
        }

        if backward_diagonal_response_time_for_sample(self, sample)
            .is_some_and(|time_secs| sample.time_secs >= time_secs)
        {
            self.backward_diagonal_body_travel_heading_error_values_degrees
                .push(sample.body_travel_heading_error_degrees);
            self.max_backward_diagonal_body_travel_heading_error_degrees = self
                .max_backward_diagonal_body_travel_heading_error_degrees
                .max(sample.body_travel_heading_error_degrees);
            if sample.movement_input_lateral_axis > 0.0 {
                self.backward_right_diagonal_body_travel_heading_samples += 1;
            } else if sample.movement_input_lateral_axis < 0.0 {
                self.backward_left_diagonal_body_travel_heading_samples += 1;
            }
        }
    }

    fn observe_desired_travel_heading_alignment(&mut self, sample: &SimSample) {
        if !desired_travel_heading_alignment_sample(sample) {
            return;
        }

        let Some(lateral_response_time) = lateral_response_time_for_sample(self, sample) else {
            return;
        };
        if sample.time_secs < lateral_response_time {
            return;
        }

        self.desired_travel_heading_error_values_degrees
            .push(sample.desired_travel_heading_error_degrees);
        self.max_desired_travel_heading_error_degrees = self
            .max_desired_travel_heading_error_degrees
            .max(sample.desired_travel_heading_error_degrees);
        if sample.movement_input_lateral_axis > 0.0 {
            self.right_desired_travel_heading_samples += 1;
        } else if sample.movement_input_lateral_axis < 0.0 {
            self.left_desired_travel_heading_samples += 1;
        }

        if sample.movement_input_forward_axis >= 0.0 {
            return;
        }

        if backward_diagonal_response_time_for_sample(self, sample)
            .is_some_and(|time_secs| sample.time_secs >= time_secs)
        {
            if sample.movement_input_lateral_axis > 0.0 {
                self.backward_right_desired_travel_heading_samples += 1;
            } else if sample.movement_input_lateral_axis < 0.0 {
                self.backward_left_desired_travel_heading_samples += 1;
            }
        }
    }

    fn observe_backward_air_control(&mut self, sample: &SimSample) {
        if sample.movement_input_forward_axis >= 0.0 || sample.mode == "grounded" {
            return;
        }

        let planar_speed = Vec2::new(sample.velocity.x, sample.velocity.z).length();
        self.backward_air_control_start_speed_mps
            .get_or_insert(sample.speed_mps);
        self.backward_air_control_start_planar_speed_mps
            .get_or_insert(planar_speed);
        self.min_backward_air_control_speed_mps = Some(
            self.min_backward_air_control_speed_mps
                .map_or(sample.speed_mps, |speed| speed.min(sample.speed_mps)),
        );
        self.min_backward_air_control_planar_speed_mps = Some(
            self.min_backward_air_control_planar_speed_mps
                .map_or(planar_speed, |speed| speed.min(planar_speed)),
        );
        if let (Some(start), Some(minimum)) = (
            self.backward_air_control_start_speed_mps,
            self.min_backward_air_control_speed_mps,
        ) {
            self.max_air_brake_speed_drop_mps = (start - minimum).max(0.0);
        }
        if let (Some(start), Some(minimum)) = (
            self.backward_air_control_start_planar_speed_mps,
            self.min_backward_air_control_planar_speed_mps,
        ) {
            self.max_air_brake_planar_speed_drop_mps = (start - minimum).max(0.0);
        }
    }
}

fn wind_load_response_sample(sample: &SimSample) -> bool {
    matches!(sample.mode, "airborne" | "gliding")
        && sample.movement_input_lateral_axis.abs() < 0.25
        && sample.crosswind_force_fields > 0
        && sample.max_crosswind_force_delta_mps >= MIN_CROSSWIND_FORCE_DELTA_MPS
        && sample.wind_lateral_load.abs() >= MIN_WIND_LOAD_LATERAL_LOAD
}

fn body_yaw_intent_axis(sample: &SimSample) -> Option<Vec2> {
    let axis = Vec2::new(
        sample.movement_input_lateral_axis,
        sample.movement_input_forward_axis,
    );
    (axis.length_squared() >= BODY_YAW_INTENT_AXIS_EPSILON.powi(2)).then(|| axis.normalize())
}

fn body_yaw_intent_changed(previous: Option<Vec2>, current: Option<Vec2>) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => previous.dot(current) < BODY_YAW_INTENT_CHANGE_DOT,
        (Some(_), None) | (None, Some(_)) => true,
        (None, None) => false,
    }
}

fn key_pose_intent_label(label: &str) -> bool {
    matches!(
        label,
        "gliding"
            | "launching"
            | "falling"
            | "air_turn"
            | "diving"
            | "air_brake"
            | "landing_anticipation"
            | "landing_recovery"
    )
}

fn body_travel_heading_alignment_sample(sample: &SimSample) -> bool {
    matches!(sample.mode, "airborne" | "gliding")
        && sample.lateral_input_active
        && sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
        && sample.body_travel_heading_error_degrees.is_finite()
}

fn desired_travel_heading_alignment_sample(sample: &SimSample) -> bool {
    matches!(sample.mode, "airborne" | "gliding")
        && sample.lateral_input_active
        && sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
        && sample.desired_travel_heading_error_degrees.is_finite()
}

fn lateral_response_time_for_sample(metrics: &SimMetrics, sample: &SimSample) -> Option<f32> {
    match sample.movement_input_lateral_axis.signum() {
        sign if sign > 0.0 => metrics.first_right_lateral_response_time_secs,
        sign if sign < 0.0 => metrics.first_left_lateral_response_time_secs,
        _ => metrics.first_lateral_response_time_secs,
    }
}

fn backward_diagonal_response_time_for_sample(
    metrics: &SimMetrics,
    sample: &SimSample,
) -> Option<f32> {
    match sample.movement_input_lateral_axis.signum() {
        sign if sign > 0.0 => metrics.first_backward_right_lateral_response_time_secs,
        sign if sign < 0.0 => metrics.first_backward_left_lateral_response_time_secs,
        _ => metrics.first_backward_lateral_response_time_secs,
    }
}
