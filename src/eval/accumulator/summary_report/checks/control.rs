use super::super::{super::EvalAccumulator, derived::SummaryDerivedMetrics};
use crate::{
    animation::{
        GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
        GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M, GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
    },
    eval::{
        scenarios::{
            AIR_CONTROL_RESPONSE, BASELINE_ROUTE, BRANCH_RECOVERY_ROUTE, CAMERA_STRAFE_STABILITY,
            EvalScenario, LONG_GLIDE_VISIBILITY, POSE_STATE_COVERAGE, UPDRAFT_ROUTE,
        },
        summary::EvalCheck,
        thresholds::{EvalThresholds, *},
    },
};

const POSE_STATE_MIN_WALK_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_RUN_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_IDLE_SAMPLES: f32 = 3.0;
const POSE_STATE_MIN_LAUNCH_SAMPLES: f32 = 3.0;
const POSE_STATE_MIN_FALLING_SAMPLES: f32 = 8.0;
const POSE_STATE_MIN_GLIDING_POSE_SAMPLES: f32 = 18.0;
const POSE_STATE_MIN_AIR_TURN_SAMPLES: f32 = 6.0;
const POSE_STATE_MIN_AIR_BRAKE_SAMPLES: f32 = 4.0;
const POSE_STATE_MIN_DIVING_SAMPLES: f32 = 1.0;
const POSE_STATE_MIN_GLIDING_DIVE_SAMPLES: f32 = 1.0;
const POSE_STATE_MIN_LANDING_POSE_SAMPLES: f32 = 1.0;
const AIR_CONTROL_MIN_GLIDING_DIVE_SAMPLES: f32 = 1.0;
const AIR_CONTROL_MIN_AUTHORED_DIVE_CLIP_SAMPLES: f32 = 1.0;
const AIR_CONTROL_MIN_AUTHORED_AIR_BRAKE_CLIP_SAMPLES: f32 = 4.0;
const AIR_CONTROL_MIN_AUTHORED_GLIDER_DIVE_MOTION_M: f32 = 0.04;
const TARGET_LANDING_MIN_AUTHORED_LAND_CLIP_SAMPLES: f32 = 2.0;

