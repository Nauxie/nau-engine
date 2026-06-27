use bevy::prelude::*;

use super::super::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning, GROUND_EPSILON,
    body_heading_error_degrees, desired_planar_movement_direction,
    desired_planar_travel_heading_error_degrees, face_flight_direction, math::horizontal,
    step_flight,
};
use super::default_state;

#[test]
fn launch_only_fires_from_ground() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let input = FlightInput {
        launch: true,
        ..default()
    };

    let launched = step_flight(default_state(), input, facing, &tuning, 1.0 / 60.0);
    assert_eq!(launched.controller.mode, FlightMode::Launching);
    assert!(!launched.controller.launch_available);
    assert!(launched.velocity.y > 35.0);

    let relaunched = step_flight(launched, input, facing, &tuning, 1.0 / 60.0);
    assert!(relaunched.velocity.y < tuning.launch_speed);
}

#[test]
fn grounded_forward_input_moves_at_walkable_speed() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::NEG_Z, Vec3::X);
    let input = FlightInput {
        forward: true,
        ..default()
    };
    let mut state = default_state();

    for _ in 0..60 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    assert_eq!(state.controller.mode, FlightMode::Grounded);
    assert!((state.position.y - tuning.floor_y).abs() <= GROUND_EPSILON);
    assert!(state.position.z < -5.0);
    assert!(state.velocity.length() >= 7.0);
}

#[test]
fn grounded_friction_stops_released_input() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::NEG_Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, tuning.floor_y, 0.0),
        Vec3::new(0.0, 0.0, -tuning.ground_max_horizontal_speed),
        FlightController::default(),
    );

    for _ in 0..90 {
        state = step_flight(state, FlightInput::default(), facing, &tuning, 1.0 / 60.0);
    }

    assert_eq!(state.controller.mode, FlightMode::Grounded);
    assert!(Vec2::new(state.velocity.x, state.velocity.z).length() < 0.5);
}

#[test]
fn glide_does_not_create_altitude() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 40.0, 0.0),
        Vec3::new(0.0, 0.0, 28.0),
        FlightController {
            mode: FlightMode::Airborne,
            launch_available: false,
            ..default()
        },
    );
    let start_y = state.position.y;
    let input = FlightInput {
        forward: true,
        glide: true,
        ..default()
    };

    for _ in 0..600 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    assert!(state.position.y < start_y);
    assert!(state.velocity.y <= 0.0);
}

#[test]
fn glide_clamps_fall_speed() {
    let tuning = FlightTuning::default();
    let state = FlightState::new(
        Vec3::new(0.0, 40.0, 0.0),
        Vec3::new(0.0, -40.0, 20.0),
        FlightController {
            mode: FlightMode::Airborne,
            launch_available: false,
            ..default()
        },
    );

    let next = step_flight(
        state,
        FlightInput {
            glide: true,
            ..default()
        },
        Facing::new(Vec3::Z, Vec3::X),
        &tuning,
        1.0 / 60.0,
    );

    assert!(next.velocity.y >= -tuning.glide_max_fall_speed);
}

#[test]
fn glider_dive_stays_deployed_and_descends_faster_than_plain_glide() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let start = FlightState::new(
        Vec3::new(0.0, 70.0, 0.0),
        Vec3::new(0.0, -2.0, 28.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let plain_input = FlightInput {
        glide: true,
        ..default()
    };
    let dive_input = FlightInput {
        glide: true,
        dive: true,
        ..default()
    };
    let mut plain = start;
    let mut diving = start;
    let mut previous_dive_y = diving.position.y;

    for _ in 0..45 {
        plain = step_flight(plain, plain_input, facing, &tuning, 1.0 / 60.0);
        diving = step_flight(diving, dive_input, facing, &tuning, 1.0 / 60.0);
        assert!(diving.position.y <= previous_dive_y);
        previous_dive_y = diving.position.y;
    }

    assert_eq!(diving.controller.mode, FlightMode::Gliding);
    assert!(diving.position.y < plain.position.y - 4.0);
    assert!(diving.velocity.y < -tuning.glide_max_fall_speed - 1.0);
}

#[test]
fn airborne_backward_input_brakes_forward_motion() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 30.0, 0.0),
        Vec3::new(0.0, 8.0, 34.0),
        FlightController {
            mode: FlightMode::Airborne,
            launch_available: false,
            ..default()
        },
    );
    let input = FlightInput {
        backward: true,
        ..default()
    };

    for _ in 0..60 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    let forward_speed = horizontal(state.velocity).dot(facing.forward);
    assert!(
        forward_speed < 3.0,
        "expected backward input to brake strongly, got {forward_speed}"
    );
    assert!(forward_speed >= -tuning.max_backward_speed - 0.5);
}

