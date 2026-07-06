use crate::eval_runtime::EvalRun;
use crate::world_collision_runtime::WorldCollisionDiagnostics;
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
    CAMERA_MAX_FOLLOW_FRAME_STEP_M, CAMERA_MAX_OBSTRUCTION_FRAME_STEP_M,
    CAMERA_MAX_OBSTRUCTION_HANDOFF_FRAME_STEP_M, CAMERA_MAX_OBSTRUCTION_ROTATION_STEP_DEGREES,
    CAMERA_MAX_PLAYER_DISTANCE_M, CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M,
    CAMERA_OBSTRUCTION_RELEASE_HANDOFF_FRAMES, CameraControlState, CameraControlTuning,
    CameraInput, CameraObstruction, CameraObstructionSmoothingState, FollowCamera,
    FollowCameraState, apply_camera_input, avoid_camera_obstructions_with_preferred_offset,
    camera_frame_step_budget, camera_obstruction_is_active, camera_orbit_alignment_degrees,
    camera_rotation_step_budget, clamp_camera_offset_step, clamp_camera_player_distance,
    clamp_camera_rotation_step, clamp_camera_step, lift_camera_above_floor,
    movement_input_stable_follow_direction, revalidate_camera_obstruction,
    smooth_camera_obstruction, step_camera_with_direction, update_follow_direction_state,
};
use nau_engine::eval::{scripted_camera_input, scripted_input};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::SkyRoute;

const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 1.45;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
pub(crate) const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct CameraDiagnostics {
    pub(crate) target_valid: bool,
    pub(crate) camera_valid: bool,
    pub(crate) player_control_valid: bool,
    pub(crate) invalid_transform_count: u32,
    pub(crate) position: Vec3,
    pub(crate) look_target: Vec3,
    pub(crate) desired_boom_length_m: f32,
    pub(crate) resolved_boom_length_m: f32,
    pub(crate) step_distance_m: f32,
    pub(crate) rotation_delta_degrees: f32,
    pub(crate) orbit_alignment_degrees: f32,
    pub(crate) follow_direction: Vec3,
    pub(crate) follow_direction_error_degrees: f32,
    pub(crate) obstruction_adjustment_m: f32,
    pub(crate) obstruction_hits: usize,
    pub(crate) obstruction_active_hits: usize,
    pub(crate) obstruction_memory_active: bool,
    pub(crate) obstruction_memory_age_frames: u32,
    pub(crate) obstruction_stale_memory_age_frames: u32,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct MouseLookState {
    pub(crate) captured: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CameraObstacle(pub(crate) CameraObstruction);

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct CameraObstructionMemory {
    state: CameraObstructionSmoothingState,
    release_handoff_frames_remaining: u8,
    previous_look_target: Option<Vec3>,
    memory_age_frames: u32,
    stale_memory_age_frames: u32,
    diagnostics_initialized: bool,
}

pub(crate) type CameraFollowFilter = (With<Camera3d>, Without<Player>);

#[derive(SystemParam)]
pub(crate) struct CameraScene<'w, 's> {
    route: Res<'w, SkyRoute>,
    camera_control: Res<'w, CameraControlState>,
    camera_diagnostics: ResMut<'w, CameraDiagnostics>,
    collision_diagnostics: Res<'w, WorldCollisionDiagnostics>,
    player: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Velocity,
            &'static FlightController,
        ),
        With<Player>,
    >,
    camera: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static FollowCamera,
            &'static mut FollowCameraState,
            &'static mut CameraObstructionMemory,
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
        follow_camera_transform(player_start, &follow_camera),
        follow_camera,
        FollowCameraState::default(),
        CameraObstructionMemory::default(),
    ));
}

