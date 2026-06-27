use bevy::prelude::Vec2;

use crate::{
    animation::MIN_KEY_POSE_READABILITY_SCORE,
    eval::thresholds::{
        AIR_CONTROL_RESPONSE_THRESHOLD_MPS, AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES,
    },
    movement::FlightMode,
};

use super::{EvalAccumulator, EvalSample};

const BODY_YAW_INTENT_AXIS_EPSILON: f32 = 0.05;
const BODY_YAW_INTENT_CHANGE_DOT: f32 = 0.98;

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.max_altitude_m = accumulator.max_altitude_m.max(sample.altitude_m);
    accumulator.min_altitude_m = accumulator.min_altitude_m.min(sample.altitude_m);
    accumulator.max_speed_mps = accumulator.max_speed_mps.max(sample.speed_mps);

    observe_grounded_visual_footing(accumulator, sample);
    observe_body_heading(accumulator, sample);
    observe_body_roll(accumulator, sample);
    observe_desired_heading_alignment(accumulator, sample);
    observe_lateral_response(accumulator, sample);
    observe_body_travel_heading_alignment(accumulator, sample);
    observe_desired_travel_heading_alignment(accumulator, sample);
    observe_air_brake(accumulator, sample);
    observe_pose_readability(accumulator, sample);
    observe_authored_clip_coverage(accumulator, sample);
    observe_pose_intent_counts(accumulator, sample);
    observe_mode_counts(accumulator, sample);
}

fn observe_grounded_visual_footing(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if sample.mode == FlightMode::Grounded.label() && sample.visual_foot_gap_m.is_finite() {
        accumulator.max_grounded_visual_foot_gap_m = accumulator
            .max_grounded_visual_foot_gap_m
            .max(sample.visual_foot_gap_m.abs());
    }
}

fn observe_body_heading(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if !sample.desired_body_yaw_error_degrees.is_finite() {
        accumulator.previous_desired_body_yaw_error_degrees = None;
        accumulator.previous_body_yaw_intent_axis = None;
        accumulator.previous_body_yaw_error_sign = None;
        return;
    }

    let current_intent_axis = body_yaw_intent_axis(sample);
    let intent_changed = body_yaw_intent_changed(
        accumulator.previous_body_yaw_intent_axis,
        current_intent_axis,
    );
    if intent_changed {
        accumulator.previous_desired_body_yaw_error_degrees = None;
        accumulator.previous_body_yaw_error_sign = None;
    }
    accumulator.previous_body_yaw_intent_axis = current_intent_axis;

    let heading_error = sample.desired_body_heading_error_degrees;
    accumulator.desired_body_heading_error_sum_degrees += heading_error;
    accumulator.desired_body_heading_samples += 1;
    accumulator
        .desired_body_heading_error_values_degrees
        .push(heading_error);
    accumulator.max_desired_body_heading_error_degrees = accumulator
        .max_desired_body_heading_error_degrees
        .max(heading_error);

    if let Some(previous_error) = accumulator.previous_desired_body_yaw_error_degrees {
        accumulator.max_body_yaw_error_step_degrees = accumulator
            .max_body_yaw_error_step_degrees
            .max((sample.desired_body_yaw_error_degrees - previous_error).abs());
    }
    accumulator.previous_desired_body_yaw_error_degrees =
        Some(sample.desired_body_yaw_error_degrees);

    if sample.desired_body_yaw_error_degrees.abs() >= AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES {
        let sign = sample.desired_body_yaw_error_degrees.signum();
        if accumulator
            .previous_body_yaw_error_sign
            .is_some_and(|previous| previous != sign)
        {
            accumulator.body_yaw_oscillation_count += 1;
        }
        accumulator.previous_body_yaw_error_sign = Some(sign);
    }
}

fn body_yaw_intent_axis(sample: &EvalSample) -> Option<Vec2> {
    let axis = Vec2::new(
        sample.movement_input_lateral_axis,
        sample.movement_input_forward_axis,
    );
    (axis.length_squared() >= BODY_YAW_INTENT_AXIS_EPSILON.powi(2)).then(|| axis.normalize())
}

fn body_yaw_intent_changed(previous: Option<Vec2>, current: Option<Vec2>) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => previous.dot(current) < BODY_YAW_INTENT_CHANGE_DOT,
        (Some(_), None) | (None, Some(_)) => true,
        (None, None) => false,
    }
}