#[test]
fn gliding_backward_input_slows_without_runaway_reverse() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 45.0, 0.0),
        Vec3::new(0.0, -2.0, 34.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let input = FlightInput {
        backward: true,
        glide: true,
        ..default()
    };

    for _ in 0..60 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    let forward_speed = horizontal(state.velocity).dot(facing.forward);
    assert!(
        forward_speed < 5.0,
        "expected glide brake to bleed speed, got {forward_speed}"
    );
    assert!(forward_speed >= -tuning.max_backward_speed - 0.5);
}

#[test]
fn gliding_backward_input_turns_travel_toward_rear_heading() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 45.0, 0.0),
        Vec3::new(0.0, -2.0, 34.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let input = FlightInput {
        backward: true,
        glide: true,
        ..default()
    };

    for _ in 0..45 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    let desired_direction = desired_planar_movement_direction(input, facing).unwrap();
    let desired_travel_error =
        desired_planar_travel_heading_error_degrees(state.velocity, desired_direction, 3.0);
    let rear_speed = horizontal(state.velocity).dot(-facing.forward);

    assert!(
        rear_speed > 3.0,
        "expected backward glide input to create rearward travel, got {rear_speed}"
    );
    assert!(
        desired_travel_error < 18.0,
        "expected backward glide travel to turn toward rear heading, got {desired_travel_error} deg"
    );
}

#[test]
fn gliding_backward_input_brakes_sideways_momentum() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 45.0, 0.0),
        Vec3::new(26.0, -2.0, 4.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let input = FlightInput {
        backward: true,
        glide: true,
        ..default()
    };

    for _ in 0..30 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    let side_speed = horizontal(state.velocity).dot(facing.right);
    let horizontal_speed = horizontal(state.velocity).length();
    assert!(
        side_speed.abs() < 5.0,
        "expected air brake to bleed sideways drift, got {side_speed}"
    );
    assert!(
        horizontal_speed < 12.0,
        "expected air brake to shed planar speed, got {horizontal_speed}"
    );
}

#[test]
fn backward_diagonal_glide_input_steers_toward_rear_quadrant() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 45.0, 0.0),
        Vec3::new(18.0, -2.0, 26.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let input = FlightInput {
        backward: true,
        left: true,
        glide: true,
        ..default()
    };

    for _ in 0..45 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    let left_speed = horizontal(state.velocity).dot(-facing.right);
    let forward_speed = horizontal(state.velocity).dot(facing.forward);
    let desired_direction = desired_planar_movement_direction(input, facing).unwrap();
    let desired_travel_error =
        desired_planar_travel_heading_error_degrees(state.velocity, desired_direction, 6.0);
    assert!(
        left_speed > 10.0,
        "expected back-left input to build leftward control, got {left_speed}"
    );
    assert!(
        forward_speed < 6.0,
        "expected back-left input to brake forward drift, got {forward_speed}"
    );
    assert!(
        desired_travel_error < 24.0,
        "expected back-left travel to align with desired rear quadrant, got {desired_travel_error} deg"
    );
}

#[test]
fn lateral_air_input_steers_velocity_toward_desired_plane() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 45.0, 0.0),
        Vec3::new(0.0, -2.0, 34.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let input = FlightInput {
        right: true,
        glide: true,
        ..default()
    };

    for _ in 0..30 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    let side_speed = horizontal(state.velocity).dot(facing.right);
    let forward_speed = horizontal(state.velocity).dot(facing.forward);
    let desired_direction = desired_planar_movement_direction(input, facing).unwrap();
    let desired_travel_error =
        desired_planar_travel_heading_error_degrees(state.velocity, desired_direction, 6.0);
    assert!(
        side_speed > 24.0,
        "expected right input to build meaningful planar side speed, got {side_speed}"
    );
    assert!(
        forward_speed < 14.0,
        "expected steering to rotate velocity away from pure forward drift, got {forward_speed}"
    );
    assert!(
        desired_travel_error < 18.0,
        "expected right input to align travel with desired side heading, got {desired_travel_error} deg"
    );
}

#[test]
fn lateral_glide_input_turns_body_and_travel_together() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 45.0, 0.0),
        Vec3::new(0.0, -2.0, 34.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let mut rotation = Transform::from_translation(Vec3::ZERO)
        .looking_to(facing.forward, Vec3::Y)
        .rotation;
    let input = FlightInput {
        right: true,
        glide: true,
        ..default()
    };

    for _ in 0..30 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
        rotation = face_flight_direction(
            rotation,
            state.velocity,
            input,
            facing,
            state.controller,
            &tuning,
            1.0 / 60.0,
        );
    }

    let desired_direction = desired_planar_movement_direction(input, facing).unwrap();
    let desired_travel_error =
        desired_planar_travel_heading_error_degrees(state.velocity, desired_direction, 6.0);
    let body_desired_error = body_heading_error_degrees(rotation, desired_direction);
    let body_travel_error = body_heading_error_degrees(rotation, horizontal(state.velocity));

    assert!(
        desired_travel_error < 12.0,
        "expected lateral glide travel to turn toward input, got {desired_travel_error} deg"
    );
    assert!(
        body_desired_error < 12.0,
        "expected body yaw to face lateral input, got {body_desired_error} deg"
    );
    assert!(
        body_travel_error < 8.0,
        "expected body and travel headings to stay coupled, got {body_travel_error} deg"
    );
}