pub(super) fn append_scenario_checks(
    checks: &mut Vec<EvalCheck>,
    acc: &EvalAccumulator,
    scenario: EvalScenario,
    derived: &SummaryDerivedMetrics,
    thresholds: &EvalThresholds,
) {
    if thresholds.min_lifted_samples > 0 {
        checks.push(EvalCheck::at_least(
            "readable_lift_samples",
            acc.readable_lift_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_most(
            "unreadable_lift_samples",
            acc.unreadable_lift_samples as f32,
            0.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "dynamic_readable_lift_samples",
            acc.dynamic_readable_lift_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "max_wind_flow_speed",
            acc.max_wind_flow_speed_mps,
            MIN_DYNAMIC_WIND_FLOW_SPEED_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "max_wind_flow_variation",
            acc.max_wind_flow_variation,
            MIN_DYNAMIC_WIND_FLOW_VARIATION,
            "ratio",
        ));
        checks.push(EvalCheck::at_least(
            "max_wind_flow_direction_change",
            acc.max_wind_flow_direction_change_degrees,
            MIN_DYNAMIC_WIND_FLOW_DIRECTION_CHANGE_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_least(
            "max_wind_flow_variation_range",
            acc.max_wind_flow_variation_range,
            MIN_DYNAMIC_WIND_FLOW_VARIATION_RANGE,
            "ratio",
        ));
    }
    if thresholds.require_target_landing {
        checks.push(EvalCheck::at_most(
            "final_target_distance",
            derived.final_target_distance_m,
            thresholds.max_final_target_distance_m,
            "m",
        ));
        checks.push(EvalCheck::at_least(
            "target_landing_samples",
            acc.target_landing_samples as f32,
            thresholds.min_target_landing_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_anticipation_samples",
            acc.pose_landing_anticipation_samples as f32,
            1.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_recovery_samples",
            acc.pose_landing_recovery_samples as f32,
            1.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "authored_landing_clip_samples",
            acc.authored_land_clip_samples as f32,
            TARGET_LANDING_MIN_AUTHORED_LAND_CLIP_SAMPLES,
            "samples",
        ));
        checks.push(EvalCheck::at_most(
            "authored_clip_mismatch_samples",
            acc.authored_clip_mismatch_samples as f32,
            0.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_crouch",
            acc.max_pose_landing_crouch_m,
            LANDING_MIN_POSE_CROUCH_M,
            "m",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_foot_forward",
            acc.max_pose_landing_foot_forward_m,
            LANDING_MIN_POSE_FOOT_FORWARD_M,
            "m",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_foot_split",
            acc.max_pose_landing_foot_split_m,
            LANDING_MIN_POSE_FOOT_SPLIT_M,
            "m",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_flare",
            acc.max_pose_landing_flare_degrees,
            LANDING_MIN_POSE_FLARE_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_least(
            "pose_landing_recovery_flip",
            acc.max_pose_landing_recovery_flip_degrees,
            LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "unreadable_key_pose_samples",
            acc.unreadable_key_pose_samples as f32,
            0.0,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "landing_pose_temporal_stability_samples",
            acc.landing_pose_temporal_stability_samples as f32,
            MIN_POSE_TEMPORAL_STABILITY_SAMPLES as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_most(
            "landing_pose_part_rotation_delta",
            acc.max_landing_pose_part_rotation_delta_degrees,
            MAX_POSE_PART_ROTATION_DELTA_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_most(
            "landing_pose_part_translation_delta",
            acc.max_landing_pose_part_translation_delta_m,
            MAX_POSE_PART_TRANSLATION_DELTA_M,
            "m",
        ));
    }
    if wind_force_scenario(scenario) {
        checks.push(EvalCheck::at_least(
            "wind_force_samples",
            acc.wind_force_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "meaningful_wind_force_samples",
            acc.meaningful_wind_force_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "aligned_wind_force_samples",
            acc.aligned_wind_force_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "active_wind_force_fields",
            acc.max_active_wind_force_fields as f32,
            1.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_delta",
            acc.max_wind_force_delta_mps,
            MIN_WIND_FORCE_DELTA_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_flow_speed",
            acc.max_wind_force_flow_speed_mps,
            MIN_WIND_FORCE_FLOW_SPEED_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_variation",
            acc.max_wind_force_variation,
            MIN_WIND_FORCE_VARIATION,
            "ratio",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_flow_alignment",
            acc.max_wind_force_flow_alignment,
            MIN_WIND_FORCE_FLOW_ALIGNMENT,
            "dot",
        ));
        checks.push(EvalCheck::at_least(
            "wind_force_aligned_delta",
            acc.max_wind_force_aligned_delta_mps,
            MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
            "m/s",
        ));
    }
    if crosswind_force_scenario(scenario) {
        checks.push(EvalCheck::at_least(
            "crosswind_force_samples",
            acc.crosswind_force_samples as f32,
            MIN_CROSSWIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "aligned_crosswind_force_samples",
            acc.aligned_crosswind_force_samples as f32,
            MIN_CROSSWIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "crosswind_force_fields",
            acc.max_crosswind_force_fields as f32,
            1.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "crosswind_force_delta",
            acc.max_crosswind_force_delta_mps,
            MIN_CROSSWIND_FORCE_DELTA_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "crosswind_force_flow_alignment",
            acc.max_crosswind_force_flow_alignment,
            MIN_WIND_FORCE_FLOW_ALIGNMENT,
            "dot",
        ));
        checks.push(EvalCheck::at_least(
            "crosswind_force_aligned_delta",
            acc.max_crosswind_force_aligned_delta_mps,
            MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
            "m/s",
        ));
    }
    if thresholds.min_lifted_samples > 0 {
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_samples",
            acc.updraft_swirl_force_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "aligned_updraft_swirl_force_samples",
            acc.aligned_updraft_swirl_force_samples as f32,
            thresholds.min_lifted_samples as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_fields",
            acc.max_updraft_swirl_force_fields as f32,
            1.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_delta",
            acc.max_updraft_swirl_force_delta_mps,
            MIN_UPDRAFT_SWIRL_FORCE_DELTA_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_flow_alignment",
            acc.max_updraft_swirl_force_flow_alignment,
            MIN_WIND_FORCE_FLOW_ALIGNMENT,
            "dot",
        ));
        checks.push(EvalCheck::at_least(
            "updraft_swirl_force_aligned_delta",
            acc.max_updraft_swirl_force_aligned_delta_mps,
            MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
            "m/s",
        ));
    }
    if layered_wind_force_scenario(scenario) {
        checks.push(EvalCheck::at_least(
            "layered_dynamic_wind_flow_fields",
            acc.max_dynamic_wind_flow_fields as f32,
            2.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "layered_wind_force_samples",
            acc.layered_wind_force_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "aligned_layered_wind_force_samples",
            acc.aligned_layered_wind_force_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "crosswind_updraft_overlap_samples",
            acc.crosswind_updraft_overlap_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "aligned_crosswind_updraft_overlap_samples",
            acc.aligned_crosswind_updraft_overlap_samples as f32,
            MIN_WIND_FORCE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "layered_wind_force_fields",
            acc.max_layered_wind_force_fields as f32,
            2.0,
            "fields",
        ));
        checks.push(EvalCheck::at_least(
            "layered_wind_force_delta",
            acc.max_layered_wind_force_delta_mps,
            MIN_WIND_FORCE_DELTA_MPS,
            "m/s",
        ));
        checks.push(EvalCheck::at_least(
            "layered_wind_force_flow_alignment",
            acc.max_layered_wind_force_flow_alignment,
            MIN_WIND_FORCE_FLOW_ALIGNMENT,
            "dot",
        ));
        checks.push(EvalCheck::at_least(
            "layered_wind_force_aligned_delta",
            acc.max_layered_wind_force_aligned_delta_mps,
            MIN_WIND_FORCE_ALIGNED_DELTA_MPS,
            "m/s",
        ));
    }
    if wind_load_response_scenario(scenario) {
        checks.push(EvalCheck::at_least(
            "wind_load_response_samples",
            acc.wind_load_response_samples as f32,
            MIN_WIND_LOAD_RESPONSE_SAMPLE_COUNT as f32,
            "samples",
        ));
        checks.push(EvalCheck::at_least(
            "wind_load_lateral_load",
            acc.max_wind_load_lateral_load,
            MIN_WIND_LOAD_LATERAL_LOAD,
            "normalized",
        ));
        checks.push(EvalCheck::at_least(
            "wind_load_pose_lean",
            acc.max_wind_load_pose_lean_degrees,
            MIN_WIND_LOAD_POSE_LEAN_DEGREES,
            "deg",
        ));
        checks.push(EvalCheck::at_least(
            "wind_load_glider_response",
            acc.max_wind_load_glider_response_degrees,
            MIN_WIND_LOAD_GLIDER_RESPONSE_DEGREES,
            "deg",
        ));
    }
    if scenario.name == AIR_CONTROL_RESPONSE {
        append_air_control_checks(checks, acc, derived);
    }
    if scenario.name == POSE_STATE_COVERAGE {
        append_pose_state_coverage_checks(checks, acc);
    }
    if scenario.name == CAMERA_STRAFE_STABILITY {
        append_camera_strafe_checks(checks, acc);
    }
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

fn append_air_control_checks(
    checks: &mut Vec<EvalCheck>,
    acc: &EvalAccumulator,
    derived: &SummaryDerivedMetrics,
) {
    checks.extend([
        EvalCheck::at_most(
            "air_control_lateral_response_latency",
            derived.lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_lateral_response",
            acc.max_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_right_lateral_response_latency",
            derived.right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_right_lateral_response",
            acc.max_right_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_left_lateral_response_latency",
            derived.left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_left_lateral_response",
            acc.max_left_lateral_response_mps,
            AIR_CONTROL_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_backward_lateral_response_latency",
            derived.backward_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_backward_lateral_response",
            acc.max_backward_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_backward_right_lateral_response_latency",
            derived.backward_right_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_backward_right_lateral_response",
            acc.max_backward_right_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_backward_right_rear_response",
            acc.max_backward_right_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_backward_left_lateral_response_latency",
            derived.backward_left_lateral_response_latency_secs,
            AIR_CONTROL_MAX_LATERAL_RESPONSE_LATENCY_SECS,
            "s",
        ),
        EvalCheck::at_least(
            "air_control_backward_left_lateral_response",
            acc.max_backward_left_lateral_response_mps,
            AIR_CONTROL_MIN_BACKWARD_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_backward_left_rear_response",
            acc.max_backward_left_rear_response_mps,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_REAR_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_air_brake_speed_drop",
            acc.max_air_brake_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_SPEED_DROP_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_air_brake_planar_speed_drop",
            acc.max_air_brake_planar_speed_drop_mps,
            AIR_CONTROL_MIN_AIR_BRAKE_PLANAR_SPEED_DROP_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_pose_air_turn_samples",
            acc.pose_air_turn_samples as f32,
            AIR_CONTROL_MIN_POSE_AIR_TURN_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_right_pose_air_turn_samples",
            acc.right_pose_air_turn_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_TURN_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_left_pose_air_turn_samples",
            acc.left_pose_air_turn_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_TURN_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_pose_air_brake_samples",
            acc.pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_POSE_AIR_BRAKE_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_right_pose_air_brake_samples",
            acc.right_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_BRAKE_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_left_pose_air_brake_samples",
            acc.left_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_BRAKE_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_backward_right_pose_air_brake_samples",
            acc.backward_right_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_BRAKE_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_backward_left_pose_air_brake_samples",
            acc.backward_left_pose_air_brake_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_BRAKE_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_pose_diving_samples",
            acc.pose_diving_samples as f32,
            1.0,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_gliding_dive_samples",
            acc.gliding_dive_samples as f32,
            AIR_CONTROL_MIN_GLIDING_DIVE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_authored_bank_left_clip_samples",
            acc.authored_bank_left_clip_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_TURN_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_authored_bank_right_clip_samples",
            acc.authored_bank_right_clip_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_POSE_AIR_TURN_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_authored_dive_clip_samples",
            acc.authored_dive_clip_samples as f32,
            AIR_CONTROL_MIN_AUTHORED_DIVE_CLIP_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_authored_air_brake_clip_samples",
            acc.authored_air_brake_clip_samples as f32,
            AIR_CONTROL_MIN_AUTHORED_AIR_BRAKE_CLIP_SAMPLES,
            "samples",
        ),
        EvalCheck::at_most(
            "air_control_authored_clip_mismatch_samples",
            acc.authored_clip_mismatch_samples as f32,
            0.0,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_pose_torso_pitch",
            acc.max_pose_torso_pitch_degrees,
            AIR_CONTROL_MIN_POSE_TORSO_PITCH_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_arm_spread",
            acc.max_pose_arm_spread_degrees,
            AIR_CONTROL_MIN_POSE_ARM_SPREAD_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_leg_tuck",
            acc.max_pose_leg_tuck_degrees,
            AIR_CONTROL_MIN_POSE_LEG_TUCK_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_lateral_lean",
            acc.max_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_right_pose_lateral_lean",
            acc.max_right_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_left_pose_lateral_lean",
            acc.max_left_pose_lateral_lean_degrees,
            AIR_CONTROL_MIN_SIGNED_POSE_LATERAL_LEAN_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pose_wing_airflow",
            acc.max_pose_wing_airflow_strength,
            AIR_CONTROL_MIN_POSE_WING_AIRFLOW_STRENGTH,
            "ratio",
        ),
        EvalCheck::at_least(
            "air_control_pose_scarf_stream",
            acc.max_pose_scarf_stream_m,
            MIN_POSE_SCARF_STREAM_M,
            "m",
        ),
        EvalCheck::at_least(
            "air_control_pose_scarf_lateral_sway",
            acc.max_pose_scarf_lateral_sway_m,
            MIN_POSE_SCARF_LATERAL_SWAY_M,
            "m",
        ),
        EvalCheck::at_least(
            "air_control_pose_scarf_tail_flex",
            acc.max_pose_scarf_tail_flex_degrees,
            MIN_POSE_SCARF_TAIL_FLEX_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_authored_glider_response",
            acc.max_authored_glider_response_degrees,
            AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_authored_glider_dive_response",
            acc.max_authored_glider_dive_response_degrees,
            AIR_CONTROL_MIN_AUTHORED_GLIDER_RESPONSE_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_authored_glider_dive_motion",
            acc.max_authored_glider_dive_motion_m,
            AIR_CONTROL_MIN_AUTHORED_GLIDER_DIVE_MOTION_M,
            "m",
        ),
        EvalCheck::at_most(
            "air_control_unreadable_key_pose_samples",
            acc.unreadable_key_pose_samples as f32,
            0.0,
            "samples",
        ),
        EvalCheck::at_most(
            "air_control_key_pose_transition_grace_samples",
            acc.key_pose_transition_grace_samples as f32,
            AIR_CONTROL_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_pose_temporal_stability_samples",
            acc.pose_temporal_stability_samples as f32,
            MIN_POSE_TEMPORAL_STABILITY_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "air_control_max_pose_part_rotation_delta",
            acc.max_pose_part_rotation_delta_degrees,
            MAX_POSE_PART_ROTATION_DELTA_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_pose_part_translation_delta",
            acc.max_pose_part_translation_delta_m,
            MAX_POSE_PART_TRANSLATION_DELTA_M,
            "m",
        ),
        EvalCheck::at_least(
            "air_control_post_brake_forward_alignment",
            acc.max_post_brake_forward_alignment_mps,
            AIR_CONTROL_MIN_POST_BRAKE_ALIGNMENT_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "air_control_desired_heading_alignment",
            acc.max_desired_heading_alignment_mps,
            AIR_CONTROL_MIN_DESIRED_ALIGNMENT_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "air_control_avg_body_heading_error",
            derived.avg_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_AVG_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_p95_body_heading_error",
            derived.p95_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_P95_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_body_heading_error",
            acc.max_desired_body_heading_error_degrees,
            AIR_CONTROL_MAX_BODY_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_lateral_body_travel_heading_samples",
            acc.lateral_body_travel_heading_error_values_degrees.len() as f32,
            AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_right_body_travel_heading_samples",
            acc.right_lateral_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_left_body_travel_heading_samples",
            acc.left_lateral_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_LATERAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "air_control_p95_lateral_body_travel_heading_error",
            derived.p95_lateral_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_P95_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_lateral_body_travel_heading_error",
            acc.max_lateral_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_backward_diagonal_body_travel_heading_samples",
            acc.backward_diagonal_body_travel_heading_error_values_degrees
                .len() as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_backward_right_diagonal_body_travel_heading_samples",
            acc.backward_right_diagonal_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_backward_left_diagonal_body_travel_heading_samples",
            acc.backward_left_diagonal_body_travel_heading_samples as f32,
            AIR_CONTROL_MIN_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "air_control_p95_backward_diagonal_body_travel_heading_error",
            derived.p95_backward_diagonal_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_P95_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_backward_diagonal_body_travel_heading_error",
            acc.max_backward_diagonal_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_BACKWARD_DIAGONAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_desired_travel_heading_samples",
            acc.desired_travel_heading_error_values_degrees.len() as f32,
            AIR_CONTROL_MIN_DESIRED_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_right_desired_travel_heading_samples",
            acc.right_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_DESIRED_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_left_desired_travel_heading_samples",
            acc.left_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_DESIRED_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_backward_right_desired_travel_heading_samples",
            acc.backward_right_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_DESIRED_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_backward_left_desired_travel_heading_samples",
            acc.backward_left_desired_travel_heading_samples as f32,
            AIR_CONTROL_MIN_DIRECTIONAL_DESIRED_TRAVEL_HEADING_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "air_control_p95_desired_travel_heading_error",
            derived.p95_desired_travel_heading_error_degrees,
            AIR_CONTROL_MAX_P95_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_desired_travel_heading_error",
            acc.max_desired_travel_heading_error_degrees,
            AIR_CONTROL_MAX_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_pure_air_turn_sideways_samples",
            acc.pure_air_turn_sideways_body_travel_heading_error_values_degrees
                .len() as f32,
            AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_right_pure_air_turn_sideways_samples",
            acc.right_pure_air_turn_sideways_samples as f32,
            AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_least(
            "air_control_left_pure_air_turn_sideways_samples",
            acc.left_pure_air_turn_sideways_samples as f32,
            AIR_CONTROL_MIN_PURE_AIR_TURN_SIDEWAYS_SAMPLES as f32,
            "samples",
        ),
        EvalCheck::at_most(
            "air_control_p95_pure_air_turn_sideways_body_travel_heading_error",
            derived.p95_pure_air_turn_sideways_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_P95_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_pure_air_turn_sideways_body_travel_heading_error",
            acc.max_pure_air_turn_sideways_body_travel_heading_error_degrees,
            AIR_CONTROL_MAX_LATERAL_BODY_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_p95_pure_air_turn_sideways_desired_travel_heading_error",
            derived.p95_pure_air_turn_sideways_desired_travel_heading_error_degrees,
            AIR_CONTROL_MAX_P95_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_pure_air_turn_sideways_desired_travel_heading_error",
            acc.max_pure_air_turn_sideways_desired_travel_heading_error_degrees,
            AIR_CONTROL_MAX_DESIRED_TRAVEL_HEADING_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_body_yaw_error_step",
            acc.max_body_yaw_error_step_degrees,
            AIR_CONTROL_MAX_BODY_YAW_ERROR_STEP_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_body_yaw_oscillation_count",
            acc.body_yaw_oscillation_count as f32,
            AIR_CONTROL_MAX_BODY_YAW_OSCILLATIONS,
            "sign changes",
        ),
        EvalCheck::at_least(
            "air_control_right_body_bank_response",
            acc.max_right_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "air_control_left_body_bank_response",
            acc.max_left_body_bank_degrees,
            AIR_CONTROL_MIN_BODY_BANK_RESPONSE_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_max_body_roll_step",
            acc.max_body_roll_step_degrees,
            AIR_CONTROL_MAX_BODY_ROLL_STEP_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_orbit_yaw_offset",
            acc.max_abs_camera_yaw_offset_degrees,
            AIR_CONTROL_MAX_CAMERA_YAW_OFFSET_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_rotation_delta",
            acc.max_camera_rotation_delta_degrees,
            AIR_CONTROL_MAX_CAMERA_ROTATION_DELTA_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_view_yaw_drift",
            acc.max_camera_view_yaw_drift_degrees,
            AIR_CONTROL_MAX_CAMERA_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_avg_camera_follow_direction_error",
            derived.avg_camera_follow_direction_error_degrees,
            AIR_CONTROL_MAX_AVG_CAMERA_FOLLOW_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_p95_camera_follow_direction_error",
            derived.p95_camera_follow_direction_error_degrees,
            AIR_CONTROL_MAX_P95_CAMERA_FOLLOW_ERROR_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "air_control_camera_world_yaw_drift",
            acc.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ),
    ]);
}

fn append_pose_state_coverage_checks(checks: &mut Vec<EvalCheck>, acc: &EvalAccumulator) {
    checks.extend([
        EvalCheck::at_least(
            "pose_state_grounded_idle_samples",
            acc.pose_grounded_idle_samples as f32,
            POSE_STATE_MIN_IDLE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_grounded_walk_samples",
            acc.pose_grounded_walk_samples as f32,
            POSE_STATE_MIN_WALK_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_grounded_run_samples",
            acc.pose_grounded_run_samples as f32,
            POSE_STATE_MIN_RUN_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_walk_stride_foot_travel",
            acc.max_grounded_walk_stride_foot_travel_m,
            GROUNDED_WALK_STRIDE_MIN_FOOT_TRAVEL_M,
            "m",
        ),
        EvalCheck::at_least(
            "pose_state_run_stride_foot_travel",
            acc.max_grounded_run_stride_foot_travel_m,
            GROUNDED_RUN_STRIDE_MIN_FOOT_TRAVEL_M,
            "m",
        ),
        EvalCheck::at_least(
            "pose_state_walk_stride_leg_opposition",
            acc.max_grounded_walk_stride_leg_opposition_degrees,
            GROUNDED_WALK_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "pose_state_run_stride_leg_opposition",
            acc.max_grounded_run_stride_leg_opposition_degrees,
            GROUNDED_RUN_STRIDE_MIN_LEG_OPPOSITION_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "pose_state_authored_grounded_idle_clip_samples",
            acc.authored_grounded_idle_clip_samples as f32,
            POSE_STATE_MIN_IDLE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_authored_grounded_walk_clip_samples",
            acc.authored_grounded_walk_clip_samples as f32,
            POSE_STATE_MIN_WALK_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_authored_grounded_run_clip_samples",
            acc.authored_grounded_run_clip_samples as f32,
            POSE_STATE_MIN_RUN_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_launching_samples",
            acc.pose_launching_samples as f32,
            POSE_STATE_MIN_LAUNCH_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_falling_samples",
            acc.pose_falling_samples as f32,
            POSE_STATE_MIN_FALLING_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_authored_fall_clip_samples",
            acc.authored_fall_clip_samples as f32,
            POSE_STATE_MIN_FALLING_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_gliding_samples",
            acc.pose_gliding_samples as f32,
            POSE_STATE_MIN_GLIDING_POSE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_air_turn_samples",
            acc.pose_air_turn_samples as f32,
            POSE_STATE_MIN_AIR_TURN_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_right_air_turn_samples",
            acc.right_pose_air_turn_samples as f32,
            POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_left_air_turn_samples",
            acc.left_pose_air_turn_samples as f32,
            POSE_STATE_MIN_DIRECTIONAL_AIR_TURN_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_air_brake_samples",
            acc.pose_air_brake_samples as f32,
            POSE_STATE_MIN_AIR_BRAKE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_diving_samples",
            acc.pose_diving_samples as f32,
            POSE_STATE_MIN_DIVING_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_gliding_dive_samples",
            acc.gliding_dive_samples as f32,
            POSE_STATE_MIN_GLIDING_DIVE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_landing_anticipation_samples",
            acc.pose_landing_anticipation_samples as f32,
            POSE_STATE_MIN_LANDING_POSE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_landing_recovery_samples",
            acc.pose_landing_recovery_samples as f32,
            POSE_STATE_MIN_LANDING_POSE_SAMPLES,
            "samples",
        ),
        EvalCheck::at_least(
            "pose_state_landing_crouch",
            acc.max_pose_landing_crouch_m,
            LANDING_MIN_POSE_CROUCH_M,
            "m",
        ),
        EvalCheck::at_least(
            "pose_state_landing_foot_forward",
            acc.max_pose_landing_foot_forward_m,
            LANDING_MIN_POSE_FOOT_FORWARD_M,
            "m",
        ),
        EvalCheck::at_least(
            "pose_state_landing_foot_split",
            acc.max_pose_landing_foot_split_m,
            LANDING_MIN_POSE_FOOT_SPLIT_M,
            "m",
        ),
        EvalCheck::at_least(
            "pose_state_landing_flare",
            acc.max_pose_landing_flare_degrees,
            LANDING_MIN_POSE_FLARE_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "pose_state_landing_recovery_flip",
            acc.max_pose_landing_recovery_flip_degrees,
            LANDING_MIN_POSE_RECOVERY_FLIP_DEGREES,
            "deg",
        ),
        EvalCheck::at_least(
            "pose_state_scarf_stream",
            acc.max_pose_scarf_stream_m,
            MIN_POSE_SCARF_STREAM_M,
            "m",
        ),
        EvalCheck::at_least(
            "pose_state_scarf_lateral_sway",
            acc.max_pose_scarf_lateral_sway_m,
            MIN_POSE_SCARF_LATERAL_SWAY_M,
            "m",
        ),
        EvalCheck::at_least(
            "pose_state_scarf_tail_flex",
            acc.max_pose_scarf_tail_flex_degrees,
            MIN_POSE_SCARF_TAIL_FLEX_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "pose_state_unreadable_key_pose_samples",
            acc.unreadable_key_pose_samples as f32,
            0.0,
            "samples",
        ),
        EvalCheck::at_most(
            "pose_state_key_pose_transition_grace_samples",
            acc.key_pose_transition_grace_samples as f32,
            POSE_STATE_MAX_KEY_POSE_TRANSITION_GRACE_SAMPLES as f32,
            "samples",
        ),
    ]);
}

fn append_camera_strafe_checks(checks: &mut Vec<EvalCheck>, acc: &EvalAccumulator) {
    checks.extend([
        EvalCheck::at_least(
            "camera_strafe_right_lateral_response",
            acc.max_right_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_least(
            "camera_strafe_left_lateral_response",
            acc.max_left_lateral_response_mps,
            CAMERA_STRAFE_MIN_LATERAL_RESPONSE_MPS,
            "m/s",
        ),
        EvalCheck::at_most(
            "camera_strafe_view_yaw_drift",
            acc.max_camera_view_yaw_drift_degrees,
            CAMERA_STRAFE_MAX_VIEW_YAW_DRIFT_DEGREES,
            "deg",
        ),
        EvalCheck::at_most(
            "camera_strafe_world_yaw_drift",
            acc.max_camera_world_yaw_drift_degrees,
            MOVEMENT_ONLY_MAX_CAMERA_WORLD_YAW_DRIFT_DEGREES,
            "deg",
        ),
    ]);
}
