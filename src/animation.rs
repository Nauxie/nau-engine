use crate::movement::{FlightInput, FlightMode, smoothing_factor};
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerPoseIntent {
    #[default]
    GroundedIdle,
    GroundedStride,
    Launching,
    Falling,
    Gliding,
    Diving,
    AirBrake,
    LandingAnticipation,
}

impl PlayerPoseIntent {
    pub fn label(self) -> &'static str {
        match self {
            Self::GroundedIdle => "grounded_idle",
            Self::GroundedStride => "grounded_stride",
            Self::Launching => "launching",
            Self::Falling => "falling",
            Self::Gliding => "gliding",
            Self::Diving => "diving",
            Self::AirBrake => "air_brake",
            Self::LandingAnticipation => "landing_anticipation",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlayerPoseContext {
    pub mode: FlightMode,
    pub velocity: Vec3,
    pub input: FlightInput,
    pub height_above_ground_m: f32,
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
        }
    }

    pub fn intent(self) -> PlayerPoseIntent {
        player_pose_intent(self)
    }
}

pub fn advance_phase(phase: f32, speed: f32, dt: f32) -> f32 {
    (phase + (5.0 + speed.max(0.0) * 0.08) * dt.max(0.0)).rem_euclid(TAU)
}

pub fn pose_blend(dt: f32) -> f32 {
    smoothing_factor(18.0, dt)
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

pub fn player_pose_intent(context: PlayerPoseContext) -> PlayerPoseIntent {
    let horizontal_speed = Vec2::new(context.velocity.x, context.velocity.z).length();
    let near_landing = context.mode != FlightMode::Grounded
        && context.height_above_ground_m <= 6.0
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
        FlightMode::Gliding => PlayerPoseIntent::Gliding,
        FlightMode::Airborne if context.input.dive || context.velocity.y < -18.0 => {
            PlayerPoseIntent::Diving
        }
        FlightMode::Airborne => PlayerPoseIntent::Falling,
    }
}

