mod integration;
mod math;
mod orientation;
mod types;

#[cfg(test)]
mod tests;

const GROUND_EPSILON: f32 = 0.05;

pub use integration::step_flight;
pub use math::smoothing_factor;
pub use orientation::{
    body_forward, body_heading_error_degrees, body_roll_degrees, body_yaw_error_degrees,
    desired_heading_alignment_speed, desired_planar_movement_direction,
    desired_planar_travel_heading_error_degrees, face_flight_direction, face_horizontal_velocity,
    lateral_response_speed,
};
pub use types::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning,
    LAUNCH_MAX_HORIZONTAL_SPEED_MPS, LAUNCH_MAX_UPWARD_SPEED_MPS, Velocity,
    landing_recovery_strength,
};
