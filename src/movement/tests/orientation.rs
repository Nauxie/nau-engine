use bevy::prelude::*;

use super::super::{
    Facing, FlightInput, FlightTuning, body_heading_error_degrees, body_roll_degrees,
    desired_planar_movement_direction, face_flight_direction,
};
use super::gliding_controller;

#[test]
fn diagonal_glide_input_rotates_body_toward_camera_relative_heading() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let input = FlightInput {
        forward: true,
        right: true,
        glide: true,
        ..default()
    };
    let mut rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(facing.forward, Vec3::Y)
        .rotation;
    let desired_direction = desired_planar_movement_direction(input, facing)
        .expect("diagonal input has a movement direction");

    for _ in 0..12 {
        rotation = face_flight_direction(
            rotation,
            Vec3::new(0.0, -2.0, 34.0),
            input,
            facing,
            gliding_controller(0.0),
            &tuning,
            1.0 / 60.0,
        );
    }

    let heading_error = body_heading_error_degrees(rotation, desired_direction);
    assert!(
        heading_error < 8.0,
        "expected diagonal input to turn toward camera-relative heading, got {heading_error} deg"
    );
}

#[test]
fn backward_diagonal_glide_input_rotates_body_toward_rear_quadrant() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let input = FlightInput {
        backward: true,
        right: true,
        glide: true,
        ..default()
    };
    let mut rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(facing.forward, Vec3::Y)
        .rotation;
    let desired_direction = desired_planar_movement_direction(input, facing)
        .expect("backward-diagonal input has a movement direction");

    for _ in 0..18 {
        rotation = face_flight_direction(
            rotation,
            Vec3::new(0.0, -2.0, 28.0),
            input,
            facing,
            gliding_controller(0.0),
            &tuning,
            1.0 / 60.0,
        );
    }

    let heading_error = body_heading_error_degrees(rotation, desired_direction);
    assert!(
        heading_error < 18.0,
        "expected backward-diagonal input to turn toward rear quadrant, got {heading_error} deg"
    );
}

#[test]
fn flight_body_yaw_tracks_lateral_input_direction() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let input = FlightInput {
        right: true,
        glide: true,
        ..default()
    };
    let mut rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(facing.forward, Vec3::Y)
        .rotation;

    for _ in 0..30 {
        rotation = face_flight_direction(
            rotation,
            Vec3::new(0.0, -2.0, 34.0),
            input,
            facing,
            gliding_controller(0.0),
            &tuning,
            1.0 / 60.0,
        );
    }

    let heading_error = body_heading_error_degrees(rotation, facing.right);
    assert!(
        heading_error < 20.0,
        "expected body yaw to turn toward right input, got {heading_error} deg"
    );
}

#[test]
fn flight_body_yaw_reverses_lateral_air_input_quickly() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(facing.right, Vec3::Y)
        .rotation;
    let input = FlightInput {
        left: true,
        glide: true,
        ..default()
    };

    for _ in 0..6 {
        rotation = face_flight_direction(
            rotation,
            Vec3::new(26.0, -2.0, 18.0),
            input,
            facing,
            gliding_controller(0.0),
            &tuning,
            1.0 / 60.0,
        );
    }

    let heading_error = body_heading_error_degrees(rotation, -facing.right);
    assert!(
        heading_error < 35.0,
        "expected rapid body-yaw recovery after lateral reversal, got {heading_error} deg"
    );
}

#[test]
fn flight_body_yaw_limits_first_frame_reversal_spike() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(facing.right, Vec3::Y)
        .rotation;
    let input = FlightInput {
        left: true,
        glide: true,
        ..default()
    };

    let rotation = face_flight_direction(
        rotation,
        Vec3::new(24.0, -2.0, 5.0),
        input,
        facing,
        gliding_controller(0.0),
        &tuning,
        1.0 / 60.0,
    );

    let heading_error = body_heading_error_degrees(rotation, -facing.right);
    assert!(
        heading_error < 45.0,
        "expected first-frame lateral reversal to stay bounded, got {heading_error} deg"
    );
}

#[test]
fn body_roll_reports_smoothed_bank_without_heading_error() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let input = FlightInput {
        right: true,
        glide: true,
        ..default()
    };
    let rotation = face_flight_direction(
        Quat::IDENTITY,
        Vec3::new(0.0, -2.0, 24.0),
        input,
        facing,
        gliding_controller(-12.0),
        &tuning,
        1.0 / 60.0,
    );

    assert!(body_roll_degrees(rotation) < -1.0);
    assert!(body_heading_error_degrees(rotation, facing.right) < 45.0);
}
