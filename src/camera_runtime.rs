use crate::eval_runtime::EvalRun;
use crate::{Player, keyboard_flight_input};
use bevy::camera::{CameraOutputMode, ClearColorConfig, Exposure};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseMotion;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use nau_engine::camera::{
    CameraControlState, CameraControlTuning, CameraInput, CameraObstruction, FollowCamera,
    FollowCameraState, apply_camera_input, avoid_camera_obstructions,
    camera_orbit_alignment_degrees, clamp_camera_step, lift_camera_above_floor,
    movement_input_stable_follow_direction, step_camera_with_direction,
    update_follow_direction_state,
};
use nau_engine::eval::{scripted_camera_input, scripted_input};
use nau_engine::movement::Velocity;
use nau_engine::world::SkyRoute;

const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_MAX_STEP_M: f32 = 9.5;
pub(crate) const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct CameraDiagnostics {
    pub(crate) step_distance_m: f32,
    pub(crate) rotation_delta_degrees: f32,
    pub(crate) orbit_alignment_degrees: f32,
    pub(crate) follow_direction: Vec3,
    pub(crate) follow_direction_error_degrees: f32,
    pub(crate) obstruction_adjustment_m: f32,
    pub(crate) obstruction_hits: usize,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct MouseLookState {
    pub(crate) captured: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CameraObstacle(pub(crate) CameraObstruction);

pub(crate) type CameraFollowFilter = (With<Camera3d>, Without<Player>);

#[derive(SystemParam)]
pub(crate) struct CameraScene<'w, 's> {
    route: Res<'w, SkyRoute>,
    camera_control: Res<'w, CameraControlState>,
    camera_diagnostics: ResMut<'w, CameraDiagnostics>,
    player: Query<'w, 's, (&'static Transform, &'static Velocity), With<Player>>,
    camera: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static FollowCamera,
            &'static mut FollowCameraState,
        ),
        CameraFollowFilter,
    >,
    obstacles: Query<'w, 's, &'static CameraObstacle>,
}

pub(crate) fn spawn_follow_camera(
    commands: &mut Commands,
    scattering_mediums: &mut ResMut<Assets<ScatteringMedium>>,
    player_start: Vec3,
    world_radius: f32,
    clear_color: Color,
) {
    let follow_camera = FollowCamera::default();
    let initial_camera_direction = Vec3::NEG_Z;
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(clear_color),
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::REPLACE),
                clear_color: ClearColorConfig::Custom(clear_color),
            },
            ..default()
        },
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings {
            scene_units_to_m: 18.0,
            aerial_view_lut_max_distance: 26_000.0,
            ..default()
        },
        Exposure { ev100: 12.6 },
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight::default(),
        VolumetricFog {
            ambient_color: Color::srgb(0.66, 0.72, 0.84),
            ambient_intensity: 0.035,
            jitter: 0.35,
            step_count: 48,
        },
        DistanceFog {
            color: Color::srgba(0.56, 0.70, 0.88, 0.48),
            directional_light_color: Color::srgba(1.0, 0.84, 0.55, 0.45),
            directional_light_exponent: 18.0,
            falloff: FogFalloff::Linear {
                start: 260.0,
                end: world_radius,
            },
        },
        Transform::from_translation(
            player_start - initial_camera_direction * follow_camera.distance
                + Vec3::Y * follow_camera.height,
        )
        .looking_at(
            player_start
                + Vec3::Y * follow_camera.look_height
                + initial_camera_direction * follow_camera.look_ahead,
            Vec3::Y,
        ),
        follow_camera,
        FollowCameraState::default(),
    ));
}