fn side_cycle(phase: f32, side: Side) -> f32 {
    let offset = if side == Side::Left { 0.0 } else { TAU * 0.5 };
    (phase + offset).sin()
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
    let roll = (-context.velocity.x * 0.006).clamp(-0.14, 0.14);
    let vertical_pitch = (-context.velocity.y * 0.004).clamp(-0.1, 0.1);
    let mut translation = part.base_translation;
    let mut rotation = part.base_rotation;
    let mut visibility = PartVisibility::Inherited;

    match part.role {
        CharacterPartRole::Torso => {
            let pitch = match intent {
                PlayerPoseIntent::GroundedIdle => 0.015 + cycle * 0.01,
                PlayerPoseIntent::GroundedStride => -0.04 * gait_weight,
                PlayerPoseIntent::Falling => -0.12 + vertical_pitch,
                PlayerPoseIntent::Gliding => -0.30 + vertical_pitch * 0.5,
                PlayerPoseIntent::Diving => -0.92 + vertical_pitch * 0.25,
                PlayerPoseIntent::AirBrake => 0.08 + vertical_pitch * 0.35,
                PlayerPoseIntent::LandingAnticipation => 0.42,
                PlayerPoseIntent::Launching => 0.1,
            };
            translation.y += cycle.abs() * (0.014 + gait_weight * 0.018);
            if intent == PlayerPoseIntent::LandingAnticipation {
                translation.y += 0.08;
                translation.z += 0.08;
            }
            rotation *= Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        }
        CharacterPartRole::Head => {
            translation.y += cycle.abs() * (0.01 + gait_weight * 0.006);
            let pitch = match intent {
                PlayerPoseIntent::Diving => 0.24,
                PlayerPoseIntent::AirBrake => -0.14,
                PlayerPoseIntent::LandingAnticipation => -0.22,
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
                PlayerPoseIntent::Diving => 1.34,
                PlayerPoseIntent::AirBrake => 1.46,
                PlayerPoseIntent::LandingAnticipation => 0.78,
                PlayerPoseIntent::Launching => 0.28,
            };
            let sweep = match intent {
                PlayerPoseIntent::GroundedIdle => cycle * 0.025,
                PlayerPoseIntent::GroundedStride => gait * 0.48 * gait_weight,
                PlayerPoseIntent::Gliding => -0.58,
                PlayerPoseIntent::Diving => -0.08,
                PlayerPoseIntent::AirBrake => 0.42,
                PlayerPoseIntent::LandingAnticipation => 0.86,
                PlayerPoseIntent::Launching => 0.22,
                PlayerPoseIntent::Falling => -0.2,
            };
            translation.z += gait * 0.08 * gait_weight;
            translation.y += match context.mode {
                _ if intent == PlayerPoseIntent::Diving => 0.12,
                _ if intent == PlayerPoseIntent::AirBrake => 0.08,
                _ if intent == PlayerPoseIntent::LandingAnticipation => -0.08,
                FlightMode::Gliding => 0.04,
                FlightMode::Airborne => -0.02,
                _ => 0.0,
            };
            rotation *= Quat::from_rotation_z(sign * spread) * Quat::from_rotation_x(sweep);
        }
        CharacterPartRole::Leg(side) => {
            let sign = side.sign();
            let gait = side_cycle(phase, side);
            let spread = match intent {
                PlayerPoseIntent::GroundedIdle => 0.04,
                PlayerPoseIntent::GroundedStride => 0.04 + gait.abs() * 0.05 * gait_weight,
                PlayerPoseIntent::Falling => 0.14,
                PlayerPoseIntent::Gliding => 0.2,
                PlayerPoseIntent::Diving => 0.12,
                PlayerPoseIntent::AirBrake => 0.34,
                PlayerPoseIntent::LandingAnticipation => 0.38,
                PlayerPoseIntent::Launching => 0.02,
            };
            let trail = match intent {
                PlayerPoseIntent::GroundedIdle => 0.02,
                PlayerPoseIntent::GroundedStride => gait * 0.52 * gait_weight,
                PlayerPoseIntent::Gliding => 0.46 + cycle * 0.04,
                PlayerPoseIntent::Diving => 0.86 + cycle * 0.02,
                PlayerPoseIntent::AirBrake => -0.34,
                PlayerPoseIntent::LandingAnticipation => -0.82,
                PlayerPoseIntent::Falling => 0.22 + vertical_pitch,
                PlayerPoseIntent::Launching => -0.12,
            };
            translation.z += gait * 0.18 * gait_weight;
            translation.y += gait.max(0.0) * 0.045 * gait_weight;
            if intent == PlayerPoseIntent::LandingAnticipation {
                translation.z += 0.14;
                translation.y += 0.05;
            }
            rotation *= Quat::from_rotation_z(sign * spread) * Quat::from_rotation_x(trail);
        }
        CharacterPartRole::Wing(side) => {
            visibility = if context.mode == FlightMode::Gliding {
                PartVisibility::Visible
            } else {
                PartVisibility::Hidden
            };

            let sign = side.sign();
            let bank = (context.velocity.x * 0.012).clamp(-0.2, 0.2);
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
                Vec3::new(0.0, -3.0, -18.0),
                FlightInput::default(),
                4.5,
            )),
            PlayerPoseIntent::LandingAnticipation
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
    fn landing_anticipation_pose_tucks_legs_forward() {
        let leg = CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        let falling_context = PlayerPoseContext::new(
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

        let falling = part_pose_with_context(&leg, falling_context, 0.0);
        let landing = part_pose_with_context(&leg, landing_context, 0.0);

        assert!(landing.translation.z > falling.translation.z + 0.1);
        assert!(landing.translation.y > falling.translation.y + 0.04);
        assert!(landing.rotation.angle_between(falling.rotation) > 1.0);
    }
}
