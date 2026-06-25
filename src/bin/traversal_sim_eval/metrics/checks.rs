use nau_engine::eval::{AIR_CONTROL_RESPONSE, CAMERA_STRAFE_STABILITY, EvalScenario};
use serde_json::{Value, json};

use super::super::{
    AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES, AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES, AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
    AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS, AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
    AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES, AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
    AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS, AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS, AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
    AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS, AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
    AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS, AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
    AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS, CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
    CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS, MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
    round4,
};
use super::{
    SimMetrics,
    util::{avg_body_heading_error_degrees, p95_body_heading_error_degrees, response_latency_secs},
};

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

    pub(super) fn to_json(&self) -> Value {
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

impl SimMetrics {
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
                    avg_body_heading_error_degrees(self),
                    AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
                    "deg",
                ),
                SimCheck::at_most(
                    "air_control_p95_body_heading_error",
                    p95_body_heading_error_degrees(self),
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
}
