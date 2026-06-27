#[path = "checks/air_control.rs"]
mod air_control;
#[path = "checks/camera_strafe.rs"]
mod camera_strafe;
#[path = "checks/core.rs"]
mod core;

use nau_engine::eval::{
    AIR_CONTROL_RESPONSE, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_STRAFE_STABILITY,
    EvalScenario, LANDING_MIN_POSE_FLARE_DEGREES, LANDING_MIN_POSE_FOOT_FORWARD_M,
    LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES, LONG_GLIDE_VISIBILITY, MIN_CROSSWIND_FORCE_DELTA_MPS,
    MIN_CROSSWIND_FORCE_SAMPLE_COUNT, MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES,
    MIN_DYNAMIC_WIND_FLOW_SPEED_MPS, MIN_DYNAMIC_WIND_FLOW_VARIATION,
    MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE, MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
    MIN_WIND_FORCE_ALIGNED_DELTA_MPS, MIN_WIND_FORCE_DELTA_MPS, MIN_WIND_FORCE_FLOW_ALIGNMENT,
    MIN_WIND_FORCE_FLOW_SPEED_MPS, MIN_WIND_FORCE_SAMPLE_COUNT, MIN_WIND_FORCE_VARIATION,
    MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES, MIN_WIND_LOAD_LATERAL_LOAD,
    MIN_WIND_LOAD_POSE_LEAN_DEGREES, MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT, POSE_STATE_COVERAGE,
    UPDRAFT_ROUTE,
};
use serde_json::{Value, json};

use super::{super::round4, SimMetrics};
use crate::{
    GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
    GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
    LANDING_MIN_POSE_CROUCH_M, MIN_POSE_SCARF_LATERAL_SWAY_M, MIN_POSE_SCARF_STREAM_M,
    MIN_POSE_SCARF_TAIL_FLEX_DEGREES,
};

const POSE_STATE_MIN_WALK_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_RUN_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_LAUNCH_SAMPLES: f32 = 3.0;
const POSE_STATE_MIN_FALLING_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_GLIDING_POSE_SAMPLES: f32 = 18.0;
const POSE_STATE_MIN_AIR_TURN_SAMPLES: f32 = 6.0;
const POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES: f32 = 3.0;
const POSE_STATE_MIN_AIR_BRAKE_SAMPLES: f32 = 4.0;
const POSE_STATE_MIN_DIVING_SAMPLES: f32 = 1.0;
const POSE_STATE_MIN_GLIDING_DIVE_SAMPLES: f32 = 1.0;
const POSE_STATE_MIN_LANDING_POSE_SAMPLES: f32 = 1.0;

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

        if scenario.thresholds.require_target_landing {
            checks.push(SimCheck::at_least(
                "pose_landing_anticipation_samples",
                self.pose_landing_anticipation_samples as f32,
                1.0,
                "samples",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_recovery_samples",
                self.pose_landing_recovery_samples as f32,
                1.0,
                "samples",
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
                "pose_landing_flare",
                self.max_pose_landing_flare_degrees,
                LANDING_MIN_POSE_FLARE_DEGREES,
                "deg",
            ));
            checks.push(SimCheck::at_least(
                "pose_landing_recovery_flip",
                self.max_pose_landing_recovery_flip_degrees,
                LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES,
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
    checks.extend([
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
            "pose_state_air_brake_samples",
            metrics.pose_air_brake_samples as f32,
            POSE_STATE_MIN_AIR_BRAKE_SAMPLES,
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
        SimCheck::at_least(
            "pose_state_landing_anticipation_samples",
            metrics.pose_landing_anticipation_samples as f32,
            POSE_STATE_MIN_LANDING_POSE_SAMPLES,
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
            "pose_state_landing_flare",
            metrics.max_pose_landing_flare_degrees,
            LANDING_MIN_POSE_FLARE_DEGREES,
            "deg",
        ),
        SimCheck::at_least(
            "pose_state_landing_recovery_flip",
            metrics.max_pose_landing_recovery_flip_degrees,
            LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES,
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
    ]);
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