fn observe_body_roll(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if !sample.body_roll_degrees.is_finite() || sample.mode == FlightMode::Grounded.label() {
        accumulator.previous_body_roll_degrees = None;
        return;
    }

    if let Some(previous_roll) = accumulator.previous_body_roll_degrees {
        accumulator.max_body_roll_step_degrees = accumulator
            .max_body_roll_step_degrees
            .max((sample.body_roll_degrees - previous_roll).abs());
    }
    accumulator.previous_body_roll_degrees = Some(sample.body_roll_degrees);

    match sample.movement_input_lateral_axis.signum() {
        sign if sign > 0.0 => {
            accumulator.max_right_body_bank_degrees = accumulator
                .max_right_body_bank_degrees
                .max((-sample.body_roll_degrees).max(0.0));
        }
        sign if sign < 0.0 => {
            accumulator.max_left_body_bank_degrees = accumulator
                .max_left_body_bank_degrees
                .max(sample.body_roll_degrees.max(0.0));
        }
        _ => {}
    }
}

fn observe_desired_heading_alignment(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if sample.desired_heading_alignment_mps.is_finite() {
        accumulator.max_desired_heading_alignment_mps = accumulator
            .max_desired_heading_alignment_mps
            .max(sample.desired_heading_alignment_mps);
    }
}

fn observe_lateral_response(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    let lateral_axis_active =
        sample.lateral_input_active || sample.movement_input_lateral_axis.abs() > f32::EPSILON;
    if !lateral_axis_active {
        return;
    }

    if accumulator.first_lateral_input_time_secs.is_none() {
        accumulator.first_lateral_input_time_secs = Some(sample.time_secs);
    }
    accumulator.max_lateral_response_mps = accumulator
        .max_lateral_response_mps
        .max(sample.lateral_response_mps);
    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
        && accumulator.first_lateral_response_time_secs.is_none()
    {
        accumulator.first_lateral_response_time_secs = Some(sample.time_secs);
    }

    match sample.movement_input_lateral_axis.signum() {
        sign if sign > 0.0 => observe_right_lateral_response(accumulator, sample),
        sign if sign < 0.0 => observe_left_lateral_response(accumulator, sample),
        _ => {}
    }

    if sample.movement_input_forward_axis < 0.0 {
        if accumulator.first_backward_lateral_input_time_secs.is_none() {
            accumulator.first_backward_lateral_input_time_secs = Some(sample.time_secs);
        }
        accumulator.max_backward_lateral_response_mps = accumulator
            .max_backward_lateral_response_mps
            .max(sample.lateral_response_mps);
        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
            && accumulator
                .first_backward_lateral_response_time_secs
                .is_none()
        {
            accumulator.first_backward_lateral_response_time_secs = Some(sample.time_secs);
        }
    }
}

fn observe_right_lateral_response(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if accumulator.first_right_lateral_input_time_secs.is_none() {
        accumulator.first_right_lateral_input_time_secs = Some(sample.time_secs);
    }
    accumulator.max_right_lateral_response_mps = accumulator
        .max_right_lateral_response_mps
        .max(sample.lateral_response_mps);
    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
        && accumulator.first_right_lateral_response_time_secs.is_none()
    {
        accumulator.first_right_lateral_response_time_secs = Some(sample.time_secs);
    }
    if sample.movement_input_forward_axis < 0.0 {
        if accumulator
            .first_backward_right_lateral_input_time_secs
            .is_none()
        {
            accumulator.first_backward_right_lateral_input_time_secs = Some(sample.time_secs);
        }
        accumulator.max_backward_right_lateral_response_mps = accumulator
            .max_backward_right_lateral_response_mps
            .max(sample.lateral_response_mps);
        if let Some(rear_response_mps) = backward_diagonal_rear_response_mps(sample) {
            accumulator.max_backward_right_rear_response_mps = accumulator
                .max_backward_right_rear_response_mps
                .max(rear_response_mps);
        }
        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
            && accumulator
                .first_backward_right_lateral_response_time_secs
                .is_none()
        {
            accumulator.first_backward_right_lateral_response_time_secs = Some(sample.time_secs);
        }
    }
}

