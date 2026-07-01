#[path = "checks/air_control.rs"]
mod air_control;
#[path = "checks/camera_strafe.rs"]
mod camera_strafe;
#[path = "checks/core.rs"]
mod core;

use nau_engine::animation::LANDING_MAX_FOOT_SPLIT_READABILITY_M;
use nau_engine::eval::{
    AIR_CONTROL_RESPONSE, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_STRAFE_STABILITY,
    EvalScenario, LANDING_MAX_POSE_ANTICIPATION_BACKBEND_DEGREES,
    LANDING_MAX_POSE_BACKWARD_BEND_DEGREES, LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES,
    LANDING_MIN_POSE_FLARE_DEGREES, LANDING_MIN_POSE_FOOT_FORWARD_M, LANDING_MIN_POSE_FOOT_SPLIT_M,
    LANDING_MIN_POSE_FORWARD_FOLD_DEGREES, LONG_GLIDE_VISIBILITY,
    MAX_CROSSWIND_NEUTRAL_HORIZONTAL_STEP_M, MIN_CROSSWIND_FORCE_DELTA_MPS,
    MIN_CROSSWIND_FORCE_SAMPLE_COUNT, MIN_CROSSWIND_NEUTRAL_DRIFT_SAMPLE_COUNT,
    MIN_CROSSWIND_NEUTRAL_HORIZONTAL_DRIFT_M, MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS,
    MIN_DYNAMIC_LIFT_MULTIPLIER_RANGE, MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES,
    MIN_DYNAMIC_WIND_FLOW_SPEED_MPS, MIN_DYNAMIC_WIND_FLOW_VARIATION,
    MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE, MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
    MIN_WIND_FORCE_ALIGNED_DELTA_MPS, MIN_WIND_FORCE_DELTA_MPS, MIN_WIND_FORCE_FLOW_ALIGNMENT,
    MIN_WIND_FORCE_FLOW_SPEED_MPS, MIN_WIND_FORCE_SAMPLE_COUNT, MIN_WIND_FORCE_VARIATION,
    MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES, MIN_WIND_LOAD_LATERAL_LOAD,
    MIN_WIND_LOAD_POSE_LEAN_DEGREES, MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT, POSE_STATE_COVERAGE,
    POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES, POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES,
    UPDRAFT_ROUTE,
};
use nau_engine::movement::{LAUNCH_MAX_HORIZONTAL_SPEED_MPS, LAUNCH_MAX_UPWARD_SPEED_MPS};
use serde_json::{Value, json};

use super::{super::round4, SimMetrics};
use crate::{
    AIR_CONTROL_MAX_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_DIVE_POSE_ARM_SPREAD_DEGREES,
    AIR_CONTROL_MAX_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_P95_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MAX_P95_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
    AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES,
    AIR_CONTROL_MIN_DIVE_POSE_LEG_TUCK_DEGREES, AIR_CONTROL_MIN_DIVE_POSE_TORSO_PITCH_DEGREES,
    AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES,
    AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES, GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M,
    GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES, GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M,
    GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES, LANDING_MIN_POSE_CROUCH_M,
    MIN_POSE_LIMB_CLEARANCE_M, MIN_POSE_SCARF_LATERAL_SWAY_M, MIN_POSE_SCARF_STREAM_M,
    MIN_POSE_SCARF_TAIL_FLEX_DEGREES,
};

