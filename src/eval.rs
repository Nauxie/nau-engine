use bevy::prelude::Vec3;

mod accumulator;
mod sample;
mod scenarios;
mod summary;
mod thresholds;
pub use accumulator::EvalAccumulator;
pub use sample::{
    EvalMovementMetrics, EvalObjectiveProgress, EvalPoseReadabilityMetrics,
    EvalPoseTemporalMetrics, EvalSample,
};
pub use scenarios::{
    AIR_CONTROL_RESPONSE, APP_ONLY_SCENARIO_NAMES, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE,
    CAMERA_MOUSE_CONTROL, CAMERA_STRAFE_STABILITY, CAMERA_TURN_STABILITY, CAMERA_YAW_STABILITY,
    EvalCheckpoint, EvalScenario, GROUND_TAXI_CONTROL, ISLAND_LAUNCH_TO_LANDING,
    LONG_GLIDE_VISIBILITY, POSE_STATE_COVERAGE, SCENARIO_NAMES, TERRAIN_BODY_COLLISION_CONTACT,
    TERRAIN_RIM_COLLISION_CONTACT, UPDRAFT_ROUTE, WORLD_COLLISION_CONTACT, scenario_named,
    scripted_camera_input, scripted_input,
};
pub use summary::{EvalArtifacts, EvalCheck, EvalMetricsSummary, EvalSummary};
#[cfg(test)]
use thresholds::*;
pub use thresholds::{
    AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES, EvalThresholds,
    LANDING_MIN_POSE_FLARE_DEGREES, LANDING_MIN_POSE_FOOT_FORWARD_M, LANDING_MIN_POSE_FOOT_SPLIT_M,
    LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES, MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
    MIN_CROSSWIND_FORCE_DELTA_MPS, MIN_CROSSWIND_FORCE_SAMPLE_COUNT,
    MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS, MIN_DYNAMIC_LIFT_MULTIPLIER_RANGE,
    MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES, MIN_DYNAMIC_WIND_FLOW_SPEED_MPS,
    MIN_DYNAMIC_WIND_FLOW_VARIATION, MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE,
    MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS, MIN_WIND_FORCE_ALIGNED_DELTA_MPS, MIN_WIND_FORCE_DELTA_MPS,
    MIN_WIND_FORCE_FLOW_ALIGNMENT, MIN_WIND_FORCE_FLOW_SPEED_MPS, MIN_WIND_FORCE_SAMPLE_COUNT,
    MIN_WIND_FORCE_VARIATION, MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES, MIN_WIND_LOAD_LATERAL_LOAD,
    MIN_WIND_LOAD_POSE_LEAN_DEGREES, MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT,
    POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES, POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES,
};

fn vec3_array(value: Vec3) -> [f32; 3] {
    [value.x, value.y, value.z]
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