fn observe_left_lateral_response(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if accumulator.first_left_lateral_input_time_secs.is_none() {
        accumulator.first_left_lateral_input_time_secs = Some(sample.time_secs);
    }
    accumulator.max_left_lateral_response_mps = accumulator
        .max_left_lateral_response_mps
        .max(sample.lateral_response_mps);
    if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
        && accumulator.first_left_lateral_response_time_secs.is_none()
    {
        accumulator.first_left_lateral_response_time_secs = Some(sample.time_secs);
    }
    if sample.movement_input_forward_axis < 0.0 {
        if accumulator
            .first_backward_left_lateral_input_time_secs
            .is_none()
        {
            accumulator.first_backward_left_lateral_input_time_secs = Some(sample.time_secs);
        }
        accumulator.max_backward_left_lateral_response_mps = accumulator
            .max_backward_left_lateral_response_mps
            .max(sample.lateral_response_mps);
        if let Some(rear_response_mps) = backward_diagonal_rear_response_mps(sample) {
            accumulator.max_backward_left_rear_response_mps = accumulator
                .max_backward_left_rear_response_mps
                .max(rear_response_mps);
        }
        if sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
            && accumulator
                .first_backward_left_lateral_response_time_secs
                .is_none()
        {
            accumulator.first_backward_left_lateral_response_time_secs = Some(sample.time_secs);
        }
    }
}

fn observe_air_brake(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if sample.movement_input_forward_axis < 0.0 && sample.mode != FlightMode::Grounded.label() {
        let planar_speed = Vec2::new(sample.velocity[0], sample.velocity[2]).length();
        if accumulator.backward_air_control_start_speed_mps.is_none() {
            accumulator.backward_air_control_start_speed_mps = Some(sample.speed_mps);
        }
        if accumulator
            .backward_air_control_start_planar_speed_mps
            .is_none()
        {
            accumulator.backward_air_control_start_planar_speed_mps = Some(planar_speed);
        }
        let min_speed = accumulator
            .min_backward_air_control_speed_mps
            .map_or(sample.speed_mps, |speed| speed.min(sample.speed_mps));
        accumulator.min_backward_air_control_speed_mps = Some(min_speed);
        let min_planar_speed = accumulator
            .min_backward_air_control_planar_speed_mps
            .map_or(planar_speed, |speed| speed.min(planar_speed));
        accumulator.min_backward_air_control_planar_speed_mps = Some(min_planar_speed);
        if let Some(start_speed) = accumulator.backward_air_control_start_speed_mps {
            accumulator.max_air_brake_speed_drop_mps = accumulator
                .max_air_brake_speed_drop_mps
                .max(start_speed - min_speed);
        }
        if let Some(start_planar_speed) = accumulator.backward_air_control_start_planar_speed_mps {
            accumulator.max_air_brake_planar_speed_drop_mps = accumulator
                .max_air_brake_planar_speed_drop_mps
                .max(start_planar_speed - min_planar_speed);
        }
    } else if accumulator.backward_air_control_start_speed_mps.is_some()
        && sample.movement_input_forward_axis > 0.0
        && sample.desired_heading_alignment_mps.is_finite()
    {
        accumulator.max_post_brake_forward_alignment_mps = accumulator
            .max_post_brake_forward_alignment_mps
            .max(sample.desired_heading_alignment_mps);
    }
}

fn observe_body_travel_heading_alignment(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    if !body_travel_heading_alignment_sample(sample) {
        return;
    }

    let Some(lateral_response_time) = lateral_response_time_for_sample(accumulator, sample) else {
        return;
    };
    if sample.time_secs < lateral_response_time {
        return;
    }

    accumulator
        .lateral_body_travel_heading_error_values_degrees
        .push(sample.body_travel_heading_error_degrees);
    accumulator.max_lateral_body_travel_heading_error_degrees = accumulator
        .max_lateral_body_travel_heading_error_degrees
        .max(sample.body_travel_heading_error_degrees);
    if sample.movement_input_lateral_axis > 0.0 {
        accumulator.right_lateral_body_travel_heading_samples += 1;
    } else if sample.movement_input_lateral_axis < 0.0 {
        accumulator.left_lateral_body_travel_heading_samples += 1;
    }

    if sample.movement_input_forward_axis >= 0.0 {
        return;
    }

    if backward_diagonal_response_time_for_sample(accumulator, sample)
        .is_some_and(|time_secs| sample.time_secs >= time_secs)
    {
        accumulator
            .backward_diagonal_body_travel_heading_error_values_degrees
            .push(sample.body_travel_heading_error_degrees);
        accumulator.max_backward_diagonal_body_travel_heading_error_degrees = accumulator
            .max_backward_diagonal_body_travel_heading_error_degrees
            .max(sample.body_travel_heading_error_degrees);
        if sample.movement_input_lateral_axis > 0.0 {
            accumulator.backward_right_diagonal_body_travel_heading_samples += 1;
        } else if sample.movement_input_lateral_axis < 0.0 {
            accumulator.backward_left_diagonal_body_travel_heading_samples += 1;
        }
    }
}

