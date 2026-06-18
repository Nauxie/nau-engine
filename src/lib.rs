pub mod movement {
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
    }

    impl Default for FlightController {
        fn default() -> Self {
            Self {
                mode: FlightMode::Grounded,
                launch_cooldown_remaining: 0.0,
                launch_timer: 0.0,
                launch_available: true,
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
        pub forward_accel: f32,
        pub backward_accel: f32,
        pub lateral_accel: f32,
        pub glide_forward_accel: f32,
        pub glide_lateral_accel: f32,
        pub glide_brake_drag: f32,
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
        pub turn_rate: f32,
        pub floor_y: f32,
    }

    impl Default for FlightTuning {
        fn default() -> Self {
            Self {
                forward_accel: 28.0,
                backward_accel: 10.0,
                lateral_accel: 14.0,
                glide_forward_accel: 12.0,
                glide_lateral_accel: 9.0,
                glide_brake_drag: 0.42,
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
                turn_rate: 8.0,
                floor_y: 1.2,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct FlightInput {
        pub forward: bool,
        pub backward: bool,
        pub left: bool,
        pub right: bool,
        pub glide: bool,
        pub dive: bool,
        pub launch: bool,
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

        if gliding {
            if input.forward {
                acceleration += facing.forward * tuning.glide_forward_accel;
            }
            if input.backward {
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
            if input.forward {
                acceleration += facing.forward * tuning.forward_accel;
            }
            if input.backward {
                acceleration -= facing.forward * tuning.backward_accel;
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

        let gravity_scale = if gliding {
            tuning.glide_gravity_scale
        } else {
            1.0
        };
        acceleration.y -= tuning.gravity * gravity_scale;

        state.velocity += acceleration * dt;
        state.velocity *= tuning.drag.powf(dt);

        if gliding {
            state.velocity.y = state.velocity.y.max(-tuning.glide_max_fall_speed);
        }

        state.velocity = clamp_velocity(state.velocity, tuning);
        state.position += state.velocity * dt;

        if state.position.y < tuning.floor_y {
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

        state
    }

    pub fn face_horizontal_velocity(
        current: Quat,
        velocity: Vec3,
        turn_rate: f32,
        dt: f32,
    ) -> Quat {
        let horizontal_velocity = horizontal(velocity);
        if horizontal_velocity.length_squared() <= 0.1 {
            return current;
        }

        let target = Transform::from_translation(Vec3::ZERO)
            .looking_to(horizontal_velocity.normalize(), Vec3::Y)
            .rotation;
        current.slerp(target, smoothing_factor(turn_rate, dt))
    }

    pub fn smoothing_factor(rate: f32, dt: f32) -> f32 {
        (1.0 - (-rate.max(0.0) * dt.max(0.0)).exp()).clamp(0.0, 1.0)
    }

    fn clamp_velocity(mut velocity: Vec3, tuning: &FlightTuning) -> Vec3 {
        let horizontal_velocity = horizontal(velocity);
        let horizontal_speed = horizontal_velocity.length();

        if horizontal_speed > tuning.max_horizontal_speed {
            let horizontal_velocity = horizontal_velocity.normalize() * tuning.max_horizontal_speed;
            velocity.x = horizontal_velocity.x;
            velocity.z = horizontal_velocity.z;
        }

        velocity.y = velocity
            .y
            .clamp(-tuning.max_fall_speed, tuning.launch_speed);
        velocity
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
}

pub mod camera {
    use crate::movement::smoothing_factor;
    use bevy::prelude::*;

    #[derive(Component, Clone, Copy, Debug)]
    pub struct FollowCamera {
        pub distance: f32,
        pub height: f32,
        pub look_height: f32,
        pub look_ahead: f32,
        pub position_smoothing: f32,
        pub rotation_smoothing: f32,
        pub min_height: f32,
    }

    impl Default for FollowCamera {
        fn default() -> Self {
            Self {
                distance: 12.0,
                height: 5.0,
                look_height: 1.4,
                look_ahead: 2.0,
                position_smoothing: 10.0,
                rotation_smoothing: 14.0,
                min_height: 1.6,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct CameraFrame {
        pub position: Vec3,
        pub rotation: Quat,
        pub look_target: Vec3,
    }

    pub fn step_camera(
        current_position: Vec3,
        current_rotation: Quat,
        player_position: Vec3,
        player_forward: Vec3,
        player_velocity: Vec3,
        follow: &FollowCamera,
        dt: f32,
    ) -> CameraFrame {
        let direction = horizontal_follow_direction(player_velocity, player_forward);
        let look_target =
            player_position + Vec3::Y * follow.look_height + direction * follow.look_ahead;
        let mut desired_position =
            player_position - direction * follow.distance + Vec3::Y * follow.height;
        desired_position.y = desired_position.y.max(follow.min_height);

        let position = current_position.lerp(
            desired_position,
            smoothing_factor(follow.position_smoothing, dt),
        );
        let target_rotation = Transform::from_translation(position)
            .looking_at(look_target, Vec3::Y)
            .rotation;
        let rotation = current_rotation.slerp(
            target_rotation,
            smoothing_factor(follow.rotation_smoothing, dt),
        );

        CameraFrame {
            position,
            rotation,
            look_target,
        }
    }

    pub fn horizontal_follow_direction(velocity: Vec3, player_forward: Vec3) -> Vec3 {
        let horizontal_velocity = Vec3::new(velocity.x, 0.0, velocity.z);
        if horizontal_velocity.length_squared() > 1.0 {
            horizontal_velocity.normalize()
        } else {
            let horizontal_forward = Vec3::new(player_forward.x, 0.0, player_forward.z);
            if horizontal_forward.length_squared() > 0.0001 {
                horizontal_forward.normalize()
            } else {
                Vec3::Z
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn vertical_launch_velocity_does_not_pull_camera_under_player() {
            let follow = FollowCamera::default();
            let frame = step_camera(
                Vec3::new(0.0, 6.0, -12.0),
                Quat::IDENTITY,
                Vec3::new(0.0, 20.0, 0.0),
                Vec3::Z,
                Vec3::new(0.0, 40.0, 0.0),
                &follow,
                1.0,
            );

            assert!(frame.position.y > 20.0);
            assert!(frame.position.z < 0.0);
        }

        #[test]
        fn horizontal_velocity_controls_follow_direction() {
            let direction = horizontal_follow_direction(Vec3::new(10.0, 40.0, 0.0), Vec3::Z);
            assert!(direction.x > 0.99);
            assert!(direction.y.abs() < 0.001);
        }
    }
}

pub mod animation {
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

    pub fn part_pose(
        part: &CharacterPart,
        mode: FlightMode,
        velocity: Vec3,
        phase: f32,
    ) -> PartPose {
        let cycle = phase.sin();
        let mut translation = part.base_translation;
        let mut rotation = part.base_rotation;
        let mut visibility = PartVisibility::Inherited;

        match part.role {
            CharacterPartRole::Torso => {
                let pitch = match mode {
                    FlightMode::Grounded => 0.0,
                    FlightMode::Airborne => -0.12,
                    FlightMode::Gliding => -0.26,
                    FlightMode::Launching => 0.1,
                };
                translation.y += cycle.abs() * 0.018;
                rotation *= Quat::from_rotation_x(pitch);
            }
            CharacterPartRole::Head => {
                translation.y += cycle.abs() * 0.012;
                rotation *= Quat::from_rotation_x(-0.06);
            }
            CharacterPartRole::Arm(side) => {
                let sign = side.sign();
                let spread = match mode {
                    FlightMode::Grounded => cycle * 0.12,
                    FlightMode::Airborne => 0.65,
                    FlightMode::Gliding => 1.05,
                    FlightMode::Launching => 0.28,
                };
                let sweep = match mode {
                    FlightMode::Gliding => -0.52,
                    FlightMode::Launching => 0.22,
                    FlightMode::Airborne => -0.2,
                    FlightMode::Grounded => 0.0,
                };
                rotation *= Quat::from_rotation_z(sign * spread) * Quat::from_rotation_x(sweep);
            }
            CharacterPartRole::Leg(side) => {
                let sign = side.sign();
                let spread = match mode {
                    FlightMode::Grounded => cycle * 0.08,
                    FlightMode::Airborne => 0.12,
                    FlightMode::Gliding => 0.18,
                    FlightMode::Launching => 0.02,
                };
                let trail = match mode {
                    FlightMode::Gliding => 0.38,
                    FlightMode::Airborne => 0.18,
                    FlightMode::Launching => -0.12,
                    FlightMode::Grounded => 0.0,
                };
                rotation *= Quat::from_rotation_z(sign * spread) * Quat::from_rotation_x(trail);
            }
            CharacterPartRole::Wing(side) => {
                visibility = if mode == FlightMode::Gliding {
                    PartVisibility::Visible
                } else {
                    PartVisibility::Hidden
                };

                let sign = side.sign();
                let bank = (velocity.x * 0.012).clamp(-0.18, 0.18);
                let flutter = (phase * 2.4).sin() * 0.025;
                rotation *= Quat::from_rotation_z(sign * bank) * Quat::from_rotation_x(flutter);
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
    }
}
