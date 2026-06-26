use bevy::prelude::Vec2;

use crate::{
    eval::thresholds::{
        AIR_CONTROL_RESPONSE_THRESHOLD_MPS, AIR_CONTROL_YAW_OSCILLATION_DEADZONE_DEGREES,
    },
    movement::FlightMode,
};

use super::{EvalAccumulator, EvalSample};

pub(super) fn observe(accumulator: &mut EvalAccumulator, sample: &EvalSample) {
    accumulator.max_altitude_m = accumulator.max_altitude_m.max(sample.altitude_m);
    accumulator.min_altitude_m = accumulator.min_altitude_m.min(sample.altitude_m);
    accumulator.max_speed_mps = accumulator.max_speed_mps.max(sample.speed_mps);

    observe_grounded_visual_footing(accumulator, sample);
    observe_body_heading(accumulator, sample);
    observe_body_roll(accumulator, sample);
    observe_desired_heading_alignment(accumulator, sample);
    observe_lateral_response(accumulator, sample);
    observe_air_brake(accumulator, sample);
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
        return;
    }

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
        "gliding" => accumulator.pose_gliding_samples += 1,
        "diving" => accumulator.pose_diving_samples += 1,
        "air_brake" => accumulator.pose_air_brake_samples += 1,
        "landing_anticipation" => accumulator.pose_landing_anticipation_samples += 1,
        _ => {}
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
