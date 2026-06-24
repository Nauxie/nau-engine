pub mod asset_pipeline {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum VisualAssetKind {
        PlayerCharacter,
        Glider,
        IslandTerrain,
        IslandFoliage,
        IslandRock,
        IslandWater,
        RouteMarker,
        WeatherLayer,
        DistantImpostor,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum VisualAssetResidency {
        Always,
        StreamWindow,
        NearLod,
        FarLod,
        Weather,
    }

    impl VisualAssetResidency {
        pub fn is_stream_managed(self) -> bool {
            matches!(
                self,
                Self::StreamWindow | Self::NearLod | Self::FarLod | Self::Weather
            )
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct VisualAssetSpec {
        pub kind: VisualAssetKind,
        pub label: &'static str,
        pub gltf_scene_path: &'static str,
        pub residency: VisualAssetResidency,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum VisualAssetLoadState {
        Missing,
        Queued,
        Loading,
        Loaded,
        Failed,
    }

    impl VisualAssetLoadState {
        pub fn from_asset_exists(asset_exists: bool) -> Self {
            if asset_exists {
                Self::Queued
            } else {
                Self::Missing
            }
        }

        pub fn is_available(self) -> bool {
            self != Self::Missing
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct VisualAssetPipelineMetrics {
        pub slot_count: usize,
        pub gltf_scene_slot_count: usize,
        pub ready_slot_count: usize,
        pub placeholder_slot_count: usize,
        pub streaming_slot_count: usize,
        pub missing_slot_count: usize,
        pub queued_scene_count: usize,
        pub loading_scene_count: usize,
        pub loaded_scene_count: usize,
        pub failed_scene_count: usize,
        pub always_slot_count: usize,
        pub stream_window_slot_count: usize,
        pub near_lod_slot_count: usize,
        pub far_lod_slot_count: usize,
        pub weather_slot_count: usize,
    }

    pub const VISUAL_ASSET_SPECS: [VisualAssetSpec; 9] = [
        VisualAssetSpec {
            kind: VisualAssetKind::PlayerCharacter,
            label: "player character rig",
            gltf_scene_path: "models/player/player.glb",
            residency: VisualAssetResidency::Always,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::Glider,
            label: "player glider",
            gltf_scene_path: "models/player/glider.glb",
            residency: VisualAssetResidency::Always,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::IslandTerrain,
            label: "island terrain kit",
            gltf_scene_path: "models/world/island_terrain.glb",
            residency: VisualAssetResidency::StreamWindow,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::IslandFoliage,
            label: "island foliage kit",
            gltf_scene_path: "models/world/foliage.glb",
            residency: VisualAssetResidency::NearLod,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::IslandRock,
            label: "island rock kit",
            gltf_scene_path: "models/world/rocks.glb",
            residency: VisualAssetResidency::StreamWindow,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::IslandWater,
            label: "pond and water kit",
            gltf_scene_path: "models/world/water.glb",
            residency: VisualAssetResidency::NearLod,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::RouteMarker,
            label: "route marker kit",
            gltf_scene_path: "models/world/route_markers.glb",
            residency: VisualAssetResidency::Always,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::WeatherLayer,
            label: "weather cloud layer kit",
            gltf_scene_path: "models/world/weather_layers.glb",
            residency: VisualAssetResidency::Weather,
        },
        VisualAssetSpec {
            kind: VisualAssetKind::DistantImpostor,
            label: "sky island distant impostor kit",
            gltf_scene_path: "models/world/island_impostors.glb",
            residency: VisualAssetResidency::FarLod,
        },
    ];
    pub const VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SPECS.len();
    pub const GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT: usize = VISUAL_ASSET_SPECS.len();
    pub const STREAMING_VISUAL_ASSET_SLOT_COUNT: usize = 6;
    pub const ALWAYS_VISUAL_ASSET_SLOT_COUNT: usize = 3;
    pub const STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT: usize = 2;
    pub const NEAR_LOD_VISUAL_ASSET_SLOT_COUNT: usize = 2;
    pub const FAR_LOD_VISUAL_ASSET_SLOT_COUNT: usize = 1;
    pub const WEATHER_VISUAL_ASSET_SLOT_COUNT: usize = 1;

    pub fn visual_asset_pipeline_metrics(
        specs: &[VisualAssetSpec],
        mut asset_exists: impl FnMut(&str) -> bool,
    ) -> VisualAssetPipelineMetrics {
        visual_asset_pipeline_metrics_with_load_states(specs, |spec| {
            VisualAssetLoadState::from_asset_exists(asset_exists(spec.gltf_scene_path))
        })
    }

    pub fn visual_asset_pipeline_metrics_with_load_states(
        specs: &[VisualAssetSpec],
        mut asset_load_state: impl FnMut(&VisualAssetSpec) -> VisualAssetLoadState,
    ) -> VisualAssetPipelineMetrics {
        let mut metrics = VisualAssetPipelineMetrics::default();

        for spec in specs {
            metrics.slot_count += 1;
            if !spec.gltf_scene_path.is_empty() {
                metrics.gltf_scene_slot_count += 1;
            }
            match spec.residency {
                VisualAssetResidency::Always => metrics.always_slot_count += 1,
                VisualAssetResidency::StreamWindow => {
                    metrics.stream_window_slot_count += 1;
                    metrics.streaming_slot_count += 1;
                }
                VisualAssetResidency::NearLod => {
                    metrics.near_lod_slot_count += 1;
                    metrics.streaming_slot_count += 1;
                }
                VisualAssetResidency::FarLod => {
                    metrics.far_lod_slot_count += 1;
                    metrics.streaming_slot_count += 1;
                }
                VisualAssetResidency::Weather => {
                    metrics.weather_slot_count += 1;
                    metrics.streaming_slot_count += 1;
                }
            }

            match asset_load_state(spec) {
                VisualAssetLoadState::Missing => {
                    metrics.placeholder_slot_count += 1;
                    metrics.missing_slot_count += 1;
                }
                VisualAssetLoadState::Queued => {
                    metrics.ready_slot_count += 1;
                    metrics.queued_scene_count += 1;
                }
                VisualAssetLoadState::Loading => {
                    metrics.ready_slot_count += 1;
                    metrics.queued_scene_count += 1;
                    metrics.loading_scene_count += 1;
                }
                VisualAssetLoadState::Loaded => {
                    metrics.ready_slot_count += 1;
                    metrics.queued_scene_count += 1;
                    metrics.loaded_scene_count += 1;
                }
                VisualAssetLoadState::Failed => {
                    metrics.ready_slot_count += 1;
                    metrics.queued_scene_count += 1;
                    metrics.failed_scene_count += 1;
                }
            }
        }

        metrics
    }

    #[cfg(test)]
    mod tests {
        use crate::asset_pipeline::{
            ALWAYS_VISUAL_ASSET_SLOT_COUNT, FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT, STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
            WEATHER_VISUAL_ASSET_SLOT_COUNT,
        };

        use super::*;

        #[test]
        fn asset_specs_cover_streamed_world_and_player_slots() {
            let metrics = visual_asset_pipeline_metrics(&VISUAL_ASSET_SPECS, |_| false);

            assert_eq!(metrics.slot_count, VISUAL_ASSET_SPECS.len());
            assert!(metrics.gltf_scene_slot_count >= 8);
            assert!(metrics.streaming_slot_count >= 5);
            assert_eq!(metrics.ready_slot_count, 0);
            assert_eq!(metrics.placeholder_slot_count, VISUAL_ASSET_SPECS.len());
            assert_eq!(metrics.missing_slot_count, VISUAL_ASSET_SPECS.len());
            assert_eq!(metrics.queued_scene_count, 0);
            assert_eq!(metrics.slot_count, VISUAL_ASSET_SLOT_COUNT);
            assert_eq!(
                metrics.gltf_scene_slot_count,
                GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT
            );
            assert_eq!(
                metrics.streaming_slot_count,
                STREAMING_VISUAL_ASSET_SLOT_COUNT
            );
            assert_eq!(metrics.always_slot_count, ALWAYS_VISUAL_ASSET_SLOT_COUNT);
            assert_eq!(
                metrics.stream_window_slot_count,
                STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT
            );
            assert_eq!(
                metrics.near_lod_slot_count,
                NEAR_LOD_VISUAL_ASSET_SLOT_COUNT
            );
            assert_eq!(metrics.far_lod_slot_count, FAR_LOD_VISUAL_ASSET_SLOT_COUNT);
            assert_eq!(metrics.weather_slot_count, WEATHER_VISUAL_ASSET_SLOT_COUNT);
            assert!(
                VISUAL_ASSET_SPECS
                    .iter()
                    .any(|spec| spec.kind == VisualAssetKind::PlayerCharacter)
            );
            assert!(
                VISUAL_ASSET_SPECS
                    .iter()
                    .any(|spec| spec.kind == VisualAssetKind::DistantImpostor)
            );
        }

        #[test]
        fn asset_metrics_count_ready_and_placeholder_slots() {
            let metrics = visual_asset_pipeline_metrics(&VISUAL_ASSET_SPECS, |path| {
                path == "models/player/player.glb" || path == "models/world/foliage.glb"
            });

            assert_eq!(metrics.ready_slot_count, 2);
            assert_eq!(metrics.queued_scene_count, 2);
            assert_eq!(metrics.missing_slot_count, VISUAL_ASSET_SPECS.len() - 2);
            assert_eq!(metrics.placeholder_slot_count, VISUAL_ASSET_SPECS.len() - 2);
        }

        #[test]
        fn asset_metrics_track_bevy_load_state_buckets() {
            let metrics =
                visual_asset_pipeline_metrics_with_load_states(&VISUAL_ASSET_SPECS, |spec| {
                    match spec.kind {
                        VisualAssetKind::PlayerCharacter => VisualAssetLoadState::Loading,
                        VisualAssetKind::Glider => VisualAssetLoadState::Loaded,
                        VisualAssetKind::DistantImpostor => VisualAssetLoadState::Failed,
                        _ => VisualAssetLoadState::Missing,
                    }
                });

            assert_eq!(metrics.ready_slot_count, 3);
            assert_eq!(metrics.placeholder_slot_count, VISUAL_ASSET_SPECS.len() - 3);
            assert_eq!(metrics.queued_scene_count, 3);
            assert_eq!(metrics.loading_scene_count, 1);
            assert_eq!(metrics.loaded_scene_count, 1);
            assert_eq!(metrics.failed_scene_count, 1);
        }
    }
}

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
        pub turn_rate: f32,
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
                lateral_accel: 14.0,
                glide_forward_accel: 12.0,
                glide_lateral_accel: 9.0,
                glide_brake_drag: 0.42,
                air_brake_accel: 46.0,
                glide_brake_accel: 38.0,
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
                turn_rate: 8.0,
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

    fn clamp_velocity(
        mut velocity: Vec3,
        tuning: &FlightTuning,
        max_horizontal_speed: f32,
    ) -> Vec3 {
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
        dt: f32,
    ) {
        let forward = horizontal_or(forward, Vec3::Z);
        let forward_speed = horizontal(*velocity).dot(forward);

        if forward_speed > 0.0 {
            let reduction = forward_speed.min(brake_accel.max(0.0) * dt);
            velocity.x -= forward.x * reduction;
            velocity.z -= forward.z * reduction;
            return;
        }

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

pub mod environment {
    use bevy::prelude::*;

    const DIRECTION_EPSILON: f32 = 0.0001;
    const FIELD_PAIR_EPSILON: f32 = 0.001;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum WindFieldKind {
        Crosswind,
        Updraft,
    }

    #[derive(Component, Clone, Copy, Debug, PartialEq)]
    pub struct WindField {
        pub center: Vec3,
        pub half_extents: Vec3,
        pub direction: Vec3,
        pub visual_speed: f32,
        pub kind: WindFieldKind,
    }

    impl WindField {
        pub fn crosswind(
            center: Vec3,
            half_extents: Vec3,
            direction: Vec3,
            visual_speed: f32,
        ) -> Self {
            let horizontal_direction = Vec3::new(direction.x, 0.0, direction.z);
            let direction = if horizontal_direction.length_squared() > DIRECTION_EPSILON {
                horizontal_direction.normalize()
            } else {
                Vec3::X
            };

            Self {
                center,
                half_extents,
                direction,
                visual_speed: visual_speed.max(0.0),
                kind: WindFieldKind::Crosswind,
            }
        }

        pub fn updraft(center: Vec3, half_extents: Vec3, visual_speed: f32) -> Self {
            Self {
                center,
                half_extents,
                direction: Vec3::Y,
                visual_speed: visual_speed.max(0.0),
                kind: WindFieldKind::Updraft,
            }
        }

        pub fn contains(self, position: Vec3) -> bool {
            let offset = position - self.center;
            offset.x.abs() <= self.half_extents.x
                && offset.y.abs() <= self.half_extents.y
                && offset.z.abs() <= self.half_extents.z
        }

        pub fn flow_vector(self) -> Vec3 {
            self.direction * self.visual_speed
        }

        pub fn stream_origin(self, index: usize, stream_count: usize) -> Vec3 {
            let stream_count = stream_count.max(1);
            let columns = (stream_count as f32).sqrt().ceil() as usize;
            let column = index % columns;
            let row = (index / columns).min(columns.saturating_sub(1));
            let x_t = centered_unit(column, columns);
            let y_t = centered_unit(row, columns);

            match self.kind {
                WindFieldKind::Crosswind => {
                    let leading_edge = self.center - self.direction * self.half_extents.x;
                    leading_edge
                        + Vec3::Y * (y_t * self.half_extents.y * 0.72)
                        + Vec3::Z * (x_t * self.half_extents.z * 0.72)
                }
                WindFieldKind::Updraft => {
                    let base = self.center - Vec3::Y * self.half_extents.y;
                    base + Vec3::X * (x_t * self.half_extents.x * 0.72)
                        + Vec3::Z * (y_t * self.half_extents.z * 0.72)
                }
            }
        }
    }

    #[derive(Component, Clone, Copy, Debug, PartialEq)]
    pub struct LiftField {
        pub center: Vec3,
        pub half_extents: Vec3,
        pub lift_accel: f32,
        pub max_upward_speed: f32,
    }

    impl LiftField {
        pub fn updraft(
            center: Vec3,
            half_extents: Vec3,
            lift_accel: f32,
            max_upward_speed: f32,
        ) -> Self {
            Self {
                center,
                half_extents: half_extents.max(Vec3::splat(0.1)),
                lift_accel: lift_accel.max(0.0),
                max_upward_speed: max_upward_speed.max(0.0),
            }
        }

        pub fn contains(self, position: Vec3) -> bool {
            let offset = position - self.center;
            offset.x.abs() <= self.half_extents.x
                && offset.y.abs() <= self.half_extents.y
                && offset.z.abs() <= self.half_extents.z
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct LiftRouteNode {
        pub name: &'static str,
        pub center: Vec3,
        pub half_extents: Vec3,
        pub lift_accel: f32,
        pub max_upward_speed: f32,
        pub visual_speed: f32,
    }

    impl LiftRouteNode {
        pub fn lift_field(self) -> LiftField {
            LiftField::updraft(
                self.center,
                self.half_extents,
                self.lift_accel,
                self.max_upward_speed,
            )
        }

        pub fn visual_field(self) -> WindField {
            WindField::updraft(self.center, self.half_extents, self.visual_speed)
        }
    }

    pub const GAMEPLAY_LIFT_ROUTE: [LiftRouteNode; 2] = [
        LiftRouteNode {
            name: "near route updraft",
            center: Vec3::new(38.0, 68.0, -112.0),
            half_extents: Vec3::new(20.0, 34.0, 22.0),
            lift_accel: 28.0,
            max_upward_speed: 20.0,
            visual_speed: 12.0,
        },
        LiftRouteNode {
            name: "distant recovery updraft",
            center: Vec3::new(24.0, 74.0, -430.0),
            half_extents: Vec3::new(26.0, 42.0, 26.0),
            lift_accel: 24.0,
            max_upward_speed: 22.0,
            visual_speed: 14.0,
        },
    ];

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct AerialPowerUp {
        pub name: &'static str,
        pub center: Vec3,
        pub radius_m: f32,
        pub forward_direction: Vec3,
        pub forward_speed_boost: f32,
        pub upward_speed_boost: f32,
        pub max_upward_speed: f32,
        pub effect_duration_secs: f32,
    }

    impl AerialPowerUp {
        pub fn contains(self, position: Vec3) -> bool {
            (position - self.center).length() <= self.radius_m.max(0.0)
        }
    }

    pub const AERIAL_POWER_UP_ROUTE: [AerialPowerUp; 3] = [
        AerialPowerUp {
            name: "midair gust gate",
            center: Vec3::new(26.0, 92.0, -126.0),
            radius_m: 24.0,
            forward_direction: Vec3::NEG_Z,
            forward_speed_boost: 7.5,
            upward_speed_boost: 5.0,
            max_upward_speed: 20.0,
            effect_duration_secs: 0.75,
        },
        AerialPowerUp {
            name: "drift boost gate",
            center: Vec3::new(32.0, 124.0, -300.0),
            radius_m: 26.0,
            forward_direction: Vec3::NEG_Z,
            forward_speed_boost: 7.0,
            upward_speed_boost: 4.0,
            max_upward_speed: 18.0,
            effect_duration_secs: 0.75,
        },
        AerialPowerUp {
            name: "recovery lift gate",
            center: Vec3::new(42.0, 114.0, -430.0),
            radius_m: 26.0,
            forward_direction: Vec3::NEG_Z,
            forward_speed_boost: 6.0,
            upward_speed_boost: 5.0,
            max_upward_speed: 18.0,
            effect_duration_secs: 0.75,
        },
    ];

    pub fn apply_aerial_power_up(mut velocity: Vec3, power_up: AerialPowerUp) -> Vec3 {
        let forward = horizontal_or(power_up.forward_direction, Vec3::NEG_Z);
        velocity += forward * power_up.forward_speed_boost.max(0.0);

        if velocity.y < power_up.max_upward_speed {
            velocity.y =
                (velocity.y + power_up.upward_speed_boost.max(0.0)).min(power_up.max_upward_speed);
        }

        velocity
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct LiftApplication {
        pub velocity: Vec3,
        pub active_fields: usize,
        pub applied_delta_y: f32,
    }

    pub fn apply_lift_fields(
        position: Vec3,
        mut velocity: Vec3,
        fields: impl IntoIterator<Item = LiftField>,
        dt: f32,
        enabled: bool,
    ) -> LiftApplication {
        let mut active_fields = 0;
        let mut lift_accel = 0.0_f32;
        let mut max_upward_speed = velocity.y;

        for field in fields {
            if field.contains(position) {
                active_fields += 1;
                lift_accel += field.lift_accel;
                max_upward_speed = max_upward_speed.max(field.max_upward_speed);
            }
        }

        let applied_delta_y = if enabled && active_fields > 0 && velocity.y < max_upward_speed {
            let delta = (lift_accel * dt.max(0.0)).min(max_upward_speed - velocity.y);
            velocity.y += delta;
            delta
        } else {
            0.0
        };

        LiftApplication {
            velocity,
            active_fields,
            applied_delta_y,
        }
    }

    pub fn active_lift_fields_at(
        position: Vec3,
        fields: impl IntoIterator<Item = LiftField>,
    ) -> usize {
        fields
            .into_iter()
            .filter(|field| field.contains(position))
            .count()
    }

    fn centered_unit(index: usize, count: usize) -> f32 {
        if count <= 1 {
            0.0
        } else {
            (index as f32 / (count - 1) as f32) * 2.0 - 1.0
        }
    }

    fn horizontal_or(v: Vec3, fallback: Vec3) -> Vec3 {
        let horizontal = Vec3::new(v.x, 0.0, v.z);
        if horizontal.length_squared() > DIRECTION_EPSILON {
            horizontal.normalize()
        } else {
            fallback.normalize()
        }
    }

    pub fn visible_fields_at(position: Vec3, fields: impl IntoIterator<Item = WindField>) -> usize {
        fields
            .into_iter()
            .filter(|field| field.contains(position))
            .count()
    }

    pub fn readable_lift_fields_at(
        position: Vec3,
        lift_fields: impl IntoIterator<Item = LiftField>,
        visual_fields: impl IntoIterator<Item = WindField>,
    ) -> usize {
        let visible_updrafts = visual_fields
            .into_iter()
            .filter(|field| field.kind == WindFieldKind::Updraft && field.contains(position))
            .collect::<Vec<_>>();

        lift_fields
            .into_iter()
            .filter(|lift| {
                lift.contains(position)
                    && visible_updrafts
                        .iter()
                        .any(|visual| lift_matches_visual_updraft(*lift, *visual))
            })
            .count()
    }

    fn lift_matches_visual_updraft(lift: LiftField, visual: WindField) -> bool {
        vec3_near(lift.center, visual.center) && vec3_near(lift.half_extents, visual.half_extents)
    }

    fn vec3_near(left: Vec3, right: Vec3) -> bool {
        (left - right).abs().max_element() <= FIELD_PAIR_EPSILON
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct WindSwayMotion {
        pub offset: Vec3,
        pub rotation: Quat,
        pub scale: Vec3,
        pub offset_magnitude_m: f32,
    }

    pub fn wind_sway_motion(
        elapsed_secs: f32,
        phase: f32,
        amplitude_m: f32,
        bend_radians: f32,
        gust_speed: f32,
        wind_direction: Vec3,
    ) -> WindSwayMotion {
        let direction = horizontal_or(wind_direction, Vec3::X);
        let time = elapsed_secs.max(0.0);
        let amplitude = amplitude_m.max(0.0);
        let bend = bend_radians.max(0.0);
        let speed = gust_speed.max(0.0);
        let wave = (time * speed + phase).sin();
        let gust = 0.62 + 0.38 * (time * speed * 0.43 + phase * 1.7).sin();
        let strength = wave * gust.clamp(0.2, 1.0);
        let flutter = (time * speed * 1.9 + phase * 0.6).cos() * 0.12;
        let axis = Vec3::new(direction.z, 0.0, -direction.x).normalize_or_zero();
        let rotation_axis = if axis.length_squared() > DIRECTION_EPSILON {
            axis
        } else {
            Vec3::Z
        };
        let offset = direction * amplitude * strength
            + Vec3::Y * amplitude * flutter * (0.5 + strength.abs() * 0.5);
        let scale_pulse = 1.0 + strength.abs() * 0.018;

        WindSwayMotion {
            offset,
            rotation: Quat::from_axis_angle(rotation_axis, bend * strength),
            scale: Vec3::new(scale_pulse, 1.0 - strength.abs() * 0.01, scale_pulse),
            offset_magnitude_m: offset.length(),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn crosswind_is_visual_only_and_horizontal() {
            let field = WindField::crosswind(
                Vec3::ZERO,
                Vec3::new(4.0, 2.0, 4.0),
                Vec3::new(1.0, 1.0, 0.0),
                8.0,
            );

            assert_eq!(field.kind, WindFieldKind::Crosswind);
            assert_eq!(field.direction, Vec3::X);
            assert_eq!(field.flow_vector(), Vec3::new(8.0, 0.0, 0.0));
        }

        #[test]
        fn updraft_is_visual_and_vertical() {
            let field = WindField::updraft(Vec3::ZERO, Vec3::new(2.0, 8.0, 2.0), 6.0);

            assert_eq!(field.kind, WindFieldKind::Updraft);
            assert_eq!(field.flow_vector(), Vec3::new(0.0, 6.0, 0.0));
        }

        #[test]
        fn field_contains_only_inside_bounds() {
            let field = WindField::crosswind(Vec3::ZERO, Vec3::splat(4.0), Vec3::X, 8.0);

            assert!(field.contains(Vec3::new(4.0, 0.0, 0.0)));
            assert!(!field.contains(Vec3::new(4.1, 0.0, 0.0)));
        }

        #[test]
        fn stream_origins_stay_inside_visual_field() {
            let field = WindField::updraft(Vec3::ZERO, Vec3::new(4.0, 8.0, 4.0), 6.0);

            for index in 0..16 {
                assert!(field.contains(field.stream_origin(index, 16)));
            }
        }

        #[test]
        fn visible_field_count_is_deterministic() {
            let near = WindField::crosswind(Vec3::ZERO, Vec3::splat(4.0), Vec3::X, 8.0);
            let far = WindField::updraft(Vec3::new(20.0, 0.0, 0.0), Vec3::splat(4.0), 6.0);

            assert_eq!(visible_fields_at(Vec3::ZERO, [near, far]), 1);
            assert_eq!(visible_fields_at(Vec3::new(20.0, 0.0, 0.0), [near, far]), 1);
            assert_eq!(visible_fields_at(Vec3::new(10.0, 0.0, 0.0), [near, far]), 0);
        }

        #[test]
        fn gameplay_lift_route_pairs_lift_and_visual_volumes() {
            for node in GAMEPLAY_LIFT_ROUTE {
                let lift = node.lift_field();
                let visual = node.visual_field();

                assert_eq!(lift.center, visual.center);
                assert_eq!(lift.half_extents, visual.half_extents);
                assert!(lift.contains(node.center));
                assert!(visual.contains(node.center));
                assert_eq!(visual.kind, WindFieldKind::Updraft);
            }
        }

        #[test]
        fn aerial_power_up_route_is_collectible_and_directional() {
            for power_up in AERIAL_POWER_UP_ROUTE {
                assert!(power_up.contains(power_up.center));
                assert!(power_up.radius_m >= 20.0);
                assert!(power_up.forward_speed_boost > 0.0);
                assert!(power_up.upward_speed_boost > 0.0);
                assert!(power_up.effect_duration_secs > 0.0);
            }
        }

        #[test]
        fn aerial_power_up_applies_capped_forward_and_upward_boost() {
            let power_up = AERIAL_POWER_UP_ROUTE[0];
            let boosted = apply_aerial_power_up(Vec3::new(0.0, 16.0, -12.0), power_up);

            assert!(boosted.z < -12.0);
            assert!(boosted.y > 16.0);
            assert!(boosted.y <= power_up.max_upward_speed);

            let already_fast_up = apply_aerial_power_up(Vec3::new(0.0, 28.0, -12.0), power_up);
            assert_eq!(already_fast_up.y, 28.0);
        }

        #[test]
        fn readable_lift_requires_overlapping_paired_updraft_visual() {
            let node = GAMEPLAY_LIFT_ROUTE[0];
            let lift = node.lift_field();
            let paired_visual = node.visual_field();
            let crosswind =
                WindField::crosswind(node.center, node.half_extents, Vec3::X, node.visual_speed);
            let shifted_visual =
                WindField::updraft(node.center + Vec3::X, node.half_extents, node.visual_speed);

            assert_eq!(
                readable_lift_fields_at(node.center, [lift], [paired_visual]),
                1
            );
            assert_eq!(readable_lift_fields_at(node.center, [lift], [crosswind]), 0);
            assert_eq!(
                readable_lift_fields_at(node.center, [lift], [shifted_visual]),
                0
            );
        }

        #[test]
        fn lift_field_only_applies_inside_bounds_when_enabled() {
            let field = LiftField::updraft(Vec3::ZERO, Vec3::splat(4.0), 20.0, 12.0);
            let outside =
                apply_lift_fields(Vec3::new(10.0, 0.0, 0.0), Vec3::ZERO, [field], 0.5, true);
            let disabled = apply_lift_fields(Vec3::ZERO, Vec3::ZERO, [field], 0.5, false);
            let active = apply_lift_fields(Vec3::ZERO, Vec3::ZERO, [field], 0.5, true);

            assert_eq!(outside.active_fields, 0);
            assert_eq!(outside.velocity, Vec3::ZERO);
            assert_eq!(disabled.active_fields, 1);
            assert_eq!(disabled.applied_delta_y, 0.0);
            assert_eq!(active.active_fields, 1);
            assert!(active.velocity.y > 0.0);
            assert!(active.velocity.y <= field.max_upward_speed);
        }

        #[test]
        fn wind_sway_motion_is_bounded_and_horizontal() {
            let motion = wind_sway_motion(1.2, 0.4, 0.35, 0.08, 1.6, Vec3::new(0.0, 4.0, -2.0));

            assert!(motion.offset.z < 0.0);
            assert!(motion.offset.x.abs() < 0.001);
            assert!(motion.offset_magnitude_m <= 0.38);
            assert!(motion.scale.x > 1.0);
            assert!(motion.scale.y <= 1.0);
        }

        #[test]
        fn wind_sway_motion_clamps_negative_inputs_to_stillness() {
            let motion = wind_sway_motion(-1.0, 0.0, -0.2, -0.1, -2.0, Vec3::ZERO);

            assert_eq!(motion.offset, Vec3::ZERO);
            assert_eq!(motion.scale, Vec3::ONE);
            assert_eq!(motion.offset_magnitude_m, 0.0);
        }
    }
}

pub mod world {
    use crate::environment::{GAMEPLAY_LIFT_ROUTE, LiftRouteNode};
    use crate::movement::{FlightMode, FlightState};
    use bevy::prelude::*;

    pub const PLAYER_STANDING_OFFSET: f32 = 0.24;
    pub const START_FLOOR_Y: f32 = 28.0;
    pub const START_POSITION: Vec3 = Vec3::new(0.0, START_FLOOR_Y, 0.0);
    pub const RECOVERY_BRANCH_ISLANDS: [&str; 2] = ["sunlit terrace", "western refuge"];
    pub const STREAM_CHUNK_SIZE_M: f32 = 160.0;
    pub const STREAM_ACTIVE_CHUNK_RADIUS: i32 = 2;
    pub const LOD_NEAR_DISTANCE_M: f32 = 220.0;
    pub const LOD_MID_DISTANCE_M: f32 = 520.0;
    pub const TERRAIN_MAX_RISE_M: f32 = 0.45;
    pub const TERRAIN_MAX_DROP_M: f32 = 0.75;
    pub const TERRAIN_VISUAL_FOOTING_OFFSET_M: f32 = 0.18;
    const GROUND_CONTACT_EPSILON: f32 = 0.05;
    const GROUND_CONTACT_HORIZONTAL_DAMPING: f32 = 0.58;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct StreamChunkCoord {
        pub x: i32,
        pub z: i32,
    }

    impl StreamChunkCoord {
        pub fn from_world(position: Vec3) -> Self {
            Self {
                x: (position.x / STREAM_CHUNK_SIZE_M).floor() as i32,
                z: (position.z / STREAM_CHUNK_SIZE_M).floor() as i32,
            }
        }

        pub fn is_inside_active_window(self, center: Self) -> bool {
            (self.x - center.x).abs() <= STREAM_ACTIVE_CHUNK_RADIUS
                && (self.z - center.z).abs() <= STREAM_ACTIVE_CHUNK_RADIUS
        }
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct StreamingLodStats {
        pub player_chunk: StreamChunkCoord,
        pub active_chunk_count: usize,
        pub active_island_count: usize,
        pub near_lod_islands: usize,
        pub mid_lod_islands: usize,
        pub far_lod_islands: usize,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum LodBand {
        Near,
        Mid,
        Far,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum StreamActivation {
        Active,
        Inactive,
    }

    impl StreamActivation {
        pub fn is_active(self) -> bool {
            self == Self::Active
        }
    }

    #[derive(Resource, Clone, Debug)]
    pub struct SkyRoute {
        pub fallback_floor_y: f32,
        islands: Vec<SkyIsland>,
    }

    impl Default for SkyRoute {
        fn default() -> Self {
            Self {
                fallback_floor_y: PLAYER_STANDING_OFFSET,
                islands: vec![
                    SkyIsland::new(
                        "launch mesa",
                        Vec3::new(0.0, START_FLOOR_Y, 0.0),
                        Vec2::new(34.0, 28.0),
                        11.0,
                        false,
                    ),
                    SkyIsland::new(
                        "midpoint shelf",
                        Vec3::new(-12.0, 44.0, -128.0),
                        Vec2::new(28.0, 24.0),
                        9.0,
                        false,
                    ),
                    SkyIsland::new(
                        "landing garden",
                        Vec3::new(-38.0, 52.0, -263.0),
                        Vec2::new(46.0, 36.0),
                        12.0,
                        true,
                    ),
                    SkyIsland::new(
                        "distant crown",
                        Vec3::new(82.0, 62.0, -356.0),
                        Vec2::new(38.0, 32.0),
                        14.0,
                        false,
                    ),
                    SkyIsland::new(
                        "wind overlook",
                        Vec3::new(-112.0, 52.0, -204.0),
                        Vec2::new(30.0, 26.0),
                        10.0,
                        false,
                    ),
                    SkyIsland::new(
                        "copper stair",
                        Vec3::new(36.0, 58.0, -332.0),
                        Vec2::new(26.0, 22.0),
                        9.0,
                        false,
                    ),
                    SkyIsland::new(
                        "sunlit terrace",
                        Vec3::new(42.0, 64.0, -444.0),
                        Vec2::new(54.0, 30.0),
                        13.0,
                        false,
                    ),
                    SkyIsland::new(
                        "western refuge",
                        Vec3::new(-150.0, 70.0, -432.0),
                        Vec2::new(38.0, 30.0),
                        12.0,
                        false,
                    ),
                    SkyIsland::new(
                        "storm porch",
                        Vec3::new(-74.0, 76.0, -548.0),
                        Vec2::new(42.0, 28.0),
                        15.0,
                        false,
                    ),
                    SkyIsland::new(
                        "high orchard",
                        Vec3::new(18.0, 82.0, -662.0),
                        Vec2::new(58.0, 38.0),
                        14.0,
                        false,
                    ),
                    SkyIsland::new(
                        "far needle",
                        Vec3::new(142.0, 92.0, -742.0),
                        Vec2::new(24.0, 22.0),
                        18.0,
                        false,
                    ),
                    SkyIsland::new(
                        "sapphire basin",
                        Vec3::new(-58.0, 88.0, -818.0),
                        Vec2::new(46.0, 34.0),
                        16.0,
                        false,
                    ),
                ],
            }
        }
    }

    impl SkyRoute {
        pub fn islands(&self) -> &[SkyIsland] {
            &self.islands
        }

        pub fn route_objectives(&self, island_name: Option<&str>) -> Vec<RouteObjective> {
            let mut objectives = vec![RouteObjective::fly_through(GAMEPLAY_LIFT_ROUTE[0])];
            if self
                .tracked_target_island(island_name)
                .is_some_and(|island| !island.is_target)
            {
                objectives.push(RouteObjective::fly_through(GAMEPLAY_LIFT_ROUTE[1]));
            }
            if let Some(target) = self.tracked_target_island(island_name) {
                objectives.push(RouteObjective::land_on(target));
            }

            objectives
        }

        pub fn streaming_lod_stats(&self, position: Vec3) -> StreamingLodStats {
            let player_chunk = StreamChunkCoord::from_world(position);
            let active_chunk_width = STREAM_ACTIVE_CHUNK_RADIUS * 2 + 1;
            let mut stats = StreamingLodStats {
                player_chunk,
                active_chunk_count: (active_chunk_width * active_chunk_width) as usize,
                ..default()
            };

            for island in &self.islands {
                if island.stream_activation(position).is_active() {
                    stats.active_island_count += 1;
                }

                match island.lod_band(position) {
                    LodBand::Near => stats.near_lod_islands += 1,
                    LodBand::Mid => stats.mid_lod_islands += 1,
                    LodBand::Far => stats.far_lod_islands += 1,
                }
            }

            stats
        }

        pub fn ground_at(&self, position: Vec3) -> GroundSurface {
            self.islands
                .iter()
                .copied()
                .filter(|island| island.contains_horizontal(position))
                .map(|island| GroundSurface::from_island_at(island, position))
                .max_by(|a, b| a.floor_y.total_cmp(&b.floor_y))
                .unwrap_or(GroundSurface {
                    floor_y: self.fallback_floor_y,
                    is_target: false,
                    island_name: None,
                })
        }

        pub fn is_grounded_at(&self, position: Vec3) -> bool {
            let ground = self.ground_at(position);
            position.y <= ground.floor_y + GROUND_CONTACT_EPSILON
        }

        pub fn resolve_ground_contact(&self, state: FlightState) -> FlightState {
            self.resolve_ground_contact_with_landing(state, true)
        }

        pub fn resolve_ground_contact_after_step(
            &self,
            state: FlightState,
            was_grounded: bool,
        ) -> FlightState {
            self.resolve_ground_contact_with_landing(state, !was_grounded)
        }

        fn resolve_ground_contact_with_landing(
            &self,
            mut state: FlightState,
            apply_landing_damping: bool,
        ) -> FlightState {
            let ground = self.ground_at(state.position);
            if state.position.y <= ground.floor_y + GROUND_CONTACT_EPSILON {
                state.position.y = ground.floor_y;
                if apply_landing_damping {
                    state.velocity.x *= GROUND_CONTACT_HORIZONTAL_DAMPING;
                    state.velocity.z *= GROUND_CONTACT_HORIZONTAL_DAMPING;
                }
                state.velocity.y = state.velocity.y.max(0.0);
                state.controller.launch_timer = 0.0;
                state.controller.launch_available = true;
                state.controller.mode = FlightMode::Grounded;
            } else if state.controller.mode == FlightMode::Grounded {
                state.controller.mode = FlightMode::Airborne;
                state.controller.launch_timer = 0.0;
            }

            state
        }

        pub fn target_distance(&self, position: Vec3) -> f32 {
            self.target_distance_to(position, None)
        }

        pub fn target_distance_to(&self, position: Vec3, island_name: Option<&str>) -> f32 {
            self.tracked_target_island(island_name)
                .map(|island| island.horizontal_distance(position))
                .unwrap_or(0.0)
        }

        pub fn on_landing_target(&self, position: Vec3, mode: FlightMode) -> bool {
            self.on_landing_target_named(position, mode, None)
        }

        pub fn on_landing_target_named(
            &self,
            position: Vec3,
            mode: FlightMode,
            island_name: Option<&str>,
        ) -> bool {
            let ground = self.ground_at(position);
            self.tracked_target_island(island_name)
                .is_some_and(|island| ground.island_name == Some(island.name))
                && mode == FlightMode::Grounded
                && (position.y - ground.floor_y).abs() <= 0.1
        }

        pub fn target_island(&self) -> Option<SkyIsland> {
            self.islands.iter().copied().find(|island| island.is_target)
        }

        pub fn island_named(&self, name: &str) -> Option<SkyIsland> {
            self.islands
                .iter()
                .copied()
                .find(|island| island.name == name)
        }

        fn tracked_target_island(&self, island_name: Option<&str>) -> Option<SkyIsland> {
            island_name
                .and_then(|name| self.island_named(name))
                .or_else(|| self.target_island())
        }
    }

    pub fn is_recovery_branch_island(name: &str) -> bool {
        RECOVERY_BRANCH_ISLANDS.contains(&name)
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum RouteObjectiveKind {
        FlyThrough,
        Land,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct RouteObjective {
        pub label: &'static str,
        pub position: Vec3,
        pub radius_m: f32,
        pub kind: RouteObjectiveKind,
        pub island_name: Option<&'static str>,
    }

    impl RouteObjective {
        pub fn fly_through(node: LiftRouteNode) -> Self {
            Self {
                label: node.name,
                position: node.center,
                radius_m: node.half_extents.x.max(node.half_extents.z) + 8.0,
                kind: RouteObjectiveKind::FlyThrough,
                island_name: None,
            }
        }

        pub fn land_on(island: SkyIsland) -> Self {
            Self {
                label: island.name,
                position: island.center,
                radius_m: island.half_extents.x.max(island.half_extents.y),
                kind: RouteObjectiveKind::Land,
                island_name: Some(island.name),
            }
        }

        pub fn horizontal_distance(self, position: Vec3) -> f32 {
            Vec2::new(position.x - self.position.x, position.z - self.position.z).length()
        }

        pub fn is_complete(self, route: &SkyRoute, position: Vec3, mode: FlightMode) -> bool {
            match self.kind {
                RouteObjectiveKind::FlyThrough => {
                    self.horizontal_distance(position) <= self.radius_m
                }
                RouteObjectiveKind::Land => {
                    route.on_landing_target_named(position, mode, self.island_name)
                }
            }
        }
    }

    #[derive(Component, Clone, Copy, Debug, PartialEq)]
    pub struct SkyIsland {
        pub name: &'static str,
        pub center: Vec3,
        pub half_extents: Vec2,
        pub thickness: f32,
        pub is_target: bool,
    }

    impl SkyIsland {
        pub fn new(
            name: &'static str,
            center: Vec3,
            half_extents: Vec2,
            thickness: f32,
            is_target: bool,
        ) -> Self {
            Self {
                name,
                center,
                half_extents,
                thickness: thickness.max(1.0),
                is_target,
            }
        }

        pub fn floor_y(self) -> f32 {
            self.center.y
        }

        pub fn mesh_top_y(self) -> f32 {
            self.floor_y() - PLAYER_STANDING_OFFSET
        }

        pub fn terrain_surface_y_at(self, position: Vec3) -> f32 {
            let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
            let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
            let radius = Vec2::new(dx, dz).length().clamp(0.0, 1.0);
            let angle = dz.atan2(dx);

            self.terrain_surface_y_at_polar(radius, angle)
        }

        pub fn terrain_surface_y_at_polar(self, radius: f32, angle: f32) -> f32 {
            self.floor_y() + self.terrain_relief_m(radius, angle)
        }

        pub fn mesh_top_y_at(self, position: Vec3) -> f32 {
            self.terrain_surface_y_at(position) - TERRAIN_VISUAL_FOOTING_OFFSET_M
        }

        pub fn mesh_top_y_at_polar(self, radius: f32, angle: f32) -> f32 {
            self.terrain_surface_y_at_polar(radius, angle) - TERRAIN_VISUAL_FOOTING_OFFSET_M
        }

        pub fn terrain_relief_m(self, radius: f32, angle: f32) -> f32 {
            let radius = radius.clamp(0.0, 1.0);
            if radius <= f32::EPSILON {
                return 0.0;
            }

            let phase = self.terrain_phase();
            let ridge = radius
                * ((angle * 3.0 + phase).sin() * 0.28 + (angle * 7.0 - phase * 0.5).cos() * 0.14);
            let shoulder = (radius * std::f32::consts::PI).sin() * 0.24;
            let center_falloff = ((1.0 - radius).powi(2) - 1.0) * 0.16;
            let edge_drop = -radius.powf(2.35) * 0.42;

            (ridge + shoulder + center_falloff + edge_drop)
                .clamp(-TERRAIN_MAX_DROP_M, TERRAIN_MAX_RISE_M)
        }

        pub fn contains_horizontal(self, position: Vec3) -> bool {
            let dx = (position.x - self.center.x) / self.half_extents.x.max(0.001);
            let dz = (position.z - self.center.z) / self.half_extents.y.max(0.001);
            dx * dx + dz * dz <= 1.0
        }

        pub fn horizontal_distance(self, position: Vec3) -> f32 {
            Vec2::new(position.x - self.center.x, position.z - self.center.z).length()
        }

        pub fn lod_band(self, position: Vec3) -> LodBand {
            let distance = self.horizontal_distance(position);
            if distance <= LOD_NEAR_DISTANCE_M {
                LodBand::Near
            } else if distance <= LOD_MID_DISTANCE_M {
                LodBand::Mid
            } else {
                LodBand::Far
            }
        }

        pub fn streaming_chunk(self) -> StreamChunkCoord {
            StreamChunkCoord::from_world(self.center)
        }

        pub fn stream_activation(self, position: Vec3) -> StreamActivation {
            let player_chunk = StreamChunkCoord::from_world(position);
            if self.streaming_chunk().is_inside_active_window(player_chunk) {
                StreamActivation::Active
            } else {
                StreamActivation::Inactive
            }
        }

        fn terrain_phase(self) -> f32 {
            self.center.x * 0.013
                + self.center.z * 0.009
                + self.half_extents.x * 0.021
                + self.half_extents.y * 0.017
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct GroundSurface {
        pub floor_y: f32,
        pub is_target: bool,
        pub island_name: Option<&'static str>,
    }

    impl GroundSurface {
        fn from_island_at(island: SkyIsland, position: Vec3) -> Self {
            Self {
                floor_y: island.terrain_surface_y_at(position),
                is_target: island.is_target,
                island_name: Some(island.name),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::movement::{FlightController, FlightState};

        #[test]
        fn route_reports_highest_island_surface_under_player() {
            let route = SkyRoute::default();
            let launch_surface = route.ground_at(START_POSITION);

            assert_eq!(launch_surface.floor_y, START_FLOOR_Y);
            assert_eq!(launch_surface.island_name, Some("launch mesa"));
        }

        #[test]
        fn route_surface_follows_island_relief() {
            let route = SkyRoute::default();
            let island = route.islands()[0];
            let ridge_position = Vec3::new(
                island.center.x + island.half_extents.x * 0.46,
                START_FLOOR_Y,
                island.center.z + island.half_extents.y * 0.18,
            );
            let edge_position = Vec3::new(
                island.center.x + island.half_extents.x * 0.84,
                START_FLOOR_Y,
                island.center.z - island.half_extents.y * 0.22,
            );

            let center_surface = route.ground_at(island.center);
            let ridge_surface = route.ground_at(ridge_position);
            let edge_surface = route.ground_at(edge_position);

            assert_eq!(center_surface.floor_y, START_FLOOR_Y);
            assert_ne!(ridge_surface.floor_y, center_surface.floor_y);
            assert_ne!(edge_surface.floor_y, center_surface.floor_y);
            assert!(
                ridge_surface.floor_y <= island.floor_y() + TERRAIN_MAX_RISE_M
                    && edge_surface.floor_y >= island.floor_y() - TERRAIN_MAX_DROP_M
            );
        }

        #[test]
        fn target_distance_reaches_zero_near_landing_island_center() {
            let route = SkyRoute::default();
            let target = route.target_island().expect("target island exists");

            assert_eq!(route.target_distance(target.center), 0.0);
            assert!(route.target_distance(START_POSITION) > 200.0);
        }

        #[test]
        fn route_can_track_named_recovery_branch_islands() {
            let route = SkyRoute::default();
            let branch = route
                .island_named("sunlit terrace")
                .expect("recovery branch exists");

            assert!(is_recovery_branch_island(branch.name));
            assert_eq!(
                route.target_distance_to(branch.center, Some(branch.name)),
                0.0
            );
            assert!(
                route.target_distance_to(START_POSITION, Some(branch.name))
                    > route.target_distance(START_POSITION)
            );
        }

        #[test]
        fn route_objectives_track_main_and_branch_targets() {
            let route = SkyRoute::default();
            let main = route.route_objectives(None);
            let branch = route.route_objectives(Some("sunlit terrace"));

            assert_eq!(main.len(), 2);
            assert_eq!(main[0].label, "near route updraft");
            assert_eq!(main[1].label, "landing garden");
            assert_eq!(branch.len(), 3);
            assert_eq!(branch[1].label, "distant recovery updraft");
            assert_eq!(branch[2].label, "sunlit terrace");
        }

        #[test]
        fn route_objective_completion_tracks_flythrough_and_landing() {
            let route = SkyRoute::default();
            let objectives = route.route_objectives(None);
            let target = route.target_island().expect("target island exists");

            assert!(objectives[0].is_complete(
                &route,
                GAMEPLAY_LIFT_ROUTE[0].center,
                FlightMode::Gliding
            ));
            assert!(!objectives[1].is_complete(
                &route,
                target.center + Vec3::Y * 8.0,
                FlightMode::Gliding
            ));
            assert!(objectives[1].is_complete(&route, target.center, FlightMode::Grounded));
        }

        #[test]
        fn route_has_archipelago_scale_and_distant_landmarks() {
            let route = SkyRoute::default();
            let farthest_z = route
                .islands()
                .iter()
                .map(|island| island.center.z)
                .fold(0.0_f32, f32::min);

            assert!(route.islands().len() >= 12);
            assert!(farthest_z < -800.0);
        }

        #[test]
        fn streaming_lod_stats_track_active_window_and_distance_bands() {
            let route = SkyRoute::default();
            let stats = route.streaming_lod_stats(START_POSITION);

            assert_eq!(stats.player_chunk, StreamChunkCoord { x: 0, z: 0 });
            assert_eq!(stats.active_chunk_count, 25);
            assert!(stats.active_island_count < route.islands().len());
            assert!(stats.active_island_count >= 4);
            assert!(stats.near_lod_islands >= 2);
            assert!(stats.mid_lod_islands >= 3);
            assert!(stats.far_lod_islands >= 3);
        }

        #[test]
        fn island_lod_band_uses_route_distance_thresholds() {
            let island =
                SkyIsland::new("test island", Vec3::ZERO, Vec2::new(10.0, 10.0), 4.0, false);

            assert_eq!(island.lod_band(Vec3::new(0.0, 0.0, 0.0)), LodBand::Near);
            assert_eq!(
                island.lod_band(Vec3::new(LOD_NEAR_DISTANCE_M + 1.0, 0.0, 0.0)),
                LodBand::Mid
            );
            assert_eq!(
                island.lod_band(Vec3::new(LOD_MID_DISTANCE_M + 1.0, 0.0, 0.0)),
                LodBand::Far
            );
        }

        #[test]
        fn island_stream_activation_uses_chunk_window() {
            let island =
                SkyIsland::new("test island", Vec3::ZERO, Vec2::new(10.0, 10.0), 4.0, false);

            assert_eq!(
                island.stream_activation(START_POSITION),
                StreamActivation::Active
            );
            assert_eq!(
                island.stream_activation(Vec3::new(
                    0.0,
                    START_FLOOR_Y,
                    STREAM_CHUNK_SIZE_M * (STREAM_ACTIVE_CHUNK_RADIUS + 2) as f32,
                )),
                StreamActivation::Inactive
            );
        }

        #[test]
        fn ground_contact_marks_target_landing_as_grounded() {
            let route = SkyRoute::default();
            let target = route.target_island().expect("target island exists");
            let state = FlightState::new(
                Vec3::new(target.center.x, target.floor_y() - 2.0, target.center.z),
                Vec3::new(20.0, -10.0, 10.0),
                FlightController::default(),
            );

            let resolved = route.resolve_ground_contact(state);

            assert_eq!(resolved.position.y, target.floor_y());
            assert!(resolved.velocity.x < state.velocity.x);
            assert!(resolved.velocity.z < state.velocity.z);
            assert_eq!(resolved.controller.mode, FlightMode::Grounded);
            assert!(route.on_landing_target(resolved.position, resolved.controller.mode));
        }

        #[test]
        fn already_grounded_route_contact_does_not_damp_wasd_motion() {
            let route = SkyRoute::default();
            let state = FlightState::new(
                START_POSITION,
                Vec3::new(8.0, 0.0, -4.0),
                FlightController::default(),
            );

            let resolved = route.resolve_ground_contact_after_step(state, true);

            assert_eq!(resolved.position.y, START_FLOOR_Y);
            assert_eq!(resolved.velocity.x, state.velocity.x);
            assert_eq!(resolved.velocity.z, state.velocity.z);
            assert_eq!(resolved.controller.mode, FlightMode::Grounded);
        }

        #[test]
        fn walking_off_island_clears_stale_grounded_mode() {
            let route = SkyRoute::default();
            let state = FlightState::new(
                Vec3::new(200.0, START_FLOOR_Y, 200.0),
                Vec3::new(6.0, 0.0, 0.0),
                FlightController::default(),
            );

            let resolved = route.resolve_ground_contact_after_step(state, true);

            assert_eq!(resolved.controller.mode, FlightMode::Airborne);
            assert_eq!(resolved.position.y, START_FLOOR_Y);
        }

        #[test]
        fn island_visual_top_stays_close_to_player_footing() {
            let route = SkyRoute::default();
            let island = route.islands()[0];
            let sample = Vec3::new(
                island.center.x + island.half_extents.x * 0.35,
                island.center.y,
                island.center.z - island.half_extents.y * 0.25,
            );
            let visual_offset = island.terrain_surface_y_at(sample) - island.mesh_top_y_at(sample);

            assert!((0.15..=0.3).contains(&visual_offset));
        }
    }
}

pub mod camera {
    use crate::movement::smoothing_factor;
    use bevy::prelude::*;
    use std::f32::consts::PI;

    #[derive(Component, Clone, Copy, Debug)]
    pub struct FollowCamera {
        pub distance: f32,
        pub height: f32,
        pub look_height: f32,
        pub look_ahead: f32,
        pub position_smoothing: f32,
        pub rotation_smoothing: f32,
        pub direction_smoothing: f32,
        pub min_height: f32,
    }

    impl Default for FollowCamera {
        fn default() -> Self {
            Self {
                distance: 12.0,
                height: 5.0,
                look_height: 1.4,
                look_ahead: 0.5,
                position_smoothing: 10.0,
                rotation_smoothing: 24.0,
                direction_smoothing: 5.0,
                min_height: 1.6,
            }
        }
    }

    #[derive(Component, Clone, Copy, Debug)]
    pub struct FollowCameraState {
        pub direction: Vec3,
        initialized: bool,
    }

    impl Default for FollowCameraState {
        fn default() -> Self {
            Self {
                direction: Vec3::NEG_Z,
                initialized: false,
            }
        }
    }

    #[derive(Resource, Clone, Copy, Debug)]
    pub struct CameraControlTuning {
        pub sensitivity_x: f32,
        pub sensitivity_y: f32,
        pub min_pitch: f32,
        pub max_pitch: f32,
        pub invert_y: bool,
    }

    impl Default for CameraControlTuning {
        fn default() -> Self {
            Self {
                sensitivity_x: 0.0042,
                sensitivity_y: 0.0036,
                min_pitch: -35.0_f32.to_radians(),
                max_pitch: 35.0_f32.to_radians(),
                invert_y: false,
            }
        }
    }

    #[derive(Resource, Clone, Copy, Debug, Default)]
    pub struct CameraControlState {
        pub orbit: CameraOrbit,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct CameraInput {
        pub mouse_delta: Vec2,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct CameraOrbit {
        pub yaw: f32,
        pub pitch: f32,
    }

    impl CameraOrbit {
        pub fn yaw_degrees(self) -> f32 {
            self.yaw.to_degrees()
        }

        pub fn pitch_degrees(self) -> f32 {
            self.pitch.to_degrees()
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct CameraFrame {
        pub position: Vec3,
        pub rotation: Quat,
        pub look_target: Vec3,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct CameraObstruction {
        pub center: Vec3,
        pub half_extents: Vec3,
    }

    impl CameraObstruction {
        pub fn new(center: Vec3, half_extents: Vec3) -> Self {
            Self {
                center,
                half_extents: half_extents.abs(),
            }
        }

        fn expanded(self, clearance: f32) -> Self {
            Self {
                center: self.center,
                half_extents: self.half_extents + Vec3::splat(clearance.max(0.0)),
            }
        }

        fn contains(self, point: Vec3) -> bool {
            let min = self.center - self.half_extents;
            let max = self.center + self.half_extents;

            point.x >= min.x
                && point.x <= max.x
                && point.y >= min.y
                && point.y <= max.y
                && point.z >= min.z
                && point.z <= max.z
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct CameraObstructionResolution {
        pub frame: CameraFrame,
        pub adjusted_distance_m: f32,
        pub hit_count: usize,
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
        step_camera_with_orbit(
            current_position,
            current_rotation,
            player_position,
            player_forward,
            player_velocity,
            follow,
            CameraOrbit::default(),
            dt,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn step_camera_with_orbit(
        current_position: Vec3,
        current_rotation: Quat,
        player_position: Vec3,
        player_forward: Vec3,
        player_velocity: Vec3,
        follow: &FollowCamera,
        orbit: CameraOrbit,
        dt: f32,
    ) -> CameraFrame {
        let direction = horizontal_follow_direction(player_velocity, player_forward);
        step_camera_with_direction(
            current_position,
            current_rotation,
            player_position,
            direction,
            follow,
            orbit,
            dt,
        )
    }

    pub fn update_follow_direction_state(
        state: &mut FollowCameraState,
        desired_direction: Vec3,
        follow: &FollowCamera,
        dt: f32,
    ) -> Vec3 {
        let fallback = if state.initialized {
            state.direction
        } else {
            Vec3::NEG_Z
        };
        let desired_direction = horizontal_or(desired_direction, fallback);
        if !state.initialized {
            state.direction = desired_direction;
            state.initialized = true;
            return state.direction;
        }

        state.direction = horizontal_or(
            state.direction.lerp(
                desired_direction,
                smoothing_factor(follow.direction_smoothing, dt),
            ),
            desired_direction,
        );
        state.direction
    }

    #[allow(clippy::too_many_arguments)]
    pub fn step_camera_with_direction(
        current_position: Vec3,
        current_rotation: Quat,
        player_position: Vec3,
        follow_direction: Vec3,
        follow: &FollowCamera,
        orbit: CameraOrbit,
        dt: f32,
    ) -> CameraFrame {
        let direction = horizontal_or(follow_direction, Vec3::NEG_Z);
        let direction = yawed_horizontal_direction(direction, orbit.yaw);
        let look_target =
            player_position + Vec3::Y * follow.look_height + direction * follow.look_ahead;
        let base_horizontal_distance = follow.distance + follow.look_ahead;
        let base_vertical_offset = follow.height - follow.look_height;
        let boom_distance = Vec2::new(base_horizontal_distance, base_vertical_offset)
            .length()
            .max(0.001);
        let base_elevation = base_vertical_offset.atan2(base_horizontal_distance);
        let elevation = base_elevation - orbit.pitch;
        let horizontal_distance = elevation.cos().max(0.0) * boom_distance;
        let vertical_offset = elevation.sin() * boom_distance;
        let mut desired_position =
            look_target - direction * horizontal_distance + Vec3::Y * vertical_offset;
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

    pub fn apply_camera_input(
        orbit: CameraOrbit,
        input: CameraInput,
        tuning: &CameraControlTuning,
    ) -> CameraOrbit {
        let yaw = wrap_radians(orbit.yaw - input.mouse_delta.x * tuning.sensitivity_x);
        let y_sign = if tuning.invert_y { 1.0 } else { -1.0 };
        let pitch = (orbit.pitch + input.mouse_delta.y * tuning.sensitivity_y * y_sign)
            .clamp(tuning.min_pitch, tuning.max_pitch);

        CameraOrbit { yaw, pitch }
    }

    fn yawed_horizontal_direction(direction: Vec3, yaw: f32) -> Vec3 {
        let rotated = Quat::from_rotation_y(yaw) * direction;
        horizontal_or(rotated, direction)
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

    fn horizontal_or(value: Vec3, fallback: Vec3) -> Vec3 {
        let horizontal = Vec3::new(value.x, 0.0, value.z);
        if horizontal.length_squared() > 0.0001 {
            horizontal.normalize()
        } else {
            let fallback = Vec3::new(fallback.x, 0.0, fallback.z);
            if fallback.length_squared() > 0.0001 {
                fallback.normalize()
            } else {
                Vec3::NEG_Z
            }
        }
    }

    fn wrap_radians(value: f32) -> f32 {
        (value + PI).rem_euclid(PI * 2.0) - PI
    }

    pub fn camera_distance(camera_position: Vec3, target_position: Vec3) -> f32 {
        let distance = camera_position.distance(target_position);
        if distance.is_finite() { distance } else { 0.0 }
    }

    pub fn camera_surface_clearance(camera_position: Vec3, floor_y: f32) -> f32 {
        (camera_position.y - floor_y).max(0.0)
    }

    pub fn camera_target_angle_degrees(
        camera_position: Vec3,
        camera_rotation: Quat,
        target_position: Vec3,
    ) -> f32 {
        let to_target = target_position - camera_position;
        if to_target.length_squared() <= 0.0001 {
            return 0.0;
        }

        let forward = camera_rotation * Vec3::NEG_Z;
        let dot = forward
            .normalize_or_zero()
            .dot(to_target.normalize())
            .clamp(-1.0, 1.0);
        if dot.is_finite() {
            dot.acos().to_degrees()
        } else {
            0.0
        }
    }

    pub fn camera_orbit_alignment_degrees(
        camera_position: Vec3,
        look_target: Vec3,
        follow_direction: Vec3,
        orbit: CameraOrbit,
    ) -> f32 {
        let expected_direction = yawed_horizontal_direction(follow_direction, orbit.yaw);
        let actual_direction = horizontal_or(look_target - camera_position, expected_direction);
        let angle = actual_direction
            .angle_between(expected_direction)
            .to_degrees();

        if angle.is_finite() { angle } else { 0.0 }
    }

    pub fn camera_view_yaw_degrees(camera_rotation: Quat, reference_direction: Vec3) -> f32 {
        let reference_direction = horizontal_or(reference_direction, Vec3::NEG_Z);
        let view_direction = horizontal_or(camera_rotation * Vec3::NEG_Z, reference_direction);
        let cross_y = reference_direction.cross(view_direction).y;
        let dot = reference_direction.dot(view_direction).clamp(-1.0, 1.0);
        let yaw = cross_y.atan2(dot).to_degrees();

        if yaw.is_finite() { yaw } else { 0.0 }
    }

    pub fn lift_camera_above_floor(
        mut frame: CameraFrame,
        floor_y: f32,
        min_clearance: f32,
    ) -> CameraFrame {
        let min_y = floor_y + min_clearance.max(0.0);
        if frame.position.y < min_y {
            frame.position.y = min_y;
            frame.rotation = Transform::from_translation(frame.position)
                .looking_at(frame.look_target, Vec3::Y)
                .rotation;
        }

        frame
    }

    pub fn avoid_camera_obstructions(
        frame: CameraFrame,
        obstructions: impl IntoIterator<Item = CameraObstruction>,
        clearance: f32,
    ) -> CameraObstructionResolution {
        let segment = frame.position - frame.look_target;
        let segment_length = segment.length();
        if segment_length <= 0.001 || !segment_length.is_finite() {
            return CameraObstructionResolution {
                frame,
                adjusted_distance_m: 0.0,
                hit_count: 0,
            };
        }

        let direction = segment / segment_length;
        let mut nearest_hit_distance = segment_length;
        let mut hit_count = 0;

        for obstruction in obstructions {
            let obstruction = obstruction.expanded(clearance);
            if obstruction.contains(frame.look_target) {
                continue;
            }
            let Some(hit_distance) = segment_aabb_hit_distance(
                frame.look_target,
                direction,
                segment_length,
                obstruction,
            ) else {
                continue;
            };
            hit_count += 1;
            nearest_hit_distance = nearest_hit_distance.min(hit_distance);
        }

        if hit_count == 0 || nearest_hit_distance >= segment_length {
            return CameraObstructionResolution {
                frame,
                adjusted_distance_m: 0.0,
                hit_count,
            };
        }

        let min_target_distance = 2.4;
        let adjusted_distance = nearest_hit_distance.max(min_target_distance);
        let mut adjusted = frame;
        adjusted.position = frame.look_target + direction * adjusted_distance;
        adjusted.rotation = Transform::from_translation(adjusted.position)
            .looking_at(adjusted.look_target, Vec3::Y)
            .rotation;

        CameraObstructionResolution {
            frame: adjusted,
            adjusted_distance_m: frame.position.distance(adjusted.position),
            hit_count,
        }
    }

    fn segment_aabb_hit_distance(
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
        obstruction: CameraObstruction,
    ) -> Option<f32> {
        let min = obstruction.center - obstruction.half_extents;
        let max = obstruction.center + obstruction.half_extents;
        let mut t_min = 0.0;
        let mut t_max = max_distance;

        update_slab_interval(origin.x, direction.x, min.x, max.x, &mut t_min, &mut t_max)?;
        update_slab_interval(origin.y, direction.y, min.y, max.y, &mut t_min, &mut t_max)?;
        update_slab_interval(origin.z, direction.z, min.z, max.z, &mut t_min, &mut t_max)?;

        if t_min <= max_distance && t_max >= 0.0 {
            Some(t_min.max(0.0))
        } else {
            None
        }
    }

    fn update_slab_interval(
        origin: f32,
        direction: f32,
        min: f32,
        max: f32,
        t_min: &mut f32,
        t_max: &mut f32,
    ) -> Option<()> {
        if direction.abs() <= 0.0001 {
            return (origin >= min && origin <= max).then_some(());
        }

        let inverse_direction = direction.recip();
        let mut near = (min - origin) * inverse_direction;
        let mut far = (max - origin) * inverse_direction;
        if near > far {
            std::mem::swap(&mut near, &mut far);
        }

        *t_min = (*t_min).max(near);
        *t_max = (*t_max).min(far);
        (*t_min <= *t_max).then_some(())
    }

    pub fn camera_pitch_degrees(rotation: Quat) -> f32 {
        let forward = rotation * Vec3::NEG_Z;
        let y = forward.y.clamp(-1.0, 1.0);

        if y.is_finite() {
            y.asin().to_degrees()
        } else {
            0.0
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

        #[test]
        fn mouse_x_changes_camera_yaw_without_touching_pitch() {
            let tuning = CameraControlTuning::default();
            let orbit = apply_camera_input(
                CameraOrbit::default(),
                CameraInput {
                    mouse_delta: Vec2::new(20.0, 0.0),
                },
                &tuning,
            );

            assert!(orbit.yaw < -0.08);
            assert_eq!(orbit.pitch, 0.0);
        }

        #[test]
        fn mouse_y_maps_to_pitch_and_clamps() {
            let tuning = CameraControlTuning::default();
            let up = apply_camera_input(
                CameraOrbit::default(),
                CameraInput {
                    mouse_delta: Vec2::new(0.0, -20.0),
                },
                &tuning,
            );
            let clamped = apply_camera_input(
                CameraOrbit::default(),
                CameraInput {
                    mouse_delta: Vec2::new(0.0, -1000.0),
                },
                &tuning,
            );

            assert!(up.pitch > 0.07);
            assert_eq!(clamped.pitch, tuning.max_pitch);
        }

        #[test]
        fn orbit_pitch_moves_view_pitch_in_expected_direction() {
            let follow = FollowCamera::default();
            let low = step_camera_with_orbit(
                Vec3::new(0.0, 6.0, -12.0),
                Quat::IDENTITY,
                Vec3::ZERO,
                Vec3::NEG_Z,
                Vec3::NEG_Z * 10.0,
                &follow,
                CameraOrbit {
                    pitch: -0.25,
                    yaw: 0.0,
                },
                1.0,
            );
            let high = step_camera_with_orbit(
                Vec3::new(0.0, 6.0, -12.0),
                Quat::IDENTITY,
                Vec3::ZERO,
                Vec3::NEG_Z,
                Vec3::NEG_Z * 10.0,
                &follow,
                CameraOrbit {
                    pitch: 0.25,
                    yaw: 0.0,
                },
                1.0,
            );

            assert!(camera_pitch_degrees(high.rotation) > camera_pitch_degrees(low.rotation));
        }

        #[test]
        fn orbit_pitch_keeps_player_focus_centered() {
            let follow = FollowCamera::default();
            let player_position = Vec3::ZERO;
            let frame = step_camera_with_orbit(
                Vec3::new(0.0, follow.height, follow.distance),
                Quat::IDENTITY,
                player_position,
                Vec3::NEG_Z,
                Vec3::ZERO,
                &follow,
                CameraOrbit {
                    pitch: CameraControlTuning::default().max_pitch,
                    yaw: 0.0,
                },
                1.0,
            );
            let player_focus = player_position + Vec3::Y * follow.look_height;

            assert!(
                camera_target_angle_degrees(frame.position, frame.rotation, player_focus) < 3.0
            );
        }

        #[test]
        fn follow_direction_smoothing_limits_turnaround_snap() {
            let follow = FollowCamera::default();
            let mut state = FollowCameraState {
                direction: Vec3::Z,
                initialized: true,
            };
            let follow_direction =
                update_follow_direction_state(&mut state, Vec3::NEG_Z, &follow, 1.0 / 60.0);
            let frame = step_camera_with_direction(
                Vec3::new(0.0, 6.0, 12.0),
                Quat::IDENTITY,
                Vec3::ZERO,
                follow_direction,
                &follow,
                CameraOrbit::default(),
                1.0 / 60.0,
            );

            assert!(
                frame.position.z > 8.0,
                "camera should not instantly orbit across the player on a velocity flip"
            );
        }

        #[test]
        fn persistent_yaw_offset_does_not_compound_into_spin() {
            let follow = FollowCamera::default();
            let orbit = CameraOrbit {
                yaw: 0.2,
                pitch: 0.0,
            };
            let player_position = Vec3::ZERO;
            let player_forward = Vec3::NEG_Z;
            let mut camera_position = Vec3::new(0.0, follow.height, follow.distance);
            let mut camera_rotation = Transform::from_translation(camera_position)
                .looking_at(player_position + Vec3::Y * follow.look_height, Vec3::Y)
                .rotation;
            let expected_direction = yawed_horizontal_direction(
                horizontal_follow_direction(Vec3::ZERO, player_forward),
                orbit.yaw,
            );

            for _ in 0..240 {
                let frame = step_camera_with_orbit(
                    camera_position,
                    camera_rotation,
                    player_position,
                    player_forward,
                    Vec3::ZERO,
                    &follow,
                    orbit,
                    1.0 / 60.0,
                );
                camera_position = frame.position;
                camera_rotation = frame.rotation;
            }

            let drift_degrees = camera_orbit_alignment_degrees(
                camera_position,
                player_position + Vec3::Y * follow.look_height,
                expected_direction,
                CameraOrbit::default(),
            );

            assert!(
                drift_degrees < 3.0,
                "persistent yaw drifted by {drift_degrees} degrees"
            );
        }

        #[test]
        fn camera_pitch_is_negative_when_looking_downward() {
            let rotation = Transform::from_xyz(0.0, 6.0, -12.0)
                .looking_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y)
                .rotation;

            assert!(camera_pitch_degrees(rotation) < -15.0);
        }

        #[test]
        fn camera_pitch_is_level_for_horizontal_forward() {
            assert!(camera_pitch_degrees(Quat::IDENTITY).abs() < 0.001);
        }

        #[test]
        fn camera_view_yaw_tracks_horizontal_rotation() {
            let yaw_radians = 0.35_f32;
            let yaw_degrees =
                camera_view_yaw_degrees(Quat::from_rotation_y(yaw_radians), Vec3::NEG_Z);

            assert!((yaw_degrees.abs() - yaw_radians.to_degrees()).abs() < 0.001);
        }

        #[test]
        fn camera_distance_matches_vector_length() {
            assert_eq!(camera_distance(Vec3::new(0.0, 3.0, 4.0), Vec3::ZERO), 5.0);
        }

        #[test]
        fn camera_surface_clearance_lifts_clipping_frame() {
            let frame = CameraFrame {
                position: Vec3::new(0.0, 3.0, 0.0),
                rotation: Quat::IDENTITY,
                look_target: Vec3::new(0.0, 4.0, -4.0),
            };

            let lifted = lift_camera_above_floor(frame, 2.5, 2.0);

            assert_eq!(lifted.position.y, 4.5);
            assert_eq!(camera_surface_clearance(lifted.position, 2.5), 2.0);
        }

        #[test]
        fn camera_obstruction_moves_camera_in_front_of_blocker() {
            let frame = CameraFrame {
                position: Vec3::new(0.0, 2.0, 10.0),
                rotation: Quat::IDENTITY,
                look_target: Vec3::new(0.0, 2.0, 0.0),
            };

            let resolved = avoid_camera_obstructions(
                frame,
                [CameraObstruction::new(
                    Vec3::new(0.0, 2.0, 5.0),
                    Vec3::new(1.0, 1.0, 1.0),
                )],
                0.5,
            );

            assert_eq!(resolved.hit_count, 1);
            assert!(resolved.adjusted_distance_m > 5.0);
            assert!(resolved.frame.position.z < 4.0);
            assert!(
                camera_target_angle_degrees(
                    resolved.frame.position,
                    resolved.frame.rotation,
                    resolved.frame.look_target,
                ) < 0.001
            );
        }

        #[test]
        fn camera_obstruction_keeps_clear_view_when_blocker_is_off_segment() {
            let frame = CameraFrame {
                position: Vec3::new(0.0, 2.0, 10.0),
                rotation: Quat::IDENTITY,
                look_target: Vec3::new(0.0, 2.0, 0.0),
            };

            let resolved = avoid_camera_obstructions(
                frame,
                [CameraObstruction::new(
                    Vec3::new(5.0, 2.0, 5.0),
                    Vec3::new(1.0, 1.0, 1.0),
                )],
                0.5,
            );

            assert_eq!(resolved.hit_count, 0);
            assert_eq!(resolved.adjusted_distance_m, 0.0);
            assert_eq!(resolved.frame.position, frame.position);
        }

        #[test]
        fn camera_obstruction_uses_nearest_blocker() {
            let frame = CameraFrame {
                position: Vec3::new(0.0, 2.0, 12.0),
                rotation: Quat::IDENTITY,
                look_target: Vec3::new(0.0, 2.0, 0.0),
            };

            let resolved = avoid_camera_obstructions(
                frame,
                [
                    CameraObstruction::new(Vec3::new(0.0, 2.0, 8.0), Vec3::splat(1.0)),
                    CameraObstruction::new(Vec3::new(0.0, 2.0, 4.0), Vec3::splat(1.0)),
                ],
                0.25,
            );

            assert_eq!(resolved.hit_count, 2);
            assert!(resolved.frame.position.z < 3.0);
            assert!(resolved.frame.position.z > 2.3);
        }
    }
}

pub mod diagnostics {
    pub fn frame_ms(delta_seconds: f32) -> f32 {
        if delta_seconds.is_finite() {
            delta_seconds.max(0.0) * 1000.0
        } else {
            0.0
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn frame_ms_never_emits_nan_for_zero_or_invalid_delta() {
            assert_eq!(frame_ms(0.0), 0.0);
            assert_eq!(frame_ms(f32::NAN), 0.0);
            assert_eq!(frame_ms(f32::NEG_INFINITY), 0.0);
            assert!(frame_ms(1.0 / 60.0).is_finite());
        }
    }
}

pub mod eval {
    use crate::{
        asset_pipeline::{
            GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT, STREAMING_VISUAL_ASSET_SLOT_COUNT,
            VISUAL_ASSET_SLOT_COUNT,
        },
        camera::CameraInput,
        movement::{FlightInput, FlightMode},
    };
    use bevy::prelude::*;

    pub const BASELINE_ROUTE: &str = "baseline_route";
    pub const ISLAND_LAUNCH_TO_LANDING: &str = "island_launch_to_landing";
    pub const GROUND_TAXI_CONTROL: &str = "ground_taxi_control";
    pub const UPDRAFT_ROUTE: &str = "updraft_route";
    pub const CAMERA_MOUSE_CONTROL: &str = "camera_mouse_control";
    pub const CAMERA_YAW_STABILITY: &str = "camera_yaw_stability";
    pub const CAMERA_TURN_STABILITY: &str = "camera_turn_stability";
    pub const CAMERA_STRAFE_STABILITY: &str = "camera_strafe_stability";
    pub const LONG_GLIDE_VISIBILITY: &str = "long_glide_visibility";
    pub const BRANCH_RECOVERY_ROUTE: &str = "branch_recovery_route";
    pub const SCENARIO_NAMES: &[&str] = &[
        BASELINE_ROUTE,
        ISLAND_LAUNCH_TO_LANDING,
        GROUND_TAXI_CONTROL,
        UPDRAFT_ROUTE,
        BRANCH_RECOVERY_ROUTE,
        CAMERA_MOUSE_CONTROL,
        CAMERA_YAW_STABILITY,
        CAMERA_TURN_STABILITY,
        CAMERA_STRAFE_STABILITY,
        LONG_GLIDE_VISIBILITY,
    ];
    pub const MAX_RESIDENT_ISLAND_VISUAL_FRACTION: f32 = 0.70;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EvalCheckpoint {
        pub frame: u32,
        pub name: &'static str,
    }

    const BASELINE_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 90,
            name: "launch_clear",
        },
        EvalCheckpoint {
            frame: 260,
            name: "glide_midroute",
        },
    ];
    const ISLAND_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 120,
            name: "outbound_glide",
        },
        EvalCheckpoint {
            frame: 320,
            name: "landing_approach",
        },
    ];
    const GROUND_TAXI_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 60,
            name: "ground_turn",
        },
        EvalCheckpoint {
            frame: 150,
            name: "reverse_check",
        },
    ];
    const UPDRAFT_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 150,
            name: "updraft_entry",
        },
        EvalCheckpoint {
            frame: 280,
            name: "high_glide",
        },
    ];
    const BRANCH_RECOVERY_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 180,
            name: "branch_choice",
        },
        EvalCheckpoint {
            frame: 500,
            name: "recovery_approach",
        },
        EvalCheckpoint {
            frame: 690,
            name: "branch_landing",
        },
    ];
    const CAMERA_MOUSE_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 5,
            name: "launch_obstruction",
        },
        EvalCheckpoint {
            frame: 50,
            name: "yaw_check",
        },
        EvalCheckpoint {
            frame: 120,
            name: "pitch_check",
        },
        EvalCheckpoint {
            frame: 180,
            name: "settled_view",
        },
    ];
    const CAMERA_YAW_STABILITY_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 30,
            name: "small_yaw_input",
        },
        EvalCheckpoint {
            frame: 180,
            name: "yaw_settle",
        },
        EvalCheckpoint {
            frame: 260,
            name: "drift_check",
        },
    ];
    const CAMERA_TURN_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 90,
            name: "first_turn",
        },
        EvalCheckpoint {
            frame: 180,
            name: "counter_turn",
        },
        EvalCheckpoint {
            frame: 300,
            name: "air_brake",
        },
    ];
    const CAMERA_STRAFE_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 60,
            name: "right_strafe",
        },
        EvalCheckpoint {
            frame: 150,
            name: "left_strafe",
        },
        EvalCheckpoint {
            frame: 230,
            name: "settled_strafe",
        },
    ];
    const LONG_GLIDE_CHECKPOINTS: &[EvalCheckpoint] = &[
        EvalCheckpoint {
            frame: 180,
            name: "far_route_entry",
        },
        EvalCheckpoint {
            frame: 420,
            name: "archipelago_midroute",
        },
        EvalCheckpoint {
            frame: 640,
            name: "distant_islands",
        },
    ];

    #[derive(Clone, Copy, Debug)]
    pub struct EvalScenario {
        pub name: &'static str,
        pub fixed_dt: f32,
        pub frame_count: u32,
        pub sample_stride: u32,
        pub target_island_name: Option<&'static str>,
        pub checkpoints: &'static [EvalCheckpoint],
        pub thresholds: EvalThresholds,
    }

    impl EvalScenario {
        pub fn duration_secs(self) -> f32 {
            self.frame_count as f32 * self.fixed_dt
        }

        pub fn should_sample(self, frame: u32) -> bool {
            frame == 0 || frame >= self.frame_count || frame.is_multiple_of(self.sample_stride)
        }

        pub fn checkpoint_at(self, frame: u32) -> Option<EvalCheckpoint> {
            self.checkpoints
                .iter()
                .copied()
                .find(|checkpoint| checkpoint.frame == frame)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct EvalThresholds {
        pub min_samples: u32,
        pub min_horizontal_distance_m: f32,
        pub min_max_altitude_m: f32,
        pub min_max_speed_mps: f32,
        pub min_gliding_samples: u32,
        pub min_grounded_samples: u32,
        pub min_lifted_samples: u32,
        pub min_sky_island_count: usize,
        pub min_active_island_count: usize,
        pub max_active_chunk_count: usize,
        pub min_near_lod_island_count: usize,
        pub min_mid_lod_island_count: usize,
        pub min_far_lod_island_count: usize,
        pub max_visible_island_terrain_count: usize,
        pub min_hidden_island_terrain_count: usize,
        pub min_visible_island_impostor_count: usize,
        pub max_visible_island_detail_count: usize,
        pub min_hidden_island_detail_count: usize,
        pub min_visible_route_beacon_count: usize,
        pub min_weather_cloud_count: usize,
        pub min_environment_motion_visual_count: usize,
        pub min_environment_motion_offset_m: f32,
        pub max_resident_island_visual_count: usize,
        pub max_stream_visibility_changes_per_frame: usize,
        pub min_entity_count: usize,
        pub max_camera_distance_m: f32,
        pub min_camera_surface_clearance_m: f32,
        pub max_camera_player_angle_degrees: f32,
        pub max_camera_step_distance_m: f32,
        pub max_camera_rotation_delta_degrees: f32,
        pub max_camera_orbit_alignment_degrees: f32,
        pub max_abs_camera_view_yaw_degrees: f32,
        pub min_camera_obstruction_adjustment_m: f32,
        pub min_abs_camera_yaw_degrees: f32,
        pub min_camera_pitch_offset_degrees: f32,
        pub max_camera_pitch_offset_degrees: f32,
        pub min_objective_total_count: usize,
        pub min_completed_objective_count: usize,
        pub min_visual_asset_slot_count: usize,
        pub min_gltf_scene_asset_slot_count: usize,
        pub min_streaming_visual_asset_slot_count: usize,
        pub max_failed_visual_asset_scene_count: usize,
        pub min_power_up_count: usize,
        pub min_collected_power_up_count: usize,
        pub min_power_up_effect_samples: u32,
        pub require_target_landing: bool,
        pub max_final_target_distance_m: f32,
        pub min_target_landing_samples: u32,
    }

    impl EvalThresholds {
        fn to_json(self, indent: &str) -> String {
            format!(
                "{{\n{indent}  \"min_samples\": {},\n{indent}  \"min_horizontal_distance_m\": {},\n{indent}  \"min_max_altitude_m\": {},\n{indent}  \"min_max_speed_mps\": {},\n{indent}  \"min_gliding_samples\": {},\n{indent}  \"min_grounded_samples\": {},\n{indent}  \"min_lifted_samples\": {},\n{indent}  \"min_sky_island_count\": {},\n{indent}  \"min_active_island_count\": {},\n{indent}  \"max_active_chunk_count\": {},\n{indent}  \"min_near_lod_island_count\": {},\n{indent}  \"min_mid_lod_island_count\": {},\n{indent}  \"min_far_lod_island_count\": {},\n{indent}  \"max_visible_island_terrain_count\": {},\n{indent}  \"min_hidden_island_terrain_count\": {},\n{indent}  \"min_visible_island_impostor_count\": {},\n{indent}  \"max_visible_island_detail_count\": {},\n{indent}  \"min_hidden_island_detail_count\": {},\n{indent}  \"min_visible_route_beacon_count\": {},\n{indent}  \"min_weather_cloud_count\": {},\n{indent}  \"min_environment_motion_visual_count\": {},\n{indent}  \"min_environment_motion_offset_m\": {},\n{indent}  \"max_resident_island_visual_count\": {},\n{indent}  \"max_stream_visibility_changes_per_frame\": {},\n{indent}  \"min_entity_count\": {},\n{indent}  \"max_camera_distance_m\": {},\n{indent}  \"min_camera_surface_clearance_m\": {},\n{indent}  \"max_camera_player_angle_degrees\": {},\n{indent}  \"max_camera_step_distance_m\": {},\n{indent}  \"max_camera_rotation_delta_degrees\": {},\n{indent}  \"max_camera_orbit_alignment_degrees\": {},\n{indent}  \"max_abs_camera_view_yaw_degrees\": {},\n{indent}  \"min_camera_obstruction_adjustment_m\": {},\n{indent}  \"min_abs_camera_yaw_degrees\": {},\n{indent}  \"min_camera_pitch_offset_degrees\": {},\n{indent}  \"max_camera_pitch_offset_degrees\": {},\n{indent}  \"min_objective_total_count\": {},\n{indent}  \"min_completed_objective_count\": {},\n{indent}  \"min_visual_asset_slot_count\": {},\n{indent}  \"min_gltf_scene_asset_slot_count\": {},\n{indent}  \"min_streaming_visual_asset_slot_count\": {},\n{indent}  \"max_failed_visual_asset_scene_count\": {},\n{indent}  \"min_power_up_count\": {},\n{indent}  \"min_collected_power_up_count\": {},\n{indent}  \"min_power_up_effect_samples\": {},\n{indent}  \"require_target_landing\": {},\n{indent}  \"max_final_target_distance_m\": {},\n{indent}  \"min_target_landing_samples\": {}\n{indent}}}",
                self.min_samples,
                json_number(self.min_horizontal_distance_m),
                json_number(self.min_max_altitude_m),
                json_number(self.min_max_speed_mps),
                self.min_gliding_samples,
                self.min_grounded_samples,
                self.min_lifted_samples,
                self.min_sky_island_count,
                self.min_active_island_count,
                self.max_active_chunk_count,
                self.min_near_lod_island_count,
                self.min_mid_lod_island_count,
                self.min_far_lod_island_count,
                self.max_visible_island_terrain_count,
                self.min_hidden_island_terrain_count,
                self.min_visible_island_impostor_count,
                self.max_visible_island_detail_count,
                self.min_hidden_island_detail_count,
                self.min_visible_route_beacon_count,
                self.min_weather_cloud_count,
                self.min_environment_motion_visual_count,
                json_number(self.min_environment_motion_offset_m),
                self.max_resident_island_visual_count,
                self.max_stream_visibility_changes_per_frame,
                self.min_entity_count,
                json_number(self.max_camera_distance_m),
                json_number(self.min_camera_surface_clearance_m),
                json_number(self.max_camera_player_angle_degrees),
                json_number(self.max_camera_step_distance_m),
                json_number(self.max_camera_rotation_delta_degrees),
                json_number(self.max_camera_orbit_alignment_degrees),
                json_number(self.max_abs_camera_view_yaw_degrees),
                json_number(self.min_camera_obstruction_adjustment_m),
                json_number(self.min_abs_camera_yaw_degrees),
                json_number(self.min_camera_pitch_offset_degrees),
                json_number(self.max_camera_pitch_offset_degrees),
                self.min_objective_total_count,
                self.min_completed_objective_count,
                self.min_visual_asset_slot_count,
                self.min_gltf_scene_asset_slot_count,
                self.min_streaming_visual_asset_slot_count,
                self.max_failed_visual_asset_scene_count,
                self.min_power_up_count,
                self.min_collected_power_up_count,
                self.min_power_up_effect_samples,
                self.require_target_landing,
                json_number(self.max_final_target_distance_m),
                self.min_target_landing_samples,
            )
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    pub struct EvalObjectiveProgress {
        pub completed_count: usize,
        pub total_count: usize,
        pub current_label: &'static str,
        pub current_distance_m: f32,
        pub complete: bool,
    }

    impl EvalObjectiveProgress {
        pub fn new(
            completed_count: usize,
            total_count: usize,
            current_label: &'static str,
            current_distance_m: f32,
            complete: bool,
        ) -> Self {
            Self {
                completed_count: completed_count.min(total_count),
                total_count,
                current_label,
                current_distance_m,
                complete,
            }
        }

        pub fn current_step(self) -> usize {
            if self.total_count == 0 {
                0
            } else {
                (self.completed_count + 1).min(self.total_count)
            }
        }

        fn to_json(self) -> String {
            format!(
                "{{\"completed_count\":{},\"total_count\":{},\"current_step\":{},\"current_label\":{},\"current_distance_m\":{},\"complete\":{}}}",
                self.completed_count,
                self.total_count,
                self.current_step(),
                json_string(self.current_label),
                json_number(self.current_distance_m),
                self.complete,
            )
        }
    }

    #[derive(Clone, Debug)]
    pub struct EvalSample {
        pub frame: u32,
        pub time_secs: f32,
        pub position: [f32; 3],
        pub velocity: [f32; 3],
        pub speed_mps: f32,
        pub altitude_m: f32,
        pub mode: &'static str,
        pub camera_distance_m: f32,
        pub camera_surface_clearance_m: f32,
        pub camera_player_angle_degrees: f32,
        pub camera_pitch_degrees: f32,
        pub camera_yaw_offset_degrees: f32,
        pub camera_pitch_offset_degrees: f32,
        pub camera_step_distance_m: f32,
        pub camera_rotation_delta_degrees: f32,
        pub camera_orbit_alignment_degrees: f32,
        pub camera_view_yaw_degrees: f32,
        pub camera_obstruction_adjustment_m: f32,
        pub camera_obstruction_hits: usize,
        pub visible_wind_fields: usize,
        pub wind_field_count: usize,
        pub active_lift_fields: usize,
        pub readable_lift_fields: usize,
        pub lift_field_count: usize,
        pub target_distance_m: f32,
        pub on_landing_target: bool,
        pub objective: EvalObjectiveProgress,
        pub sky_island_count: usize,
        pub active_chunk_count: usize,
        pub active_island_count: usize,
        pub near_lod_islands: usize,
        pub mid_lod_islands: usize,
        pub far_lod_islands: usize,
        pub visible_island_terrain_count: usize,
        pub hidden_island_terrain_count: usize,
        pub visible_island_impostor_count: usize,
        pub hidden_island_impostor_count: usize,
        pub visible_island_detail_count: usize,
        pub hidden_island_detail_count: usize,
        pub visible_route_beacon_count: usize,
        pub weather_cloud_count: usize,
        pub environment_motion_visual_count: usize,
        pub max_environment_motion_offset_m: f32,
        pub resident_island_visual_count: usize,
        pub stream_visibility_changes_this_frame: usize,
        pub max_stream_visibility_changes_per_frame: usize,
        pub total_stream_visibility_changes: usize,
        pub catalog_island_visual_count: usize,
        pub hidden_island_visual_count: usize,
        pub resident_island_visual_fraction: f32,
        pub stream_spawned_visuals_this_frame: usize,
        pub stream_despawned_visuals_this_frame: usize,
        pub max_stream_spawned_visuals_per_frame: usize,
        pub max_stream_despawned_visuals_per_frame: usize,
        pub total_stream_spawned_visuals: usize,
        pub total_stream_despawned_visuals: usize,
        pub entity_count: usize,
        pub visual_asset_slot_count: usize,
        pub gltf_scene_asset_slot_count: usize,
        pub ready_visual_asset_slot_count: usize,
        pub placeholder_visual_asset_slot_count: usize,
        pub streaming_visual_asset_slot_count: usize,
        pub missing_visual_asset_slot_count: usize,
        pub queued_visual_asset_scene_count: usize,
        pub loading_visual_asset_scene_count: usize,
        pub loaded_visual_asset_scene_count: usize,
        pub failed_visual_asset_scene_count: usize,
        pub always_visual_asset_slot_count: usize,
        pub stream_window_visual_asset_slot_count: usize,
        pub near_lod_visual_asset_slot_count: usize,
        pub far_lod_visual_asset_slot_count: usize,
        pub weather_visual_asset_slot_count: usize,
        pub power_up_count: usize,
        pub visible_power_up_count: usize,
        pub collected_power_up_count: usize,
        pub active_power_up_effects: usize,
        pub total_power_up_activations: usize,
    }

    impl EvalSample {
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            frame: u32,
            fixed_dt: f32,
            position: Vec3,
            velocity: Vec3,
            mode: FlightMode,
            camera_distance_m: f32,
            camera_surface_clearance_m: f32,
            camera_player_angle_degrees: f32,
            camera_pitch_degrees: f32,
            camera_yaw_offset_degrees: f32,
            camera_pitch_offset_degrees: f32,
            camera_step_distance_m: f32,
            camera_rotation_delta_degrees: f32,
            camera_orbit_alignment_degrees: f32,
            camera_view_yaw_degrees: f32,
            camera_obstruction_adjustment_m: f32,
            camera_obstruction_hits: usize,
            visible_wind_fields: usize,
            wind_field_count: usize,
            active_lift_fields: usize,
            readable_lift_fields: usize,
            lift_field_count: usize,
            target_distance_m: f32,
            on_landing_target: bool,
            objective: EvalObjectiveProgress,
            sky_island_count: usize,
            active_chunk_count: usize,
            active_island_count: usize,
            near_lod_islands: usize,
            mid_lod_islands: usize,
            far_lod_islands: usize,
            visible_island_terrain_count: usize,
            hidden_island_terrain_count: usize,
            visible_island_impostor_count: usize,
            hidden_island_impostor_count: usize,
            visible_island_detail_count: usize,
            hidden_island_detail_count: usize,
            visible_route_beacon_count: usize,
            weather_cloud_count: usize,
            environment_motion_visual_count: usize,
            max_environment_motion_offset_m: f32,
            resident_island_visual_count: usize,
            stream_visibility_changes_this_frame: usize,
            max_stream_visibility_changes_per_frame: usize,
            total_stream_visibility_changes: usize,
            catalog_island_visual_count: usize,
            hidden_island_visual_count: usize,
            resident_island_visual_fraction: f32,
            stream_spawned_visuals_this_frame: usize,
            stream_despawned_visuals_this_frame: usize,
            max_stream_spawned_visuals_per_frame: usize,
            max_stream_despawned_visuals_per_frame: usize,
            total_stream_spawned_visuals: usize,
            total_stream_despawned_visuals: usize,
            entity_count: usize,
            visual_asset_slot_count: usize,
            gltf_scene_asset_slot_count: usize,
            ready_visual_asset_slot_count: usize,
            placeholder_visual_asset_slot_count: usize,
            streaming_visual_asset_slot_count: usize,
            missing_visual_asset_slot_count: usize,
            queued_visual_asset_scene_count: usize,
            loading_visual_asset_scene_count: usize,
            loaded_visual_asset_scene_count: usize,
            failed_visual_asset_scene_count: usize,
            always_visual_asset_slot_count: usize,
            stream_window_visual_asset_slot_count: usize,
            near_lod_visual_asset_slot_count: usize,
            far_lod_visual_asset_slot_count: usize,
            weather_visual_asset_slot_count: usize,
            power_up_count: usize,
            visible_power_up_count: usize,
            collected_power_up_count: usize,
            active_power_up_effects: usize,
            total_power_up_activations: usize,
        ) -> Self {
            Self {
                frame,
                time_secs: frame as f32 * fixed_dt,
                position: vec3_array(position),
                velocity: vec3_array(velocity),
                speed_mps: velocity.length(),
                altitude_m: position.y,
                mode: mode.label(),
                camera_distance_m,
                camera_surface_clearance_m,
                camera_player_angle_degrees,
                camera_pitch_degrees,
                camera_yaw_offset_degrees,
                camera_pitch_offset_degrees,
                camera_step_distance_m,
                camera_rotation_delta_degrees,
                camera_orbit_alignment_degrees,
                camera_view_yaw_degrees,
                camera_obstruction_adjustment_m,
                camera_obstruction_hits,
                visible_wind_fields,
                wind_field_count,
                active_lift_fields,
                readable_lift_fields,
                lift_field_count,
                target_distance_m,
                on_landing_target,
                objective,
                sky_island_count,
                active_chunk_count,
                active_island_count,
                near_lod_islands,
                mid_lod_islands,
                far_lod_islands,
                visible_island_terrain_count,
                hidden_island_terrain_count,
                visible_island_impostor_count,
                hidden_island_impostor_count,
                visible_island_detail_count,
                hidden_island_detail_count,
                visible_route_beacon_count,
                weather_cloud_count,
                environment_motion_visual_count,
                max_environment_motion_offset_m,
                resident_island_visual_count,
                stream_visibility_changes_this_frame,
                max_stream_visibility_changes_per_frame,
                total_stream_visibility_changes,
                catalog_island_visual_count,
                hidden_island_visual_count,
                resident_island_visual_fraction,
                stream_spawned_visuals_this_frame,
                stream_despawned_visuals_this_frame,
                max_stream_spawned_visuals_per_frame,
                max_stream_despawned_visuals_per_frame,
                total_stream_spawned_visuals,
                total_stream_despawned_visuals,
                entity_count,
                visual_asset_slot_count,
                gltf_scene_asset_slot_count,
                ready_visual_asset_slot_count,
                placeholder_visual_asset_slot_count,
                streaming_visual_asset_slot_count,
                missing_visual_asset_slot_count,
                queued_visual_asset_scene_count,
                loading_visual_asset_scene_count,
                loaded_visual_asset_scene_count,
                failed_visual_asset_scene_count,
                always_visual_asset_slot_count,
                stream_window_visual_asset_slot_count,
                near_lod_visual_asset_slot_count,
                far_lod_visual_asset_slot_count,
                weather_visual_asset_slot_count,
                power_up_count,
                visible_power_up_count,
                collected_power_up_count,
                active_power_up_effects,
                total_power_up_activations,
            }
        }

        pub fn to_json(&self) -> String {
            format!(
                "{{\"frame\":{},\"time_secs\":{},\"position\":{},\"velocity\":{},\"speed_mps\":{},\"altitude_m\":{},\"mode\":{},\"camera_distance_m\":{},\"camera_surface_clearance_m\":{},\"camera_player_angle_degrees\":{},\"camera_pitch_degrees\":{},\"camera_yaw_offset_degrees\":{},\"camera_pitch_offset_degrees\":{},\"camera_step_distance_m\":{},\"camera_rotation_delta_degrees\":{},\"camera_orbit_alignment_degrees\":{},\"camera_view_yaw_degrees\":{},\"camera_obstruction_adjustment_m\":{},\"camera_obstruction_hits\":{},\"visible_wind_fields\":{},\"wind_field_count\":{},\"active_lift_fields\":{},\"readable_lift_fields\":{},\"lift_field_count\":{},\"target_distance_m\":{},\"on_landing_target\":{},\"objective\":{},\"sky_island_count\":{},\"active_chunk_count\":{},\"active_island_count\":{},\"near_lod_islands\":{},\"mid_lod_islands\":{},\"far_lod_islands\":{},\"visible_island_terrain_count\":{},\"hidden_island_terrain_count\":{},\"visible_island_impostor_count\":{},\"hidden_island_impostor_count\":{},\"visible_island_detail_count\":{},\"hidden_island_detail_count\":{},\"visible_route_beacon_count\":{},\"weather_cloud_count\":{},\"environment_motion_visual_count\":{},\"max_environment_motion_offset_m\":{},\"resident_island_visual_count\":{},\"stream_visibility_changes_this_frame\":{},\"max_stream_visibility_changes_per_frame\":{},\"total_stream_visibility_changes\":{},\"catalog_island_visual_count\":{},\"hidden_island_visual_count\":{},\"resident_island_visual_fraction\":{},\"stream_spawned_visuals_this_frame\":{},\"stream_despawned_visuals_this_frame\":{},\"max_stream_spawned_visuals_per_frame\":{},\"max_stream_despawned_visuals_per_frame\":{},\"total_stream_spawned_visuals\":{},\"total_stream_despawned_visuals\":{},\"entity_count\":{},\"visual_asset_slot_count\":{},\"gltf_scene_asset_slot_count\":{},\"ready_visual_asset_slot_count\":{},\"placeholder_visual_asset_slot_count\":{},\"streaming_visual_asset_slot_count\":{},\"missing_visual_asset_slot_count\":{},\"queued_visual_asset_scene_count\":{},\"loading_visual_asset_scene_count\":{},\"loaded_visual_asset_scene_count\":{},\"failed_visual_asset_scene_count\":{},\"always_visual_asset_slot_count\":{},\"stream_window_visual_asset_slot_count\":{},\"near_lod_visual_asset_slot_count\":{},\"far_lod_visual_asset_slot_count\":{},\"weather_visual_asset_slot_count\":{},\"power_up_count\":{},\"visible_power_up_count\":{},\"collected_power_up_count\":{},\"active_power_up_effects\":{},\"total_power_up_activations\":{}}}",
                self.frame,
                json_number(self.time_secs),
                json_array3(self.position),
                json_array3(self.velocity),
                json_number(self.speed_mps),
                json_number(self.altitude_m),
                json_string(self.mode),
                json_number(self.camera_distance_m),
                json_number(self.camera_surface_clearance_m),
                json_number(self.camera_player_angle_degrees),
                json_number(self.camera_pitch_degrees),
                json_number(self.camera_yaw_offset_degrees),
                json_number(self.camera_pitch_offset_degrees),
                json_number(self.camera_step_distance_m),
                json_number(self.camera_rotation_delta_degrees),
                json_number(self.camera_orbit_alignment_degrees),
                json_number(self.camera_view_yaw_degrees),
                json_number(self.camera_obstruction_adjustment_m),
                self.camera_obstruction_hits,
                self.visible_wind_fields,
                self.wind_field_count,
                self.active_lift_fields,
                self.readable_lift_fields,
                self.lift_field_count,
                json_number(self.target_distance_m),
                self.on_landing_target,
                self.objective.to_json(),
                self.sky_island_count,
                self.active_chunk_count,
                self.active_island_count,
                self.near_lod_islands,
                self.mid_lod_islands,
                self.far_lod_islands,
                self.visible_island_terrain_count,
                self.hidden_island_terrain_count,
                self.visible_island_impostor_count,
                self.hidden_island_impostor_count,
                self.visible_island_detail_count,
                self.hidden_island_detail_count,
                self.visible_route_beacon_count,
                self.weather_cloud_count,
                self.environment_motion_visual_count,
                json_number(self.max_environment_motion_offset_m),
                self.resident_island_visual_count,
                self.stream_visibility_changes_this_frame,
                self.max_stream_visibility_changes_per_frame,
                self.total_stream_visibility_changes,
                self.catalog_island_visual_count,
                self.hidden_island_visual_count,
                json_number(self.resident_island_visual_fraction),
                self.stream_spawned_visuals_this_frame,
                self.stream_despawned_visuals_this_frame,
                self.max_stream_spawned_visuals_per_frame,
                self.max_stream_despawned_visuals_per_frame,
                self.total_stream_spawned_visuals,
                self.total_stream_despawned_visuals,
                self.entity_count,
                self.visual_asset_slot_count,
                self.gltf_scene_asset_slot_count,
                self.ready_visual_asset_slot_count,
                self.placeholder_visual_asset_slot_count,
                self.streaming_visual_asset_slot_count,
                self.missing_visual_asset_slot_count,
                self.queued_visual_asset_scene_count,
                self.loading_visual_asset_scene_count,
                self.loaded_visual_asset_scene_count,
                self.failed_visual_asset_scene_count,
                self.always_visual_asset_slot_count,
                self.stream_window_visual_asset_slot_count,
                self.near_lod_visual_asset_slot_count,
                self.far_lod_visual_asset_slot_count,
                self.weather_visual_asset_slot_count,
                self.power_up_count,
                self.visible_power_up_count,
                self.collected_power_up_count,
                self.active_power_up_effects,
                self.total_power_up_activations,
            )
        }
    }

    #[derive(Default, Clone, Debug)]
    pub struct EvalAccumulator {
        first_sample: Option<EvalSample>,
        final_sample: Option<EvalSample>,
        frame_times_ms: Vec<f32>,
        sample_count: u32,
        max_altitude_m: f32,
        min_altitude_m: f32,
        max_speed_mps: f32,
        max_camera_distance_m: f32,
        min_camera_surface_clearance_m: f32,
        max_camera_player_angle_degrees: f32,
        max_camera_step_distance_m: f32,
        max_camera_rotation_delta_degrees: f32,
        max_camera_orbit_alignment_degrees: f32,
        max_abs_camera_view_yaw_degrees: f32,
        max_camera_obstruction_adjustment_m: f32,
        max_camera_obstruction_hits: usize,
        min_target_distance_m: f32,
        min_camera_pitch_degrees: f32,
        max_camera_pitch_degrees: f32,
        max_abs_camera_yaw_offset_degrees: f32,
        min_camera_pitch_offset_degrees: f32,
        max_camera_pitch_offset_degrees: f32,
        max_visible_wind_fields: usize,
        max_active_lift_fields: usize,
        max_readable_lift_fields: usize,
        max_sky_island_count: usize,
        max_active_chunk_count: usize,
        max_active_island_count: usize,
        max_near_lod_islands: usize,
        max_mid_lod_islands: usize,
        max_far_lod_islands: usize,
        max_visible_island_terrain_count: usize,
        max_hidden_island_terrain_count: usize,
        max_visible_island_impostor_count: usize,
        max_hidden_island_impostor_count: usize,
        max_visible_island_detail_count: usize,
        max_hidden_island_detail_count: usize,
        max_visible_route_beacon_count: usize,
        max_weather_cloud_count: usize,
        max_environment_motion_visual_count: usize,
        max_environment_motion_offset_m: f32,
        max_resident_island_visual_count: usize,
        max_stream_visibility_changes_per_frame: usize,
        total_stream_visibility_changes: usize,
        max_catalog_island_visual_count: usize,
        max_hidden_island_visual_count: usize,
        max_resident_island_visual_fraction: f32,
        max_stream_spawned_visuals_per_frame: usize,
        max_stream_despawned_visuals_per_frame: usize,
        total_stream_spawned_visuals: usize,
        total_stream_despawned_visuals: usize,
        max_entity_count: usize,
        max_objective_total_count: usize,
        max_completed_objective_count: usize,
        min_objective_distance_m: f32,
        objective_complete_samples: u32,
        max_visual_asset_slot_count: usize,
        max_gltf_scene_asset_slot_count: usize,
        max_ready_visual_asset_slot_count: usize,
        max_placeholder_visual_asset_slot_count: usize,
        max_streaming_visual_asset_slot_count: usize,
        max_missing_visual_asset_slot_count: usize,
        max_queued_visual_asset_scene_count: usize,
        max_loading_visual_asset_scene_count: usize,
        max_loaded_visual_asset_scene_count: usize,
        max_failed_visual_asset_scene_count: usize,
        max_always_visual_asset_slot_count: usize,
        max_stream_window_visual_asset_slot_count: usize,
        max_near_lod_visual_asset_slot_count: usize,
        max_far_lod_visual_asset_slot_count: usize,
        max_weather_visual_asset_slot_count: usize,
        max_power_up_count: usize,
        min_visible_power_up_count: usize,
        max_collected_power_up_count: usize,
        power_up_effect_samples: u32,
        total_power_up_activations: usize,
        target_landing_samples: u32,
        lifted_samples: u32,
        readable_lift_samples: u32,
        unreadable_lift_samples: u32,
        gliding_samples: u32,
        launching_samples: u32,
        grounded_samples: u32,
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct EvalFrameTimeStats {
        avg_ms: f32,
        p95_ms: f32,
        p99_ms: f32,
        max_ms: f32,
    }

    impl EvalFrameTimeStats {
        fn from_samples(samples: &[f32]) -> Self {
            if samples.is_empty() {
                return Self::default();
            }

            let mut sorted = samples.to_vec();
            sorted.sort_by(f32::total_cmp);

            let sum: f32 = sorted.iter().sum();
            Self {
                avg_ms: sum / sorted.len() as f32,
                p95_ms: percentile(&sorted, 0.95),
                p99_ms: percentile(&sorted, 0.99),
                max_ms: *sorted.last().unwrap_or(&0.0),
            }
        }
    }

    impl EvalAccumulator {
        pub fn observe_frame_time_ms(&mut self, frame_time_ms: f32) {
            if frame_time_ms.is_finite() && frame_time_ms >= 0.0 {
                self.frame_times_ms.push(frame_time_ms);
            }
        }

        pub fn observe(&mut self, sample: EvalSample) {
            if self.first_sample.is_none() {
                self.first_sample = Some(sample.clone());
                self.min_altitude_m = sample.altitude_m;
                self.min_camera_surface_clearance_m = sample.camera_surface_clearance_m;
                self.min_target_distance_m = sample.target_distance_m;
                self.min_objective_distance_m = sample.objective.current_distance_m;
                self.min_camera_pitch_degrees = sample.camera_pitch_degrees;
                self.max_camera_pitch_degrees = sample.camera_pitch_degrees;
                self.min_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
                self.max_camera_pitch_offset_degrees = sample.camera_pitch_offset_degrees;
                self.min_visible_power_up_count = sample.visible_power_up_count;
            }

            self.sample_count += 1;
            self.max_altitude_m = self.max_altitude_m.max(sample.altitude_m);
            self.min_altitude_m = self.min_altitude_m.min(sample.altitude_m);
            self.max_speed_mps = self.max_speed_mps.max(sample.speed_mps);
            self.max_camera_distance_m = self.max_camera_distance_m.max(sample.camera_distance_m);
            self.min_camera_surface_clearance_m = self
                .min_camera_surface_clearance_m
                .min(sample.camera_surface_clearance_m);
            self.max_camera_player_angle_degrees = self
                .max_camera_player_angle_degrees
                .max(sample.camera_player_angle_degrees);
            self.max_camera_step_distance_m = self
                .max_camera_step_distance_m
                .max(sample.camera_step_distance_m);
            self.max_camera_rotation_delta_degrees = self
                .max_camera_rotation_delta_degrees
                .max(sample.camera_rotation_delta_degrees);
            self.max_camera_orbit_alignment_degrees = self
                .max_camera_orbit_alignment_degrees
                .max(sample.camera_orbit_alignment_degrees);
            self.max_abs_camera_view_yaw_degrees = self
                .max_abs_camera_view_yaw_degrees
                .max(sample.camera_view_yaw_degrees.abs());
            self.max_camera_obstruction_adjustment_m = self
                .max_camera_obstruction_adjustment_m
                .max(sample.camera_obstruction_adjustment_m);
            self.max_camera_obstruction_hits = self
                .max_camera_obstruction_hits
                .max(sample.camera_obstruction_hits);
            self.min_target_distance_m = self.min_target_distance_m.min(sample.target_distance_m);
            self.min_camera_pitch_degrees = self
                .min_camera_pitch_degrees
                .min(sample.camera_pitch_degrees);
            self.max_camera_pitch_degrees = self
                .max_camera_pitch_degrees
                .max(sample.camera_pitch_degrees);
            self.max_abs_camera_yaw_offset_degrees = self
                .max_abs_camera_yaw_offset_degrees
                .max(sample.camera_yaw_offset_degrees.abs());
            self.min_camera_pitch_offset_degrees = self
                .min_camera_pitch_offset_degrees
                .min(sample.camera_pitch_offset_degrees);
            self.max_camera_pitch_offset_degrees = self
                .max_camera_pitch_offset_degrees
                .max(sample.camera_pitch_offset_degrees);
            self.max_visible_wind_fields =
                self.max_visible_wind_fields.max(sample.visible_wind_fields);
            self.max_active_lift_fields =
                self.max_active_lift_fields.max(sample.active_lift_fields);
            self.max_readable_lift_fields = self
                .max_readable_lift_fields
                .max(sample.readable_lift_fields);
            self.max_sky_island_count = self.max_sky_island_count.max(sample.sky_island_count);
            self.max_active_chunk_count =
                self.max_active_chunk_count.max(sample.active_chunk_count);
            self.max_active_island_count =
                self.max_active_island_count.max(sample.active_island_count);
            self.max_near_lod_islands = self.max_near_lod_islands.max(sample.near_lod_islands);
            self.max_mid_lod_islands = self.max_mid_lod_islands.max(sample.mid_lod_islands);
            self.max_far_lod_islands = self.max_far_lod_islands.max(sample.far_lod_islands);
            self.max_visible_island_terrain_count = self
                .max_visible_island_terrain_count
                .max(sample.visible_island_terrain_count);
            self.max_hidden_island_terrain_count = self
                .max_hidden_island_terrain_count
                .max(sample.hidden_island_terrain_count);
            self.max_visible_island_impostor_count = self
                .max_visible_island_impostor_count
                .max(sample.visible_island_impostor_count);
            self.max_hidden_island_impostor_count = self
                .max_hidden_island_impostor_count
                .max(sample.hidden_island_impostor_count);
            self.max_visible_island_detail_count = self
                .max_visible_island_detail_count
                .max(sample.visible_island_detail_count);
            self.max_hidden_island_detail_count = self
                .max_hidden_island_detail_count
                .max(sample.hidden_island_detail_count);
            self.max_visible_route_beacon_count = self
                .max_visible_route_beacon_count
                .max(sample.visible_route_beacon_count);
            self.max_weather_cloud_count =
                self.max_weather_cloud_count.max(sample.weather_cloud_count);
            self.max_environment_motion_visual_count = self
                .max_environment_motion_visual_count
                .max(sample.environment_motion_visual_count);
            self.max_environment_motion_offset_m = self
                .max_environment_motion_offset_m
                .max(sample.max_environment_motion_offset_m);
            self.max_resident_island_visual_count = self
                .max_resident_island_visual_count
                .max(sample.resident_island_visual_count);
            self.max_stream_visibility_changes_per_frame = self
                .max_stream_visibility_changes_per_frame
                .max(sample.max_stream_visibility_changes_per_frame);
            self.total_stream_visibility_changes = self
                .total_stream_visibility_changes
                .max(sample.total_stream_visibility_changes);
            self.max_catalog_island_visual_count = self
                .max_catalog_island_visual_count
                .max(sample.catalog_island_visual_count);
            self.max_hidden_island_visual_count = self
                .max_hidden_island_visual_count
                .max(sample.hidden_island_visual_count);
            self.max_resident_island_visual_fraction = self
                .max_resident_island_visual_fraction
                .max(sample.resident_island_visual_fraction);
            self.max_stream_spawned_visuals_per_frame = self
                .max_stream_spawned_visuals_per_frame
                .max(sample.max_stream_spawned_visuals_per_frame);
            self.max_stream_despawned_visuals_per_frame = self
                .max_stream_despawned_visuals_per_frame
                .max(sample.max_stream_despawned_visuals_per_frame);
            self.total_stream_spawned_visuals = self
                .total_stream_spawned_visuals
                .max(sample.total_stream_spawned_visuals);
            self.total_stream_despawned_visuals = self
                .total_stream_despawned_visuals
                .max(sample.total_stream_despawned_visuals);
            self.max_entity_count = self.max_entity_count.max(sample.entity_count);
            self.max_objective_total_count = self
                .max_objective_total_count
                .max(sample.objective.total_count);
            self.max_completed_objective_count = self
                .max_completed_objective_count
                .max(sample.objective.completed_count);
            self.min_objective_distance_m = self
                .min_objective_distance_m
                .min(sample.objective.current_distance_m);
            if sample.objective.complete {
                self.objective_complete_samples += 1;
            }
            self.max_visual_asset_slot_count = self
                .max_visual_asset_slot_count
                .max(sample.visual_asset_slot_count);
            self.max_gltf_scene_asset_slot_count = self
                .max_gltf_scene_asset_slot_count
                .max(sample.gltf_scene_asset_slot_count);
            self.max_ready_visual_asset_slot_count = self
                .max_ready_visual_asset_slot_count
                .max(sample.ready_visual_asset_slot_count);
            self.max_placeholder_visual_asset_slot_count = self
                .max_placeholder_visual_asset_slot_count
                .max(sample.placeholder_visual_asset_slot_count);
            self.max_streaming_visual_asset_slot_count = self
                .max_streaming_visual_asset_slot_count
                .max(sample.streaming_visual_asset_slot_count);
            self.max_missing_visual_asset_slot_count = self
                .max_missing_visual_asset_slot_count
                .max(sample.missing_visual_asset_slot_count);
            self.max_queued_visual_asset_scene_count = self
                .max_queued_visual_asset_scene_count
                .max(sample.queued_visual_asset_scene_count);
            self.max_loading_visual_asset_scene_count = self
                .max_loading_visual_asset_scene_count
                .max(sample.loading_visual_asset_scene_count);
            self.max_loaded_visual_asset_scene_count = self
                .max_loaded_visual_asset_scene_count
                .max(sample.loaded_visual_asset_scene_count);
            self.max_failed_visual_asset_scene_count = self
                .max_failed_visual_asset_scene_count
                .max(sample.failed_visual_asset_scene_count);
            self.max_always_visual_asset_slot_count = self
                .max_always_visual_asset_slot_count
                .max(sample.always_visual_asset_slot_count);
            self.max_stream_window_visual_asset_slot_count = self
                .max_stream_window_visual_asset_slot_count
                .max(sample.stream_window_visual_asset_slot_count);
            self.max_near_lod_visual_asset_slot_count = self
                .max_near_lod_visual_asset_slot_count
                .max(sample.near_lod_visual_asset_slot_count);
            self.max_far_lod_visual_asset_slot_count = self
                .max_far_lod_visual_asset_slot_count
                .max(sample.far_lod_visual_asset_slot_count);
            self.max_weather_visual_asset_slot_count = self
                .max_weather_visual_asset_slot_count
                .max(sample.weather_visual_asset_slot_count);
            self.max_power_up_count = self.max_power_up_count.max(sample.power_up_count);
            self.min_visible_power_up_count = self
                .min_visible_power_up_count
                .min(sample.visible_power_up_count);
            self.max_collected_power_up_count = self
                .max_collected_power_up_count
                .max(sample.collected_power_up_count);
            self.total_power_up_activations = self
                .total_power_up_activations
                .max(sample.total_power_up_activations);
            if sample.active_power_up_effects > 0 {
                self.power_up_effect_samples += 1;
            }
            if sample.on_landing_target {
                self.target_landing_samples += 1;
            }
            if sample.active_lift_fields > 0 {
                self.lifted_samples += 1;
                if sample.readable_lift_fields > 0 {
                    self.readable_lift_samples += 1;
                } else {
                    self.unreadable_lift_samples += 1;
                }
            }

            match sample.mode {
                "gliding" => self.gliding_samples += 1,
                "launching" => self.launching_samples += 1,
                "grounded" => self.grounded_samples += 1,
                _ => {}
            }

            self.final_sample = Some(sample);
        }

        pub fn summary(&self, scenario: EvalScenario, artifacts: EvalArtifacts) -> EvalSummary {
            let horizontal_distance_m = match (&self.first_sample, &self.final_sample) {
                (Some(first), Some(final_sample)) => {
                    horizontal_distance(first.position, final_sample.position)
                }
                _ => 0.0,
            };
            let thresholds = scenario.thresholds;
            let final_target_distance_m = self
                .final_sample
                .as_ref()
                .map_or(0.0, |sample| sample.target_distance_m);
            let final_objective_completed_count = self
                .final_sample
                .as_ref()
                .map_or(0, |sample| sample.objective.completed_count);
            let final_objective_distance_m = self
                .final_sample
                .as_ref()
                .map_or(0.0, |sample| sample.objective.current_distance_m);
            let frame_time_stats = EvalFrameTimeStats::from_samples(&self.frame_times_ms);
            let mut checks = vec![
                EvalCheck::at_least(
                    "sample_count",
                    self.sample_count as f32,
                    thresholds.min_samples as f32,
                    "samples",
                ),
                EvalCheck::at_least(
                    "horizontal_distance",
                    horizontal_distance_m,
                    thresholds.min_horizontal_distance_m,
                    "m",
                ),
                EvalCheck::at_least(
                    "max_altitude",
                    self.max_altitude_m,
                    thresholds.min_max_altitude_m,
                    "m",
                ),
                EvalCheck::at_least(
                    "max_speed",
                    self.max_speed_mps,
                    thresholds.min_max_speed_mps,
                    "m/s",
                ),
                EvalCheck::at_least(
                    "gliding_samples",
                    self.gliding_samples as f32,
                    thresholds.min_gliding_samples as f32,
                    "samples",
                ),
                EvalCheck::at_least(
                    "grounded_samples",
                    self.grounded_samples as f32,
                    thresholds.min_grounded_samples as f32,
                    "samples",
                ),
                EvalCheck::at_least(
                    "lifted_samples",
                    self.lifted_samples as f32,
                    thresholds.min_lifted_samples as f32,
                    "samples",
                ),
                EvalCheck::at_least(
                    "sky_island_count",
                    self.max_sky_island_count as f32,
                    thresholds.min_sky_island_count as f32,
                    "islands",
                ),
                EvalCheck::at_least(
                    "active_island_count",
                    self.max_active_island_count as f32,
                    thresholds.min_active_island_count as f32,
                    "islands",
                ),
                EvalCheck::at_most(
                    "active_chunk_count",
                    self.max_active_chunk_count as f32,
                    thresholds.max_active_chunk_count as f32,
                    "chunks",
                ),
                EvalCheck::at_least(
                    "near_lod_island_count",
                    self.max_near_lod_islands as f32,
                    thresholds.min_near_lod_island_count as f32,
                    "islands",
                ),
                EvalCheck::at_least(
                    "mid_lod_island_count",
                    self.max_mid_lod_islands as f32,
                    thresholds.min_mid_lod_island_count as f32,
                    "islands",
                ),
                EvalCheck::at_least(
                    "far_lod_island_count",
                    self.max_far_lod_islands as f32,
                    thresholds.min_far_lod_island_count as f32,
                    "islands",
                ),
                EvalCheck::at_most(
                    "visible_island_terrain_count",
                    self.max_visible_island_terrain_count as f32,
                    thresholds.max_visible_island_terrain_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "hidden_island_terrain_count",
                    self.max_hidden_island_terrain_count as f32,
                    thresholds.min_hidden_island_terrain_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "visible_island_impostor_count",
                    self.max_visible_island_impostor_count as f32,
                    thresholds.min_visible_island_impostor_count as f32,
                    "entities",
                ),
                EvalCheck::at_most(
                    "visible_island_detail_count",
                    self.max_visible_island_detail_count as f32,
                    thresholds.max_visible_island_detail_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "hidden_island_detail_count",
                    self.max_hidden_island_detail_count as f32,
                    thresholds.min_hidden_island_detail_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "visible_route_beacon_count",
                    self.max_visible_route_beacon_count as f32,
                    thresholds.min_visible_route_beacon_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "weather_cloud_count",
                    self.max_weather_cloud_count as f32,
                    thresholds.min_weather_cloud_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "environment_motion_visual_count",
                    self.max_environment_motion_visual_count as f32,
                    thresholds.min_environment_motion_visual_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "environment_motion_offset",
                    self.max_environment_motion_offset_m,
                    thresholds.min_environment_motion_offset_m,
                    "m",
                ),
                EvalCheck::at_most(
                    "resident_island_visual_count",
                    self.max_resident_island_visual_count as f32,
                    thresholds.max_resident_island_visual_count as f32,
                    "entities",
                ),
                EvalCheck::at_most(
                    "stream_visibility_changes_per_frame",
                    self.max_stream_visibility_changes_per_frame as f32,
                    thresholds.max_stream_visibility_changes_per_frame as f32,
                    "entities/frame",
                ),
                EvalCheck::at_least(
                    "hidden_island_visual_count",
                    self.max_hidden_island_visual_count as f32,
                    (thresholds.min_hidden_island_terrain_count
                        + thresholds.min_hidden_island_detail_count) as f32,
                    "entities",
                ),
                EvalCheck::at_most(
                    "resident_island_visual_fraction",
                    self.max_resident_island_visual_fraction,
                    MAX_RESIDENT_ISLAND_VISUAL_FRACTION,
                    "ratio",
                ),
                EvalCheck::at_most(
                    "stream_spawned_visuals_per_frame",
                    self.max_stream_spawned_visuals_per_frame as f32,
                    thresholds.max_stream_visibility_changes_per_frame as f32,
                    "entities/frame",
                ),
                EvalCheck::at_most(
                    "stream_despawned_visuals_per_frame",
                    self.max_stream_despawned_visuals_per_frame as f32,
                    thresholds.max_stream_visibility_changes_per_frame as f32,
                    "entities/frame",
                ),
                EvalCheck::at_least(
                    "entity_count",
                    self.max_entity_count as f32,
                    thresholds.min_entity_count as f32,
                    "entities",
                ),
                EvalCheck::at_least(
                    "objective_total_count",
                    self.max_objective_total_count as f32,
                    thresholds.min_objective_total_count as f32,
                    "objectives",
                ),
                EvalCheck::at_least(
                    "completed_objective_count",
                    self.max_completed_objective_count as f32,
                    thresholds.min_completed_objective_count as f32,
                    "objectives",
                ),
                EvalCheck::at_least(
                    "visual_asset_slot_count",
                    self.max_visual_asset_slot_count as f32,
                    thresholds.min_visual_asset_slot_count as f32,
                    "assets",
                ),
                EvalCheck::at_least(
                    "gltf_scene_asset_slot_count",
                    self.max_gltf_scene_asset_slot_count as f32,
                    thresholds.min_gltf_scene_asset_slot_count as f32,
                    "assets",
                ),
                EvalCheck::at_least(
                    "streaming_visual_asset_slot_count",
                    self.max_streaming_visual_asset_slot_count as f32,
                    thresholds.min_streaming_visual_asset_slot_count as f32,
                    "assets",
                ),
                EvalCheck::at_most(
                    "failed_visual_asset_scene_count",
                    self.max_failed_visual_asset_scene_count as f32,
                    thresholds.max_failed_visual_asset_scene_count as f32,
                    "assets",
                ),
                EvalCheck::at_least(
                    "power_up_count",
                    self.max_power_up_count as f32,
                    thresholds.min_power_up_count as f32,
                    "power-ups",
                ),
                EvalCheck::at_least(
                    "collected_power_up_count",
                    self.max_collected_power_up_count as f32,
                    thresholds.min_collected_power_up_count as f32,
                    "power-ups",
                ),
                EvalCheck::at_least(
                    "power_up_effect_samples",
                    self.power_up_effect_samples as f32,
                    thresholds.min_power_up_effect_samples as f32,
                    "samples",
                ),
                EvalCheck::at_most(
                    "max_camera_distance",
                    self.max_camera_distance_m,
                    thresholds.max_camera_distance_m,
                    "m",
                ),
                EvalCheck::at_least(
                    "min_camera_surface_clearance",
                    self.min_camera_surface_clearance_m,
                    thresholds.min_camera_surface_clearance_m,
                    "m",
                ),
                EvalCheck::at_most(
                    "max_camera_player_angle",
                    self.max_camera_player_angle_degrees,
                    thresholds.max_camera_player_angle_degrees,
                    "deg",
                ),
                EvalCheck::at_most(
                    "max_camera_step_distance",
                    self.max_camera_step_distance_m,
                    thresholds.max_camera_step_distance_m,
                    "m",
                ),
                EvalCheck::at_most(
                    "max_camera_rotation_delta",
                    self.max_camera_rotation_delta_degrees,
                    thresholds.max_camera_rotation_delta_degrees,
                    "deg",
                ),
                EvalCheck::at_most(
                    "max_camera_orbit_alignment",
                    self.max_camera_orbit_alignment_degrees,
                    thresholds.max_camera_orbit_alignment_degrees,
                    "deg",
                ),
                EvalCheck::at_most(
                    "max_abs_camera_view_yaw",
                    self.max_abs_camera_view_yaw_degrees,
                    thresholds.max_abs_camera_view_yaw_degrees,
                    "deg",
                ),
                EvalCheck::at_least(
                    "max_camera_obstruction_adjustment",
                    self.max_camera_obstruction_adjustment_m,
                    thresholds.min_camera_obstruction_adjustment_m,
                    "m",
                ),
                EvalCheck::at_least(
                    "max_abs_camera_yaw_offset",
                    self.max_abs_camera_yaw_offset_degrees,
                    thresholds.min_abs_camera_yaw_degrees,
                    "deg",
                ),
                EvalCheck::at_most(
                    "min_camera_pitch_offset",
                    self.min_camera_pitch_offset_degrees,
                    thresholds.min_camera_pitch_offset_degrees,
                    "deg",
                ),
                EvalCheck::at_least(
                    "max_camera_pitch_offset",
                    self.max_camera_pitch_offset_degrees,
                    thresholds.max_camera_pitch_offset_degrees,
                    "deg",
                ),
            ];
            if thresholds.min_lifted_samples > 0 {
                checks.push(EvalCheck::at_least(
                    "readable_lift_samples",
                    self.readable_lift_samples as f32,
                    thresholds.min_lifted_samples as f32,
                    "samples",
                ));
                checks.push(EvalCheck::at_most(
                    "unreadable_lift_samples",
                    self.unreadable_lift_samples as f32,
                    0.0,
                    "samples",
                ));
            }
            if thresholds.require_target_landing {
                checks.push(EvalCheck::at_most(
                    "final_target_distance",
                    final_target_distance_m,
                    thresholds.max_final_target_distance_m,
                    "m",
                ));
                checks.push(EvalCheck::at_least(
                    "target_landing_samples",
                    self.target_landing_samples as f32,
                    thresholds.min_target_landing_samples as f32,
                    "samples",
                ));
            }
            let passed = checks.iter().all(|check| check.passed);

            EvalSummary {
                scenario_name: scenario.name,
                target_island_name: scenario.target_island_name,
                passed,
                frame_count: scenario.frame_count,
                duration_secs: scenario.duration_secs(),
                thresholds,
                metrics: EvalMetricsSummary {
                    sample_count: self.sample_count,
                    avg_frame_time_ms: frame_time_stats.avg_ms,
                    p95_frame_time_ms: frame_time_stats.p95_ms,
                    p99_frame_time_ms: frame_time_stats.p99_ms,
                    max_frame_time_ms: frame_time_stats.max_ms,
                    horizontal_distance_m,
                    max_altitude_m: self.max_altitude_m,
                    min_altitude_m: self.min_altitude_m,
                    max_speed_mps: self.max_speed_mps,
                    max_camera_distance_m: self.max_camera_distance_m,
                    min_camera_surface_clearance_m: self.min_camera_surface_clearance_m,
                    max_camera_player_angle_degrees: self.max_camera_player_angle_degrees,
                    max_camera_step_distance_m: self.max_camera_step_distance_m,
                    max_camera_rotation_delta_degrees: self.max_camera_rotation_delta_degrees,
                    max_camera_orbit_alignment_degrees: self.max_camera_orbit_alignment_degrees,
                    max_abs_camera_view_yaw_degrees: self.max_abs_camera_view_yaw_degrees,
                    max_camera_obstruction_adjustment_m: self.max_camera_obstruction_adjustment_m,
                    max_camera_obstruction_hits: self.max_camera_obstruction_hits,
                    min_target_distance_m: self.min_target_distance_m,
                    final_target_distance_m,
                    min_camera_pitch_degrees: self.min_camera_pitch_degrees,
                    max_camera_pitch_degrees: self.max_camera_pitch_degrees,
                    max_abs_camera_yaw_offset_degrees: self.max_abs_camera_yaw_offset_degrees,
                    min_camera_pitch_offset_degrees: self.min_camera_pitch_offset_degrees,
                    max_camera_pitch_offset_degrees: self.max_camera_pitch_offset_degrees,
                    max_visible_wind_fields: self.max_visible_wind_fields,
                    max_active_lift_fields: self.max_active_lift_fields,
                    max_readable_lift_fields: self.max_readable_lift_fields,
                    max_sky_island_count: self.max_sky_island_count,
                    max_active_chunk_count: self.max_active_chunk_count,
                    max_active_island_count: self.max_active_island_count,
                    max_near_lod_islands: self.max_near_lod_islands,
                    max_mid_lod_islands: self.max_mid_lod_islands,
                    max_far_lod_islands: self.max_far_lod_islands,
                    max_visible_island_terrain_count: self.max_visible_island_terrain_count,
                    max_hidden_island_terrain_count: self.max_hidden_island_terrain_count,
                    max_visible_island_impostor_count: self.max_visible_island_impostor_count,
                    max_hidden_island_impostor_count: self.max_hidden_island_impostor_count,
                    max_visible_island_detail_count: self.max_visible_island_detail_count,
                    max_hidden_island_detail_count: self.max_hidden_island_detail_count,
                    max_visible_route_beacon_count: self.max_visible_route_beacon_count,
                    max_weather_cloud_count: self.max_weather_cloud_count,
                    max_environment_motion_visual_count: self.max_environment_motion_visual_count,
                    max_environment_motion_offset_m: self.max_environment_motion_offset_m,
                    max_resident_island_visual_count: self.max_resident_island_visual_count,
                    max_stream_visibility_changes_per_frame: self
                        .max_stream_visibility_changes_per_frame,
                    total_stream_visibility_changes: self.total_stream_visibility_changes,
                    max_catalog_island_visual_count: self.max_catalog_island_visual_count,
                    max_hidden_island_visual_count: self.max_hidden_island_visual_count,
                    max_resident_island_visual_fraction: self.max_resident_island_visual_fraction,
                    max_stream_spawned_visuals_per_frame: self.max_stream_spawned_visuals_per_frame,
                    max_stream_despawned_visuals_per_frame: self
                        .max_stream_despawned_visuals_per_frame,
                    total_stream_spawned_visuals: self.total_stream_spawned_visuals,
                    total_stream_despawned_visuals: self.total_stream_despawned_visuals,
                    max_entity_count: self.max_entity_count,
                    objective_total_count: self.max_objective_total_count,
                    max_completed_objective_count: self.max_completed_objective_count,
                    final_objective_completed_count,
                    min_objective_distance_m: self.min_objective_distance_m,
                    final_objective_distance_m,
                    objective_complete_samples: self.objective_complete_samples,
                    max_visual_asset_slot_count: self.max_visual_asset_slot_count,
                    max_gltf_scene_asset_slot_count: self.max_gltf_scene_asset_slot_count,
                    max_ready_visual_asset_slot_count: self.max_ready_visual_asset_slot_count,
                    max_placeholder_visual_asset_slot_count: self
                        .max_placeholder_visual_asset_slot_count,
                    max_streaming_visual_asset_slot_count: self
                        .max_streaming_visual_asset_slot_count,
                    max_missing_visual_asset_slot_count: self.max_missing_visual_asset_slot_count,
                    max_queued_visual_asset_scene_count: self.max_queued_visual_asset_scene_count,
                    max_loading_visual_asset_scene_count: self.max_loading_visual_asset_scene_count,
                    max_loaded_visual_asset_scene_count: self.max_loaded_visual_asset_scene_count,
                    max_failed_visual_asset_scene_count: self.max_failed_visual_asset_scene_count,
                    max_always_visual_asset_slot_count: self.max_always_visual_asset_slot_count,
                    max_stream_window_visual_asset_slot_count: self
                        .max_stream_window_visual_asset_slot_count,
                    max_near_lod_visual_asset_slot_count: self.max_near_lod_visual_asset_slot_count,
                    max_far_lod_visual_asset_slot_count: self.max_far_lod_visual_asset_slot_count,
                    max_weather_visual_asset_slot_count: self.max_weather_visual_asset_slot_count,
                    max_power_up_count: self.max_power_up_count,
                    min_visible_power_up_count: self.min_visible_power_up_count,
                    max_collected_power_up_count: self.max_collected_power_up_count,
                    power_up_effect_samples: self.power_up_effect_samples,
                    total_power_up_activations: self.total_power_up_activations,
                    target_landing_samples: self.target_landing_samples,
                    lifted_samples: self.lifted_samples,
                    readable_lift_samples: self.readable_lift_samples,
                    unreadable_lift_samples: self.unreadable_lift_samples,
                    gliding_samples: self.gliding_samples,
                    launching_samples: self.launching_samples,
                    grounded_samples: self.grounded_samples,
                },
                checks,
                artifacts,
                final_sample: self.final_sample.clone(),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct EvalArtifacts {
        pub summary_json: String,
        pub samples_ndjson: String,
        pub screenshot_png: Option<String>,
        pub checkpoint_screenshots: Vec<String>,
    }

    impl EvalArtifacts {
        fn to_json(&self, indent: &str) -> String {
            let screenshot = self
                .screenshot_png
                .as_deref()
                .map(json_string)
                .unwrap_or_else(|| "null".to_string());
            let checkpoint_screenshots = json_string_array(&self.checkpoint_screenshots);

            format!(
                "{{\n{indent}  \"summary_json\": {},\n{indent}  \"samples_ndjson\": {},\n{indent}  \"screenshot_png\": {},\n{indent}  \"checkpoint_screenshots\": {}\n{indent}}}",
                json_string(&self.summary_json),
                json_string(&self.samples_ndjson),
                screenshot,
                checkpoint_screenshots,
            )
        }
    }

    #[derive(Clone, Debug)]
    pub struct EvalMetricsSummary {
        pub sample_count: u32,
        pub avg_frame_time_ms: f32,
        pub p95_frame_time_ms: f32,
        pub p99_frame_time_ms: f32,
        pub max_frame_time_ms: f32,
        pub horizontal_distance_m: f32,
        pub max_altitude_m: f32,
        pub min_altitude_m: f32,
        pub max_speed_mps: f32,
        pub max_camera_distance_m: f32,
        pub min_camera_surface_clearance_m: f32,
        pub max_camera_player_angle_degrees: f32,
        pub max_camera_step_distance_m: f32,
        pub max_camera_rotation_delta_degrees: f32,
        pub max_camera_orbit_alignment_degrees: f32,
        pub max_abs_camera_view_yaw_degrees: f32,
        pub max_camera_obstruction_adjustment_m: f32,
        pub max_camera_obstruction_hits: usize,
        pub min_target_distance_m: f32,
        pub final_target_distance_m: f32,
        pub min_camera_pitch_degrees: f32,
        pub max_camera_pitch_degrees: f32,
        pub max_abs_camera_yaw_offset_degrees: f32,
        pub min_camera_pitch_offset_degrees: f32,
        pub max_camera_pitch_offset_degrees: f32,
        pub max_visible_wind_fields: usize,
        pub max_active_lift_fields: usize,
        pub max_readable_lift_fields: usize,
        pub max_sky_island_count: usize,
        pub max_active_chunk_count: usize,
        pub max_active_island_count: usize,
        pub max_near_lod_islands: usize,
        pub max_mid_lod_islands: usize,
        pub max_far_lod_islands: usize,
        pub max_visible_island_terrain_count: usize,
        pub max_hidden_island_terrain_count: usize,
        pub max_visible_island_impostor_count: usize,
        pub max_hidden_island_impostor_count: usize,
        pub max_visible_island_detail_count: usize,
        pub max_hidden_island_detail_count: usize,
        pub max_visible_route_beacon_count: usize,
        pub max_weather_cloud_count: usize,
        pub max_environment_motion_visual_count: usize,
        pub max_environment_motion_offset_m: f32,
        pub max_resident_island_visual_count: usize,
        pub max_stream_visibility_changes_per_frame: usize,
        pub total_stream_visibility_changes: usize,
        pub max_catalog_island_visual_count: usize,
        pub max_hidden_island_visual_count: usize,
        pub max_resident_island_visual_fraction: f32,
        pub max_stream_spawned_visuals_per_frame: usize,
        pub max_stream_despawned_visuals_per_frame: usize,
        pub total_stream_spawned_visuals: usize,
        pub total_stream_despawned_visuals: usize,
        pub max_entity_count: usize,
        pub objective_total_count: usize,
        pub max_completed_objective_count: usize,
        pub final_objective_completed_count: usize,
        pub min_objective_distance_m: f32,
        pub final_objective_distance_m: f32,
        pub objective_complete_samples: u32,
        pub max_visual_asset_slot_count: usize,
        pub max_gltf_scene_asset_slot_count: usize,
        pub max_ready_visual_asset_slot_count: usize,
        pub max_placeholder_visual_asset_slot_count: usize,
        pub max_streaming_visual_asset_slot_count: usize,
        pub max_missing_visual_asset_slot_count: usize,
        pub max_queued_visual_asset_scene_count: usize,
        pub max_loading_visual_asset_scene_count: usize,
        pub max_loaded_visual_asset_scene_count: usize,
        pub max_failed_visual_asset_scene_count: usize,
        pub max_always_visual_asset_slot_count: usize,
        pub max_stream_window_visual_asset_slot_count: usize,
        pub max_near_lod_visual_asset_slot_count: usize,
        pub max_far_lod_visual_asset_slot_count: usize,
        pub max_weather_visual_asset_slot_count: usize,
        pub max_power_up_count: usize,
        pub min_visible_power_up_count: usize,
        pub max_collected_power_up_count: usize,
        pub power_up_effect_samples: u32,
        pub total_power_up_activations: usize,
        pub target_landing_samples: u32,
        pub lifted_samples: u32,
        pub readable_lift_samples: u32,
        pub unreadable_lift_samples: u32,
        pub gliding_samples: u32,
        pub launching_samples: u32,
        pub grounded_samples: u32,
    }

    impl EvalMetricsSummary {
        fn to_json(&self, indent: &str) -> String {
            format!(
                "{{\n{indent}  \"sample_count\": {},\n{indent}  \"avg_frame_time_ms\": {},\n{indent}  \"p95_frame_time_ms\": {},\n{indent}  \"p99_frame_time_ms\": {},\n{indent}  \"max_frame_time_ms\": {},\n{indent}  \"horizontal_distance_m\": {},\n{indent}  \"max_altitude_m\": {},\n{indent}  \"min_altitude_m\": {},\n{indent}  \"max_speed_mps\": {},\n{indent}  \"max_camera_distance_m\": {},\n{indent}  \"min_camera_surface_clearance_m\": {},\n{indent}  \"max_camera_player_angle_degrees\": {},\n{indent}  \"max_camera_step_distance_m\": {},\n{indent}  \"max_camera_rotation_delta_degrees\": {},\n{indent}  \"max_camera_orbit_alignment_degrees\": {},\n{indent}  \"max_abs_camera_view_yaw_degrees\": {},\n{indent}  \"max_camera_obstruction_adjustment_m\": {},\n{indent}  \"max_camera_obstruction_hits\": {},\n{indent}  \"min_target_distance_m\": {},\n{indent}  \"final_target_distance_m\": {},\n{indent}  \"min_camera_pitch_degrees\": {},\n{indent}  \"max_camera_pitch_degrees\": {},\n{indent}  \"max_abs_camera_yaw_offset_degrees\": {},\n{indent}  \"min_camera_pitch_offset_degrees\": {},\n{indent}  \"max_camera_pitch_offset_degrees\": {},\n{indent}  \"max_visible_wind_fields\": {},\n{indent}  \"max_active_lift_fields\": {},\n{indent}  \"max_readable_lift_fields\": {},\n{indent}  \"max_sky_island_count\": {},\n{indent}  \"max_active_chunk_count\": {},\n{indent}  \"max_active_island_count\": {},\n{indent}  \"max_near_lod_islands\": {},\n{indent}  \"max_mid_lod_islands\": {},\n{indent}  \"max_far_lod_islands\": {},\n{indent}  \"max_visible_island_terrain_count\": {},\n{indent}  \"max_hidden_island_terrain_count\": {},\n{indent}  \"max_visible_island_impostor_count\": {},\n{indent}  \"max_hidden_island_impostor_count\": {},\n{indent}  \"max_visible_island_detail_count\": {},\n{indent}  \"max_hidden_island_detail_count\": {},\n{indent}  \"max_visible_route_beacon_count\": {},\n{indent}  \"max_weather_cloud_count\": {},\n{indent}  \"max_environment_motion_visual_count\": {},\n{indent}  \"max_environment_motion_offset_m\": {},\n{indent}  \"max_resident_island_visual_count\": {},\n{indent}  \"max_stream_visibility_changes_per_frame\": {},\n{indent}  \"total_stream_visibility_changes\": {},\n{indent}  \"max_catalog_island_visual_count\": {},\n{indent}  \"max_hidden_island_visual_count\": {},\n{indent}  \"max_resident_island_visual_fraction\": {},\n{indent}  \"max_stream_spawned_visuals_per_frame\": {},\n{indent}  \"max_stream_despawned_visuals_per_frame\": {},\n{indent}  \"total_stream_spawned_visuals\": {},\n{indent}  \"total_stream_despawned_visuals\": {},\n{indent}  \"max_entity_count\": {},\n{indent}  \"objective_total_count\": {},\n{indent}  \"max_completed_objective_count\": {},\n{indent}  \"final_objective_completed_count\": {},\n{indent}  \"min_objective_distance_m\": {},\n{indent}  \"final_objective_distance_m\": {},\n{indent}  \"objective_complete_samples\": {},\n{indent}  \"max_visual_asset_slot_count\": {},\n{indent}  \"max_gltf_scene_asset_slot_count\": {},\n{indent}  \"max_ready_visual_asset_slot_count\": {},\n{indent}  \"max_placeholder_visual_asset_slot_count\": {},\n{indent}  \"max_streaming_visual_asset_slot_count\": {},\n{indent}  \"max_missing_visual_asset_slot_count\": {},\n{indent}  \"max_queued_visual_asset_scene_count\": {},\n{indent}  \"max_loading_visual_asset_scene_count\": {},\n{indent}  \"max_loaded_visual_asset_scene_count\": {},\n{indent}  \"max_failed_visual_asset_scene_count\": {},\n{indent}  \"max_always_visual_asset_slot_count\": {},\n{indent}  \"max_stream_window_visual_asset_slot_count\": {},\n{indent}  \"max_near_lod_visual_asset_slot_count\": {},\n{indent}  \"max_far_lod_visual_asset_slot_count\": {},\n{indent}  \"max_weather_visual_asset_slot_count\": {},\n{indent}  \"max_power_up_count\": {},\n{indent}  \"min_visible_power_up_count\": {},\n{indent}  \"max_collected_power_up_count\": {},\n{indent}  \"power_up_effect_samples\": {},\n{indent}  \"total_power_up_activations\": {},\n{indent}  \"target_landing_samples\": {},\n{indent}  \"lifted_samples\": {},\n{indent}  \"readable_lift_samples\": {},\n{indent}  \"unreadable_lift_samples\": {},\n{indent}  \"gliding_samples\": {},\n{indent}  \"launching_samples\": {},\n{indent}  \"grounded_samples\": {}\n{indent}}}",
                self.sample_count,
                json_number(self.avg_frame_time_ms),
                json_number(self.p95_frame_time_ms),
                json_number(self.p99_frame_time_ms),
                json_number(self.max_frame_time_ms),
                json_number(self.horizontal_distance_m),
                json_number(self.max_altitude_m),
                json_number(self.min_altitude_m),
                json_number(self.max_speed_mps),
                json_number(self.max_camera_distance_m),
                json_number(self.min_camera_surface_clearance_m),
                json_number(self.max_camera_player_angle_degrees),
                json_number(self.max_camera_step_distance_m),
                json_number(self.max_camera_rotation_delta_degrees),
                json_number(self.max_camera_orbit_alignment_degrees),
                json_number(self.max_abs_camera_view_yaw_degrees),
                json_number(self.max_camera_obstruction_adjustment_m),
                self.max_camera_obstruction_hits,
                json_number(self.min_target_distance_m),
                json_number(self.final_target_distance_m),
                json_number(self.min_camera_pitch_degrees),
                json_number(self.max_camera_pitch_degrees),
                json_number(self.max_abs_camera_yaw_offset_degrees),
                json_number(self.min_camera_pitch_offset_degrees),
                json_number(self.max_camera_pitch_offset_degrees),
                self.max_visible_wind_fields,
                self.max_active_lift_fields,
                self.max_readable_lift_fields,
                self.max_sky_island_count,
                self.max_active_chunk_count,
                self.max_active_island_count,
                self.max_near_lod_islands,
                self.max_mid_lod_islands,
                self.max_far_lod_islands,
                self.max_visible_island_terrain_count,
                self.max_hidden_island_terrain_count,
                self.max_visible_island_impostor_count,
                self.max_hidden_island_impostor_count,
                self.max_visible_island_detail_count,
                self.max_hidden_island_detail_count,
                self.max_visible_route_beacon_count,
                self.max_weather_cloud_count,
                self.max_environment_motion_visual_count,
                json_number(self.max_environment_motion_offset_m),
                self.max_resident_island_visual_count,
                self.max_stream_visibility_changes_per_frame,
                self.total_stream_visibility_changes,
                self.max_catalog_island_visual_count,
                self.max_hidden_island_visual_count,
                json_number(self.max_resident_island_visual_fraction),
                self.max_stream_spawned_visuals_per_frame,
                self.max_stream_despawned_visuals_per_frame,
                self.total_stream_spawned_visuals,
                self.total_stream_despawned_visuals,
                self.max_entity_count,
                self.objective_total_count,
                self.max_completed_objective_count,
                self.final_objective_completed_count,
                json_number(self.min_objective_distance_m),
                json_number(self.final_objective_distance_m),
                self.objective_complete_samples,
                self.max_visual_asset_slot_count,
                self.max_gltf_scene_asset_slot_count,
                self.max_ready_visual_asset_slot_count,
                self.max_placeholder_visual_asset_slot_count,
                self.max_streaming_visual_asset_slot_count,
                self.max_missing_visual_asset_slot_count,
                self.max_queued_visual_asset_scene_count,
                self.max_loading_visual_asset_scene_count,
                self.max_loaded_visual_asset_scene_count,
                self.max_failed_visual_asset_scene_count,
                self.max_always_visual_asset_slot_count,
                self.max_stream_window_visual_asset_slot_count,
                self.max_near_lod_visual_asset_slot_count,
                self.max_far_lod_visual_asset_slot_count,
                self.max_weather_visual_asset_slot_count,
                self.max_power_up_count,
                self.min_visible_power_up_count,
                self.max_collected_power_up_count,
                self.power_up_effect_samples,
                self.total_power_up_activations,
                self.target_landing_samples,
                self.lifted_samples,
                self.readable_lift_samples,
                self.unreadable_lift_samples,
                self.gliding_samples,
                self.launching_samples,
                self.grounded_samples,
            )
        }
    }

    #[derive(Clone, Debug)]
    pub struct EvalCheck {
        pub name: &'static str,
        pub passed: bool,
        pub value: f32,
        pub threshold: f32,
        pub comparator: &'static str,
        pub unit: &'static str,
    }

    impl EvalCheck {
        fn at_least(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
            Self {
                name,
                passed: value >= threshold,
                value,
                threshold,
                comparator: ">=",
                unit,
            }
        }

        fn at_most(name: &'static str, value: f32, threshold: f32, unit: &'static str) -> Self {
            Self {
                name,
                passed: value <= threshold,
                value,
                threshold,
                comparator: "<=",
                unit,
            }
        }

        fn to_json(&self, indent: &str) -> String {
            format!(
                "{{\n{indent}  \"name\": {},\n{indent}  \"passed\": {},\n{indent}  \"value\": {},\n{indent}  \"comparator\": {},\n{indent}  \"threshold\": {},\n{indent}  \"unit\": {}\n{indent}}}",
                json_string(self.name),
                self.passed,
                json_number(self.value),
                json_string(self.comparator),
                json_number(self.threshold),
                json_string(self.unit),
            )
        }
    }

    #[derive(Clone, Debug)]
    pub struct EvalSummary {
        pub scenario_name: &'static str,
        pub target_island_name: Option<&'static str>,
        pub passed: bool,
        pub frame_count: u32,
        pub duration_secs: f32,
        pub thresholds: EvalThresholds,
        pub metrics: EvalMetricsSummary,
        pub checks: Vec<EvalCheck>,
        pub artifacts: EvalArtifacts,
        pub final_sample: Option<EvalSample>,
    }

    impl EvalSummary {
        pub fn to_json(&self) -> String {
            let checks = self
                .checks
                .iter()
                .map(|check| check.to_json("      "))
                .collect::<Vec<_>>()
                .join(",\n");
            let final_sample = self
                .final_sample
                .as_ref()
                .map(EvalSample::to_json)
                .unwrap_or_else(|| "null".to_string());
            let target_island = self
                .target_island_name
                .map(json_string)
                .unwrap_or_else(|| "null".to_string());

            format!(
                "{{\n  \"scenario\": {},\n  \"target_island\": {},\n  \"passed\": {},\n  \"frame_count\": {},\n  \"duration_secs\": {},\n  \"thresholds\": {},\n  \"metrics\": {},\n  \"checks\": [\n{}\n  ],\n  \"artifacts\": {},\n  \"final_sample\": {}\n}}\n",
                json_string(self.scenario_name),
                target_island,
                self.passed,
                self.frame_count,
                json_number(self.duration_secs),
                self.thresholds.to_json("  "),
                self.metrics.to_json("  "),
                checks,
                self.artifacts.to_json("  "),
                final_sample,
            )
        }
    }

    pub fn scenario_named(name: &str) -> Option<EvalScenario> {
        match name {
            BASELINE_ROUTE | "baseline" => Some(baseline_route()),
            ISLAND_LAUNCH_TO_LANDING | "island" => Some(island_launch_to_landing()),
            GROUND_TAXI_CONTROL | "ground_taxi" | "taxi" => Some(ground_taxi_control()),
            UPDRAFT_ROUTE | "updraft" => Some(updraft_route()),
            BRANCH_RECOVERY_ROUTE | "branch_recovery" | "recovery_route" => {
                Some(branch_recovery_route())
            }
            CAMERA_MOUSE_CONTROL | "camera_mouse" | "mouse_camera" => Some(camera_mouse_control()),
            CAMERA_YAW_STABILITY | "camera_yaw" | "yaw_stability" => Some(camera_yaw_stability()),
            CAMERA_TURN_STABILITY | "camera_turn" | "turn_stability" => {
                Some(camera_turn_stability())
            }
            CAMERA_STRAFE_STABILITY | "camera_strafe" | "strafe_stability" => {
                Some(camera_strafe_stability())
            }
            LONG_GLIDE_VISIBILITY | "long_glide" | "glide_visibility" => {
                Some(long_glide_visibility())
            }
            _ => None,
        }
    }

    pub fn scripted_input(scenario: EvalScenario, frame: u32) -> FlightInput {
        let t = frame as f32 * scenario.fixed_dt;
        if matches!(scenario.name, CAMERA_MOUSE_CONTROL | CAMERA_YAW_STABILITY) {
            return FlightInput::default();
        }
        if scenario.name == CAMERA_STRAFE_STABILITY {
            return FlightInput {
                right: (0.15..=1.55).contains(&t),
                left: (1.75..=3.1).contains(&t),
                ..default()
            };
        }
        if scenario.name == GROUND_TAXI_CONTROL {
            return FlightInput {
                forward: (0.05..=1.95).contains(&t),
                right: (0.75..=1.65).contains(&t),
                backward: (2.2..=2.35).contains(&t),
                ..default()
            };
        }
        if scenario.name == UPDRAFT_ROUTE {
            return FlightInput {
                forward: t >= 0.05,
                right: (1.2..=3.4).contains(&t),
                left: (4.4..=5.0).contains(&t),
                glide: t >= 0.45,
                launch: frame == 1,
                ..default()
            };
        }
        if scenario.name == BRANCH_RECOVERY_ROUTE {
            let dive = (8.7..=10.45).contains(&t);
            return FlightInput {
                forward: (0.05..=10.25).contains(&t),
                backward: (9.25..=11.65).contains(&t),
                right: (1.1..=2.75).contains(&t) || (7.2..=8.05).contains(&t),
                left: (4.2..=5.65).contains(&t),
                glide: t >= 0.45 && !dive,
                dive,
                launch: frame == 1,
            };
        }
        if scenario.name == LONG_GLIDE_VISIBILITY {
            return FlightInput {
                forward: t >= 0.05,
                right: (1.1..=2.75).contains(&t) || (7.2..=8.05).contains(&t),
                left: (4.2..=5.65).contains(&t),
                glide: t >= 0.45,
                launch: frame == 1,
                ..default()
            };
        }
        if scenario.name == CAMERA_TURN_STABILITY {
            return FlightInput {
                forward: (0.05..=1.6).contains(&t),
                backward: (3.9..=5.1).contains(&t),
                left: (1.05..=1.65).contains(&t) || (2.2..=2.75).contains(&t),
                right: (1.65..=2.2).contains(&t) || (2.75..=3.35).contains(&t),
                glide: t >= 0.45,
                launch: frame == 1,
                ..default()
            };
        }

        let dive = match scenario.name {
            ISLAND_LAUNCH_TO_LANDING => (5.8..=6.7).contains(&t),
            _ => (6.2..=7.0).contains(&t),
        };
        let (left, right) = match scenario.name {
            ISLAND_LAUNCH_TO_LANDING => ((3.1..=4.45).contains(&t), (5.1..=5.7).contains(&t)),
            _ => ((3.1..=4.2).contains(&t), (5.1..=6.0).contains(&t)),
        };

        FlightInput {
            forward: t >= 0.05,
            left,
            right,
            glide: t >= 0.45 && !dive,
            dive,
            launch: frame == 1,
            ..default()
        }
    }

    pub fn scripted_camera_input(scenario: EvalScenario, frame: u32) -> CameraInput {
        let t = frame as f32 * scenario.fixed_dt;

        let mouse_delta = match scenario.name {
            CAMERA_MOUSE_CONTROL if (0.2..=0.7).contains(&t) => Vec2::new(5.0, 0.0),
            CAMERA_MOUSE_CONTROL if (0.9..=1.3).contains(&t) => Vec2::new(0.0, -5.0),
            CAMERA_MOUSE_CONTROL if (1.5..=2.1).contains(&t) => Vec2::new(0.0, 8.0),
            CAMERA_MOUSE_CONTROL if (2.2..=2.55).contains(&t) => Vec2::new(0.0, -8.0),
            CAMERA_YAW_STABILITY if (0.2..=0.45).contains(&t) => Vec2::new(3.0, 0.0),
            _ => Vec2::ZERO,
        };

        CameraInput { mouse_delta }
    }

    fn baseline_route() -> EvalScenario {
        EvalScenario {
            name: BASELINE_ROUTE,
            fixed_dt: 1.0 / 60.0,
            frame_count: 420,
            sample_stride: 10,
            target_island_name: None,
            checkpoints: BASELINE_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 20,
                min_horizontal_distance_m: 80.0,
                min_max_altitude_m: 18.0,
                min_max_speed_mps: 20.0,
                min_gliding_samples: 20,
                min_grounded_samples: 0,
                min_lifted_samples: 0,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 35.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 12.0,
                max_camera_rotation_delta_degrees: 28.0,
                max_camera_orbit_alignment_degrees: 45.0,
                max_abs_camera_view_yaw_degrees: 8.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: false,
                max_final_target_distance_m: 40.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn island_launch_to_landing() -> EvalScenario {
        EvalScenario {
            name: ISLAND_LAUNCH_TO_LANDING,
            fixed_dt: 1.0 / 60.0,
            frame_count: 455,
            sample_stride: 5,
            target_island_name: None,
            checkpoints: ISLAND_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 50,
                min_horizontal_distance_m: 220.0,
                min_max_altitude_m: 52.0,
                min_max_speed_mps: 30.0,
                min_gliding_samples: 45,
                min_grounded_samples: 1,
                min_lifted_samples: 0,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 36.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 12.0,
                max_camera_rotation_delta_degrees: 28.0,
                max_camera_orbit_alignment_degrees: 45.0,
                max_abs_camera_view_yaw_degrees: 8.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: true,
                max_final_target_distance_m: 26.0,
                min_target_landing_samples: 1,
            },
        }
    }

    fn ground_taxi_control() -> EvalScenario {
        EvalScenario {
            name: GROUND_TAXI_CONTROL,
            fixed_dt: 1.0 / 60.0,
            frame_count: 180,
            sample_stride: 5,
            target_island_name: None,
            checkpoints: GROUND_TAXI_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 30,
                min_horizontal_distance_m: 14.0,
                min_max_altitude_m: 28.0,
                min_max_speed_mps: 8.0,
                min_gliding_samples: 0,
                min_grounded_samples: 28,
                min_lifted_samples: 0,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 28.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 10.0,
                max_camera_rotation_delta_degrees: 25.0,
                max_camera_orbit_alignment_degrees: 45.0,
                max_abs_camera_view_yaw_degrees: 8.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: false,
                max_final_target_distance_m: 280.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn updraft_route() -> EvalScenario {
        EvalScenario {
            name: UPDRAFT_ROUTE,
            fixed_dt: 1.0 / 60.0,
            frame_count: 360,
            sample_stride: 5,
            target_island_name: None,
            checkpoints: UPDRAFT_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 60,
                min_horizontal_distance_m: 150.0,
                min_max_altitude_m: 90.0,
                min_max_speed_mps: 35.0,
                min_gliding_samples: 45,
                min_grounded_samples: 1,
                min_lifted_samples: 4,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 36.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 12.0,
                max_camera_rotation_delta_degrees: 28.0,
                max_camera_orbit_alignment_degrees: 45.0,
                max_abs_camera_view_yaw_degrees: 8.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 1,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: false,
                max_final_target_distance_m: 180.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn branch_recovery_route() -> EvalScenario {
        EvalScenario {
            name: BRANCH_RECOVERY_ROUTE,
            fixed_dt: 1.0 / 60.0,
            frame_count: 760,
            sample_stride: 5,
            target_island_name: Some("sunlit terrace"),
            checkpoints: BRANCH_RECOVERY_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 80,
                min_horizontal_distance_m: 390.0,
                min_max_altitude_m: 100.0,
                min_max_speed_mps: 45.0,
                min_gliding_samples: 55,
                min_grounded_samples: 2,
                min_lifted_samples: 4,
                min_sky_island_count: 12,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 14,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 220,
                max_camera_distance_m: 38.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 14.0,
                max_camera_rotation_delta_degrees: 30.0,
                max_camera_orbit_alignment_degrees: 45.0,
                max_abs_camera_view_yaw_degrees: 8.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 3,
                min_completed_objective_count: 3,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: true,
                max_final_target_distance_m: 18.0,
                min_target_landing_samples: 2,
            },
        }
    }

    fn camera_mouse_control() -> EvalScenario {
        EvalScenario {
            name: CAMERA_MOUSE_CONTROL,
            fixed_dt: 1.0 / 60.0,
            frame_count: 200,
            sample_stride: 5,
            target_island_name: None,
            checkpoints: CAMERA_MOUSE_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 40,
                min_horizontal_distance_m: 0.0,
                min_max_altitude_m: 28.0,
                min_max_speed_mps: 0.0,
                min_gliding_samples: 0,
                min_grounded_samples: 30,
                min_lifted_samples: 0,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 36.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 14.0,
                max_camera_rotation_delta_degrees: 35.0,
                max_camera_orbit_alignment_degrees: 30.0,
                max_abs_camera_view_yaw_degrees: 45.0,
                min_camera_obstruction_adjustment_m: 1.0,
                min_abs_camera_yaw_degrees: 25.0,
                min_camera_pitch_offset_degrees: -10.0,
                max_camera_pitch_offset_degrees: 10.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: false,
                max_final_target_distance_m: 280.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn camera_yaw_stability() -> EvalScenario {
        EvalScenario {
            name: CAMERA_YAW_STABILITY,
            fixed_dt: 1.0 / 60.0,
            frame_count: 300,
            sample_stride: 5,
            target_island_name: None,
            checkpoints: CAMERA_YAW_STABILITY_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 50,
                min_horizontal_distance_m: 0.0,
                min_max_altitude_m: 28.0,
                min_max_speed_mps: 0.0,
                min_gliding_samples: 0,
                min_grounded_samples: 50,
                min_lifted_samples: 0,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 36.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 14.0,
                max_camera_rotation_delta_degrees: 25.0,
                max_camera_orbit_alignment_degrees: 15.0,
                max_abs_camera_view_yaw_degrees: 25.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 8.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: false,
                max_final_target_distance_m: 280.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn camera_turn_stability() -> EvalScenario {
        EvalScenario {
            name: CAMERA_TURN_STABILITY,
            fixed_dt: 1.0 / 60.0,
            frame_count: 360,
            sample_stride: 5,
            target_island_name: None,
            checkpoints: CAMERA_TURN_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 60,
                min_horizontal_distance_m: 55.0,
                min_max_altitude_m: 42.0,
                min_max_speed_mps: 28.0,
                min_gliding_samples: 40,
                min_grounded_samples: 0,
                min_lifted_samples: 0,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 36.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 10.0,
                max_camera_rotation_delta_degrees: 25.0,
                max_camera_orbit_alignment_degrees: 45.0,
                max_abs_camera_view_yaw_degrees: 8.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: false,
                max_final_target_distance_m: 280.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn camera_strafe_stability() -> EvalScenario {
        EvalScenario {
            name: CAMERA_STRAFE_STABILITY,
            fixed_dt: 1.0 / 60.0,
            frame_count: 260,
            sample_stride: 5,
            target_island_name: None,
            checkpoints: CAMERA_STRAFE_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 45,
                min_horizontal_distance_m: 1.0,
                min_max_altitude_m: 28.0,
                min_max_speed_mps: 8.0,
                min_gliding_samples: 0,
                min_grounded_samples: 45,
                min_lifted_samples: 0,
                min_sky_island_count: 10,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 100,
                max_camera_distance_m: 28.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 10.0,
                max_camera_rotation_delta_degrees: 8.0,
                max_camera_orbit_alignment_degrees: 15.0,
                max_abs_camera_view_yaw_degrees: 6.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 0,
                min_power_up_effect_samples: 0,
                require_target_landing: false,
                max_final_target_distance_m: 280.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn long_glide_visibility() -> EvalScenario {
        EvalScenario {
            name: LONG_GLIDE_VISIBILITY,
            fixed_dt: 1.0 / 60.0,
            frame_count: 660,
            sample_stride: 10,
            target_island_name: None,
            checkpoints: LONG_GLIDE_CHECKPOINTS,
            thresholds: EvalThresholds {
                min_samples: 60,
                min_horizontal_distance_m: 430.0,
                min_max_altitude_m: 80.0,
                min_max_speed_mps: 45.0,
                min_gliding_samples: 55,
                min_grounded_samples: 0,
                min_lifted_samples: 0,
                min_sky_island_count: 12,
                min_active_island_count: 4,
                max_active_chunk_count: 25,
                min_near_lod_island_count: 2,
                min_mid_lod_island_count: 3,
                min_far_lod_island_count: 3,
                max_visible_island_terrain_count: 55,
                min_hidden_island_terrain_count: 5,
                min_visible_island_impostor_count: 2,
                max_visible_island_detail_count: 95,
                min_hidden_island_detail_count: 20,
                min_visible_route_beacon_count: 12,
                min_weather_cloud_count: 12,
                min_environment_motion_visual_count: 6,
                min_environment_motion_offset_m: 0.03,
                max_resident_island_visual_count: 180,
                max_stream_visibility_changes_per_frame: 32,
                min_entity_count: 220,
                max_camera_distance_m: 38.0,
                min_camera_surface_clearance_m: 1.0,
                max_camera_player_angle_degrees: 18.0,
                max_camera_step_distance_m: 12.0,
                max_camera_rotation_delta_degrees: 28.0,
                max_camera_orbit_alignment_degrees: 45.0,
                max_abs_camera_view_yaw_degrees: 8.0,
                min_camera_obstruction_adjustment_m: 0.0,
                min_abs_camera_yaw_degrees: 0.0,
                min_camera_pitch_offset_degrees: 0.0,
                max_camera_pitch_offset_degrees: 0.0,
                min_objective_total_count: 2,
                min_completed_objective_count: 0,
                min_visual_asset_slot_count: VISUAL_ASSET_SLOT_COUNT,
                min_gltf_scene_asset_slot_count: GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                min_streaming_visual_asset_slot_count: STREAMING_VISUAL_ASSET_SLOT_COUNT,
                max_failed_visual_asset_scene_count: 0,
                min_power_up_count: 3,
                min_collected_power_up_count: 3,
                min_power_up_effect_samples: 3,
                require_target_landing: false,
                max_final_target_distance_m: 520.0,
                min_target_landing_samples: 0,
            },
        }
    }

    fn vec3_array(value: Vec3) -> [f32; 3] {
        [value.x, value.y, value.z]
    }

    fn horizontal_distance(start: [f32; 3], end: [f32; 3]) -> f32 {
        let dx = end[0] - start[0];
        let dz = end[2] - start[2];
        (dx * dx + dz * dz).sqrt()
    }

    fn percentile(sorted_values: &[f32], percentile: f32) -> f32 {
        if sorted_values.is_empty() {
            return 0.0;
        }

        let index = ((sorted_values.len() as f32 * percentile).ceil() as usize)
            .saturating_sub(1)
            .min(sorted_values.len() - 1);
        sorted_values[index]
    }

    fn json_array3(values: [f32; 3]) -> String {
        format!(
            "[{},{},{}]",
            json_number(values[0]),
            json_number(values[1]),
            json_number(values[2])
        )
    }

    fn json_string_array(values: &[String]) -> String {
        let values = values
            .iter()
            .map(|value| json_string(value))
            .collect::<Vec<_>>()
            .join(",");
        format!("[{values}]")
    }

    fn json_number(value: f32) -> String {
        if value.is_finite() {
            format!("{value:.4}")
        } else {
            "null".to_string()
        }
    }

    fn json_string(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len() + 2);
        escaped.push('"');
        for character in value.chars() {
            match character {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                character if character.is_control() => {
                    escaped.push_str(&format!("\\u{:04x}", character as u32));
                }
                character => escaped.push(character),
            }
        }
        escaped.push('"');
        escaped
    }

    #[cfg(test)]
    mod tests {
        use crate::asset_pipeline::{
            ALWAYS_VISUAL_ASSET_SLOT_COUNT, FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
            NEAR_LOD_VISUAL_ASSET_SLOT_COUNT, STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
            WEATHER_VISUAL_ASSET_SLOT_COUNT,
        };
        use crate::environment::AERIAL_POWER_UP_ROUTE;

        use super::*;

        #[test]
        fn baseline_route_has_scripted_launch_and_glide() {
            let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");

            assert!(scripted_input(scenario, 1).launch);
            assert!(!scripted_input(scenario, 2).launch);
            assert!(scripted_input(scenario, 60).glide);
        }

        #[test]
        fn ground_taxi_script_exercises_wasd_without_launching() {
            let scenario = scenario_named(GROUND_TAXI_CONTROL).expect("ground taxi route exists");

            assert!(scripted_input(scenario, 20).forward);
            assert!(scripted_input(scenario, 60).right);
            assert!(scripted_input(scenario, 135).backward);
            assert!(!scripted_input(scenario, 1).launch);
            assert!(!scripted_input(scenario, 60).glide);
        }

        #[test]
        fn updraft_route_steers_toward_lift_without_diving() {
            let scenario = scenario_named(UPDRAFT_ROUTE).expect("updraft route exists");

            assert!(scripted_input(scenario, 1).launch);
            assert!(scripted_input(scenario, 120).right);
            assert!(scripted_input(scenario, 180).glide);
            assert!(!scripted_input(scenario, 180).dive);
            assert_eq!(scenario.thresholds.min_completed_objective_count, 1);
        }

        #[test]
        fn branch_recovery_route_targets_named_recovery_island() {
            let scenario = scenario_named(BRANCH_RECOVERY_ROUTE).expect("branch route exists");

            assert_eq!(scenario.target_island_name, Some("sunlit terrace"));
            assert!(scenario.thresholds.require_target_landing);
            assert_eq!(scenario.thresholds.min_objective_total_count, 3);
            assert_eq!(scenario.thresholds.min_completed_objective_count, 3);
            assert!(scripted_input(scenario, 1).launch);
            assert!(scripted_input(scenario, 540).dive);
            assert!(scripted_input(scenario, 600).backward);
            assert!(!scripted_input(scenario, 720).forward);
        }

        #[test]
        fn camera_mouse_script_exercises_x_and_y_axes() {
            let scenario = scenario_named(CAMERA_MOUSE_CONTROL).expect("camera route exists");

            assert!(scripted_camera_input(scenario, 30).mouse_delta.x > 0.0);
            assert!(scripted_camera_input(scenario, 70).mouse_delta.y < 0.0);
            assert!(scripted_camera_input(scenario, 105).mouse_delta.y > 0.0);
            assert_eq!(
                scripted_input(scenario, 1),
                FlightInput::default(),
                "camera eval should not hide mouse regressions behind movement"
            );
        }

        #[test]
        fn camera_yaw_stability_script_applies_small_yaw_then_settles() {
            let scenario = scenario_named(CAMERA_YAW_STABILITY).expect("camera yaw route exists");

            assert!(scripted_camera_input(scenario, 18).mouse_delta.x > 0.0);
            assert_eq!(scripted_camera_input(scenario, 80), CameraInput::default());
            assert_eq!(
                scripted_input(scenario, 18),
                FlightInput::default(),
                "yaw stability eval should isolate mouse drift from movement"
            );
        }

        #[test]
        fn camera_turn_script_exercises_air_turns_and_air_brake() {
            let scenario = scenario_named(CAMERA_TURN_STABILITY).expect("turn route exists");

            assert!(scripted_input(scenario, 1).launch);
            assert!(scripted_input(scenario, 80).glide);
            assert!(scripted_input(scenario, 85).left);
            assert!(scripted_input(scenario, 115).right);
            assert!(scripted_input(scenario, 255).backward);
        }

        #[test]
        fn camera_strafe_script_exercises_lateral_input_without_mouse() {
            let scenario = scenario_named(CAMERA_STRAFE_STABILITY).expect("strafe route exists");

            assert!(scripted_input(scenario, 30).right);
            assert!(scripted_input(scenario, 130).left);
            assert_eq!(scripted_camera_input(scenario, 30), CameraInput::default());
            assert_eq!(scripted_camera_input(scenario, 130), CameraInput::default());
        }

        #[test]
        fn long_glide_visibility_script_crosses_archipelago() {
            let scenario = scenario_named(LONG_GLIDE_VISIBILITY).expect("long glide route exists");

            assert!(scripted_input(scenario, 1).launch);
            assert!(scripted_input(scenario, 120).right);
            assert!(scripted_input(scenario, 285).left);
            assert!(scripted_input(scenario, 620).glide);
            assert!(!scripted_input(scenario, 620).dive);
            assert!(scenario.thresholds.min_sky_island_count >= 12);
            assert_eq!(scenario.thresholds.min_power_up_count, 3);
            assert_eq!(scenario.thresholds.min_collected_power_up_count, 3);
            assert!(scenario.thresholds.min_power_up_effect_samples >= 3);
        }

        #[test]
        fn scenarios_define_non_final_camera_checkpoints() {
            for name in SCENARIO_NAMES {
                let scenario = scenario_named(name).expect("scenario exists");

                assert!(!scenario.checkpoints.is_empty());
                assert!(
                    scenario
                        .checkpoints
                        .iter()
                        .all(|checkpoint| checkpoint.frame < scenario.frame_count)
                );
                assert_eq!(
                    scenario.checkpoint_at(scenario.checkpoints[0].frame),
                    Some(scenario.checkpoints[0])
                );
            }
        }

        #[test]
        fn accumulator_summarizes_frame_time_percentiles() {
            let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
            let mut accumulator = EvalAccumulator::default();
            for frame_time_ms in [8.0, 16.0, 33.0, 50.0] {
                accumulator.observe_frame_time_ms(frame_time_ms);
            }

            let summary = accumulator.summary(
                scenario,
                EvalArtifacts {
                    summary_json: "summary.json".to_string(),
                    samples_ndjson: "samples.ndjson".to_string(),
                    screenshot_png: None,
                    checkpoint_screenshots: Vec::new(),
                },
            );

            assert_eq!(summary.metrics.avg_frame_time_ms, 26.75);
            assert_eq!(summary.metrics.p95_frame_time_ms, 50.0);
            assert_eq!(summary.metrics.p99_frame_time_ms, 50.0);
            assert_eq!(summary.metrics.max_frame_time_ms, 50.0);
        }

        #[test]
        fn accumulator_marks_current_baseline_shape_as_passing() {
            let scenario = scenario_named(BASELINE_ROUTE).expect("baseline route exists");
            let mut accumulator = EvalAccumulator::default();
            let objective = EvalObjectiveProgress::new(0, 2, "near route updraft", 140.0, false);

            accumulator.observe(EvalSample::new(
                0,
                scenario.fixed_dt,
                Vec3::new(0.0, 1.2, 0.0),
                Vec3::ZERO,
                FlightMode::Grounded,
                12.0,
                3.0,
                4.0,
                -20.0,
                0.0,
                0.0,
                0.2,
                2.0,
                0.0,
                0.0,
                0.0,
                0,
                0,
                3,
                0,
                0,
                1,
                140.0,
                false,
                objective,
                12,
                25,
                6,
                2,
                4,
                6,
                24,
                36,
                8,
                4,
                26,
                118,
                16,
                12,
                8,
                0.08,
                160,
                0,
                12,
                12,
                335,
                175,
                0.48,
                0,
                0,
                12,
                12,
                20,
                20,
                130,
                VISUAL_ASSET_SLOT_COUNT,
                GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                0,
                VISUAL_ASSET_SLOT_COUNT,
                STREAMING_VISUAL_ASSET_SLOT_COUNT,
                VISUAL_ASSET_SLOT_COUNT,
                0,
                0,
                0,
                0,
                ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                WEATHER_VISUAL_ASSET_SLOT_COUNT,
                AERIAL_POWER_UP_ROUTE.len(),
                AERIAL_POWER_UP_ROUTE.len(),
                0,
                0,
                0,
            ));
            accumulator.observe(EvalSample::new(
                scenario.frame_count,
                scenario.fixed_dt,
                Vec3::new(0.0, 32.0, 140.0),
                Vec3::new(0.0, -4.0, 30.0),
                FlightMode::Gliding,
                14.0,
                3.0,
                4.0,
                -18.0,
                0.0,
                0.0,
                0.2,
                2.0,
                0.0,
                0.0,
                0.0,
                0,
                0,
                3,
                0,
                0,
                1,
                0.0,
                false,
                objective,
                12,
                25,
                6,
                2,
                4,
                6,
                24,
                36,
                8,
                4,
                26,
                118,
                16,
                12,
                8,
                0.08,
                160,
                0,
                12,
                12,
                335,
                175,
                0.48,
                0,
                0,
                12,
                12,
                20,
                20,
                130,
                VISUAL_ASSET_SLOT_COUNT,
                GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                0,
                VISUAL_ASSET_SLOT_COUNT,
                STREAMING_VISUAL_ASSET_SLOT_COUNT,
                VISUAL_ASSET_SLOT_COUNT,
                0,
                0,
                0,
                0,
                ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                WEATHER_VISUAL_ASSET_SLOT_COUNT,
                AERIAL_POWER_UP_ROUTE.len(),
                AERIAL_POWER_UP_ROUTE.len(),
                0,
                0,
                0,
            ));
            for frame in 1..=scenario.thresholds.min_gliding_samples {
                accumulator.observe(EvalSample::new(
                    frame,
                    scenario.fixed_dt,
                    Vec3::new(0.0, 24.0, frame as f32 * 4.0),
                    Vec3::new(0.0, -3.0, 25.0),
                    FlightMode::Gliding,
                    13.0,
                    3.0,
                    4.0,
                    -18.0,
                    0.0,
                    0.0,
                    0.2,
                    2.0,
                    0.0,
                    0.0,
                    0.0,
                    0,
                    0,
                    3,
                    0,
                    0,
                    1,
                    140.0 - frame as f32 * 4.0,
                    false,
                    objective,
                    12,
                    25,
                    6,
                    2,
                    4,
                    6,
                    24,
                    36,
                    8,
                    4,
                    26,
                    118,
                    16,
                    12,
                    8,
                    0.08,
                    160,
                    0,
                    12,
                    12,
                    335,
                    175,
                    0.48,
                    0,
                    0,
                    12,
                    12,
                    20,
                    20,
                    130,
                    VISUAL_ASSET_SLOT_COUNT,
                    GLTF_SCENE_VISUAL_ASSET_SLOT_COUNT,
                    0,
                    VISUAL_ASSET_SLOT_COUNT,
                    STREAMING_VISUAL_ASSET_SLOT_COUNT,
                    VISUAL_ASSET_SLOT_COUNT,
                    0,
                    0,
                    0,
                    0,
                    ALWAYS_VISUAL_ASSET_SLOT_COUNT,
                    STREAM_WINDOW_VISUAL_ASSET_SLOT_COUNT,
                    NEAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                    FAR_LOD_VISUAL_ASSET_SLOT_COUNT,
                    WEATHER_VISUAL_ASSET_SLOT_COUNT,
                    AERIAL_POWER_UP_ROUTE.len(),
                    AERIAL_POWER_UP_ROUTE.len(),
                    0,
                    0,
                    0,
                ));
            }

            let summary = accumulator.summary(
                scenario,
                EvalArtifacts {
                    summary_json: "summary.json".to_string(),
                    samples_ndjson: "samples.ndjson".to_string(),
                    screenshot_png: None,
                    checkpoint_screenshots: vec!["checkpoints/glide_midroute.png".to_string()],
                },
            );

            assert!(summary.passed);
            assert_eq!(summary.metrics.objective_total_count, 2);
            assert_eq!(summary.metrics.max_completed_objective_count, 0);
            assert!(summary.to_json().contains("\"passed\": true"));
            assert!(summary.to_json().contains("\"objective\":"));
            assert!(
                summary
                    .to_json()
                    .contains("\"checkpoint_screenshots\": [\"checkpoints/glide_midroute.png\"]")
            );
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

    pub fn part_pose(
        part: &CharacterPart,
        mode: FlightMode,
        velocity: Vec3,
        phase: f32,
    ) -> PartPose {
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
            let fast_glide =
                wing_airflow_strength(FlightMode::Gliding, Vec3::new(0.0, -18.0, -55.0));
            let slow_glide = wing_airflow_strength(FlightMode::Gliding, Vec3::new(0.0, 0.0, -8.0));
            let fast_ground =
                wing_airflow_strength(FlightMode::Grounded, Vec3::new(0.0, -18.0, -55.0));

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
}
