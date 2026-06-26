use bevy::prelude::*;

use super::{
    math::{horizontal, horizontal_or, smoothing_factor},
    types::{Facing, FlightController, FlightInput, FlightMode, FlightTuning},
};

pub fn desired_planar_movement_direction(input: FlightInput, facing: Facing) -> Option<Vec3> {
    let axis = input.planar_axis();
    if axis.length_squared() <= f32::EPSILON {
        return None;
    }

    let direction = facing.right * axis.x + facing.forward * axis.y;
    let direction = horizontal(direction);
    if direction.length_squared() > 0.0001 {
        Some(direction.normalize())
    } else {
        None
    }
}

pub fn face_flight_direction(
    current: Quat,
    velocity: Vec3,
    input: FlightInput,
    facing: Facing,
    controller: FlightController,
    tuning: &FlightTuning,
    dt: f32,
) -> Quat {
    let desired_direction = if controller.mode == FlightMode::Grounded {
        None
    } else {
        desired_air_body_direction(input, facing)
    };
    let target_direction = desired_direction.or_else(|| {
        let horizontal_velocity = horizontal(velocity);
        (horizontal_velocity.length_squared() > 0.1).then(|| horizontal_velocity.normalize())
    });
    let Some(target_direction) = target_direction else {
        return current;
    };

    let target = Transform::from_translation(Vec3::ZERO)
        .looking_to(target_direction, Vec3::Y)
        .rotation
        * Quat::from_rotation_z(controller.bank_degrees.to_radians());
    let yaw_error = body_heading_error_degrees(current, target_direction);
    let planar_input_weight = input.planar_axis().length().clamp(0.0, 1.0);
    let turn_rate = tuning.turn_rate
        * (1.0 + yaw_error / 180.0 * planar_input_weight * tuning.input_turn_rate_boost);
    current.slerp(target, smoothing_factor(turn_rate, dt))
}

pub fn face_horizontal_velocity(current: Quat, velocity: Vec3, turn_rate: f32, dt: f32) -> Quat {
    let horizontal_velocity = horizontal(velocity);
    if horizontal_velocity.length_squared() <= 0.1 {
        return current;
    }

    let target = Transform::from_translation(Vec3::ZERO)
        .looking_to(horizontal_velocity.normalize(), Vec3::Y)
        .rotation;
    current.slerp(target, smoothing_factor(turn_rate, dt))
}

pub fn body_heading_error_degrees(rotation: Quat, desired_direction: Vec3) -> f32 {
    body_yaw_error_degrees(rotation, desired_direction).abs()
}

pub fn body_yaw_error_degrees(rotation: Quat, desired_direction: Vec3) -> f32 {
    let forward = body_forward(rotation);
    let desired = horizontal_or(desired_direction, Vec3::Z);
    forward
        .cross(desired)
        .y
        .atan2(forward.dot(desired).clamp(-1.0, 1.0))
        .to_degrees()
}

pub fn body_forward(rotation: Quat) -> Vec3 {
    horizontal_or(rotation * Vec3::NEG_Z, Vec3::Z)
}

pub fn body_roll_degrees(rotation: Quat) -> f32 {
    let level_rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(body_forward(rotation), Vec3::Y)
        .rotation;
    let local_roll = level_rotation.inverse() * rotation;
    let (_, _, roll) = local_roll.to_euler(EulerRot::XYZ);
    roll.to_degrees()
}

pub fn desired_heading_alignment_speed(velocity: Vec3, desired_direction: Vec3) -> f32 {
    horizontal(velocity).dot(horizontal_or(desired_direction, Vec3::Z))
}

pub fn lateral_response_speed(velocity: Vec3, input: FlightInput, facing: Facing) -> f32 {
    let lateral = input.planar_axis().x;
    if lateral.abs() <= f32::EPSILON {
        return 0.0;
    }

    horizontal(velocity).dot(facing.right * lateral.signum())
}

fn desired_air_body_direction(input: FlightInput, facing: Facing) -> Option<Vec3> {
    desired_planar_movement_direction(input, facing)
}

pub(super) fn desired_air_steering_direction(input: FlightInput, facing: Facing) -> Option<Vec3> {
    if !input.forward && !input.left && !input.right {
        return None;
    }

    desired_planar_movement_direction(input, facing)
}
