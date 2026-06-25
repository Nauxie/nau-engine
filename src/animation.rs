use crate::movement::{FlightMode, smoothing_factor};
use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct AnimationState {
    pub phase: f32,
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

fn side_cycle(phase: f32, side: Side) -> f32 {
    let offset = if side == Side::Left { 0.0 } else { TAU * 0.5 };
    (phase + offset).sin()
}

pub fn part_pose(part: &CharacterPart, mode: FlightMode, velocity: Vec3, phase: f32) -> PartPose {
    let cycle = phase.sin();
    let horizontal_speed = Vec2::new(velocity.x, velocity.z).length();
    let gait_weight = (horizontal_speed / 16.0).clamp(0.0, 1.0);
    let roll = (-velocity.x * 0.006).clamp(-0.14, 0.14);
    let vertical_pitch = (-velocity.y * 0.004).clamp(-0.1, 0.1);
    let mut translation = part.base_translation;
    let mut rotation = part.base_rotation;
    let mut visibility = PartVisibility::Inherited;

    match part.role {
        CharacterPartRole::Torso => {
            let pitch = match mode {
                FlightMode::Grounded => -0.04 * gait_weight,
                FlightMode::Airborne => -0.12 + vertical_pitch,
                FlightMode::Gliding => -0.30 + vertical_pitch * 0.5,
                FlightMode::Launching => 0.1,
            };
            translation.y += cycle.abs() * (0.014 + gait_weight * 0.018);
            rotation *= Quat::from_rotation_x(pitch) * Quat::from_rotation_z(roll);
        }
        CharacterPartRole::Head => {
            translation.y += cycle.abs() * (0.01 + gait_weight * 0.006);
            rotation *= Quat::from_rotation_x(-0.05) * Quat::from_rotation_z(roll * 0.35);
        }
        CharacterPartRole::Arm(side) => {
            let sign = side.sign();
            let gait = -side_cycle(phase, side);
            let spread = match mode {
                FlightMode::Grounded => 0.08 + gait.abs() * 0.06 * gait_weight,
                FlightMode::Airborne => 0.65,
                FlightMode::Gliding => 1.08,
                FlightMode::Launching => 0.28,
            };
            let sweep = match mode {
                FlightMode::Grounded => gait * 0.48 * gait_weight,
                FlightMode::Gliding => -0.58,
                FlightMode::Launching => 0.22,
                FlightMode::Airborne => -0.2,
            };
            translation.z += gait * 0.08 * gait_weight;
            translation.y += match mode {
                FlightMode::Gliding => 0.04,
                FlightMode::Airborne => -0.02,
                _ => 0.0,
            };
            rotation *= Quat::from_rotation_z(sign * spread) * Quat::from_rotation_x(sweep);
        }
        CharacterPartRole::Leg(side) => {
            let sign = side.sign();
            let gait = side_cycle(phase, side);
            let spread = match mode {
                FlightMode::Grounded => 0.04 + gait.abs() * 0.05 * gait_weight,
                FlightMode::Airborne => 0.14,
                FlightMode::Gliding => 0.2,
                FlightMode::Launching => 0.02,
            };
            let trail = match mode {
                FlightMode::Grounded => gait * 0.52 * gait_weight,
                FlightMode::Gliding => 0.46 + cycle * 0.04,
                FlightMode::Airborne => 0.22 + vertical_pitch,
                FlightMode::Launching => -0.12,
            };
            translation.z += gait * 0.18 * gait_weight;
            translation.y += gait.max(0.0) * 0.045 * gait_weight;
            rotation *= Quat::from_rotation_z(sign * spread) * Quat::from_rotation_x(trail);
        }
        CharacterPartRole::Wing(side) => {
            visibility = if mode == FlightMode::Gliding {
                PartVisibility::Visible
            } else {
                PartVisibility::Hidden
            };

            let sign = side.sign();
            let bank = (velocity.x * 0.012).clamp(-0.2, 0.2);
            let airflow = wing_airflow_strength(mode, velocity);
            let flutter = (phase * 2.4).sin() * (0.018 + airflow * 0.038);
            translation.y += flutter * 0.5 + airflow * 0.045;
            translation.z += airflow * 0.06;
            rotation *= Quat::from_rotation_z(sign * (bank + airflow * 0.05))
                * Quat::from_rotation_y(sign * airflow * 0.08)
                * Quat::from_rotation_x(flutter - airflow * 0.09);
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
}
