use crate::{camera::CameraInput, movement::FlightInput};
use bevy::prelude::*;

use super::{
    AIR_CONTROL_RESPONSE, BRANCH_RECOVERY_ROUTE, CAMERA_MOUSE_CONTROL, CAMERA_STRAFE_STABILITY,
    CAMERA_TURN_STABILITY, CAMERA_YAW_STABILITY, EvalScenario, GROUND_TAXI_CONTROL,
    ISLAND_LAUNCH_TO_LANDING, LONG_GLIDE_VISIBILITY, POSE_STATE_COVERAGE,
    TERRAIN_RIM_COLLISION_CONTACT, UPDRAFT_ROUTE, WORLD_COLLISION_CONTACT,
};

pub fn scripted_input(scenario: EvalScenario, frame: u32) -> FlightInput {
    let t = frame as f32 * scenario.fixed_dt;
    if matches!(scenario.name, CAMERA_MOUSE_CONTROL | CAMERA_YAW_STABILITY) {
        return FlightInput::default();
    }
    if scenario.name == CAMERA_STRAFE_STABILITY {
        return FlightInput {
            right: (0.15..=1.65).contains(&t),
            left: (1.75..=3.1).contains(&t),
            ..default()
        };
    }
    if scenario.name == AIR_CONTROL_RESPONSE {
        let dive = (5.75..=6.0).contains(&t);
        return FlightInput {
            forward: (0.05..=1.55).contains(&t) || (6.1..=6.45).contains(&t),
            right: (1.0..=2.45).contains(&t) || (4.0..=4.55).contains(&t),
            left: (2.65..=3.75).contains(&t) || (4.75..=5.3).contains(&t),
            backward: (4.0..=5.65).contains(&t),
            glide: t >= 0.45,
            dive,
            launch: frame == 1,
        };
    }
    if scenario.name == POSE_STATE_COVERAGE {
        let route_forward = (0.05..=7.05).contains(&t);
        let post_landing_stride = (8.45..=10.75).contains(&t);
        return FlightInput {
            forward: route_forward || post_landing_stride,
            right: (5.1..=5.35).contains(&t),
            left: (3.1..=4.2).contains(&t),
            backward: (4.65..=5.05).contains(&t),
            launch: frame == 1,
            glide: (1.15..=8.0).contains(&t),
            dive: (4.25..=4.45).contains(&t) || (5.8..=6.7).contains(&t),
        };
    }
    if scenario.name == GROUND_TAXI_CONTROL {
        return FlightInput {
            forward: (0.05..=1.95).contains(&t),
            right: (0.75..=1.65).contains(&t),
            backward: (2.2..=2.35).contains(&t),
            ..default()
        };
    }
    if scenario.name == WORLD_COLLISION_CONTACT {
        return FlightInput {
            backward: (0.05..=2.8).contains(&t),
            ..default()
        };
    }
    if scenario.name == TERRAIN_RIM_COLLISION_CONTACT {
        return FlightInput {
            forward: (0.05..=3.2).contains(&t),
            ..default()
        };
    }
    if scenario.name == UPDRAFT_ROUTE {
        return FlightInput {
            forward: t >= 0.05,
            right: (1.2..=2.2).contains(&t),
            left: (4.7..=5.0).contains(&t),
            glide: t >= 0.45,
            launch: frame == 1,
            ..default()
        };
    }
    if scenario.name == BRANCH_RECOVERY_ROUTE {
        let dive = (8.45..=10.9).contains(&t);
        return FlightInput {
            forward: (0.05..=10.55).contains(&t),
            backward: (10.75..=11.1).contains(&t),
            right: (1.2..=2.2).contains(&t) || (9.1..=9.75).contains(&t),
            left: (4.7..=5.0).contains(&t),
            glide: t >= 0.45,
            dive,
            launch: frame == 1,
        };
    }
    if scenario.name == LONG_GLIDE_VISIBILITY {
        return FlightInput {
            forward: t >= 0.05,
            right: (1.1..=2.25).contains(&t),
            left: (2.45..=2.7).contains(&t),
            glide: t >= 0.45,
            launch: frame == 1,
            ..default()
        };
    }
    if scenario.name == CAMERA_TURN_STABILITY {
        return FlightInput {
            forward: (0.05..=3.35).contains(&t),
            backward: (3.9..=5.1).contains(&t),
            left: (1.05..=1.65).contains(&t) || (2.2..=2.75).contains(&t),
            right: (1.65..=2.2).contains(&t) || (2.75..=3.35).contains(&t),
            glide: t >= 0.45,
            launch: frame == 1,
            ..default()
        };
    }

    let dive = match scenario.name {
        ISLAND_LAUNCH_TO_LANDING => (5.8..=6.7).contains(&t),
        _ => (6.2..=7.0).contains(&t),
    };
    let left = (3.1..=4.2).contains(&t);
    let right = (5.1..=5.35).contains(&t);

    let forward = if scenario.name == ISLAND_LAUNCH_TO_LANDING {
        (0.05..=7.05).contains(&t)
    } else {
        t >= 0.05
    };
    let backward = scenario.name == ISLAND_LAUNCH_TO_LANDING && (7.05..=7.55).contains(&t);

    FlightInput {
        forward,
        backward,
        left,
        right,
        glide: t >= 0.45,
        dive,
        launch: frame == 1,
    }
}

pub fn scripted_camera_input(scenario: EvalScenario, frame: u32) -> CameraInput {
    let t = frame as f32 * scenario.fixed_dt;

    let mouse_delta = match scenario.name {
        CAMERA_MOUSE_CONTROL if (0.2..=0.7).contains(&t) => Vec2::new(5.0, 0.0),
        CAMERA_MOUSE_CONTROL if (0.9..=1.3).contains(&t) => Vec2::new(0.0, -5.0),
        CAMERA_MOUSE_CONTROL if (1.5..=2.1).contains(&t) => Vec2::new(0.0, 8.0),
        CAMERA_MOUSE_CONTROL if (2.2..=2.55).contains(&t) => Vec2::new(0.0, -8.0),
        CAMERA_YAW_STABILITY if (0.2..=0.45).contains(&t) => Vec2::new(3.0, 0.0),
        _ => Vec2::ZERO,
    };

    CameraInput { mouse_delta }
}
