use bevy::prelude::{Vec2, Vec3};
use nau_engine::{
    environment::AERIAL_POWER_UP_ROUTE,
    eval::{AIR_CONTROL_RESPONSE, CAMERA_STRAFE_STABILITY, EvalScenario},
    world::{START_POSITION, SkyRoute},
};
use serde_json::{Value, json};

use super::{
    AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES, AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES, AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
    AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS, AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
    AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES, AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS, AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS, AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS, AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
    AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS, AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
    AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS, AIR_CONTROL_RESPONSE_THRESHOLD_MPS,
    AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES, CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
    CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS, MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
    SimSample, round4, round4_f64, vec3_json,
};

#[derive(Clone, Debug)]
pub(crate) struct SimMetrics {
    pub(crate) sample_count: u32,
    pub(crate) start_position: Vec3,
    pub(crate) final_position: Vec3,
    pub(crate) horizontal_distance_m: f32,
    pub(crate) max_altitude_m: f32,
    pub(crate) min_altitude_m: f32,
    pub(crate) max_speed_mps: f32,
    pub(crate) max_camera_distance_m: f32,
    pub(crate) min_camera_surface_clearance_m: f32,
    pub(crate) max_camera_player_angle_degrees: f32,
    pub(crate) max_camera_step_distance_m: f32,
    pub(crate) max_camera_rotation_delta_degrees: f32,
    pub(crate) max_camera_orbit_alignment_degrees: f32,
    pub(crate) max_abs_camera_view_yaw_degrees: f32,
    pub(crate) first_camera_view_yaw_degrees: Option<f32>,
    pub(crate) max_camera_view_yaw_drift_degrees: f32,
    pub(crate) first_camera_world_yaw_degrees: Option<f32>,
    pub(crate) max_camera_world_yaw_drift_degrees: f32,
    pub(crate) max_camera_obstruction_adjustment_m: f32,
    pub(crate) max_camera_obstruction_hits: usize,
    pub(crate) max_abs_camera_yaw_offset_degrees: f32,
    pub(crate) min_camera_pitch_offset_degrees: f32,
    pub(crate) max_camera_pitch_offset_degrees: f32,
    pub(crate) desired_body_heading_error_sum_degrees: f32,
    pub(crate) desired_body_heading_samples: u32,
    pub(crate) desired_body_heading_error_values_degrees: Vec<f32>,
    pub(crate) max_desired_body_heading_error_degrees: f32,
    pub(crate) previous_desired_body_yaw_error_degrees: Option<f32>,
    pub(crate) max_body_yaw_error_step_degrees: f32,
    pub(crate) previous_body_yaw_error_sign: Option<f32>,
    pub(crate) body_yaw_oscillation_count: u32,
    pub(crate) previous_body_roll_degrees: Option<f32>,
    pub(crate) max_body_roll_step_degrees: f32,
    pub(crate) max_right_body_bank_degrees: f32,
    pub(crate) max_left_body_bank_degrees: f32,
    pub(crate) max_desired_heading_alignment_mps: f32,
    pub(crate) max_lateral_response_mps: f32,
    pub(crate) first_lateral_input_time_secs: Option<f32>,
    pub(crate) first_lateral_response_time_secs: Option<f32>,
    pub(crate) max_right_lateral_response_mps: f32,
    pub(crate) first_right_lateral_input_time_secs: Option<f32>,
    pub(crate) first_right_lateral_response_time_secs: Option<f32>,
    pub(crate) max_left_lateral_response_mps: f32,
    pub(crate) first_left_lateral_input_time_secs: Option<f32>,
    pub(crate) first_left_lateral_response_time_secs: Option<f32>,
    pub(crate) max_backward_lateral_response_mps: f32,
    pub(crate) first_backward_lateral_input_time_secs: Option<f32>,
    pub(crate) first_backward_lateral_response_time_secs: Option<f32>,
    pub(crate) max_backward_right_lateral_response_mps: f32,
    pub(crate) max_backward_right_rear_response_mps: f32,
    pub(crate) first_backward_right_lateral_input_time_secs: Option<f32>,
    pub(crate) first_backward_right_lateral_response_time_secs: Option<f32>,
    pub(crate) max_backward_left_lateral_response_mps: f32,
    pub(crate) max_backward_left_rear_response_mps: f32,
    pub(crate) first_backward_left_lateral_input_time_secs: Option<f32>,
    pub(crate) first_backward_left_lateral_response_time_secs: Option<f32>,
    pub(crate) backward_air_control_start_speed_mps: Option<f32>,
    pub(crate) min_backward_air_control_speed_mps: Option<f32>,
    pub(crate) backward_air_control_start_planar_speed_mps: Option<f32>,
    pub(crate) min_backward_air_control_planar_speed_mps: Option<f32>,
    pub(crate) max_air_brake_speed_drop_mps: f32,
    pub(crate) max_air_brake_planar_speed_drop_mps: f32,
    pub(crate) max_post_brake_forward_alignment_mps: f32,
    pub(crate) min_target_distance_m: f32,
    pub(crate) final_target_distance_m: f32,
    pub(crate) objective_total_count: usize,
    pub(crate) max_completed_objective_count: usize,
    pub(crate) final_objective_completed_count: usize,
    pub(crate) min_objective_distance_m: f32,
    pub(crate) final_objective_distance_m: f32,
    pub(crate) objective_complete_samples: u32,
    pub(crate) max_sky_island_count: usize,
    pub(crate) max_active_chunk_count: usize,
    pub(crate) max_active_island_count: usize,
    pub(crate) max_near_lod_islands: usize,
    pub(crate) max_mid_lod_islands: usize,
    pub(crate) max_far_lod_islands: usize,
    pub(crate) max_power_up_count: usize,
    pub(crate) min_visible_power_up_count: usize,
    pub(crate) max_collected_power_up_count: usize,
    pub(crate) power_up_effect_samples: u32,
    pub(crate) total_power_up_activations: usize,
    pub(crate) target_landing_samples: u32,
    pub(crate) lifted_samples: u32,
    pub(crate) readable_lift_samples: u32,
    pub(crate) unreadable_lift_samples: u32,
    pub(crate) gliding_samples: u32,
    pub(crate) launching_samples: u32,
    pub(crate) grounded_samples: u32,
}

