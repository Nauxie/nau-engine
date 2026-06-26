#[path = "checks/air_control.rs"]
mod air_control;
#[path = "checks/camera_strafe.rs"]
mod camera_strafe;
#[path = "checks/core.rs"]
mod core;

use nau_engine::eval::{
    AIR_CONTROL_RESPONSE, CAMERA_STRAFE_STABILITY, EvalScenario, MIN_DYNAMIC_WIND_FLOW_SPEED_MPS,
    MIN_DYNAMIC_WIND_FLOW_VARIATION, MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE,
};
use serde_json::{Value, json};

use super::{super::round4, SimMetrics};
use crate::LANDING_MIN_POSE_CROUCH_M;

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
                "pose_landing_crouch",
                self.max_pose_landing_crouch_m,
                LANDING_MIN_POSE_CROUCH_M,
                "m",
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

        if scenario.name == CAMERA_STRAFE_STABILITY {
            camera_strafe::append_checks(&mut checks, self);
        }

        if scenario.name == AIR_CONTROL_RESPONSE {
            air_control::append_checks(&mut checks, self);
        }

        checks
    }
}
