use bevy::prelude::*;

const GROUND_EPSILON: f32 = 0.05;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Component, Clone, Copy, Debug)]
pub struct FlightController {
    pub mode: FlightMode,
    pub launch_cooldown_remaining: f32,
    pub launch_timer: f32,
    pub launch_available: bool,
    pub bank_degrees: f32,
}

impl Default for FlightController {
    fn default() -> Self {
        Self {
            mode: FlightMode::Grounded,
            launch_cooldown_remaining: 0.0,
            launch_timer: 0.0,
            launch_available: true,
            bank_degrees: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlightMode {
    Grounded,
    Airborne,
    Gliding,
    Launching,
}

impl FlightMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Grounded => "grounded",
            Self::Airborne => "airborne",
            Self::Gliding => "gliding",
            Self::Launching => "launching",
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct FlightTuning {
    pub ground_accel: f32,
    pub ground_backward_accel: f32,
    pub ground_lateral_accel: f32,
    pub ground_friction: f32,
    pub ground_max_horizontal_speed: f32,
    pub forward_accel: f32,
    pub backward_accel: f32,
    pub lateral_accel: f32,
    pub glide_forward_accel: f32,
    pub glide_lateral_accel: f32,
    pub glide_brake_drag: f32,
    pub air_brake_accel: f32,
    pub glide_brake_accel: f32,
    pub max_backward_speed: f32,
    pub dive_accel: f32,
    pub gravity: f32,
    pub glide_gravity_scale: f32,
    pub glide_max_fall_speed: f32,
    pub launch_speed: f32,
    pub launch_forward_bonus: f32,
    pub launch_cooldown: f32,
    pub launch_duration: f32,
    pub drag: f32,
    pub max_horizontal_speed: f32,
    pub max_fall_speed: f32,
    pub air_steer_accel: f32,
    pub glide_steer_accel: f32,
    pub air_counter_steer_accel: f32,
    pub glide_counter_steer_accel: f32,
    pub air_steer_min_speed: f32,
    pub max_bank_degrees: f32,
    pub bank_response_rate: f32,
    pub turn_rate: f32,
    pub input_turn_rate_boost: f32,
    pub floor_y: f32,
}

impl Default for FlightTuning {
    fn default() -> Self {
        Self {
            ground_accel: 34.0,
            ground_backward_accel: 22.0,
            ground_lateral_accel: 30.0,
            ground_friction: 0.08,
            ground_max_horizontal_speed: 11.0,
            forward_accel: 28.0,
            backward_accel: 10.0,
            lateral_accel: 16.0,
            glide_forward_accel: 12.0,
            glide_lateral_accel: 16.0,
            glide_brake_drag: 0.30,
            air_brake_accel: 46.0,
            glide_brake_accel: 66.0,
            max_backward_speed: 12.0,
            dive_accel: 32.0,
            gravity: 18.0,
            glide_gravity_scale: 0.28,
            glide_max_fall_speed: 7.5,
            launch_speed: 38.0,
            launch_forward_bonus: 12.0,
            launch_cooldown: 1.4,
            launch_duration: 0.35,
            drag: 0.82,
            max_horizontal_speed: 58.0,
            max_fall_speed: 70.0,
            air_steer_accel: 54.0,
            glide_steer_accel: 44.0,
            air_counter_steer_accel: 120.0,
            glide_counter_steer_accel: 340.0,
            air_steer_min_speed: 16.0,
            max_bank_degrees: 20.0,
            bank_response_rate: 6.0,
            turn_rate: 12.5,
            input_turn_rate_boost: 8.5,
            floor_y: 1.2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FlightInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub glide: bool,
    pub dive: bool,
    pub launch: bool,
}

impl FlightInput {
    pub fn planar_axis(self) -> Vec2 {
        Vec2::new(
            axis_value(self.left, self.right),
            axis_value(self.backward, self.forward),
        )
    }

    pub fn has_lateral_axis(self) -> bool {
        self.left != self.right
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Facing {
    pub forward: Vec3,
    pub right: Vec3,
}

impl Facing {
    pub fn new(forward: Vec3, right: Vec3) -> Self {
        Self {
            forward: horizontal_or(forward, Vec3::Z),
            right: horizontal_or(right, Vec3::X),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FlightState {
    pub position: Vec3,
    pub velocity: Vec3,
    pub controller: FlightController,
}

impl FlightState {
    pub fn new(position: Vec3, velocity: Vec3, controller: FlightController) -> Self {
        Self {
            position,
            velocity,
            controller,
        }
    }
}

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

    let was_grounded = is_grounded(state.position, tuning);
    if was_grounded {
        state.controller.launch_available = true;
    }

    if input.launch
        && was_grounded
        && state.controller.launch_available
        && state.controller.launch_cooldown_remaining <= 0.0
    {
        state.velocity.y = tuning.launch_speed;
        state.velocity += facing.forward * tuning.launch_forward_bonus;
        state.controller.launch_available = false;
        state.controller.launch_cooldown_remaining = tuning.launch_cooldown;
        state.controller.launch_timer = tuning.launch_duration;
    }

    let launching = state.controller.launch_timer > 0.0;
    let gliding = input.glide && !was_grounded && !input.dive && !launching;
    let mut acceleration = Vec3::ZERO;

    if was_grounded && !launching {
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

    if was_grounded && !launching && !input.dive {
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
    if was_grounded && !launching {
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

    let max_horizontal_speed = if was_grounded && !launching {
        tuning.ground_max_horizontal_speed
    } else {
        tuning.max_horizontal_speed
    };
    state.velocity = clamp_velocity(state.velocity, tuning, max_horizontal_speed);
    state.position += state.velocity * dt;

    if state.position.y <= tuning.floor_y + GROUND_EPSILON && state.velocity.y <= 0.0 {
        state.position.y = tuning.floor_y;
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
        desired_air_steering_direction(input, facing)
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

pub fn smoothing_factor(rate: f32, dt: f32) -> f32 {
    (1.0 - (-rate.max(0.0) * dt.max(0.0)).exp()).clamp(0.0, 1.0)
}

fn desired_air_steering_direction(input: FlightInput, facing: Facing) -> Option<Vec3> {
    if !input.forward && !input.left && !input.right {
        return None;
    }

    desired_planar_movement_direction(input, facing)
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

fn horizontal(v: Vec3) -> Vec3 {
    Vec3::new(v.x, 0.0, v.z)
}

fn horizontal_or(v: Vec3, fallback: Vec3) -> Vec3 {
    let horizontal = horizontal(v);
    if horizontal.length_squared() > 0.0001 {
        horizontal.normalize()
    } else {
        fallback.normalize()
    }
}

fn axis_value(negative: bool, positive: bool) -> f32 {
    match (negative, positive) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            left_speed > 10.0,
            "expected back-left input to build leftward control, got {left_speed}"
        );
        assert!(
            forward_speed < 6.0,
            "expected back-left input to brake forward drift, got {forward_speed}"
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

        for _ in 0..60 {
            state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
        }

        let side_speed = horizontal(state.velocity).dot(facing.right);
        let forward_speed = horizontal(state.velocity).dot(facing.forward);
        assert!(
            side_speed > 14.0,
            "expected right input to build meaningful planar side speed, got {side_speed}"
        );
        assert!(
            forward_speed < 28.0,
            "expected steering to rotate velocity away from pure forward drift, got {forward_speed}"
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

        for _ in 0..15 {
            state = step_flight(state, input, facing, &tuning, 1.0 / 60.0);
        }

        let left_response = horizontal(state.velocity).dot(-facing.right);
        assert!(
            left_response > 4.0,
            "expected left reversal to recover promptly, got {left_response}"
        );
    }

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

    #[test]
    fn floor_collision_clears_downward_velocity() {
        let tuning = FlightTuning::default();
        let state = FlightState::new(
            Vec3::new(0.0, tuning.floor_y + 0.01, 0.0),
            Vec3::new(0.0, -20.0, 0.0),
            FlightController::default(),
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
    }

    #[test]
    fn smoothing_factor_never_overshoots() {
        assert!((0.0..=1.0).contains(&smoothing_factor(8.0, 0.5)));
        assert!((0.0..=1.0).contains(&smoothing_factor(8.0, 3.0)));
    }
}