#[test]
fn lateral_air_input_reverses_side_velocity_before_it_feels_stuck() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let mut state = FlightState::new(
        Vec3::new(0.0, 45.0, 0.0),
        Vec3::new(26.0, -2.0, 18.0),
        FlightController {
            mode: FlightMode::Gliding,
            launch_available: false,
            ..default()
        },
    );
    let input = FlightInput {
        left: true,
        glide: true,
        ..default()
    };

    for _ in 0..12 {
        state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    }

    let left_response = horizontal(state.velocity).dot(-facing.right);
    assert!(
        left_response > 8.0,
        "expected left reversal to recover promptly, got {left_response}"
    );
}

#[test]
fn lateral_air_bank_smooths_toward_input() {
    let tuning = FlightTuning::default();
    let facing = Facing::new(Vec3::Z, Vec3::X);
    let input = FlightInput {
        right: true,
        glide: true,
        ..default()
    };
    let state = FlightState::new(
        Vec3::new(0.0, 20.0, 0.0),
        Vec3::new(0.0, -2.0, 24.0),
        FlightController::default(),
    );

    let first = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
    let second = step_flight(first, input, facing, &tuning, 1.0 / 60.0);

    assert!(first.controller.bank_degrees < -1.0);
    assert!(first.controller.bank_degrees > -5.0);
    assert!(second.controller.bank_degrees < first.controller.bank_degrees);
}

#[test]
fn floor_collision_clears_downward_velocity() {
    let tuning = FlightTuning::default();
    let state = FlightState::new(
        Vec3::new(0.0, tuning.floor_y + 0.2, 0.0),
        Vec3::new(0.0, -20.0, 0.0),
        FlightController {
            mode: FlightMode::Airborne,
            launch_available: false,
            ..default()
        },
    );

    let next = step_flight(
        state,
        FlightInput::default(),
        Facing::new(Vec3::Z, Vec3::X),
        &tuning,
        0.2,
    );

    assert_eq!(next.position.y, tuning.floor_y);
    assert!(next.velocity.y >= 0.0);
    assert_eq!(next.controller.mode, FlightMode::Grounded);
    assert!(next.controller.landing_recovery_timer > 0.0);
    assert!(next.controller.landing_impact_speed_mps > 20.0);
}

#[test]
fn near_floor_airborne_collision_records_landing_recovery() {
    let tuning = FlightTuning::default();
    let state = FlightState::new(
        Vec3::new(0.0, tuning.floor_y + GROUND_EPSILON * 0.5, 0.0),
        Vec3::new(0.0, -12.0, 0.0),
        FlightController {
            mode: FlightMode::Airborne,
            launch_available: false,
            ..default()
        },
    );

    let next = step_flight(
        state,
        FlightInput::default(),
        Facing::new(Vec3::Z, Vec3::X),
        &tuning,
        1.0 / 60.0,
    );

    assert_eq!(next.position.y, tuning.floor_y);
    assert_eq!(next.controller.mode, FlightMode::Grounded);
    assert!(next.controller.landing_recovery_timer > 0.0);
    assert!(next.controller.landing_impact_speed_mps > 12.0);
}

#[test]
fn landing_recovery_timer_expires_after_touchdown() {
    let tuning = FlightTuning::default();
    let mut state = FlightState::new(
        Vec3::new(0.0, tuning.floor_y + 0.2, 0.0),
        Vec3::new(0.0, -20.0, 0.0),
        FlightController {
            mode: FlightMode::Airborne,
            launch_available: false,
            ..default()
        },
    );

    state = step_flight(
        state,
        FlightInput::default(),
        Facing::new(Vec3::Z, Vec3::X),
        &tuning,
        0.2,
    );
    assert!(state.controller.landing_recovery_timer > 0.0);

    for _ in 0..60 {
        state = step_flight(
            state,
            FlightInput::default(),
            Facing::new(Vec3::Z, Vec3::X),
            &tuning,
            1.0 / 60.0,
        );
    }

    assert_eq!(state.controller.landing_recovery_timer, 0.0);
    assert_eq!(state.controller.landing_impact_speed_mps, 0.0);
}
