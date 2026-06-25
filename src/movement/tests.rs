use bevy::prelude::*;

use super::{FlightController, FlightMode, FlightState};

mod integration;
mod math;
mod orientation;

fn default_state() -> FlightState {
    FlightState::new(
        Vec3::new(0.0, 1.2, 0.0),
        Vec3::ZERO,
        FlightController::default(),
    )
}

fn gliding_controller(bank_degrees: f32) -> FlightController {
    FlightController {
        mode: FlightMode::Gliding,
        bank_degrees,
        ..default()
    }
}