pub(crate) fn follow_camera_transform(player_position: Vec3, follow: &FollowCamera) -> Transform {
    let initial_camera_direction = Vec3::NEG_Z;
    Transform::from_translation(
        player_position - initial_camera_direction * follow.distance + Vec3::Y * follow.height,
    )
    .looking_at(
        player_position
            + Vec3::Y * follow.look_height
            + initial_camera_direction * follow.look_ahead,
        Vec3::Y,
    )
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
    scene.camera_diagnostics.target_valid = false;
    scene.camera_diagnostics.camera_valid = false;
    scene.camera_diagnostics.player_control_valid = false;
    scene.camera_diagnostics.invalid_transform_count = 0;

    let Ok((player_transform, player_velocity, player_controller)) = scene.player.single() else {
        return;
    };
    let player_transform_valid = transform_is_finite(player_transform);
    let player_velocity_valid = player_velocity.0.is_finite();
    scene.camera_diagnostics.target_valid = player_transform_valid && player_velocity_valid;
    scene.camera_diagnostics.player_control_valid = flight_controller_is_finite(player_controller);
    if !player_transform_valid {
        scene.camera_diagnostics.invalid_transform_count += 1;
    }

    let Ok((mut camera_transform, follow, mut follow_state, mut obstruction_memory)) =
        scene.camera.single_mut()
    else {
        return;
    };
    let camera_transform_valid = transform_is_finite(&camera_transform);
    scene.camera_diagnostics.camera_valid = camera_transform_valid;
    if !camera_transform_valid {
        scene.camera_diagnostics.invalid_transform_count += 1;
        return;
    }
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
    let desired_boom_length_m = frame.position.distance(frame.look_target);
    let orbit_alignment_degrees = camera_orbit_alignment_degrees(
        frame.position,
        frame.look_target,
        follow_direction,
        scene.camera_control.orbit,
    );
    let camera_floor_y = scene.route.contact_ground_at(frame.position).floor_y;
    let frame = lift_camera_above_floor(frame, camera_floor_y, CAMERA_MIN_SURFACE_CLEARANCE);
    let obstructions = scene
        .obstacles
        .iter()
        .map(|obstacle| obstacle.0)
        .collect::<Vec<_>>();
    let preferred_obstruction_offset = obstruction_memory.state.readable_offset();
    let obstruction = avoid_camera_obstructions_with_preferred_offset(
        frame,
        obstructions.iter().copied(),
        CAMERA_OBSTRUCTION_CLEARANCE,
        preferred_obstruction_offset,
    );
    let active_obstruction =
        camera_obstruction_is_active(obstruction.hit_count, obstruction.adjusted_distance_m);
    let active_obstruction_hits = if active_obstruction {
        obstruction.hit_count
    } else {
        0
    };
    let active_obstruction_adjustment_m = if active_obstruction {
        obstruction.adjusted_distance_m
    } else {
        0.0
    };
    let obstruction_frame = if active_obstruction {
        obstruction.frame
    } else {
        frame
    };
    let camera_floor_y = scene
        .route
        .contact_ground_at(obstruction_frame.position)
        .floor_y;
    let frame = lift_camera_above_floor(
        obstruction_frame,
        camera_floor_y,
        CAMERA_MIN_SURFACE_CLEARANCE,
    );
    let pre_smoothing_frame = frame;
    let frame = smooth_camera_obstruction(
        frame,
        &mut obstruction_memory.state,
        active_obstruction_hits,
        active_obstruction_adjustment_m,
        dt,
    );
    let revalidated_obstruction = revalidate_camera_obstruction(
        frame,
        obstructions.iter().copied(),
        CAMERA_OBSTRUCTION_CLEARANCE,
        preferred_obstruction_offset,
    );
    let revalidated_active = camera_obstruction_is_active(
        revalidated_obstruction.hit_count,
        revalidated_obstruction.adjusted_distance_m,
    );
    let (frame, active_obstruction_hits, active_obstruction_adjustment_m) = if revalidated_active {
        let camera_floor_y = scene
            .route
            .contact_ground_at(revalidated_obstruction.frame.position)
            .floor_y;
        (
            lift_camera_above_floor(
                revalidated_obstruction.frame,
                camera_floor_y,
                CAMERA_MIN_SURFACE_CLEARANCE,
            ),
            revalidated_obstruction.hit_count,
            active_obstruction_adjustment_m.max(revalidated_obstruction.adjusted_distance_m),
        )
    } else {
        (
            frame,
            active_obstruction_hits,
            active_obstruction_adjustment_m,
        )
    };
    let release_smoothing_active = active_obstruction_hits == 0
        && (preferred_obstruction_offset.is_some()
            || pre_smoothing_frame.position.distance(frame.position) > 0.001);
    let release_handoff_active =
        active_obstruction_hits == 0 && obstruction_memory.release_handoff_frames_remaining > 0;
    let reported_obstruction_hits = if release_smoothing_active || release_handoff_active {
        1
    } else {
        active_obstruction_hits
    };
    let reported_obstruction_adjustment_m = if release_smoothing_active || release_handoff_active {
        CAMERA_OBSTRUCTION_MIN_ACTIVE_ADJUSTMENT_M
    } else {
        active_obstruction_adjustment_m
    };
    let frame = clamp_camera_player_distance(
        frame,
        player_transform.translation,
        CAMERA_MAX_PLAYER_DISTANCE_M,
    );
    let pre_cap_rotation_delta_degrees = previous_camera_rotation
        .angle_between(frame.rotation)
        .to_degrees();
    let obstruction_position_controlled = active_obstruction_hits > 0 || release_smoothing_active;
    let base_max_camera_step_m = if obstruction_position_controlled {
        CAMERA_MAX_OBSTRUCTION_FRAME_STEP_M
    } else if release_handoff_active {
        CAMERA_MAX_OBSTRUCTION_HANDOFF_FRAME_STEP_M
    } else {
        CAMERA_MAX_FOLLOW_FRAME_STEP_M
    };
    let max_camera_step_m = camera_frame_step_budget(base_max_camera_step_m, dt);
    let frame = if reported_obstruction_hits > 0 {
        clamp_camera_offset_step(
            frame,
            previous_camera_position,
            obstruction_memory.previous_look_target,
            max_camera_step_m,
        )
    } else {
        let smoothed_rotation = frame.rotation;
        let mut clamped = clamp_camera_step(frame, previous_camera_position, max_camera_step_m);
        clamped.rotation = smoothed_rotation;
        clamped
    };
    let frame = if reported_obstruction_hits > 0 {
        clamp_camera_rotation_step(
            frame,
            previous_camera_rotation,
            camera_rotation_step_budget(CAMERA_MAX_OBSTRUCTION_ROTATION_STEP_DEGREES, dt),
        )
    } else {
        frame
    };
    let terrain_collision_target_jump = scene.collision_diagnostics.terrain_rim_resolved_count
        + scene.collision_diagnostics.terrain_body_resolved_count
        > 0
        && scene
            .collision_diagnostics
            .max_terrain_rim_push_m
            .max(scene.collision_diagnostics.max_terrain_body_push_m)
            > 0.0;
    let frame = if terrain_collision_target_jump {
        clamp_camera_step(frame, previous_camera_position, max_camera_step_m)
    } else {
        frame
    };
    let frame = clamp_camera_player_distance(
        frame,
        player_transform.translation,
        CAMERA_MAX_PLAYER_DISTANCE_M,
    );
    obstruction_memory.state.sync_resolved_frame(
        frame,
        active_obstruction_hits,
        active_obstruction_adjustment_m,
    );
    let release_handoff_still_settling = release_handoff_active
        && pre_cap_rotation_delta_degrees > CAMERA_MAX_OBSTRUCTION_ROTATION_STEP_DEGREES;
    if active_obstruction_hits > 0 || release_smoothing_active || release_handoff_still_settling {
        obstruction_memory.release_handoff_frames_remaining =
            CAMERA_OBSTRUCTION_RELEASE_HANDOFF_FRAMES;
    } else {
        obstruction_memory.release_handoff_frames_remaining = obstruction_memory
            .release_handoff_frames_remaining
            .saturating_sub(1);
    }
    obstruction_memory.previous_look_target = Some(frame.look_target);
    scene.camera_diagnostics.obstruction_memory_active =
        obstruction_memory.state.readable_offset().is_some()
            || obstruction_memory.release_handoff_frames_remaining > 0;
    if scene.camera_diagnostics.obstruction_memory_active {
        obstruction_memory.memory_age_frames =
            obstruction_memory.memory_age_frames.saturating_add(1);
        if active_obstruction_hits == 0 {
            obstruction_memory.stale_memory_age_frames =
                obstruction_memory.stale_memory_age_frames.saturating_add(1);
        } else {
            obstruction_memory.stale_memory_age_frames = 0;
        }
    } else {
        obstruction_memory.memory_age_frames = 0;
        obstruction_memory.stale_memory_age_frames = 0;
    }
    scene.camera_diagnostics.obstruction_memory_age_frames = obstruction_memory.memory_age_frames;
    scene.camera_diagnostics.obstruction_stale_memory_age_frames =
        obstruction_memory.stale_memory_age_frames;

    let (diagnostics_previous_position, diagnostics_previous_rotation) =
        if obstruction_memory.diagnostics_initialized {
            (previous_camera_position, previous_camera_rotation)
        } else {
            (frame.position, frame.rotation)
        };
    obstruction_memory.diagnostics_initialized = true;

    scene.camera_diagnostics.position = frame.position;
    scene.camera_diagnostics.look_target = frame.look_target;
    scene.camera_diagnostics.desired_boom_length_m = desired_boom_length_m;
    scene.camera_diagnostics.resolved_boom_length_m = frame.position.distance(frame.look_target);
    scene.camera_diagnostics.step_distance_m =
        diagnostics_previous_position.distance(frame.position);
    scene.camera_diagnostics.rotation_delta_degrees = diagnostics_previous_rotation
        .angle_between(frame.rotation)
        .to_degrees();
    scene.camera_diagnostics.orbit_alignment_degrees = orbit_alignment_degrees;
    scene.camera_diagnostics.follow_direction = follow_direction;
    scene.camera_diagnostics.follow_direction_error_degrees = follow_direction
        .angle_between(desired_follow_direction)
        .to_degrees();
    scene.camera_diagnostics.obstruction_adjustment_m = reported_obstruction_adjustment_m;
    scene.camera_diagnostics.obstruction_hits = reported_obstruction_hits;
    scene.camera_diagnostics.obstruction_active_hits = active_obstruction_hits;

    camera_transform.translation = frame.position;
    camera_transform.rotation = frame.rotation;
}

fn eval_dt(time: &Time, eval: Option<&EvalRun>) -> f32 {
    eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt)
}

fn transform_is_finite(transform: &Transform) -> bool {
    transform.translation.is_finite()
        && transform.rotation.is_finite()
        && transform.scale.is_finite()
}

fn flight_controller_is_finite(controller: &FlightController) -> bool {
    controller.launch_cooldown_remaining.is_finite()
        && controller.launch_timer.is_finite()
        && controller.bank_degrees.is_finite()
        && controller.landing_recovery_timer.is_finite()
        && controller.landing_impact_speed_mps.is_finite()
}