impl SimMetrics {
    pub(crate) fn new(route: &SkyRoute) -> Self {
        Self {
            sample_count: 0,
            start_position: START_POSITION,
            final_position: START_POSITION,
            horizontal_distance_m: 0.0,
            max_altitude_m: START_POSITION.y,
            min_altitude_m: START_POSITION.y,
            max_speed_mps: 0.0,
            max_camera_distance_m: 0.0,
            min_camera_surface_clearance_m: f32::MAX,
            max_camera_player_angle_degrees: 0.0,
            max_camera_step_distance_m: 0.0,
            max_camera_rotation_delta_degrees: 0.0,
            max_camera_orbit_alignment_degrees: 0.0,
            max_abs_camera_view_yaw_degrees: 0.0,
            first_camera_view_yaw_degrees: None,
            max_camera_view_yaw_drift_degrees: 0.0,
            first_camera_world_yaw_degrees: None,
            max_camera_world_yaw_drift_degrees: 0.0,
            max_camera_obstruction_adjustment_m: 0.0,
            max_camera_obstruction_hits: 0,
            max_abs_camera_yaw_offset_degrees: 0.0,
            min_camera_pitch_offset_degrees: f32::MAX,
            max_camera_pitch_offset_degrees: f32::MIN,
            desired_body_heading_error_sum_degrees: 0.0,
            desired_body_heading_samples: 0,
            desired_body_heading_error_values_degrees: Vec::new(),
            max_desired_body_heading_error_degrees: 0.0,
            previous_desired_body_yaw_error_degrees: None,
            max_body_yaw_error_step_degrees: 0.0,
            previous_body_yaw_error_sign: None,
            body_yaw_oscillation_count: 0,
            previous_body_roll_degrees: None,
            max_body_roll_step_degrees: 0.0,
            max_right_body_bank_degrees: 0.0,
            max_left_body_bank_degrees: 0.0,
            max_desired_heading_alignment_mps: 0.0,
            max_lateral_response_mps: 0.0,
            first_lateral_input_time_secs: None,
            first_lateral_response_time_secs: None,
            max_right_lateral_response_mps: 0.0,
            first_right_lateral_input_time_secs: None,
            first_right_lateral_response_time_secs: None,
            max_left_lateral_response_mps: 0.0,
            first_left_lateral_input_time_secs: None,
            first_left_lateral_response_time_secs: None,
            max_backward_lateral_response_mps: 0.0,
            first_backward_lateral_input_time_secs: None,
            first_backward_lateral_response_time_secs: None,
            max_backward_right_lateral_response_mps: 0.0,
            max_backward_right_rear_response_mps: 0.0,
            first_backward_right_lateral_input_time_secs: None,
            first_backward_right_lateral_response_time_secs: None,
            max_backward_left_lateral_response_mps: 0.0,
            max_backward_left_rear_response_mps: 0.0,
            first_backward_left_lateral_input_time_secs: None,
            first_backward_left_lateral_response_time_secs: None,
            backward_air_control_start_speed_mps: None,
            min_backward_air_control_speed_mps: None,
            backward_air_control_start_planar_speed_mps: None,
            min_backward_air_control_planar_speed_mps: None,
            max_air_brake_speed_drop_mps: 0.0,
            max_air_brake_planar_speed_drop_mps: 0.0,
            max_post_brake_forward_alignment_mps: 0.0,
            min_target_distance_m: f32::MAX,
            final_target_distance_m: route.target_distance_to(START_POSITION, None),
            objective_total_count: 0,
            max_completed_objective_count: 0,
            final_objective_completed_count: 0,
            min_objective_distance_m: f32::MAX,
            final_objective_distance_m: 0.0,
            objective_complete_samples: 0,
            max_sky_island_count: route.islands().len(),
            max_active_chunk_count: 0,
            max_active_island_count: 0,
            max_near_lod_islands: 0,
            max_mid_lod_islands: 0,
            max_far_lod_islands: 0,
            max_power_up_count: AERIAL_POWER_UP_ROUTE.len(),
            min_visible_power_up_count: AERIAL_POWER_UP_ROUTE.len(),
            max_collected_power_up_count: 0,
            power_up_effect_samples: 0,
            total_power_up_activations: 0,
            target_landing_samples: 0,
            lifted_samples: 0,
            readable_lift_samples: 0,
            unreadable_lift_samples: 0,
            gliding_samples: 0,
            launching_samples: 0,
            grounded_samples: 0,
        }
    }

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
        self.observe_backward_air_control(sample);

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

