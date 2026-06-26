use bevy::prelude::*;

use super::{
    GROUND_EPSILON,
    math::{horizontal, horizontal_or, smoothing_factor},
    orientation::desired_air_steering_direction,
    types::{Facing, FlightInput, FlightMode, FlightState, FlightTuning},
};

pub fn step_flight(
    mut state: FlightState,
    input: FlightInput,
    facing: Facing,
    tuning: &FlightTuning,
    dt: f32,
) -> FlightState {
    let dt = dt.max(0.0);
    state.controller.launch_cooldown_remaining =
        (state.controller.launch_cooldown_remaining - dt).max(0.0);
    state.controller.launch_timer = (state.controller.launch_timer - dt).max(0.0);
    state.controller.step_landing_recovery(dt);

    let touching_ground = is_grounded(state.position, tuning);
    let started_grounded = touching_ground && state.controller.mode == FlightMode::Grounded;
    if started_grounded {
        state.controller.launch_available = true;
    }

    if input.launch
        && started_grounded
        && state.controller.launch_available
        && state.controller.launch_cooldown_remaining <= 0.0
    {
        state.velocity.y = tuning.launch_speed;
        state.velocity += facing.forward * tuning.launch_forward_bonus;
        state.controller.launch_available = false;
        state.controller.launch_cooldown_remaining = tuning.launch_cooldown;
        state.controller.launch_timer = tuning.launch_duration;
        state.controller.clear_landing_recovery();
    }

    let launching = state.controller.launch_timer > 0.0;
    let gliding = input.glide && !started_grounded && !input.dive && !launching;
    let mut acceleration = Vec3::ZERO;

    if started_grounded && !launching {
        if input.forward {
            acceleration += facing.forward * tuning.ground_accel;
        }
        if input.backward {
            acceleration -= facing.forward * tuning.ground_backward_accel;
        }
        if input.left {
            acceleration -= facing.right * tuning.ground_lateral_accel;
        }
        if input.right {
            acceleration += facing.right * tuning.ground_lateral_accel;
        }
    } else if gliding {
        if let Some(desired_direction) = desired_air_steering_direction(input, facing) {
            acceleration += directional_air_steering_acceleration(
                state.velocity,
                desired_direction,
                tuning.glide_steer_accel,
                tuning.glide_counter_steer_accel,
                tuning.air_steer_min_speed,
            );
        }
        if input.forward {
            acceleration += facing.forward * tuning.glide_forward_accel;
        }
        if input.backward {
            apply_backward_air_control(
                &mut state.velocity,
                facing.forward,
                tuning.glide_brake_accel,
                tuning.backward_accel,
                tuning.max_backward_speed,
                !input.has_lateral_axis(),
                dt,
            );
            state.velocity.x *= tuning.glide_brake_drag.powf(dt);
            state.velocity.z *= tuning.glide_brake_drag.powf(dt);
        }
        if input.left {
            acceleration -= facing.right * tuning.glide_lateral_accel;
        }
        if input.right {
            acceleration += facing.right * tuning.glide_lateral_accel;
        }
    } else {
        if let Some(desired_direction) = desired_air_steering_direction(input, facing) {
            acceleration += directional_air_steering_acceleration(
                state.velocity,
                desired_direction,
                tuning.air_steer_accel,
                tuning.air_counter_steer_accel,
                tuning.air_steer_min_speed,
            );
        }
        if input.forward {
            acceleration += facing.forward * tuning.forward_accel;
        }
        if input.backward {
            apply_backward_air_control(
                &mut state.velocity,
                facing.forward,
                tuning.air_brake_accel,
                tuning.backward_accel,
                tuning.max_backward_speed,
                !input.has_lateral_axis(),
                dt,
            );
        }
        if input.left {
            acceleration -= facing.right * tuning.lateral_accel;
        }
        if input.right {
            acceleration += facing.right * tuning.lateral_accel;
        }
    }

    if input.dive {
        acceleration.y -= tuning.dive_accel;
    }

    if started_grounded && !launching && !input.dive {
        state.velocity.y = state.velocity.y.max(0.0);
    } else {
        let gravity_scale = if gliding {
            tuning.glide_gravity_scale
        } else {
            1.0
        };
        acceleration.y -= tuning.gravity * gravity_scale;
    }

    state.velocity += acceleration * dt;
    state.velocity *= tuning.drag.powf(dt);
    if started_grounded && !launching {
        let ground_friction = tuning.ground_friction.clamp(0.0, 1.0).powf(dt);
        state.velocity.x *= ground_friction;
        state.velocity.z *= ground_friction;
        if horizontal(state.velocity).length_squared() < 0.01 {
            state.velocity.x = 0.0;
            state.velocity.z = 0.0;
        }
    }

    if gliding {
        state.velocity.y = state.velocity.y.max(-tuning.glide_max_fall_speed);
    }

    let max_horizontal_speed = if started_grounded && !launching {
        tuning.ground_max_horizontal_speed
    } else {
        tuning.max_horizontal_speed
    };
    state.velocity = clamp_velocity(state.velocity, tuning, max_horizontal_speed);
    state.position += state.velocity * dt;

    if state.position.y <= tuning.floor_y + GROUND_EPSILON && state.velocity.y <= 0.0 {
        let impact_speed_mps = (-state.velocity.y).max(0.0);
        state.position.y = tuning.floor_y;
        if !started_grounded {
            state.controller.record_landing_impact(impact_speed_mps);
        }
        state.velocity.y = state.velocity.y.max(0.0);
    }

    let grounded = is_grounded(state.position, tuning);
    if grounded {
        state.controller.launch_timer = 0.0;
        state.controller.launch_available = true;
    }

    state.controller.mode = if grounded {
        FlightMode::Grounded
    } else if state.controller.launch_timer > 0.0 {
        FlightMode::Launching
    } else if gliding {
        FlightMode::Gliding
    } else {
        FlightMode::Airborne
    };
    let target_bank_degrees = target_bank_degrees(input, state.controller.mode, tuning);
    state.controller.bank_degrees += (target_bank_degrees - state.controller.bank_degrees)
        * smoothing_factor(tuning.bank_response_rate, dt);
    if target_bank_degrees == 0.0 && state.controller.bank_degrees.abs() < 0.01 {
        state.controller.bank_degrees = 0.0;
    }

    state
}