fn observe_desired_travel_heading_alignment(
    accumulator: &mut EvalAccumulator,
    sample: &EvalSample,
) {
    if !desired_travel_heading_alignment_sample(sample) {
        return;
    }

    let Some(lateral_response_time) = lateral_response_time_for_sample(accumulator, sample) else {
        return;
    };
    if sample.time_secs < lateral_response_time {
        return;
    }

    accumulator
        .desired_travel_heading_error_values_degrees
        .push(sample.desired_travel_heading_error_degrees);
    accumulator.max_desired_travel_heading_error_degrees = accumulator
        .max_desired_travel_heading_error_degrees
        .max(sample.desired_travel_heading_error_degrees);
    if sample.movement_input_lateral_axis > 0.0 {
        accumulator.right_desired_travel_heading_samples += 1;
    } else if sample.movement_input_lateral_axis < 0.0 {
        accumulator.left_desired_travel_heading_samples += 1;
    }
    if sample.movement_input_forward_axis < 0.0
        && backward_diagonal_response_time_for_sample(accumulator, sample)
            .is_some_and(|time_secs| sample.time_secs >= time_secs)
    {
        if sample.movement_input_lateral_axis > 0.0 {
            accumulator.backward_right_desired_travel_heading_samples += 1;
        } else if sample.movement_input_lateral_axis < 0.0 {
            accumulator.backward_left_desired_travel_heading_samples += 1;
        }
    }
}