        if scenario.name == CAMERA_STRAFE_STABILITY {
            self.max_camera_obstruction_adjustment_m = 0.0;
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

    pub(crate) fn checks(&self, scenario: EvalScenario) -> Vec<SimCheck> {
        let thresholds = scenario.thresholds;
        let mut checks = vec![
            SimCheck::at_least(
                "sample_count",
                self.sample_count as f32,
                thresholds.min_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "horizontal_distance",
                self.horizontal_distance_m,
                thresholds.min_horizontal_distance_m,
                "m",
            ),
            SimCheck::at_least(
                "max_altitude",
                self.max_altitude_m,
                thresholds.min_max_altitude_m,
                "m",
            ),
            SimCheck::at_least(
                "max_speed",
                self.max_speed_mps,
                thresholds.min_max_speed_mps,
                "mps",
            ),
            SimCheck::at_least(
                "gliding_samples",
                self.gliding_samples as f32,
                thresholds.min_gliding_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "grounded_samples",
                self.grounded_samples as f32,
                thresholds.min_grounded_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "lifted_samples",
                self.lifted_samples as f32,
                thresholds.min_lifted_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "sky_island_count",
                self.max_sky_island_count as f32,
                thresholds.min_sky_island_count as f32,
                "islands",
            ),
            SimCheck::at_least(
                "active_island_count",
                self.max_active_island_count as f32,
                thresholds.min_active_island_count as f32,
                "islands",
            ),
            SimCheck::at_most(
                "active_chunk_count",
                self.max_active_chunk_count as f32,
                thresholds.max_active_chunk_count as f32,
                "chunks",
            ),
            SimCheck::at_least(
                "near_lod_island_count",
                self.max_near_lod_islands as f32,
                thresholds.min_near_lod_island_count as f32,
                "islands",
            ),
            SimCheck::at_least(
                "mid_lod_island_count",
                self.max_mid_lod_islands as f32,
                thresholds.min_mid_lod_island_count as f32,
                "islands",
            ),
            SimCheck::at_least(
                "far_lod_island_count",
                self.max_far_lod_islands as f32,
                thresholds.min_far_lod_island_count as f32,
                "islands",
            ),
            SimCheck::at_most(
                "camera_distance",
                self.max_camera_distance_m,
                thresholds.max_camera_distance_m,
                "m",
            ),
            SimCheck::at_least(
                "camera_surface_clearance",
                self.min_camera_surface_clearance_m,
                thresholds.min_camera_surface_clearance_m,
                "m",
            ),
            SimCheck::at_most(
                "camera_player_angle",
                self.max_camera_player_angle_degrees,
                thresholds.max_camera_player_angle_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_step_distance",
                self.max_camera_step_distance_m,
                thresholds.max_camera_step_distance_m,
                "m",
            ),
            SimCheck::at_most(
                "camera_rotation_delta",
                self.max_camera_rotation_delta_degrees,
                thresholds.max_camera_rotation_delta_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_orbit_alignment",
                self.max_camera_orbit_alignment_degrees,
                thresholds.max_camera_orbit_alignment_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_view_yaw",
                self.max_abs_camera_view_yaw_degrees,
                thresholds.max_abs_camera_view_yaw_degrees,
                "deg",
            ),
            SimCheck::at_least(
                "camera_yaw_input",
                self.max_abs_camera_yaw_offset_degrees,
                thresholds.min_abs_camera_yaw_degrees,
                "deg",
            ),
            SimCheck::at_least(
                "camera_pitch_input_min",
                self.min_camera_pitch_offset_degrees,
                thresholds.min_camera_pitch_offset_degrees,
                "deg",
            ),
            SimCheck::at_most(
                "camera_pitch_input_max",
                self.max_camera_pitch_offset_degrees,
                thresholds.max_camera_pitch_offset_degrees,
                "deg",
            ),
            SimCheck::at_least(
                "objective_total_count",
                self.objective_total_count as f32,
                thresholds.min_objective_total_count as f32,
                "objectives",
            ),
            SimCheck::at_least(
                "completed_objective_count",
                self.max_completed_objective_count as f32,
                thresholds.min_completed_objective_count as f32,
                "objectives",
            ),
            SimCheck::at_most(
                "final_target_distance",
                self.final_target_distance_m,
                thresholds.max_final_target_distance_m,
                "m",
            ),
            SimCheck::at_least(
                "target_landing_samples",
                self.target_landing_samples as f32,
                thresholds.min_target_landing_samples as f32,
                "samples",
            ),
            SimCheck::at_least(
                "power_up_count",
                self.max_power_up_count as f32,
                thresholds.min_power_up_count as f32,
                "powerups",
            ),
            SimCheck::at_least(
                "collected_power_up_count",
                self.max_collected_power_up_count as f32,
                thresholds.min_collected_power_up_count as f32,
                "powerups",
            ),
            SimCheck::at_least(
                "power_up_effect_samples",
                self.power_up_effect_samples as f32,
                thresholds.min_power_up_effect_samples as f32,
                "samples",
            ),
        ];

        if scenario.name == CAMERA_STRAFE_STABILITY {
            checks.extend([
                SimCheck::at_most(
                    "camera_strafe_view_yaw_drift",
                    self.max_camera_view_yaw_drift_degrees,
                    CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "camera_strafe_world_yaw_drift",
                    self.max_camera_world_yaw_drift_degrees,
                    MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_least(
                    "camera_strafe_right_lateral_response",
                    self.max_right_lateral_response_mps,
                    CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "camera_strafe_left_lateral_response",
                    self.max_left_lateral_response_mps,
                    CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
            ]);
        }

        if scenario.name == AIR_CONTROL_RESPONSE {
            let lateral_response_latency_secs = response_latency_secs(
                self.first_lateral_input_time_secs,
                self.first_lateral_response_time_secs,
            );
            let right_lateral_response_latency_secs = response_latency_secs(
                self.first_right_lateral_input_time_secs,
                self.first_right_lateral_response_time_secs,
            );
            let left_lateral_response_latency_secs = response_latency_secs(
                self.first_left_lateral_input_time_secs,
                self.first_left_lateral_response_time_secs,
            );
            let backward_lateral_response_latency_secs = response_latency_secs(
                self.first_backward_lateral_input_time_secs,
                self.first_backward_lateral_response_time_secs,
            );
            let backward_right_lateral_response_latency_secs = response_latency_secs(
                self.first_backward_right_lateral_input_time_secs,
                self.first_backward_right_lateral_response_time_secs,
            );
            let backward_left_lateral_response_latency_secs = response_latency_secs(
                self.first_backward_left_lateral_input_time_secs,
                self.first_backward_left_lateral_response_time_secs,
            );

            checks.extend([
                SimCheck::at_most(
                    "air_control_lateral_response_latency",
                    lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_lateral_response",
                    self.max_lateral_response_mps,
                    AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_right_lateral_response_latency",
                    right_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_right_lateral_response",
                    self.max_right_lateral_response_mps,
                    AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_left_lateral_response_latency",
                    left_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_left_lateral_response",
                    self.max_left_lateral_response_mps,
                    AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_backward_lateral_response_latency",
                    backward_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_backward_lateral_response",
                    self.max_backward_lateral_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_backward_right_lateral_response_latency",
                    backward_right_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_backward_right_lateral_response",
                    self.max_backward_right_lateral_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_backward_right_rear_response",
                    self.max_backward_right_rear_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_backward_left_lateral_response_latency",
                    backward_left_lateral_response_latency_secs,
                    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
                    "s",
                ),
                SimCheck::at_least(
                    "air_control_backward_left_lateral_response",
                    self.max_backward_left_lateral_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_backward_left_rear_response",
                    self.max_backward_left_rear_response_mps,
                    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_desired_heading_alignment",
                    self.max_desired_heading_alignment_mps,
                    AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
                    "mps",
                ),
                SimCheck::at_most(
                    "air_control_avg_body_heading_error",
                    self.avg_body_heading_error_degrees(),
                    AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_p95_body_heading_error",
                    self.p95_body_heading_error_degrees(),
                    AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_max_body_heading_error",
                    self.max_desired_body_heading_error_degrees,
                    AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_max_body_yaw_error_step",
                    self.max_body_yaw_error_step_degrees,
                    AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_body_yaw_oscillation_count",
                    self.body_yaw_oscillation_count as f32,
                    AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
                    "oscillations",
                ),
                SimCheck::at_least(
                    "air_control_right_body_bank_response",
                    self.max_right_body_bank_degrees,
                    AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
                    "deg",
                ),
                SimCheck::at_least(
                    "air_control_left_body_bank_response",
                    self.max_left_body_bank_degrees,
                    AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_max_body_roll_step",
                    self.max_body_roll_step_degrees,
                    AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_camera_orbit_yaw_offset",
                    self.max_abs_camera_yaw_offset_degrees,
                    AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_camera_rotation_delta",
                    self.max_camera_rotation_delta_degrees,
                    AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_camera_view_yaw_drift",
                    self.max_camera_view_yaw_drift_degrees,
                    AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_camera_world_yaw_drift",
                    self.max_camera_world_yaw_drift_degrees,
                    MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
                    "deg",
                ),
                SimCheck::at_least(
                    "air_control_air_brake_speed_drop",
                    self.max_air_brake_speed_drop_mps,
                    AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_air_brake_planar_speed_drop",
                    self.max_air_brake_planar_speed_drop_mps,
                    AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
                    "mps",
                ),
                SimCheck::at_least(
                    "air_control_post_brake_forward_alignment",
                    self.max_post_brake_forward_alignment_mps,
                    AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
                    "mps",
                ),
            ]);
        }

        checks
    }

    fn avg_body_heading_error_degrees(&self) -> f32 {
        if self.desired_body_heading_samples == 0 {
            0.0
        } else {
            self.desired_body_heading_error_sum_degrees / self.desired_body_heading_samples as f32
        }
    }

    fn p95_body_heading_error_degrees(&self) -> f32 {
        percentile(&self.desired_body_heading_error_values_degrees, 0.95)
    }

    fn to_json(&self) -> Value {
        json!({
            "sample_count": self.sample_count,
            "horizontal_distance_m": round4(self.horizontal_distance_m),
            "max_altitude_m": round4(self.max_altitude_m),
            "min_altitude_m": round4(self.min_altitude_m),
            "max_speed_mps": round4(self.max_speed_mps),
            "max_camera_distance_m": round4(self.max_camera_distance_m),
            "min_camera_surface_clearance_m": round4(self.min_camera_surface_clearance_m),
            "max_camera_player_angle_degrees": round4(self.max_camera_player_angle_degrees),
            "max_camera_step_distance_m": round4(self.max_camera_step_distance_m),
            "max_camera_rotation_delta_degrees": round4(self.max_camera_rotation_delta_degrees),
            "max_camera_orbit_alignment_degrees": round4(self.max_camera_orbit_alignment_degrees),
            "max_abs_camera_view_yaw_degrees": round4(self.max_abs_camera_view_yaw_degrees),
            "max_camera_view_yaw_drift_degrees": round4(self.max_camera_view_yaw_drift_degrees),
            "max_camera_world_yaw_drift_degrees": round4(self.max_camera_world_yaw_drift_degrees),
            "max_camera_obstruction_adjustment_m": round4(self.max_camera_obstruction_adjustment_m),
            "max_camera_obstruction_hits": self.max_camera_obstruction_hits,
            "max_abs_camera_yaw_offset_degrees": round4(self.max_abs_camera_yaw_offset_degrees),
            "min_camera_pitch_offset_degrees": round4(self.min_camera_pitch_offset_degrees),
            "max_camera_pitch_offset_degrees": round4(self.max_camera_pitch_offset_degrees),
            "avg_desired_body_heading_error_degrees": round4(self.avg_body_heading_error_degrees()),
            "p95_desired_body_heading_error_degrees": round4(self.p95_body_heading_error_degrees()),
            "max_desired_body_heading_error_degrees": round4(self.max_desired_body_heading_error_degrees),
            "max_body_yaw_error_step_degrees": round4(self.max_body_yaw_error_step_degrees),
            "body_yaw_oscillation_count": self.body_yaw_oscillation_count,
            "max_body_roll_step_degrees": round4(self.max_body_roll_step_degrees),
            "max_right_body_bank_degrees": round4(self.max_right_body_bank_degrees),
            "max_left_body_bank_degrees": round4(self.max_left_body_bank_degrees),
            "max_desired_heading_alignment_mps": round4(self.max_desired_heading_alignment_mps),
            "max_lateral_response_mps": round4(self.max_lateral_response_mps),
            "lateral_response_latency_secs": round4(response_latency_secs(self.first_lateral_input_time_secs, self.first_lateral_response_time_secs)),
            "max_right_lateral_response_mps": round4(self.max_right_lateral_response_mps),
            "right_lateral_response_latency_secs": round4(response_latency_secs(self.first_right_lateral_input_time_secs, self.first_right_lateral_response_time_secs)),
            "max_left_lateral_response_mps": round4(self.max_left_lateral_response_mps),
            "left_lateral_response_latency_secs": round4(response_latency_secs(self.first_left_lateral_input_time_secs, self.first_left_lateral_response_time_secs)),
            "max_backward_lateral_response_mps": round4(self.max_backward_lateral_response_mps),
            "backward_lateral_response_latency_secs": round4(response_latency_secs(self.first_backward_lateral_input_time_secs, self.first_backward_lateral_response_time_secs)),
            "max_backward_right_lateral_response_mps": round4(self.max_backward_right_lateral_response_mps),
            "backward_right_lateral_response_latency_secs": round4(response_latency_secs(self.first_backward_right_lateral_input_time_secs, self.first_backward_right_lateral_response_time_secs)),
            "max_backward_right_rear_response_mps": round4(self.max_backward_right_rear_response_mps),
            "max_backward_left_lateral_response_mps": round4(self.max_backward_left_lateral_response_mps),
            "backward_left_lateral_response_latency_secs": round4(response_latency_secs(self.first_backward_left_lateral_input_time_secs, self.first_backward_left_lateral_response_time_secs)),
            "max_backward_left_rear_response_mps": round4(self.max_backward_left_rear_response_mps),
            "max_air_brake_speed_drop_mps": round4(self.max_air_brake_speed_drop_mps),
            "max_air_brake_planar_speed_drop_mps": round4(self.max_air_brake_planar_speed_drop_mps),
            "max_post_brake_forward_alignment_mps": round4(self.max_post_brake_forward_alignment_mps),
            "min_target_distance_m": round4(self.min_target_distance_m),
            "final_target_distance_m": round4(self.final_target_distance_m),
            "objective_total_count": self.objective_total_count,
            "max_completed_objective_count": self.max_completed_objective_count,
            "final_objective_completed_count": self.final_objective_completed_count,
            "min_objective_distance_m": round4(self.min_objective_distance_m),
            "final_objective_distance_m": round4(self.final_objective_distance_m),
            "objective_complete_samples": self.objective_complete_samples,
            "max_sky_island_count": self.max_sky_island_count,
            "max_active_chunk_count": self.max_active_chunk_count,
            "max_active_island_count": self.max_active_island_count,
            "max_near_lod_islands": self.max_near_lod_islands,
            "max_mid_lod_islands": self.max_mid_lod_islands,
            "max_far_lod_islands": self.max_far_lod_islands,
            "max_power_up_count": self.max_power_up_count,
            "min_visible_power_up_count": self.min_visible_power_up_count,
            "max_collected_power_up_count": self.max_collected_power_up_count,
            "power_up_effect_samples": self.power_up_effect_samples,
            "total_power_up_activations": self.total_power_up_activations,
            "target_landing_samples": self.target_landing_samples,
            "lifted_samples": self.lifted_samples,
            "readable_lift_samples": self.readable_lift_samples,
            "unreadable_lift_samples": self.unreadable_lift_samples,
            "gliding_samples": self.gliding_samples,
            "launching_samples": self.launching_samples,
            "grounded_samples": self.grounded_samples,
            "final_position": vec3_json(self.final_position),
            "native_window_created": false,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SimCheck {
    pub(crate) name: &'static str,
    pub(crate) passed: bool,
    pub(crate) value: f32,
    pub(crate) comparator: &'static str,
    pub(crate) threshold: f32,
    pub(crate) unit: &'static str,
}

impl SimCheck {
    fn at_least(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
        Self {
            name,
            passed: value >= threshold,
            value,
            comparator: ">=",
            threshold,
            unit,
        }
    }

    fn at_most(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
        Self {
            name,
            passed: value <= threshold,
            value,
            comparator: "<=",
            threshold,
            unit,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "passed": self.passed,
            "value": round4(self.value),
            "comparator": self.comparator,
            "threshold": round4(self.threshold),
            "unit": self.unit,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SimResult {
    pub(crate) scenario: EvalScenario,
    pub(crate) passed: bool,
    pub(crate) metrics: SimMetrics,
    pub(crate) checks: Vec<SimCheck>,
    pub(crate) samples: Vec<SimSample>,
    pub(crate) elapsed_ms: f64,
    pub(crate) summary_path: String,
    pub(crate) samples_path: String,
}

impl SimResult {
    pub(crate) fn to_summary_json(&self) -> String {
        serde_json::to_string_pretty(&json!({
            "schema": "nau_traversal_sim_eval.v1",
            "scenario": self.scenario.name,
            "target_island": self.scenario.target_island_name,
            "passed": self.passed,
            "mode": "simulation_only",
            "frame_count": self.scenario.frame_count,
            "duration_secs": round4(self.scenario.duration_secs()),
            "elapsed_ms": round4_f64(self.elapsed_ms),
            "metrics": self.metrics.to_json(),
            "checks": self.checks.iter().map(SimCheck::to_json).collect::<Vec<_>>(),
            "artifacts": {
                "summary_json": self.summary_path,
                "samples_ndjson": self.samples_path,
                "screenshot_png": Value::Null,
                "checkpoint_screenshots": [],
                "checkpoint_marker_metadata": [],
            },
            "final_sample": self.samples.last().map(SimSample::to_json),
        }))
        .expect("summary json")
            + "\n"
    }
}

fn backward_diagonal_rear_response_mps(sample: &SimSample) -> Option<f32> {
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

fn response_latency_secs(input_time_secs: Option<f32>, response_time_secs: Option<f32>) -> f32 {
    match (input_time_secs, response_time_secs) {
        (Some(input_time), Some(response_time)) => (response_time - input_time).max(0.0),
        (Some(_), None) => 999.0,
        _ => 0.0,
    }
}

fn percentile(values: &[f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let index =
        ((sorted.len().saturating_sub(1)) as f32 * percentile.clamp(0.0, 1.0)).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn horizontal_distance(left: Vec3, right: Vec3) -> f32 {
    Vec2::new(left.x - right.x, left.z - right.z).length()
}
