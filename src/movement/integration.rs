use bevy::prelude::*;

use super::{
    GROUND_EPSILON,
    math::{horizontal, horizontal_or, smoothing_factor},
    orientation::desired_air_steering_direction,
    types::{
        Facing, FlightInput, FlightMode, FlightState, FlightTuning,
        LAUNCH_MAX_HORIZONTAL_SPEED_MPS, LAUNCH_MAX_UPWARD_SPEED_MPS,
    },
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
        state.velocity.y = tuning.launch_speed.min(LAUNCH_MAX_UPWARD_SPEED_MPS);
        state.velocity += facing.forward * tuning.launch_forward_bonus;
        state.velocity = clamp_horizontal_velocity(state.velocity, LAUNCH_MAX_HORIZONTAL_SPEED_MPS);
        state.controller.launch_available = false;
        state.controller.launch_cooldown_remaining = tuning.launch_cooldown;
        state.controller.launch_timer = tuning.launch_duration;
        state.controller.clear_landing_recovery();
    }

    let launching = state.controller.launch_timer > 0.0;
    let gliding = input.glide && !started_grounded && !launching;
    let air_steering_direction = (!started_grounded && !launching)
        .then(|| desired_air_steering_direction(input, facing))
        .flatten();
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
        if let Some(desired_direction) = air_steering_direction {
            if input_adds_air_steering_acceleration(input) {
                acceleration += directional_air_steering_acceleration(
                    state.velocity,
                    desired_direction,
                    tuning.glide_steer_accel,
                    tuning.glide_counter_steer_accel,
                    tuning.air_steer_min_speed,
                );
            }
            if input.has_lateral_axis() {
                acceleration +=
                    desired_planar_thrust(desired_direction, input, tuning.glide_lateral_accel);
            }
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
    } else {
        if let Some(desired_direction) = air_steering_direction {
            if input_adds_air_steering_acceleration(input) {
                acceleration += directional_air_steering_acceleration(
                    state.velocity,
                    desired_direction,
                    tuning.air_steer_accel,
                    tuning.air_counter_steer_accel,
                    tuning.air_steer_min_speed,
                );
            }
            if input.has_lateral_axis() {
                acceleration +=
                    desired_planar_thrust(desired_direction, input, tuning.lateral_accel);
            }
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
    if let Some(desired_direction) =
        air_steering_direction.filter(|_| input.has_lateral_axis() || input.backward)
    {
        let orthogonal_damping = if gliding {
            tuning.glide_input_orthogonal_damping
        } else {
            tuning.air_input_orthogonal_damping
        };
        damp_horizontal_velocity_orthogonal_to_input(
            &mut state.velocity,
            desired_direction,
            orthogonal_damping,
            dt,
        );
        let (turn_rate, counter_turn_rate) = if gliding {
            (
                tuning.glide_velocity_turn_rate_degrees,
                tuning.glide_velocity_counter_turn_rate_degrees,
            )
        } else {
            (
                tuning.air_velocity_turn_rate_degrees,
                tuning.air_velocity_counter_turn_rate_degrees,
            )
        };
        state.velocity = rotate_horizontal_velocity_toward(
            state.velocity,
            desired_direction,
            turn_rate,
            counter_turn_rate,
            dt,
        );
    }
    if started_grounded && !launching {
        let ground_friction = tuning.ground_friction.clamp(0.0, 1.0).powf(dt);
        state.velocity.x *= ground_friction;
        state.velocity.z *= ground_friction;
        if horizontal(state.velocity).length_squared() < 0.01 {
            state.velocity.x = 0.0;
            state.velocity.z = 0.0;
        }
    }

    if gliding && !input.dive {
        state.velocity.y = state.velocity.y.max(-tuning.glide_max_fall_speed);
    }

    let max_horizontal_speed = if started_grounded && !launching {
        tuning.ground_max_horizontal_speed
    } else if launching {
        LAUNCH_MAX_HORIZONTAL_SPEED_MPS
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

fn desired_planar_thrust(desired_direction: Vec3, input: FlightInput, accel: f32) -> Vec3 {
    desired_direction * input.planar_axis().length().clamp(0.0, 1.0) * accel.max(0.0)
}

fn input_adds_air_steering_acceleration(input: FlightInput) -> bool {
    input.forward || input.has_lateral_axis()
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

fn damp_horizontal_velocity_orthogonal_to_input(
    velocity: &mut Vec3,
    desired_direction: Vec3,
    damping_rate: f32,
    dt: f32,
) {
    let horizontal_velocity = horizontal(*velocity);
    if horizontal_velocity.length_squared() <= 0.0001 {
        return;
    }

    let desired_direction = horizontal_or(desired_direction, Vec3::Z);
    let aligned_velocity = desired_direction * horizontal_velocity.dot(desired_direction);
    let orthogonal_velocity = horizontal_velocity - aligned_velocity;
    if orthogonal_velocity.length_squared() <= 0.0001 {
        return;
    }

    let damping = smoothing_factor(damping_rate.max(0.0), dt);
    velocity.x -= orthogonal_velocity.x * damping;
    velocity.z -= orthogonal_velocity.z * damping;
}

fn rotate_horizontal_velocity_toward(
    mut velocity: Vec3,
    desired_direction: Vec3,
    turn_rate_degrees: f32,
    counter_turn_rate_degrees: f32,
    dt: f32,
) -> Vec3 {
    let horizontal_velocity = horizontal(velocity);
    let speed = horizontal_velocity.length();
    if speed <= 0.05 {
        return velocity;
    }

    let current_direction = horizontal_velocity / speed;
    let desired_direction = horizontal_or(desired_direction, Vec3::Z);
    let signed_angle = current_direction
        .cross(desired_direction)
        .y
        .atan2(current_direction.dot(desired_direction).clamp(-1.0, 1.0));
    if signed_angle.abs() <= 0.0001 {
        return velocity;
    }

    let reversing = current_direction.dot(desired_direction) < -0.1;
    let max_step = if reversing {
        counter_turn_rate_degrees
    } else {
        turn_rate_degrees
    }
    .max(0.0)
    .to_radians()
        * dt.max(0.0);
    if max_step <= 0.0 {
        return velocity;
    }

    let step = signed_angle.clamp(-max_step, max_step);
    let rotated = Quat::from_rotation_y(step) * current_direction * speed;
    velocity.x = rotated.x;
    velocity.z = rotated.z;
    velocity
}

fn target_bank_degrees(input: FlightInput, mode: FlightMode, tuning: &FlightTuning) -> f32 {
    if mode == FlightMode::Grounded {
        return 0.0;
    }

    -input.planar_axis().x * tuning.max_bank_degrees
}

fn clamp_velocity(mut velocity: Vec3, tuning: &FlightTuning, max_horizontal_speed: f32) -> Vec3 {
    velocity = clamp_horizontal_velocity(velocity, max_horizontal_speed);
    velocity.y = velocity.y.max(-tuning.max_fall_speed);
    velocity
}

fn clamp_horizontal_velocity(mut velocity: Vec3, max_horizontal_speed: f32) -> Vec3 {
    let horizontal_velocity = horizontal(velocity);
    let horizontal_speed = horizontal_velocity.length();
    let max_horizontal_speed = max_horizontal_speed.max(0.0);

    if horizontal_speed > max_horizontal_speed {
        let horizontal_velocity = horizontal_velocity.normalize() * max_horizontal_speed;
        velocity.x = horizontal_velocity.x;
        velocity.z = horizontal_velocity.z;
    }

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