fn directional_air_steering_acceleration(
    velocity: Vec3,
    desired_direction: Vec3,
    steer_accel: f32,
    counter_steer_accel: f32,
    min_target_speed: f32,
) -> Vec3 {
    let horizontal_velocity = horizontal(velocity);
    let target_speed = horizontal_velocity.length().max(min_target_speed.max(0.0));
    let desired_direction = horizontal_or(desired_direction, Vec3::Z);
    let target_velocity = desired_direction * target_speed;
    let correction = target_velocity - horizontal_velocity;
    if correction.length_squared() <= 0.0001 {
        Vec3::ZERO
    } else {
        let reversing = horizontal_velocity.dot(desired_direction) < -0.1;
        let accel = if reversing {
            counter_steer_accel
        } else {
            steer_accel
        };
        correction.normalize() * accel.max(0.0)
    }
}

fn target_bank_degrees(input: FlightInput, mode: FlightMode, tuning: &FlightTuning) -> f32 {
    if mode == FlightMode::Grounded {
        return 0.0;
    }

    -input.planar_axis().x * tuning.max_bank_degrees
}

fn clamp_velocity(mut velocity: Vec3, tuning: &FlightTuning, max_horizontal_speed: f32) -> Vec3 {
    let horizontal_velocity = horizontal(velocity);
    let horizontal_speed = horizontal_velocity.length();
    let max_horizontal_speed = max_horizontal_speed.max(0.0);

    if horizontal_speed > max_horizontal_speed {
        let horizontal_velocity = horizontal_velocity.normalize() * max_horizontal_speed;
        velocity.x = horizontal_velocity.x;
        velocity.z = horizontal_velocity.z;
    }

    velocity.y = velocity
        .y
        .clamp(-tuning.max_fall_speed, tuning.launch_speed);
    velocity
}

fn apply_backward_air_control(
    velocity: &mut Vec3,
    forward: Vec3,
    brake_accel: f32,
    reverse_accel: f32,
    max_backward_speed: f32,
    brake_sideways_momentum: bool,
    dt: f32,
) {
    let forward = horizontal_or(forward, Vec3::Z);
    let horizontal_velocity = horizontal(*velocity);
    let horizontal_speed = horizontal_velocity.length();
    let backward_alignment = horizontal_velocity.dot(-forward);
    let sideways_speed = (horizontal_speed.powi(2) - backward_alignment.max(0.0).powi(2))
        .max(0.0)
        .sqrt();

    if brake_sideways_momentum
        && horizontal_speed > 0.01
        && (backward_alignment <= 0.1 || sideways_speed > 0.75)
    {
        let reduction = horizontal_speed.min(brake_accel.max(0.0) * dt);
        let braking = horizontal_velocity / horizontal_speed * reduction;
        velocity.x -= braking.x;
        velocity.z -= braking.z;
        if horizontal_speed - reduction > 0.05 {
            return;
        }
    }

    let forward_speed = horizontal(*velocity).dot(forward);
    let max_backward_speed = max_backward_speed.max(0.0);
    if forward_speed > -max_backward_speed {
        let next_forward_speed =
            (forward_speed - reverse_accel.max(0.0) * dt).max(-max_backward_speed);
        let delta = next_forward_speed - forward_speed;
        velocity.x += forward.x * delta;
        velocity.z += forward.z * delta;
    }
}

fn is_grounded(position: Vec3, tuning: &FlightTuning) -> bool {
    position.y <= tuning.floor_y + GROUND_EPSILON
}
