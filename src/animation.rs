use crate::movement::{
    FlightInput, FlightMode, body_forward,
    landing_recovery_strength as movement_landing_recovery_strength, smoothing_factor,
};
use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Component, Clone, Copy, Debug)]
pub struct AnimationState {
    pub phase: f32,
    pub last_input: FlightInput,
    pub height_above_ground_m: f32,
    pub pose_intent: PlayerPoseIntent,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            phase: 0.0,
            last_input: FlightInput::default(),
            height_above_ground_m: f32::INFINITY,
            pose_intent: PlayerPoseIntent::GroundedIdle,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct CharacterPart {
    pub role: CharacterPartRole,
    pub base_translation: Vec3,
    pub base_rotation: Quat,
}

impl CharacterPart {
    pub fn new(role: CharacterPartRole, base_translation: Vec3, base_rotation: Quat) -> Self {
        Self {
            role,
            base_translation,
            base_rotation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterPartRole {
    Torso,
    Head,
    Arm(Side),
    Leg(Side),
    Wing(Side),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartVisibility {
    Inherited,
    Hidden,
    Visible,
}

#[derive(Clone, Copy, Debug)]
pub struct PartPose {
    pub translation: Vec3,
    pub rotation: Quat,
    pub visibility: PartVisibility,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PoseReadabilityMetrics {
    pub torso_pitch_degrees: f32,
    pub arm_spread_degrees: f32,
    pub leg_tuck_degrees: f32,
    pub lateral_lean_degrees: f32,
    pub signed_lateral_lean_degrees: f32,
    pub landing_crouch_m: f32,
    pub wing_airflow_strength: f32,
    pub key_pose_readability_score: f32,
}

pub const MIN_KEY_POSE_READABILITY_SCORE: f32 = 0.9;
const LANDING_ANTICIPATION_BASE_HEIGHT_M: f32 = 6.0;
const LANDING_ANTICIPATION_MAX_HEIGHT_M: f32 = 20.0;
const LANDING_ANTICIPATION_SINK_LOOKAHEAD_SECS: f32 = 0.32;

#[derive(Clone, Copy, Debug)]
pub struct PoseReadabilityPartTransforms {
    pub torso_rotation: Quat,
    pub left_arm_rotation: Quat,
    pub right_arm_rotation: Quat,
    pub left_leg_rotation: Quat,
    pub right_leg_rotation: Quat,
    pub left_leg_translation: Vec3,
    pub right_leg_translation: Vec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerPoseIntent {
    #[default]
    GroundedIdle,
    GroundedStride,
    Launching,
    Falling,
    Gliding,
    AirTurn,
    Diving,
    AirBrake,
    LandingAnticipation,
    LandingRecovery,
}

impl PlayerPoseIntent {
    pub fn label(self) -> &'static str {
        match self {
            Self::GroundedIdle => "grounded_idle",
            Self::GroundedStride => "grounded_stride",
            Self::Launching => "launching",
            Self::Falling => "falling",
            Self::Gliding => "gliding",
            Self::AirTurn => "air_turn",
            Self::Diving => "diving",
            Self::AirBrake => "air_brake",
            Self::LandingAnticipation => "landing_anticipation",
            Self::LandingRecovery => "landing_recovery",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerPoseContext {
    pub mode: FlightMode,
    pub velocity: Vec3,
    pub input: FlightInput,
    pub height_above_ground_m: f32,
    pub landing_recovery_remaining_secs: f32,
    pub landing_impact_speed_mps: f32,
}

impl PlayerPoseContext {
    pub fn new(
        mode: FlightMode,
        velocity: Vec3,
        input: FlightInput,
        height_above_ground_m: f32,
    ) -> Self {
        Self {
            mode,
            velocity,
            input,
            height_above_ground_m,
            landing_recovery_remaining_secs: 0.0,
            landing_impact_speed_mps: 0.0,
        }
    }

    pub fn with_landing_recovery(mut self, remaining_secs: f32, impact_speed_mps: f32) -> Self {
        self.landing_recovery_remaining_secs = remaining_secs.max(0.0);
        self.landing_impact_speed_mps = impact_speed_mps.max(0.0);
        self
    }

    pub fn intent(self) -> PlayerPoseIntent {
        player_pose_intent(self)
    }

    pub fn landing_recovery_strength(self) -> f32 {
        movement_landing_recovery_strength(
            self.landing_recovery_remaining_secs,
            self.landing_impact_speed_mps,
        )
    }
}

pub fn advance_phase(phase: f32, speed: f32, dt: f32) -> f32 {
    (phase + (5.0 + speed.max(0.0) * 0.08) * dt.max(0.0)).rem_euclid(TAU)
}

pub fn pose_blend_for_intent(intent: PlayerPoseIntent, dt: f32) -> f32 {
    let rate = match intent {
        PlayerPoseIntent::LandingAnticipation => 36.0,
        PlayerPoseIntent::LandingRecovery => 30.0,
        PlayerPoseIntent::Gliding | PlayerPoseIntent::AirTurn => 30.0,
        PlayerPoseIntent::Diving | PlayerPoseIntent::AirBrake => 22.0,
        _ => 18.0,
    };
    smoothing_factor(rate, dt)
}

pub fn wing_airflow_strength(mode: FlightMode, velocity: Vec3) -> f32 {
    if mode != FlightMode::Gliding {
        return 0.0;
    }

    let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
    let speed_pressure = ((horizontal_speed - 18.0) / 44.0).clamp(0.0, 1.0);
    let sink_pressure = (-velocity.y / 28.0).clamp(0.0, 1.0) * 0.18;
    (speed_pressure + sink_pressure).clamp(0.0, 1.0)
}

pub fn body_local_pose_velocity(world_velocity: Vec3, player_rotation: Quat) -> Vec3 {
    let forward = body_forward(player_rotation);
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    Vec3::new(
        world_velocity.dot(right),
        world_velocity.y,
        -world_velocity.dot(forward),
    )
}

pub fn player_pose_intent(context: PlayerPoseContext) -> PlayerPoseIntent {
    let horizontal_speed = Vec2::new(context.velocity.x, context.velocity.z).length();
    if context.mode == FlightMode::Grounded && context.landing_recovery_strength() > 0.0 {
        return PlayerPoseIntent::LandingRecovery;
    }

    let near_landing = context.mode != FlightMode::Grounded
        && context.height_above_ground_m <= landing_anticipation_height_m(context.velocity.y)
        && context.velocity.y < -1.2;

    if near_landing {
        return PlayerPoseIntent::LandingAnticipation;
    }

    match context.mode {
        FlightMode::Grounded => {
            if horizontal_speed > 1.0 {
                PlayerPoseIntent::GroundedStride
            } else {
                PlayerPoseIntent::GroundedIdle
            }
        }
        FlightMode::Launching => PlayerPoseIntent::Launching,
        FlightMode::Gliding if context.input.backward => PlayerPoseIntent::AirBrake,
        FlightMode::Gliding if context.input.dive || context.velocity.y < -14.0 => {
            PlayerPoseIntent::Diving
        }
        FlightMode::Gliding if airborne_turn_input(context) => PlayerPoseIntent::AirTurn,
        FlightMode::Gliding => PlayerPoseIntent::Gliding,
        FlightMode::Airborne if context.input.dive || context.velocity.y < -18.0 => {
            PlayerPoseIntent::Diving
        }
        FlightMode::Airborne if airborne_turn_input(context) => PlayerPoseIntent::AirTurn,
        FlightMode::Airborne => PlayerPoseIntent::Falling,
    }
}

fn airborne_turn_input(context: PlayerPoseContext) -> bool {
    context.input.planar_axis().x.abs() >= 0.25
}

fn side_cycle(phase: f32, side: Side) -> f32 {
    let offset = if side == Side::Left { 0.0 } else { TAU * 0.5 };
    (phase + offset).sin()
}

fn airborne_pose_intent(intent: PlayerPoseIntent) -> bool {
    matches!(
        intent,
        PlayerPoseIntent::Falling
            | PlayerPoseIntent::Gliding
            | PlayerPoseIntent::AirTurn
            | PlayerPoseIntent::Diving
            | PlayerPoseIntent::AirBrake
            | PlayerPoseIntent::LandingAnticipation
    )
}

fn pose_turn_weight(context: PlayerPoseContext) -> f32 {
    let intent = context.intent();
    let divisor = if airborne_pose_intent(intent) {
        18.0
    } else {
        12.0
    };
    let velocity_weight = (context.velocity.x / divisor).clamp(-1.0, 1.0);
    let input_weight = context.input.planar_axis().x.clamp(-1.0, 1.0);
    if airborne_pose_intent(intent) && input_weight.abs() > f32::EPSILON {
        (velocity_weight * 0.35 + input_weight * 0.65).clamp(-1.0, 1.0)
    } else {
        velocity_weight
    }
}

fn pose_lateral_lean_radians(context: PlayerPoseContext) -> f32 {
    let intent = context.intent();
    let max_lean = if intent == PlayerPoseIntent::AirTurn {
        0.30
    } else if airborne_pose_intent(intent) {
        0.22
    } else {
        0.08
    };
    -pose_turn_weight(context) * max_lean
}

fn landing_anticipation_strength(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::LandingAnticipation {
        return 0.0;
    }

    let anticipation_height = landing_anticipation_height_m(context.velocity.y);
    let ground_proximity = ((anticipation_height - context.height_above_ground_m)
        / anticipation_height.max(0.1))
    .clamp(0.0, 1.0);
    let sink_rate = ((-context.velocity.y - 1.2) / 6.0).clamp(0.0, 1.0);
    (0.65 + ground_proximity.max(sink_rate) * 0.35).clamp(0.0, 1.0)
}

fn landing_anticipation_height_m(vertical_velocity_mps: f32) -> f32 {
    (LANDING_ANTICIPATION_BASE_HEIGHT_M
        + (-vertical_velocity_mps).max(0.0) * LANDING_ANTICIPATION_SINK_LOOKAHEAD_SECS)
        .min(LANDING_ANTICIPATION_MAX_HEIGHT_M)
}

fn landing_recovery_strength(context: PlayerPoseContext, intent: PlayerPoseIntent) -> f32 {
    if intent != PlayerPoseIntent::LandingRecovery {
        return 0.0;
    }

    context.landing_recovery_strength()
}

pub fn key_pose_readability_score(
    intent: PlayerPoseIntent,
    torso_pitch_degrees: f32,
    arm_spread_degrees: f32,
    leg_tuck_degrees: f32,
    landing_crouch_m: f32,
) -> f32 {
    match intent {
        PlayerPoseIntent::Diving => {
            readable_pair_score(torso_pitch_degrees, 54.0, arm_spread_degrees, 155.0)
        }
        PlayerPoseIntent::AirBrake => {
            readable_pair_score(torso_pitch_degrees, 4.0, arm_spread_degrees, 160.0)
        }
        PlayerPoseIntent::LandingAnticipation => {
            readable_pair_score(leg_tuck_degrees, 48.0, landing_crouch_m, 0.07)
        }
        PlayerPoseIntent::LandingRecovery => {
            readable_pair_score(leg_tuck_degrees, 32.0, landing_crouch_m, 0.055)
        }
        PlayerPoseIntent::Gliding => {
            readable_pair_score(torso_pitch_degrees, 16.0, arm_spread_degrees, 120.0)
        }
        _ => 1.0,
    }
}

fn torso_signed_lateral_lean_degrees(rotation: Quat) -> f32 {
    let local_up = rotation * Vec3::Y;
    -local_up.x.atan2(local_up.y).to_degrees()
}

fn torso_lateral_lean_degrees(rotation: Quat) -> f32 {
    torso_signed_lateral_lean_degrees(rotation).abs()
}

pub fn pose_readability_metrics_from_part_transforms(
    context: PlayerPoseContext,
    parts: PoseReadabilityPartTransforms,
) -> PoseReadabilityMetrics {
    let landing_crouch_m = if matches!(
        context.intent(),
        PlayerPoseIntent::LandingAnticipation | PlayerPoseIntent::LandingRecovery
    ) {
        let average_leg_lift =
            ((parts.left_leg_translation.y + parts.right_leg_translation.y) * 0.5).max(0.0);
        let average_forward_tuck =
            ((parts.left_leg_translation.z + parts.right_leg_translation.z) * 0.5).max(0.0);
        average_leg_lift + average_forward_tuck * 0.08
    } else {
        0.0
    };

    let torso_pitch_degrees = parts
        .torso_rotation
        .angle_between(Quat::IDENTITY)
        .to_degrees();
    let arm_spread_degrees = parts
        .left_arm_rotation
        .angle_between(parts.right_arm_rotation)
        .to_degrees();
    let leg_tuck_degrees = (parts
        .left_leg_rotation
        .angle_between(Quat::IDENTITY)
        .to_degrees()
        + parts
            .right_leg_rotation
            .angle_between(Quat::IDENTITY)
            .to_degrees())
        * 0.5;

    PoseReadabilityMetrics {
        torso_pitch_degrees,
        arm_spread_degrees,
        leg_tuck_degrees,
        lateral_lean_degrees: torso_lateral_lean_degrees(parts.torso_rotation),
        signed_lateral_lean_degrees: torso_signed_lateral_lean_degrees(parts.torso_rotation),
        landing_crouch_m,
        wing_airflow_strength: wing_airflow_strength(context.mode, context.velocity),
        key_pose_readability_score: key_pose_readability_score(
            context.intent(),
            torso_pitch_degrees,
            arm_spread_degrees,
            leg_tuck_degrees,
            landing_crouch_m,
        ),
    }
}

fn readable_pair_score(first: f32, first_target: f32, second: f32, second_target: f32) -> f32 {
    (first / first_target)
        .min(second / second_target)
        .clamp(0.0, 1.0)
}

pub fn pose_readability_metrics(context: PlayerPoseContext, phase: f32) -> PoseReadabilityMetrics {
    let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
    let left_arm = CharacterPart::new(
        CharacterPartRole::Arm(Side::Left),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let right_arm = CharacterPart::new(
        CharacterPartRole::Arm(Side::Right),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let left_leg = CharacterPart::new(
        CharacterPartRole::Leg(Side::Left),
        Vec3::ZERO,
        Quat::IDENTITY,
    );
    let right_leg = CharacterPart::new(
        CharacterPartRole::Leg(Side::Right),
        Vec3::ZERO,
        Quat::IDENTITY,
    );

    let torso_pose = part_pose_with_context(&torso, context, phase);
    let left_arm_pose = part_pose_with_context(&left_arm, context, phase);
    let right_arm_pose = part_pose_with_context(&right_arm, context, phase);
    let left_leg_pose = part_pose_with_context(&left_leg, context, phase);
    let right_leg_pose = part_pose_with_context(&right_leg, context, phase);
    let mut metrics = pose_readability_metrics_from_part_transforms(
        context,
        PoseReadabilityPartTransforms {
            torso_rotation: torso_pose.rotation,
            left_arm_rotation: left_arm_pose.rotation,
            right_arm_rotation: right_arm_pose.rotation,
            left_leg_rotation: left_leg_pose.rotation,
            right_leg_rotation: right_leg_pose.rotation,
            left_leg_translation: left_leg_pose.translation,
            right_leg_translation: right_leg_pose.translation,
        },
    );
    metrics.signed_lateral_lean_degrees = pose_lateral_lean_radians(context).to_degrees();
    metrics.lateral_lean_degrees = metrics.signed_lateral_lean_degrees.abs();
    metrics
}

pub fn part_pose(part: &CharacterPart, mode: FlightMode, velocity: Vec3, phase: f32) -> PartPose {
    part_pose_with_context(
        part,
        PlayerPoseContext::new(mode, velocity, FlightInput::default(), f32::INFINITY),
        phase,
    )
}

pub fn part_pose_with_context(
    part: &CharacterPart,
    context: PlayerPoseContext,
    phase: f32,
) -> PartPose {
    let cycle = phase.sin();
    let intent = context.intent();
    let horizontal_speed = Vec2::new(context.velocity.x, context.velocity.z).length();
    let gait_weight = (horizontal_speed / 16.0).clamp(0.0, 1.0);
    let turn_weight = pose_turn_weight(context);
    let airborne_pose = airborne_pose_intent(intent);
    let roll = pose_lateral_lean_radians(context);
    let vertical_pitch = (-context.velocity.y * 0.004).clamp(-0.1, 0.1);
    let mut translation = part.base_translation;
    let mut rotation = part.base_rotation;
    let mut visibility = PartVisibility::Inherited;
    let landing_strength = landing_anticipation_strength(context, intent);
    let recovery_strength = landing_recovery_strength(context, intent);

    match part.role {
        CharacterPartRole::Torso => {
            let pitch = match intent {
                PlayerPoseIntent::GroundedIdle => 0.015 + cycle * 0.01,
                PlayerPoseIntent::GroundedStride => -0.04 * gait_weight,
                PlayerPoseIntent::Falling => -0.12 + vertical_pitch,
                PlayerPoseIntent::Gliding => -0.30 + vertical_pitch * 0.5,
                PlayerPoseIntent::AirTurn => -0.34 + vertical_pitch * 0.45,
                PlayerPoseIntent::Diving => -1.04 + vertical_pitch * 0.18,
                PlayerPoseIntent::AirBrake => 0.08 + vertical_pitch * 0.35,
                PlayerPoseIntent::LandingAnticipation => 0.46 + landing_strength * 0.22,
                PlayerPoseIntent::LandingRecovery => 0.20 + recovery_strength * 0.16,
                PlayerPoseIntent::Launching => 0.1,
            };
            translation.y += cycle.abs() * (0.014 + gait_weight * 0.018);
            if intent == PlayerPoseIntent::LandingAnticipation {
                translation.y += 0.10 + landing_strength * 0.07;
                translation.z += 0.13 + landing_strength * 0.10;
            } else if intent == PlayerPoseIntent::LandingRecovery {
                translation.y -= 0.06 + recovery_strength * 0.05;
                translation.z += 0.04 + recovery_strength * 0.06;
            }
            rotation *= Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        }
        CharacterPartRole::Head => {
            translation.y += cycle.abs() * (0.01 + gait_weight * 0.006);
            let pitch = match intent {
                PlayerPoseIntent::AirTurn => -0.10,
                PlayerPoseIntent::Diving => 0.18,
                PlayerPoseIntent::AirBrake => -0.14,
                PlayerPoseIntent::LandingAnticipation => -0.34,
                PlayerPoseIntent::LandingRecovery => -0.12,
                _ => -0.05,
            };
            rotation *= Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll * 0.35);
        }
        CharacterPartRole::Arm(side) => {
            let sign = side.sign();
            let gait = -side_cycle(phase, side);
            let spread = match intent {
                PlayerPoseIntent::GroundedIdle => 0.08,
                PlayerPoseIntent::GroundedStride => 0.08 + gait.abs() * 0.06 * gait_weight,
                PlayerPoseIntent::Falling => 0.65,
                PlayerPoseIntent::Gliding => 1.08,
                PlayerPoseIntent::AirTurn => 1.18,
                PlayerPoseIntent::Diving => 1.50,
                PlayerPoseIntent::AirBrake => 1.50,
                PlayerPoseIntent::LandingAnticipation => 0.92 + landing_strength * 0.20,
                PlayerPoseIntent::LandingRecovery => 0.62 + recovery_strength * 0.22,
                PlayerPoseIntent::Launching => 0.28,
            };
            let sweep = match intent {
                PlayerPoseIntent::GroundedIdle => cycle * 0.025,
                PlayerPoseIntent::GroundedStride => gait * 0.48 * gait_weight,
                PlayerPoseIntent::Gliding => -0.58,
                PlayerPoseIntent::AirTurn => -0.46 + turn_weight.abs() * 0.10,
                PlayerPoseIntent::Diving => 0.10,
                PlayerPoseIntent::AirBrake => 0.42,
                PlayerPoseIntent::LandingAnticipation => 1.08 + landing_strength * 0.28,
                PlayerPoseIntent::LandingRecovery => 0.36 + recovery_strength * 0.24,
                PlayerPoseIntent::Launching => 0.22,
                PlayerPoseIntent::Falling => -0.2,
            };
            translation.z += gait * 0.08 * gait_weight;
            translation.y += match context.mode {
                _ if intent == PlayerPoseIntent::Diving => 0.12,
                _ if intent == PlayerPoseIntent::AirBrake => 0.08,
                _ if intent == PlayerPoseIntent::LandingAnticipation => -0.12,
                _ if intent == PlayerPoseIntent::LandingRecovery => -0.04,
                FlightMode::Gliding => 0.04,
                FlightMode::Airborne => -0.02,
                _ => 0.0,
            };
            if airborne_pose {
                translation.y += sign * turn_weight * 0.045;
                translation.z += turn_weight.abs() * 0.025;
            }
            rotation *= Quat::from_rotation_z(sign * spread)
                * Quat::from_rotation_x(sweep)
                * Quat::from_rotation_y(sign * turn_weight * 0.12);
        }
        CharacterPartRole::Leg(side) => {
            let sign = side.sign();
            let gait = side_cycle(phase, side);
            let spread = match intent {
                PlayerPoseIntent::GroundedIdle => 0.04,
                PlayerPoseIntent::GroundedStride => 0.04 + gait.abs() * 0.05 * gait_weight,
                PlayerPoseIntent::Falling => 0.14,
                PlayerPoseIntent::Gliding => 0.2,
                PlayerPoseIntent::AirTurn => 0.24,
                PlayerPoseIntent::Diving => 0.12,
                PlayerPoseIntent::AirBrake => 0.34,
                PlayerPoseIntent::LandingAnticipation => 0.46 + landing_strength * 0.16,
                PlayerPoseIntent::LandingRecovery => 0.30 + recovery_strength * 0.12,
                PlayerPoseIntent::Launching => 0.02,
            };
            let trail = match intent {
                PlayerPoseIntent::GroundedIdle => 0.02,
                PlayerPoseIntent::GroundedStride => gait * 0.52 * gait_weight,
                PlayerPoseIntent::Gliding => 0.46 + cycle * 0.04,
                PlayerPoseIntent::AirTurn => 0.50 + cycle * 0.04,
                PlayerPoseIntent::Diving => 0.98 + cycle * 0.02,
                PlayerPoseIntent::AirBrake => -0.34,
                PlayerPoseIntent::LandingAnticipation => -0.98 - landing_strength * 0.32,
                PlayerPoseIntent::LandingRecovery => -0.42 - recovery_strength * 0.28,
                PlayerPoseIntent::Falling => 0.22 + vertical_pitch,
                PlayerPoseIntent::Launching => -0.12,
            };
            translation.z += gait * 0.18 * gait_weight;
            translation.y += gait.max(0.0) * 0.045 * gait_weight;
            if intent == PlayerPoseIntent::LandingAnticipation {
                translation.z += 0.18 + landing_strength * 0.14;
                translation.y += 0.07 + landing_strength * 0.06;
            } else if intent == PlayerPoseIntent::LandingRecovery {
                translation.z += 0.09 + recovery_strength * 0.11;
                translation.y += 0.035 + recovery_strength * 0.055;
            }
            if airborne_pose {
                translation.x += sign * turn_weight * 0.035;
                translation.y += turn_weight.abs() * 0.018;
            }
            rotation *= Quat::from_rotation_z(sign * spread)
                * Quat::from_rotation_x(trail)
                * Quat::from_rotation_y(sign * turn_weight * 0.08);
        }
        CharacterPartRole::Wing(side) => {
            visibility = if context.mode == FlightMode::Gliding {
                PartVisibility::Visible
            } else {
                PartVisibility::Hidden
            };

            let sign = side.sign();
            let bank =
                (context.velocity.x * 0.008 + pose_turn_weight(context) * 0.18).clamp(-0.26, 0.26);
            let airflow = wing_airflow_strength(context.mode, context.velocity);
            let flutter = (phase * 2.4).sin() * (0.018 + airflow * 0.038);
            let air_brake_cup = if intent == PlayerPoseIntent::AirBrake {
                0.16
            } else {
                0.0
            };
            translation.y += flutter * 0.5 + airflow * 0.045 + air_brake_cup * 0.2;
            translation.z += airflow * 0.06 - air_brake_cup * 0.12;
            rotation *= Quat::from_rotation_z(sign * (bank + airflow * 0.05 + air_brake_cup))
                * Quat::from_rotation_y(sign * (airflow * 0.08 + air_brake_cup * 0.25))
                * Quat::from_rotation_x(flutter - airflow * 0.09 + air_brake_cup * 0.55);
        }
    }

    PartPose {
        translation,
        rotation,
        visibility,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_advances_by_delta_instead_of_recomputing_from_elapsed_time() {
        let first = advance_phase(0.0, 10.0, 1.0 / 60.0);
        let second = advance_phase(first, 60.0, 1.0 / 60.0);

        assert!(second > first);
        assert!(second - first < 0.2);
    }

    #[test]
    fn wing_visibility_tracks_glide_mode() {
        let wing = CharacterPart::new(
            CharacterPartRole::Wing(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        assert_eq!(
            part_pose(&wing, FlightMode::Airborne, Vec3::ZERO, 0.0).visibility,
            PartVisibility::Hidden
        );
        assert_eq!(
            part_pose(&wing, FlightMode::Gliding, Vec3::ZERO, 0.0).visibility,
            PartVisibility::Visible
        );
    }

    #[test]
    fn wing_airflow_strength_requires_fast_gliding_motion() {
        let fast_glide = wing_airflow_strength(FlightMode::Gliding, Vec3::new(0.0, -18.0, -55.0));
        let slow_glide = wing_airflow_strength(FlightMode::Gliding, Vec3::new(0.0, 0.0, -8.0));
        let fast_ground = wing_airflow_strength(FlightMode::Grounded, Vec3::new(0.0, -18.0, -55.0));

        assert!(fast_glide > 0.9);
        assert_eq!(slow_glide, 0.0);
        assert_eq!(fast_ground, 0.0);
    }

    #[test]
    fn fast_gliding_wings_flex_under_airflow() {
        let wing = CharacterPart::new(
            CharacterPartRole::Wing(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let slow = part_pose(&wing, FlightMode::Gliding, Vec3::new(0.0, 0.0, -8.0), 0.0);
        let fast = part_pose(
            &wing,
            FlightMode::Gliding,
            Vec3::new(0.0, -18.0, -55.0),
            0.0,
        );

        assert!(fast.translation.y > slow.translation.y + 0.035);
        assert!(fast.translation.z > slow.translation.z);
    }

    #[test]
    fn grounded_stride_moves_left_and_right_legs_opposite_each_other() {
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let left_pose = part_pose(
            &left_leg,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, 12.0),
            TAU * 0.25,
        );
        let right_pose = part_pose(
            &right_leg,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, 12.0),
            TAU * 0.25,
        );

        assert!(left_pose.translation.z > 0.1);
        assert!(right_pose.translation.z < -0.1);
    }

    #[test]
    fn gliding_pose_lifts_arms_into_glider_posture() {
        let arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::new(-0.58, 1.05, 0.0),
            Quat::IDENTITY,
        );

        let grounded = part_pose(
            &arm,
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, 12.0),
            TAU * 0.25,
        );
        let gliding = part_pose(&arm, FlightMode::Gliding, Vec3::ZERO, TAU * 0.25);

        assert!(gliding.translation.y > grounded.translation.y);
    }

    #[test]
    fn pose_intent_classifies_dive_air_brake_and_landing_anticipation() {
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -8.0, -32.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::Diving
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -26.0),
                FlightInput {
                    backward: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirBrake
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirTurn
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -2.0, -24.0),
                FlightInput {
                    left: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirTurn
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -26.0),
                FlightInput {
                    backward: true,
                    right: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::AirBrake
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -18.0, -32.0),
                FlightInput {
                    dive: true,
                    right: true,
                    ..default()
                },
                40.0,
            )),
            PlayerPoseIntent::Diving
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -3.0, -18.0),
                FlightInput::default(),
                4.5,
            )),
            PlayerPoseIntent::LandingAnticipation
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -24.0, -18.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                10.5,
            )),
            PlayerPoseIntent::LandingAnticipation
        );
        assert_eq!(
            player_pose_intent(
                PlayerPoseContext::new(
                    FlightMode::Grounded,
                    Vec3::new(0.0, 0.0, -6.0),
                    FlightInput::default(),
                    0.0,
                )
                .with_landing_recovery(0.22, 12.0)
            ),
            PlayerPoseIntent::LandingRecovery
        );
    }

    #[test]
    fn high_sink_landing_anticipation_looks_ahead_before_dive_pose() {
        assert!((landing_anticipation_height_m(-42.0) - 19.44).abs() < 0.001);
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -42.0, -18.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                19.0,
            )),
            PlayerPoseIntent::LandingAnticipation
        );
        assert_eq!(
            player_pose_intent(PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -42.0, -18.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                21.0,
            )),
            PlayerPoseIntent::Diving
        );
    }

    #[test]
    fn dive_pose_flattens_torso_and_spreads_arms() {
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );

        let gliding_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -2.0, -28.0),
            FlightInput::default(),
            40.0,
        );
        let diving_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -18.0, -42.0),
            FlightInput {
                dive: true,
                ..default()
            },
            40.0,
        );

        let gliding_torso = part_pose_with_context(&torso, gliding_context, 0.0);
        let diving_torso = part_pose_with_context(&torso, diving_context, 0.0);
        let gliding_arm = part_pose_with_context(&left_arm, gliding_context, 0.0);
        let diving_arm = part_pose_with_context(&left_arm, diving_context, 0.0);

        assert!(diving_torso.rotation.angle_between(Quat::IDENTITY) > 0.8);
        assert!(
            diving_torso.rotation.angle_between(Quat::IDENTITY)
                > gliding_torso.rotation.angle_between(Quat::IDENTITY) + 0.45
        );
        assert!(diving_arm.translation.y > gliding_arm.translation.y + 0.07);
    }

    #[test]
    fn landing_anticipation_pose_flares_torso_and_tucks_legs_forward() {
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let falling_context = PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -3.0, -18.0),
            FlightInput::default(),
            20.0,
        );
        let gliding_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -18.0),
            FlightInput::default(),
            20.0,
        );
        let landing_context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -3.0, -18.0),
            FlightInput::default(),
            4.5,
        );

        let falling_metrics = pose_readability_metrics(falling_context, 0.0);
        let gliding_metrics = pose_readability_metrics(gliding_context, 0.0);
        let landing_metrics = pose_readability_metrics(landing_context, 0.0);
        let falling = part_pose_with_context(&leg, falling_context, 0.0);
        let landing = part_pose_with_context(&leg, landing_context, 0.0);

        assert!(landing_metrics.torso_pitch_degrees > 32.0);
        assert!(landing_metrics.torso_pitch_degrees > falling_metrics.torso_pitch_degrees + 20.0);
        assert!(landing_metrics.torso_pitch_degrees > gliding_metrics.torso_pitch_degrees + 12.0);
        assert!(landing.translation.z > falling.translation.z + 0.1);
        assert!(landing.translation.y > falling.translation.y + 0.04);
        assert!(landing.rotation.angle_between(falling.rotation) > 1.0);
    }

    #[test]
    fn landing_recovery_pose_absorbs_impact_after_touchdown() {
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let stride_context = PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -6.0),
            FlightInput {
                forward: true,
                ..default()
            },
            0.0,
        );
        let recovery_context = stride_context.with_landing_recovery(0.24, 14.0);

        let stride_torso = part_pose_with_context(&torso, stride_context, 0.0);
        let recovery_torso = part_pose_with_context(&torso, recovery_context, 0.0);
        let stride_leg = part_pose_with_context(&leg, stride_context, 0.0);
        let recovery_leg = part_pose_with_context(&leg, recovery_context, 0.0);

        assert!(recovery_torso.translation.y < stride_torso.translation.y - 0.06);
        assert!(recovery_leg.translation.z > stride_leg.translation.z + 0.1);
        assert!(recovery_leg.rotation.angle_between(stride_leg.rotation) > 0.55);
    }

    #[test]
    fn body_local_pose_velocity_uses_player_facing_axes() {
        let rotation = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::X, Vec3::Y)
            .rotation;
        let pose_velocity = body_local_pose_velocity(Vec3::NEG_Z * 14.0, rotation);

        assert!(pose_velocity.x < -13.9);
        assert!(pose_velocity.z.abs() < 0.001);
    }

    #[test]
    fn airborne_turn_pose_exposes_readable_lateral_lean() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(24.0, -2.0, -32.0),
            FlightInput {
                right: true,
                ..default()
            },
            40.0,
        );
        let metrics = pose_readability_metrics(context, 0.0);

        assert_eq!(context.intent(), PlayerPoseIntent::AirTurn);
        assert!(metrics.lateral_lean_degrees > 11.0);
        assert!(metrics.arm_spread_degrees > 100.0);
        assert!(metrics.wing_airflow_strength > 0.25);
    }

    #[test]
    fn airborne_turn_pose_responds_to_lateral_input_before_velocity_builds() {
        let neutral = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput::default(),
                40.0,
            ),
            0.0,
        );
        let turning = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );

        assert_eq!(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            )
            .intent(),
            PlayerPoseIntent::AirTurn
        );
        assert!(neutral.lateral_lean_degrees < 0.5);
        assert!(turning.lateral_lean_degrees > 10.0);
    }

    #[test]
    fn airborne_turn_pose_preserves_signed_lean_direction() {
        let right = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    right: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );
        let left = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    left: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );

        assert!(right.signed_lateral_lean_degrees < -10.0);
        assert!(left.signed_lateral_lean_degrees > 10.0);
        assert!((right.lateral_lean_degrees - left.lateral_lean_degrees).abs() < 0.1);
    }

    #[test]
    fn pose_readability_metrics_distinguish_dive_air_brake_and_landing() {
        let dive = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -18.0, -42.0),
                FlightInput {
                    dive: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );
        let air_brake = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -2.0, -32.0),
                FlightInput {
                    backward: true,
                    ..default()
                },
                40.0,
            ),
            0.0,
        );
        let landing = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -18.0),
                FlightInput::default(),
                4.0,
            ),
            0.0,
        );
        let recovery = pose_readability_metrics(
            PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(0.0, 0.0, -5.0),
                FlightInput::default(),
                0.0,
            )
            .with_landing_recovery(0.24, 14.0),
            0.0,
        );

        assert!(dive.torso_pitch_degrees > 50.0);
        assert!(air_brake.arm_spread_degrees > dive.arm_spread_degrees);
        assert!(landing.torso_pitch_degrees > 32.0);
        assert!(landing.landing_crouch_m > 0.05);
        assert!(recovery.landing_crouch_m > 0.055);
        assert!(dive.key_pose_readability_score >= 0.98);
        assert!(air_brake.key_pose_readability_score >= 0.98);
        assert!(landing.key_pose_readability_score >= 0.98);
        assert!(recovery.key_pose_readability_score >= 0.98);
    }

    #[test]
    fn pose_readability_can_be_measured_from_rendered_part_transforms() {
        let context = PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(20.0, -3.0, -34.0),
            FlightInput {
                right: true,
                ..default()
            },
            30.0,
        );
        let torso = CharacterPart::new(CharacterPartRole::Torso, Vec3::ZERO, Quat::IDENTITY);
        let left_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_arm = CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let right_leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let left_leg_pose = part_pose_with_context(&left_leg, context, 0.0);
        let right_leg_pose = part_pose_with_context(&right_leg, context, 0.0);

        let metrics = pose_readability_metrics_from_part_transforms(
            context,
            PoseReadabilityPartTransforms {
                torso_rotation: part_pose_with_context(&torso, context, 0.0).rotation,
                left_arm_rotation: part_pose_with_context(&left_arm, context, 0.0).rotation,
                right_arm_rotation: part_pose_with_context(&right_arm, context, 0.0).rotation,
                left_leg_rotation: left_leg_pose.rotation,
                right_leg_rotation: right_leg_pose.rotation,
                left_leg_translation: left_leg_pose.translation,
                right_leg_translation: right_leg_pose.translation,
            },
        );

        assert!(metrics.lateral_lean_degrees > 8.0);
        assert!(metrics.arm_spread_degrees > 100.0);
        assert!(metrics.key_pose_readability_score >= 0.9);
    }
}