const POSE_STATE_MIN_IDLE_SAMPLES: f32 = 3.0;
const POSE_STATE_MIN_WALK_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_RUN_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_LAUNCH_SAMPLES: f32 = 3.0;
const POSE_STATE_MIN_FALLING_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_GLIDING_POSE_SAMPLES: f32 = 18.0;
const POSE_STATE_MIN_AIR_TURN_SAMPLES: f32 = 6.0;
const POSE_STATE_MIN_AIR_BRAKE_SAMPLES: f32 = 4.0;
const POSE_STATE_MIN_DIVING_SAMPLES: f32 = 1.0;
const POSE_STATE_MIN_GLIDING_DIVE_SAMPLES: f32 = 1.0;
const POSE_STATE_MIN_LANDING_POSE_SAMPLES: f32 = 1.0;
const POSE_STATE_MIN_LANDING_FLARE_DEGREES: f32 = LANDING_MIN_POSE_FLARE_DEGREES;
const TARGET_LANDING_MIN_GLIDER_RESPONSE_DEGREES: f32 = 4.0;
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
        let mut checks = Vec::new();
        core::append_checks(&mut checks, self, scenario);

        if self.launching_samples > 0 {
            checks.push(SimCheck::at_most(
                "launch_upward_speed",
                self.max_launch_upward_speed_mps,
                LAUNCH_MAX_UPWARD_SPEED_MPS,
                "mps",
            ));
            checks.push(SimCheck::at_most(
                "launch_horizontal_speed",
                self.max_launch_horizontal_speed_mps,
                LAUNCH_MAX_HORIZONTAL_SPEED_MPS,
                "mps",
            ));
        }

        if scenario.thresholds.require_target_landing {
            checks.push(SimCheck::at_least(
                "pose_landing_anticipation_samples",
                self.pose_landing_anticipation_samples as f32,
                1.0,
                "samples",
            ));
            checks.push(SimCheck::at_most(
                "gliding_landing_anticipation_samples",
                self.gliding_landing_anticipation_samples as f32,
                0.0,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_recovery_samples",
                self.pose_landing_recovery_samples as f32,
                1.0,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "target_landing_glider_response",
                self.max_glider_response_degrees,
                TARGET_LANDING_MIN_GLIDER_RESPONSE_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_crouch",
                self.max_pose_landing_crouch_m,
                LANDING_MIN_POSE_CROUCH_M,
                "m",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_foot_forward",
                self.max_pose_landing_foot_forward_m,
                LANDING_MIN_POSE_FOOT_FORWARD_M,
                "m",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_foot_split",
                self.max_pose_landing_foot_split_m,
                LANDING_MIN_POSE_FOOT_SPLIT_M,
                "m",
            ));
            checks.push(SimCheck::at_most(
                "pose_landing_foot_split_max",
                max_landing_foot_split_m(self),
                LANDING_MAX_FOOT_SPLIT_READABILITY_M,
                "m",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_flare",
                self.max_pose_landing_flare_degrees,
                LANDING_MIN_POSE_FLARE_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_most(
                "pose_landing_flare_backbend",
                self.max_pose_landing_flare_degrees,
                LANDING_MAX_POSE_ANTICIPATION_BACKBEND_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_forward_fold",
                self.max_pose_landing_forward_fold_degrees,
                LANDING_MIN_POSE_FORWARD_FOLD_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_most(
                "pose_landing_backward_bend",
                self.max_pose_landing_backward_bend_degrees,
                LANDING_MAX_POSE_BACKWARD_BEND_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_most(
                "pose_landing_recovery_backbend",
                self.max_pose_landing_recovery_flip_degrees,
                LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_most(
                "unreadable_key_pose_samples",
                self.unreadable_key_pose_samples as f32,
                0.0,
                "samples",
            ));
        }

        if scenario.thresholds.min_lifted_samples > 0 {
            checks.push(SimCheck::at_least(
                "dynamic_readable_lift_samples",
                self.dynamic_readable_lift_samples as f32,
                scenario.thresholds.min_lifted_samples as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "max_wind_flow_speed",
                self.max_wind_flow_speed_mps,
                MIN_DYNAMIC_WIND_FLOW_SPEED_MPS,
                "mps",
            ));
            checks.push(SimCheck::at_least(
                "max_wind_flow_variation",
                self.max_wind_flow_variation,
                MIN_DYNAMIC_WIND_FLOW_VARIATION,
                "ratio",
            ));
            checks.push(SimCheck::at_least(
                "max_wind_flow_direction_change",
                self.max_wind_flow_direction_change_degrees,
                MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_least(
                "max_wind_flow_variation_range",
                self.max_wind_flow_variation_range,
                MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE,
                "ratio",
            ));
        }

        if wind_force_scenario(scenario) {
            checks.push(SimCheck::at_least(
                "wind_force_samples",
                self.wind_force_samples as f32,
                MIN_WIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "meaningful_wind_force_samples",
                self.meaningful_wind_force_samples as f32,
                MIN_WIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "aligned_wind_force_samples",
                self.aligned_wind_force_samples as f32,
                MIN_WIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "active_wind_force_fields",
                self.max_active_wind_force_fields as f32,
                1.0,
                "fields",
            ));
            checks.push(SimCheck::at_least(
                "wind_force_delta",
                self.max_wind_force_delta_mps,
                MIN_WIND_FORCE_DELTA_MPS,
                "m/s",
            ));
            checks.push(SimCheck::at_least(
                "wind_force_flow_speed",
                self.max_wind_force_flow_speed_mps,
                MIN_WIND_FORCE_FLOW_SPEED_MPS,
                "m/s",
            ));
            checks.push(SimCheck::at_least(
                "wind_force_variation",
                self.max_wind_force_variation,
                MIN_WIND_FORCE_VARIATION,
                "ratio",
            ));
            checks.push(SimCheck::at_least(
                "wind_force_flow_alignment",
                self.max_wind_force_flow_alignment,
                MIN_WIND_FORCE_FLOW_ALIGNMENT,
                "dot",
            ));
            checks.push(SimCheck::at_least(
                "wind_force_aligned_delta",
                self.max_wind_force_aligned_delta_mps,
                MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
                "m/s",
            ));
        }

        if crosswind_force_scenario(scenario) {
            checks.push(SimCheck::at_least(
                "crosswind_force_samples",
                self.crosswind_force_samples as f32,
                MIN_CROSSWIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "aligned_crosswind_force_samples",
                self.aligned_crosswind_force_samples as f32,
                MIN_CROSSWIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "crosswind_force_fields",
                self.max_crosswind_force_fields as f32,
                1.0,
                "fields",
            ));
            checks.push(SimCheck::at_least(
                "crosswind_force_delta",
                self.max_crosswind_force_delta_mps,
                MIN_CROSSWIND_FORCE_DELTA_MPS,
                "m/s",
            ));
            checks.push(SimCheck::at_least(
                "crosswind_force_flow_alignment",
                self.max_crosswind_force_flow_alignment,
                MIN_WIND_FORCE_FLOW_ALIGNMENT,
                "dot",
            ));
            checks.push(SimCheck::at_least(
                "crosswind_force_aligned_delta",
                self.max_crosswind_force_aligned_delta_mps,
                MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
                "m/s",
            ));
        }

        if scenario.thresholds.min_lifted_samples > 0 {
            checks.push(SimCheck::at_least(
                "updraft_swirl_force_samples",
                self.updraft_swirl_force_samples as f32,
                scenario.thresholds.min_lifted_samples as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "aligned_updraft_swirl_force_samples",
                self.aligned_updraft_swirl_force_samples as f32,
                scenario.thresholds.min_lifted_samples as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "updraft_swirl_force_fields",
                self.max_updraft_swirl_force_fields as f32,
                1.0,
                "fields",
            ));
            checks.push(SimCheck::at_least(
                "updraft_swirl_force_delta",
                self.max_updraft_swirl_force_delta_mps,
                MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
                "m/s",
            ));
            checks.push(SimCheck::at_least(
                "updraft_swirl_force_flow_alignment",
                self.max_updraft_swirl_force_flow_alignment,
                MIN_WIND_FORCE_FLOW_ALIGNMENT,
                "dot",
            ));
            checks.push(SimCheck::at_least(
                "updraft_swirl_force_aligned_delta",
                self.max_updraft_swirl_force_aligned_delta_mps,
                MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
                "m/s",
            ));
        }

        if layered_wind_force_scenario(scenario) {
            checks.push(SimCheck::at_least(
                "layered_dynamic_wind_flow_fields",
                self.max_dynamic_wind_flow_fields as f32,
                2.0,
                "fields",
            ));
            checks.push(SimCheck::at_least(
                "layered_wind_force_samples",
                self.layered_wind_force_samples as f32,
                MIN_WIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "aligned_layered_wind_force_samples",
                self.aligned_layered_wind_force_samples as f32,
                MIN_WIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "crosswind_updraft_overlap_samples",
                self.crosswind_updraft_overlap_samples as f32,
                MIN_WIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "aligned_crosswind_updraft_overlap_samples",
                self.aligned_crosswind_updraft_overlap_samples as f32,
                MIN_WIND_FORCE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "layered_wind_force_fields",
                self.max_layered_wind_force_fields as f32,
                2.0,
                "fields",
            ));
            checks.push(SimCheck::at_least(
                "layered_wind_force_delta",
                self.max_layered_wind_force_delta_mps,
                MIN_WIND_FORCE_DELTA_MPS,
                "m/s",
            ));
            checks.push(SimCheck::at_least(
                "layered_wind_force_flow_alignment",
                self.max_layered_wind_force_flow_alignment,
                MIN_WIND_FORCE_FLOW_ALIGNMENT,
                "dot",
            ));
            checks.push(SimCheck::at_least(
                "layered_wind_force_aligned_delta",
                self.max_layered_wind_force_aligned_delta_mps,
                MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
                "m/s",
            ));
        }

        if wind_load_response_scenario(scenario) {
            checks.push(SimCheck::at_least(
                "wind_load_response_samples",
                self.wind_load_response_samples as f32,
                MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "wind_load_lateral_load",
                self.max_wind_load_lateral_load,
                MIN_WIND_LOAD_LATERAL_LOAD,
                "normalized",
            ));
            checks.push(SimCheck::at_least(
                "wind_load_pose_lean",
                self.max_wind_load_pose_lean_degrees,
                MIN_WIND_LOAD_POSE_LEAN_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_least(
                "wind_load_glider_response",
                self.max_wind_load_glider_response_degrees,
                MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_least(
                "crosswind_neutral_drift_samples",
                self.crosswind_neutral_drift_samples as f32,
                MIN_CROSSWIND_NEUTRAL_DRIFT_SAMPLE_COUNT as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "crosswind_neutral_horizontal_drift",
                self.crosswind_neutral_horizontal_drift_m,
                MIN_CROSSWIND_NEUTRAL_HORIZONTAL_DRIFT_M,
                "m",
            ));
            checks.push(SimCheck::at_most(
                "crosswind_neutral_horizontal_step",
                self.max_crosswind_neutral_horizontal_step_m,
                MAX_CROSSWIND_NEUTRAL_HORIZONTAL_STEP_M,
                "m/sample",
            ));
        }

        if scenario.thresholds.min_lifted_samples > 0 {
            checks.push(SimCheck::at_least(
                "dynamic_lift_samples",
                self.dynamic_lift_samples as f32,
                scenario.thresholds.min_lifted_samples as f32,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "paired_visual_lift_fields",
                self.max_paired_visual_lift_fields as f32,
                1.0,
                "fields",
            ));
            checks.push(SimCheck::at_least(
                "dynamic_lift_fields",
                self.max_dynamic_lift_fields as f32,
                1.0,
                "fields",
            ));
            checks.push(SimCheck::at_least(
                "lift_applied_delta",
                self.max_lift_applied_delta_mps,
                MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS,
                "m/s",
            ));
            checks.push(SimCheck::at_least(
                "dynamic_lift_multiplier_range",
                self.max_dynamic_lift_multiplier_range,
                MIN_DYNAMIC_LIFT_MULTIPLIER_RANGE,
                "ratio",
            ));
        }

        if scenario.name == CAMERA_STRAFE_STABILITY {
            camera_strafe::append_checks(&mut checks, self);
        }

        if scenario.name == AIR_CONTROL_RESPONSE {
            air_control::append_checks(&mut checks, self);
        }

        if scenario.name == POSE_STATE_COVERAGE {
            append_pose_state_coverage_checks(&mut checks, self);
        }

        checks
    }
}

fn append_pose_state_coverage_checks(checks: &mut Vec<SimCheck>, metrics: &SimMetrics) {
    let max_dive_pose_arm_spread_degrees = if metrics.gliding_dive_samples > 0 {
        metrics.max_dive_pose_arm_spread_degrees
    } else {
        f32::INFINITY
    };
    let min_pose_limb_clearance_m = metrics
        .min_pose_limb_clearance_m
        .unwrap_or(f32::NEG_INFINITY);

    checks.extend([
        SimCheck::at_least(
            "pose_state_grounded_idle_samples",
            metrics.pose_grounded_idle_samples as f32,
            POSE_STATE_MIN_IDLE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_grounded_walk_samples",
            metrics.pose_grounded_walk_samples as f32,
            POSE_STATE_MIN_WALK_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_grounded_run_samples",
            metrics.pose_grounded_run_samples as f32,
            POSE_STATE_MIN_RUN_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_walk_stride_foot_travel",
            metrics.max_grounded_walk_stride_foot_travel_m,
            GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M,
            "m",
        ),
        SimCheck::at_least(
            "pose_state_run_stride_foot_travel",
            metrics.max_grounded_run_stride_foot_travel_m,
            GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M,
            "m",
        ),
        SimCheck::at_least(
            "pose_state_walk_stride_leg_opposition",
            metrics.max_grounded_walk_stride_leg_opposition_degrees,
            GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_run_stride_leg_opposition",
            metrics.max_grounded_run_stride_leg_opposition_degrees,
            GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_launching_samples",
            metrics.pose_launching_samples as f32,
            POSE_STATE_MIN_LAUNCH_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_falling_samples",
            metrics.pose_falling_samples as f32,
            POSE_STATE_MIN_FALLING_SAMPLES,
            "samples",
        ),
        SimCheck::at_most(
            "pose_state_falling_upward_velocity_samples",
            metrics.falling_upward_velocity_samples as f32,
            0.0,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_gliding_samples",
            metrics.pose_gliding_samples as f32,
            POSE_STATE_MIN_GLIDING_POSE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_air_turn_samples",
            metrics.pose_air_turn_samples as f32,
            POSE_STATE_MIN_AIR_TURN_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_right_air_turn_samples",
            metrics.right_pose_air_turn_samples as f32,
            POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_left_air_turn_samples",
            metrics.left_pose_air_turn_samples as f32,
            POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_pure_air_turn_sideways_samples",
            metrics
                .pure_air_turn_sideways_body_travel_heading_error_values_degrees
                .len() as f32,
            AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_right_pure_air_turn_sideways_samples",
            metrics.right_pure_air_turn_sideways_samples as f32,
            AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_left_pure_air_turn_sideways_samples",
            metrics.left_pure_air_turn_sideways_samples as f32,
            AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_most(
            "pose_state_p95_pure_air_turn_sideways_body_travel_heading_error",
            metrics.p95_pure_air_turn_sideways_body_travel_heading_error_degrees(),
            AIR_CONTROL_MAX_P95_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_max_pure_air_turn_sideways_body_travel_heading_error",
            metrics.max_pure_air_turn_sideways_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_p95_pure_air_turn_sideways_desired_travel_heading_error",
            metrics.p95_pure_air_turn_sideways_desired_travel_heading_error_degrees(),
            AIR_CONTROL_MAX_P95_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_max_pure_air_turn_sideways_desired_travel_heading_error",
            metrics.max_pure_air_turn_sideways_desired_travel_heading_error_degrees,
            AIR_CONTROL_MAX_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_air_brake_samples",
            metrics.pose_air_brake_samples as f32,
            POSE_STATE_MIN_AIR_BRAKE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_backward_right_air_brake_pose_lateral_lean",
            metrics.max_backward_right_air_brake_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_backward_left_air_brake_pose_lateral_lean",
            metrics.max_backward_left_air_brake_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_backward_diagonal_body_travel_heading_samples",
            metrics
                .backward_diagonal_body_travel_heading_error_values_degrees
                .len() as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_backward_right_diagonal_body_travel_heading_samples",
            metrics.backward_right_diagonal_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_backward_left_diagonal_body_travel_heading_samples",
            metrics.backward_left_diagonal_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_diving_samples",
            metrics.pose_diving_samples as f32,
            POSE_STATE_MIN_DIVING_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_gliding_dive_samples",
            metrics.gliding_dive_samples as f32,
            POSE_STATE_MIN_GLIDING_DIVE_SAMPLES,
            "samples",
        ),
        SimCheck::at_most(
            "pose_state_dive_without_dive_input_samples",
            metrics.dive_without_dive_input_samples as f32,
            0.0,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_dive_pose_torso_pitch",
            metrics.max_dive_pose_torso_pitch_degrees,
            AIR_CONTROL_MIN_DIVE_POSE_TORSO_PITCH_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_dive_pose_arm_spread",
            max_dive_pose_arm_spread_degrees,
            AIR_CONTROL_MAX_DIVE_POSE_ARM_SPREAD_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_dive_pose_leg_tuck",
            metrics.max_dive_pose_leg_tuck_degrees,
            AIR_CONTROL_MIN_DIVE_POSE_LEG_TUCK_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_landing_anticipation_samples",
            metrics.pose_landing_anticipation_samples as f32,
            POSE_STATE_MIN_LANDING_POSE_SAMPLES,
            "samples",
        ),
        SimCheck::at_most(
            "pose_state_gliding_landing_anticipation_samples",
            metrics.gliding_landing_anticipation_samples as f32,
            0.0,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_landing_recovery_samples",
            metrics.pose_landing_recovery_samples as f32,
            POSE_STATE_MIN_LANDING_POSE_SAMPLES,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_landing_crouch",
            metrics.max_pose_landing_crouch_m,
            LANDING_MIN_POSE_CROUCH_M,
            "m",
        ),
        SimCheck::at_least(
            "pose_state_landing_foot_forward",
            metrics.max_pose_landing_foot_forward_m,
            LANDING_MIN_POSE_FOOT_FORWARD_M,
            "m",
        ),
        SimCheck::at_least(
            "pose_state_landing_foot_split",
            metrics.max_pose_landing_foot_split_m,
            LANDING_MIN_POSE_FOOT_SPLIT_M,
            "m",
        ),
        SimCheck::at_most(
            "pose_state_landing_foot_split_max",
            max_landing_foot_split_m(metrics),
            LANDING_MAX_FOOT_SPLIT_READABILITY_M,
            "m",
        ),
        SimCheck::at_least(
            "pose_state_landing_flare",
            metrics.max_pose_landing_flare_degrees,
            POSE_STATE_MIN_LANDING_FLARE_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_landing_flare_backbend",
            metrics.max_pose_landing_flare_degrees,
            LANDING_MAX_POSE_ANTICIPATION_BACKBEND_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_landing_forward_fold",
            metrics.max_pose_landing_forward_fold_degrees,
            LANDING_MIN_POSE_FORWARD_FOLD_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_landing_backward_bend",
            metrics.max_pose_landing_backward_bend_degrees,
            LANDING_MAX_POSE_BACKWARD_BEND_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_landing_recovery_backbend",
            metrics.max_pose_landing_recovery_flip_degrees,
            LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_scarf_stream",
            metrics.max_pose_scarf_stream_m,
            MIN_POSE_SCARF_STREAM_M,
            "m",
        ),
        SimCheck::at_least(
            "pose_state_scarf_lateral_sway",
            metrics.max_pose_scarf_lateral_sway_m,
            MIN_POSE_SCARF_LATERAL_SWAY_M,
            "m",
        ),
        SimCheck::at_least(
            "pose_state_scarf_tail_flex",
            metrics.max_pose_scarf_tail_flex_degrees,
            MIN_POSE_SCARF_TAIL_FLEX_DEGREES,
            "deg",
        ),
        SimCheck::at_most(
            "pose_state_unreadable_key_pose_samples",
            metrics.unreadable_key_pose_samples as f32,
            0.0,
            "samples",
        ),
        SimCheck::at_most(
            "pose_state_key_pose_transition_grace_samples",
            metrics.key_pose_transition_grace_samples as f32,
            POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES as f32,
            "samples",
        ),
        SimCheck::at_least(
            "pose_state_min_pose_limb_clearance",
            min_pose_limb_clearance_m,
            MIN_POSE_LIMB_CLEARANCE_M,
            "m",
        ),
    ]);
}

fn max_landing_foot_split_m(metrics: &SimMetrics) -> f32 {
    metrics
        .max_pose_landing_foot_split_m
        .max(metrics.max_pose_landing_distal_foot_split_m)
}

fn wind_force_scenario(scenario: EvalScenario) -> bool {
    matches!(
        scenario.name,
        BASELINE_ROUTE | UPDRAFT_ROUTE | BRANCH_RECOVERY_ROUTE | LONG_GLIDE_VISIBILITY
    )
}

fn crosswind_force_scenario(scenario: EvalScenario) -> bool {
    matches!(scenario.name, BASELINE_ROUTE | BRANCH_RECOVERY_ROUTE)
}

fn layered_wind_force_scenario(scenario: EvalScenario) -> bool {
    scenario.name == UPDRAFT_ROUTE
}

fn wind_load_response_scenario(scenario: EvalScenario) -> bool {
    scenario.name == UPDRAFT_ROUTE
}