fn observe_pose_readability(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.max_pose_torso_pitch_degrees = accumulator
        .max_pose_torso_pitch_degrees
        .max(sample.pose_torso_pitch_degrees);
    accumulator.max_pose_arm_spread_degrees = accumulator
        .max_pose_arm_spread_degrees
        .max(sample.pose_arm_spread_degrees);
    accumulator.max_pose_leg_tuck_degrees = accumulator
        .max_pose_leg_tuck_degrees
        .max(sample.pose_leg_tuck_degrees);
    accumulator.max_pose_lateral_lean_degrees = accumulator
        .max_pose_lateral_lean_degrees
        .max(sample.pose_lateral_lean_degrees);
    if sample.movement_input_lateral_axis > 0.25 {
        accumulator.max_right_pose_lateral_lean_degrees = accumulator
            .max_right_pose_lateral_lean_degrees
            .max((-sample.pose_signed_lateral_lean_degrees).max(0.0));
    } else if sample.movement_input_lateral_axis < -0.25 {
        accumulator.max_left_pose_lateral_lean_degrees = accumulator
            .max_left_pose_lateral_lean_degrees
            .max(sample.pose_signed_lateral_lean_degrees.max(0.0));
    }
    match sample.pose_intent_label {
        "grounded_walk" => {
            accumulator.max_grounded_walk_stride_foot_travel_m = accumulator
                .max_grounded_walk_stride_foot_travel_m
                .max(sample.pose_grounded_stride_foot_travel_m);
            accumulator.max_grounded_walk_stride_leg_opposition_degrees = accumulator
                .max_grounded_walk_stride_leg_opposition_degrees
                .max(sample.pose_grounded_stride_leg_opposition_degrees);
        }
        "grounded_run" => {
            accumulator.max_grounded_run_stride_foot_travel_m = accumulator
                .max_grounded_run_stride_foot_travel_m
                .max(sample.pose_grounded_stride_foot_travel_m);
            accumulator.max_grounded_run_stride_leg_opposition_degrees = accumulator
                .max_grounded_run_stride_leg_opposition_degrees
                .max(sample.pose_grounded_stride_leg_opposition_degrees);
        }
        _ => {}
    }
    accumulator.max_pose_landing_crouch_m = accumulator
        .max_pose_landing_crouch_m
        .max(sample.pose_landing_crouch_m);
    accumulator.max_pose_landing_foot_forward_m = accumulator
        .max_pose_landing_foot_forward_m
        .max(sample.pose_landing_foot_forward_m);
    if sample.pose_intent_label == "landing_anticipation" {
        accumulator.max_pose_landing_flare_degrees = accumulator
            .max_pose_landing_flare_degrees
            .max(sample.pose_torso_pitch_degrees);
    }
    if sample.pose_intent_label == "landing_recovery" {
        accumulator.max_pose_landing_recovery_flip_degrees = accumulator
            .max_pose_landing_recovery_flip_degrees
            .max(sample.pose_landing_recovery_flip_degrees);
    }
    accumulator.max_pose_wing_airflow_strength = accumulator
        .max_pose_wing_airflow_strength
        .max(sample.pose_wing_airflow_strength);
    accumulator.max_authored_glider_response_degrees = accumulator
        .max_authored_glider_response_degrees
        .max(sample.authored_glider_response_degrees);
    accumulator.max_authored_glider_motion_m = accumulator
        .max_authored_glider_motion_m
        .max(sample.authored_glider_motion_m);
    if sample.mode == "gliding" && sample.pose_intent_label == "diving" {
        accumulator.gliding_dive_samples += 1;
        accumulator.max_authored_glider_dive_response_degrees = accumulator
            .max_authored_glider_dive_response_degrees
            .max(sample.authored_glider_response_degrees);
        accumulator.max_authored_glider_dive_motion_m = accumulator
            .max_authored_glider_dive_motion_m
            .max(sample.authored_glider_motion_m);
    }
    accumulator.max_visible_pose_part_count = accumulator
        .max_visible_pose_part_count
        .max(sample.visible_pose_part_count);
    if key_pose_intent_label(sample.pose_intent_label) {
        accumulator.max_key_pose_readability_score = accumulator
            .max_key_pose_readability_score
            .max(sample.key_pose_readability_score);
        let min_score = accumulator
            .min_key_pose_readability_score
            .map_or(sample.key_pose_readability_score, |current| {
                current.min(sample.key_pose_readability_score)
            });
        accumulator.min_key_pose_readability_score = Some(min_score);

        if sample.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE {
            accumulator.unreadable_key_pose_samples += 1;
        }
        if sample.key_pose_transition_grace {
            accumulator.key_pose_transition_grace_samples += 1;
        }
        let has_pose_temporal_metrics = sample.max_pose_part_rotation_delta_degrees.is_finite()
            && sample.max_pose_part_translation_delta_m.is_finite();
        if has_pose_temporal_metrics {
            accumulator.pose_temporal_stability_samples += 1;
            accumulator.max_pose_part_rotation_delta_degrees = accumulator
                .max_pose_part_rotation_delta_degrees
                .max(sample.max_pose_part_rotation_delta_degrees);
            accumulator.max_pose_part_translation_delta_m = accumulator
                .max_pose_part_translation_delta_m
                .max(sample.max_pose_part_translation_delta_m);
            if landing_pose_intent_label(sample.pose_intent_label) {
                accumulator.landing_pose_temporal_stability_samples += 1;
                accumulator.max_landing_pose_part_rotation_delta_degrees = accumulator
                    .max_landing_pose_part_rotation_delta_degrees
                    .max(sample.max_pose_part_rotation_delta_degrees);
                accumulator.max_landing_pose_part_translation_delta_m = accumulator
                    .max_landing_pose_part_translation_delta_m
                    .max(sample.max_pose_part_translation_delta_m);
            }
        }
    }
}

fn observe_authored_clip_coverage(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.max_authored_transition_duration_ms = accumulator
        .max_authored_transition_duration_ms
        .max(sample.authored_transition_duration_ms);

    if matches!(
        sample.pose_intent_label,
        "grounded_walk" | "grounded_run" | "grounded_stride"
    ) && sample.authored_player_count > 0
        && sample.authored_player_current_clip_label == "jog"
        && sample.authored_player_desired_clip_label == "jog"
    {
        accumulator.authored_jog_clip_samples += 1;
    }

    if !key_pose_intent_label(sample.pose_intent_label) {
        return;
    }

    if sample.authored_player_count == 0 {
        accumulator.authored_clip_mismatch_samples += 1;
        return;
    }

    let current_clip = sample.authored_player_current_clip_label;
    if current_clip != sample.authored_player_desired_clip_label || current_clip == "none" {
        accumulator.authored_clip_mismatch_samples += 1;
        return;
    }

    accumulator.authored_clip_match_samples += 1;
    match (sample.pose_intent_label, current_clip) {
        ("air_turn", "bank_left") if sample.movement_input_lateral_axis < -0.25 => {
            accumulator.authored_bank_left_clip_samples += 1;
        }
        ("air_turn", "bank_right") if sample.movement_input_lateral_axis > 0.25 => {
            accumulator.authored_bank_right_clip_samples += 1;
        }
        ("falling", "fall") => accumulator.authored_fall_clip_samples += 1,
        ("diving", "dive") => accumulator.authored_dive_clip_samples += 1,
        ("air_brake", "air_brake") => accumulator.authored_air_brake_clip_samples += 1,
        ("landing_anticipation" | "landing_recovery", "land") => {
            accumulator.authored_land_clip_samples += 1;
        }
        _ => {}
    }
}