pub(crate) fn update_mouse_look_capture(
    eval: Option<Res<EvalRun>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_look: ResMut<MouseLookState>,
    mut window: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    if eval.is_some() {
        return;
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        mouse_look.captured = true;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        mouse_look.captured = false;
    }

    let Ok((window, mut cursor)) = window.single_mut() else {
        return;
    };
    if !window.focused {
        mouse_look.captured = false;
    }

    let grab_mode = if mouse_look.captured {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != grab_mode {
        cursor.grab_mode = grab_mode;
    }

    let visible = !mouse_look.captured;
    if cursor.visible != visible {
        cursor.visible = visible;
    }
}

pub(crate) fn update_camera_control(
    time: Res<Time>,
    eval: Option<Res<EvalRun>>,
    tuning: Res<CameraControlTuning>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_look: Res<MouseLookState>,
    mut state: ResMut<CameraControlState>,
    mut mouse_motion: MessageReader<MouseMotion>,
) {
    let input = if let Some(run) = eval.as_deref() {
        scripted_camera_input(run.scenario, run.frame)
    } else {
        let mouse_delta = mouse_motion
            .read()
            .fold(Vec2::ZERO, |delta, motion| delta + motion.delta);

        CameraInput {
            mouse_delta: if mouse_look.captured || mouse_buttons.pressed(MouseButton::Right) {
                mouse_delta
            } else {
                Vec2::ZERO
            },
        }
    };

    if input.mouse_delta.length_squared() <= 0.0 || time.delta_secs() <= 0.0 {
        return;
    }

    state.orbit = apply_camera_input(state.orbit, input, &tuning);
}

pub(crate) fn follow_camera(
    time: Res<Time>,
    eval: Option<Res<EvalRun>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scene: CameraScene,
) {
    let Ok((player_transform, player_velocity)) = scene.player.single() else {
        return;
    };
    let Ok((mut camera_transform, follow, mut follow_state)) = scene.camera.single_mut() else {
        return;
    };
    let previous_camera_position = camera_transform.translation;
    let previous_camera_rotation = camera_transform.rotation;

    let dt = eval_dt(&time, eval.as_deref());
    let movement_input = eval.as_deref().map_or_else(
        || keyboard_flight_input(&keyboard),
        |run| scripted_input(run.scenario, run.frame),
    );
    let desired_follow_direction = movement_input_stable_follow_direction(
        player_velocity.0,
        *player_transform.forward(),
        follow_state.direction,
        movement_input.planar_axis(),
    );
    let follow_direction =
        update_follow_direction_state(&mut follow_state, desired_follow_direction, follow, dt);
    let frame = step_camera_with_direction(
        camera_transform.translation,
        camera_transform.rotation,
        player_transform.translation,
        follow_direction,
        follow,
        scene.camera_control.orbit,
        dt,
    );
    let orbit_alignment_degrees = camera_orbit_alignment_degrees(
        frame.position,
        frame.look_target,
        follow_direction,
        scene.camera_control.orbit,
    );
    let camera_floor_y = scene.route.ground_at(frame.position).floor_y;
    let frame = lift_camera_above_floor(frame, camera_floor_y, CAMERA_MIN_SURFACE_CLEARANCE);
    let obstruction_resolution = avoid_camera_obstructions(
        frame,
        scene.obstacles.iter().map(|obstacle| obstacle.0),
        CAMERA_OBSTRUCTION_CLEARANCE,
    );
    let camera_floor_y = scene
        .route
        .ground_at(obstruction_resolution.frame.position)
        .floor_y;
    let frame = lift_camera_above_floor(
        obstruction_resolution.frame,
        camera_floor_y,
        CAMERA_MIN_SURFACE_CLEARANCE,
    );
    let frame = clamp_camera_step(frame, previous_camera_position, CAMERA_MAX_STEP_M);

    scene.camera_diagnostics.step_distance_m = previous_camera_position.distance(frame.position);
    scene.camera_diagnostics.rotation_delta_degrees = previous_camera_rotation
        .angle_between(frame.rotation)
        .to_degrees();
    scene.camera_diagnostics.orbit_alignment_degrees = orbit_alignment_degrees;
    scene.camera_diagnostics.follow_direction = follow_direction;
    scene.camera_diagnostics.follow_direction_error_degrees = follow_direction
        .angle_between(desired_follow_direction)
        .to_degrees();
    scene.camera_diagnostics.obstruction_adjustment_m = obstruction_resolution.adjusted_distance_m;
    scene.camera_diagnostics.obstruction_hits = obstruction_resolution.hit_count;

    camera_transform.translation = frame.position;
    camera_transform.rotation = frame.rotation;
}

fn eval_dt(time: &Time, eval: Option<&EvalRun>) -> f32 {
    eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt)
}
