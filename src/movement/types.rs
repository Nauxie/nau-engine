use bevy::prelude::*;

use super::math::{axis_value, horizontal_or};

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Velocity(pub Vec3);

#[derive(Component, Clone, Copy, Debug)]
pub struct FlightController {
    pub mode: FlightMode,
    pub launch_cooldown_remaining: f32,
    pub launch_timer: f32,
    pub launch_available: bool,
    pub bank_degrees: f32,
    pub landing_recovery_timer: f32,
    pub landing_impact_speed_mps: f32,
}

impl Default for FlightController {
    fn default() -> Self {
        Self {
            mode: FlightMode::Grounded,
            launch_cooldown_remaining: 0.0,
            launch_timer: 0.0,
            launch_available: true,
            bank_degrees: 0.0,
            landing_recovery_timer: 0.0,
            landing_impact_speed_mps: 0.0,
        }
    }
}

impl FlightController {
    pub fn step_landing_recovery(&mut self, dt: f32) {
        self.landing_recovery_timer = (self.landing_recovery_timer - dt.max(0.0)).max(0.0);
        if self.landing_recovery_timer <= f32::EPSILON {
            self.landing_recovery_timer = 0.0;
            self.landing_impact_speed_mps = 0.0;
        }
    }

    pub fn clear_landing_recovery(&mut self) {
        self.landing_recovery_timer = 0.0;
        self.landing_impact_speed_mps = 0.0;
    }

    pub fn record_landing_impact(&mut self, impact_speed_mps: f32) {
        let impact_speed_mps = impact_speed_mps.max(0.0);
        let duration = landing_recovery_duration_secs(impact_speed_mps);
        if duration <= 0.0 {
            return;
        }

        self.landing_recovery_timer = self.landing_recovery_timer.max(duration);
        self.landing_impact_speed_mps = self.landing_impact_speed_mps.max(impact_speed_mps);
    }

    pub fn landing_recovery_strength(self) -> f32 {
        landing_recovery_strength(self.landing_recovery_timer, self.landing_impact_speed_mps)
    }
}

pub const LANDING_RECOVERY_MIN_IMPACT_SPEED_MPS: f32 = 1.8;
pub const LANDING_RECOVERY_MAX_IMPACT_SPEED_MPS: f32 = 18.0;
pub const LANDING_RECOVERY_MIN_DURATION_SECS: f32 = 0.18;
pub const LANDING_RECOVERY_MAX_DURATION_SECS: f32 = 0.46;

pub fn landing_recovery_duration_secs(impact_speed_mps: f32) -> f32 {
    if impact_speed_mps < LANDING_RECOVERY_MIN_IMPACT_SPEED_MPS {
        return 0.0;
    }

    let t = ((impact_speed_mps - LANDING_RECOVERY_MIN_IMPACT_SPEED_MPS)
        / (LANDING_RECOVERY_MAX_IMPACT_SPEED_MPS - LANDING_RECOVERY_MIN_IMPACT_SPEED_MPS))
        .clamp(0.0, 1.0);
    LANDING_RECOVERY_MIN_DURATION_SECS
        + (LANDING_RECOVERY_MAX_DURATION_SECS - LANDING_RECOVERY_MIN_DURATION_SECS) * t
}

pub fn landing_recovery_strength(remaining_secs: f32, impact_speed_mps: f32) -> f32 {
    let duration = landing_recovery_duration_secs(impact_speed_mps);
    if duration <= 0.0 || remaining_secs <= 0.0 {
        return 0.0;
    }

    let impact = ((impact_speed_mps - LANDING_RECOVERY_MIN_IMPACT_SPEED_MPS)
        / (LANDING_RECOVERY_MAX_IMPACT_SPEED_MPS - LANDING_RECOVERY_MIN_IMPACT_SPEED_MPS))
        .clamp(0.0, 1.0);
    let remaining = (remaining_secs / duration).clamp(0.0, 1.0);
    (0.45 + impact * 0.35 + remaining * 0.20).clamp(0.0, 1.0)
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
    pub air_input_orthogonal_damping: f32,
    pub glide_input_orthogonal_damping: f32,
    pub air_steer_min_speed: f32,
    pub air_velocity_turn_rate_degrees: f32,
    pub glide_velocity_turn_rate_degrees: f32,
    pub air_velocity_counter_turn_rate_degrees: f32,
    pub glide_velocity_counter_turn_rate_degrees: f32,
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
            backward_accel: 14.0,
            lateral_accel: 20.0,
            glide_forward_accel: 12.0,
            glide_lateral_accel: 24.0,
            glide_brake_drag: 0.07,
            air_brake_accel: 46.0,
            glide_brake_accel: 92.0,
            max_backward_speed: 16.0,
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
            air_steer_accel: 62.0,
            glide_steer_accel: 62.0,
            air_counter_steer_accel: 170.0,
            glide_counter_steer_accel: 520.0,
            air_input_orthogonal_damping: 5.0,
            glide_input_orthogonal_damping: 14.0,
            air_steer_min_speed: 16.0,
            air_velocity_turn_rate_degrees: 185.0,
            glide_velocity_turn_rate_degrees: 320.0,
            air_velocity_counter_turn_rate_degrees: 360.0,
            glide_velocity_counter_turn_rate_degrees: 760.0,
            max_bank_degrees: 20.0,
            bank_response_rate: 6.0,
            turn_rate: 16.0,
            input_turn_rate_boost: 11.0,
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
