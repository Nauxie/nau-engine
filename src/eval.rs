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
    EvalCheckpoint, EvalScenario, GREAT_SKY_PLATEAU_ROUTE, GREAT_SKY_PLATEAU_VISTAS,
    GROUND_TAXI_CONTROL, ISLAND_LAUNCH_TO_LANDING, LONG_GLIDE_VISIBILITY, PLATEAU_ARRIVAL_CAMERA,
    PLAYTEST_RESET, POSE_STATE_COVERAGE, RETURN_DESCENT_ROUTE, SCENARIO_NAMES,
    TERRAIN_BODY_COLLISION_CONTACT, TERRAIN_EDGE_WALKOFF, TERRAIN_RIM_COLLISION_CONTACT,
    UNDERBRIDGE_UNDER_ROUTE, UPDRAFT_ROUTE, WORLD_COLLISION_CONTACT, scenario_named,
    scripted_camera_input, scripted_input, scripted_playtest_reset_requested,
};
pub use summary::{EvalArtifacts, EvalCheck, EvalMetricsSummary, EvalSummary};
#[cfg(test)]
use thresholds::*;
pub use thresholds::{
    AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES, EvalThresholds,
    LANDING_MAX_POSE_ANTICIPATION_BACKBEND_DEGREES, LANDING_MAX_POSE_BACKWARD_BEND_DEGREES,
    LANDING_MAX_POSE_RECOVERY_BACKBEND_DEGREES, LANDING_MIN_POSE_FLARE_DEGREES,
    LANDING_MIN_POSE_FOOT_FORWARD_M, LANDING_MIN_POSE_FOOT_SPLIT_M,
    LANDING_MIN_POSE_FORWARD_FOLD_DEGREES, MAX_CROSSWIND_NEUTRAL_HORIZONTAL_STEP_M,
    MAX_LOW_SPEED_PLAYER_WIND_SHEAR_VISIBLE_SAMPLES, MAX_PLAYER_WIND_SHEAR_FRAME_MOTION_M,
    MAX_PLAYER_WIND_SHEAR_PULSE_SCALE, MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
    MAX_VISIBLE_PLAYER_WIND_SHEAR_VISUAL_COUNT, MIN_CROSSWIND_FORCE_DELTA_MPS,
    MIN_CROSSWIND_FORCE_SAMPLE_COUNT, MIN_CROSSWIND_NEUTRAL_DRIFT_SAMPLE_COUNT,
    MIN_CROSSWIND_NEUTRAL_HORIZONTAL_DRIFT_M, MIN_DYNAMIC_LIFT_APPLIED_DELTA_MPS,
    MIN_DYNAMIC_LIFT_MULTIPLIER_RANGE, MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES,
    MIN_DYNAMIC_WIND_FLOW_SPEED_MPS, MIN_DYNAMIC_WIND_FLOW_VARIATION,
    MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE, MIN_PLAYER_WIND_SHEAR_ANGULAR_COVERAGE_DEGREES,
    MIN_PLAYER_WIND_SHEAR_BODY_CLEARANCE_M, MIN_PLAYER_WIND_SHEAR_CROSSWIND_DEFLECTION_M,
    MIN_PLAYER_WIND_SHEAR_DEPTH_OFFSET_M, MIN_PLAYER_WIND_SHEAR_DIVE_PRESSURE,
    MIN_PLAYER_WIND_SHEAR_FIELD_SPAN_M, MIN_PLAYER_WIND_SHEAR_FLOW_ALIGNMENT,
    MIN_PLAYER_WIND_SHEAR_FLOW_TRAVEL_M, MIN_PLAYER_WIND_SHEAR_FRAME_MOTION_M,
    MIN_PLAYER_WIND_SHEAR_LATERAL_OFFSET_M, MIN_PLAYER_WIND_SHEAR_LENGTH_SCALE,
    MIN_PLAYER_WIND_SHEAR_ORBIT_RADIUS_M, MIN_PLAYER_WIND_SHEAR_PULSE_SCALE,
    MIN_PLAYER_WIND_SHEAR_RELATIVE_AIR_SPEED_MPS, MIN_PLAYER_WIND_SHEAR_VERTICAL_COVERAGE_M,
    MIN_PLAYER_WIND_SHEAR_VISUAL_COUNT, MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
    MIN_VISIBLE_PLAYER_WIND_SHEAR_KIND_COUNT, MIN_VISIBLE_PLAYER_WIND_SHEAR_VISUAL_COUNT,
    MIN_WIND_FORCE_ALIGNED_DELTA_MPS, MIN_WIND_FORCE_DELTA_MPS, MIN_WIND_FORCE_FLOW_ALIGNMENT,
    MIN_WIND_FORCE_FLOW_SPEED_MPS, MIN_WIND_FORCE_SAMPLE_COUNT, MIN_WIND_FORCE_VARIATION,
    MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES, MIN_WIND_LOAD_LATERAL_LOAD,
    MIN_WIND_LOAD_POSE_LEAN_DEGREES, MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT,
    POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES, POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES,
    UNDER_ROUTE_MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS, min_updraft_swirl_force_delta_mps_for_scenario,
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
