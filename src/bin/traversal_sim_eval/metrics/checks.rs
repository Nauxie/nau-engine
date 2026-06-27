#[path = "checks/air_control.rs"]
mod air_control;
#[path = "checks/camera_strafe.rs"]
mod camera_strafe;
#[path = "checks/core.rs"]
mod core;

use nau_engine::eval::{
    AIR_CONTROL_RESPONSE, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_STRAFE_STABILITY,
    EvalScenario, LANDING_MIN_POSE_FLARE_DEGREES, LONG_GLIDE_VISIBILITY,
    MIN_CROSSWIND_FORCE_DELTA_MPS, MIN_CROSSWIND_FORCE_SAMPLE_COUNT,
    MIN_DYNAMIC_WIND_FLOW_SPEED_MPS, MIN_DYNAMIC_WIND_FLOW_VARIATION,
    MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE, MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
    MIN_WIND_FORCE_DELTA_MPS, MIN_WIND_FORCE_FLOW_SPEED_MPS, MIN_WIND_FORCE_SAMPLE_COUNT,
    MIN_WIND_FORCE_VARIATION, POSE_STATE_COVERAGE, UPDRAFT_ROUTE,
};
use serde_json::{Value, json};

use super::{super::round4, SimMetrics};
use crate::LANDING_MIN_POSE_CROUCH_M;

const POSE_STATE_MIN_WALK_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_RUN_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_LAUNCH_SAMPLES: f32 = 3.0;
const POSE_STATE_MIN_FALLING_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_GLIDING_POSE_SAMPLES: f32 = 18.0;

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
                "pose_landing_flare",
                self.max_pose_landing_flare_degrees,
                LANDING_MIN_POSE_FLARE_DEGREES,
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
        }

        if crosswind_force_scenario(scenario) {
            checks.push(SimCheck::at_least(
                "crosswind_force_samples",
                self.crosswind_force_samples as f32,
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
        }

        if scenario.thresholds.min_lifted_samples > 0 {
            checks.push(SimCheck::at_least(
                "updraft_swirl_force_samples",
                self.updraft_swirl_force_samples as f32,
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
