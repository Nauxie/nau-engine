use nau_engine::eval::EvalScenario;
use serde_json::{Value, json};

use super::super::{SimSample, round4, round4_f64, vec3_json};
use super::{
    SimMetrics,
    checks::SimCheck,
    util::{avg_body_heading_error_degrees, p95_body_heading_error_degrees, response_latency_secs},
};

impl SimMetrics {
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
            "avg_desired_body_heading_error_degrees": round4(avg_body_heading_error_degrees(self)),
            "p95_desired_body_heading_error_degrees": round4(p95_body_heading_error_degrees(self)),
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
            "max_dynamic_wind_flow_fields": self.max_dynamic_wind_flow_fields,
            "max_wind_flow_speed_mps": self.max_wind_flow_speed_mps,
            "max_wind_flow_variation": self.max_wind_flow_variation,
            "max_wind_flow_variation_range": self.max_wind_flow_variation_range,
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
            "pose_gliding_samples": self.pose_gliding_samples,
            "pose_diving_samples": self.pose_diving_samples,
            "pose_air_brake_samples": self.pose_air_brake_samples,
            "pose_landing_anticipation_samples": self.pose_landing_anticipation_samples,
            "lifted_samples": self.lifted_samples,
            "readable_lift_samples": self.readable_lift_samples,
            "unreadable_lift_samples": self.unreadable_lift_samples,
            "dynamic_readable_lift_samples": self.dynamic_readable_lift_samples,
            "gliding_samples": self.gliding_samples,
            "launching_samples": self.launching_samples,
            "grounded_samples": self.grounded_samples,
            "final_position": vec3_json(self.final_position),
            "native_window_created": false,
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