fn observe_mode_counts(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    match sample.mode {
        "gliding" => accumulator.gliding_samples += 1,
        "launching" => accumulator.launching_samples += 1,
        "grounded" => accumulator.grounded_samples += 1,
        _ => {}
    }
}

fn observe_pose_intent_counts(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    match sample.pose_intent_label {
        "grounded_walk" => accumulator.pose_grounded_walk_samples += 1,
        "grounded_run" => accumulator.pose_grounded_run_samples += 1,
        _ => {}
    }

    if key_pose_intent_label(sample.pose_intent_label)
        && sample.key_pose_readability_score < MIN_KEY_POSE_READABILITY_SCORE
    {
        return;
    }

    match sample.pose_intent_label {
        "launching" => accumulator.pose_launching_samples += 1,
        "falling" => accumulator.pose_falling_samples += 1,
        "gliding" => accumulator.pose_gliding_samples += 1,
        "air_turn" => {
            accumulator.pose_air_turn_samples += 1;
            if sample.movement_input_lateral_axis > 0.25 {
                accumulator.right_pose_air_turn_samples += 1;
            } else if sample.movement_input_lateral_axis < -0.25 {
                accumulator.left_pose_air_turn_samples += 1;
            }
        }
        "diving" => accumulator.pose_diving_samples += 1,
        "air_brake" => accumulator.pose_air_brake_samples += 1,
        "landing_anticipation" => accumulator.pose_landing_anticipation_samples += 1,
        "landing_recovery" => accumulator.pose_landing_recovery_samples += 1,
        _ => {}
    }
}

fn key_pose_intent_label(label: &str) -> bool {
    matches!(
        label,
        "launching"
            | "falling"
            | "gliding"
            | "air_turn"
            | "diving"
            | "air_brake"
            | "landing_anticipation"
            | "landing_recovery"
    )
}

fn landing_pose_intent_label(label: &str) -> bool {
    matches!(label, "landing_anticipation" | "landing_recovery")
}

fn body_travel_heading_alignment_sample(sample: &EvalSample) -> bool {
    matches!(sample.mode, "airborne" | "gliding")
        && sample.lateral_input_active
        && sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
        && sample.body_travel_heading_error_degrees.is_finite()
}

fn desired_travel_heading_alignment_sample(sample: &EvalSample) -> bool {
    matches!(sample.mode, "airborne" | "gliding")
        && sample.lateral_input_active
        && sample.lateral_response_mps >= AIR_CONTROL_RESPONSE_THRESHOLD_MPS
        && sample.desired_travel_heading_error_degrees.is_finite()
}

fn lateral_response_time_for_sample(
    accumulator: &EvalAccumulator,
    sample: &EvalSample,
) -> Option<f32> {
    match sample.movement_input_lateral_axis.signum() {
        sign if sign > 0.0 => accumulator.first_right_lateral_response_time_secs,
        sign if sign < 0.0 => accumulator.first_left_lateral_response_time_secs,
        _ => accumulator.first_lateral_response_time_secs,
    }
}

fn backward_diagonal_response_time_for_sample(
    accumulator: &EvalAccumulator,
    sample: &EvalSample,
) -> Option<f32> {
    match sample.movement_input_lateral_axis.signum() {
        sign if sign > 0.0 => accumulator.first_backward_right_lateral_response_time_secs,
        sign if sign < 0.0 => accumulator.first_backward_left_lateral_response_time_secs,
        _ => accumulator.first_backward_lateral_response_time_secs,
    }
}

fn backward_diagonal_rear_response_mps(sample: &EvalSample) -> Option<f32> {
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
