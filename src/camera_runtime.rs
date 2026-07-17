use crate::eval_runtime::{EvalRun, ISLAND_HERO_GALLERY};
use crate::play_profile_runtime::PlayProfileRun;
use crate::{Player, keyboard_flight_input};
use bevy::camera::{CameraOutputMode, ClearColorConfig, Exposure};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use nau_engine::camera::{
    CameraControlState, CameraControlTuning, CameraInput, CameraObstruction,
    CameraObstructionHandoffState, FollowCamera, FollowCameraState, camera_orbit_alignment_degrees,
    lift_camera_above_floor, movement_input_stable_follow_direction,
    resolve_camera_obstruction_handoff, step_camera_control, step_camera_with_direction_and_input,
    update_follow_direction_state,
};
use nau_engine::eval::{
    GREAT_SKY_PLATEAU_VISTAS, ISLAND_SURFACE_REVIEW, scripted_camera_input, scripted_input,
};
use nau_engine::movement::Velocity;
use nau_engine::world::{IslandPlateauRegion, SkyRoute, world_terrain_floor_y_at};

const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_CAPTURE_STALE_DELTA_THRESHOLD_PX: f32 = 64.0;
const PLATEAU_VISTA_TRANSITION_START_FRAME: u32 = 90;
const PLATEAU_VISTA_TRANSITION_END_FRAME: u32 = 150;
const SURFACE_REVIEW_FIRST_TRANSITION_START_FRAME: u32 = 75;
const SURFACE_REVIEW_FIRST_TRANSITION_END_FRAME: u32 = 165;
const SURFACE_REVIEW_SECOND_TRANSITION_START_FRAME: u32 = 195;
const SURFACE_REVIEW_SECOND_TRANSITION_END_FRAME: u32 = 285;
pub(crate) const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;

#[derive(Clone, Copy, Debug, PartialEq)]
struct DirectCameraPose {
    position: Vec3,
    target: Vec3,
}

impl DirectCameraPose {
    fn interpolate(self, next: Self, amount: f32) -> Self {
        if amount <= 0.0 {
            self
        } else if amount >= 1.0 {
            next
        } else {
            let position = self.position.lerp(next.position, amount);
            let rotation = self.rotation().slerp(next.rotation(), amount);
            let look_distance = self
                .position
                .distance(self.target)
                .lerp(next.position.distance(next.target), amount);
            Self {
                position,
                target: position + rotation * Vec3::NEG_Z * look_distance,
            }
        }
    }

    fn rotation(self) -> Quat {
        Transform::from_translation(self.position)
            .looking_at(self.target, Vec3::Y)
            .rotation
    }
}

const ISLAND_SURFACE_REVIEW_CAMERA_POSES: [DirectCameraPose; 3] = [
    DirectCameraPose {
        position: Vec3::new(-306.2, 729.7, -2694.5),
        target: Vec3::new(-221.2, 695.7, -2629.5),
    },
    DirectCameraPose {
        position: Vec3::new(-150.0, 720.0, -2482.0),
        target: Vec3::new(-225.0, 692.0, -2566.0),
    },
    DirectCameraPose {
        position: Vec3::new(205.0, 725.0, -2482.0),
        target: Vec3::new(-20.0, 675.0, -2605.0),
    },
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum CameraCorrectionSource {
    #[default]
    None,
    Input,
    Follow,
    Floor,
    Obstruction,
    Distance,
    Scripted,
}

impl CameraCorrectionSource {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Input => "input",
            Self::Follow => "follow",
            Self::Floor => "floor",
            Self::Obstruction => "obstruction",
            Self::Distance => "distance",
            Self::Scripted => "scripted",
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct CameraDiagnostics {
    pub(crate) step_distance_m: f32,
    pub(crate) rotation_delta_degrees: f32,
    pub(crate) orbit_alignment_degrees: f32,
    pub(crate) follow_direction: Vec3,
    pub(crate) follow_direction_error_degrees: f32,
    pub(crate) obstruction_adjustment_m: f32,
    pub(crate) obstruction_hits: usize,
    pub(crate) correction_source: CameraCorrectionSource,
    pub(crate) continuity_offset_limited: bool,
    pub(crate) continuity_rotation_limited: bool,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct MouseLookState {
    pub(crate) captured: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct CameraObstacle(pub(crate) CameraObstruction);

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct CameraObstructionMemory {
    state: CameraObstructionHandoffState,
    diagnostics_initialized: bool,
}

pub(crate) type CameraFollowFilter = (With<Camera3d>, Without<Player>);

#[derive(SystemParam)]
pub(crate) struct CameraControlInputSources<'w> {
    mouse_buttons: Res<'w, ButtonInput<MouseButton>>,
    mouse_look: Res<'w, MouseLookState>,
    mouse_motion: Res<'w, AccumulatedMouseMotion>,
}

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
    spawn_follow_camera_with_settings(
        commands,
        scattering_mediums,
        player_start,
        FollowCamera::default(),
        world_radius,
        clear_color,
    );
}

pub(crate) fn spawn_follow_camera_with_settings(
    commands: &mut Commands,
    scattering_mediums: &mut ResMut<Assets<ScatteringMedium>>,
    player_start: Vec3,
    follow_camera: FollowCamera,
    world_radius: f32,
    clear_color: Color,
) {
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
    profile: Option<Res<PlayProfileRun>>,
    tuning: Res<CameraControlTuning>,
    mut state: ResMut<CameraControlState>,
    sources: CameraControlInputSources,
    mut manual_look_active_last_frame: Local<bool>,
) {
    let input = if let Some(run) = eval.as_deref() {
        *manual_look_active_last_frame = false;
        scripted_camera_input(run.scenario, run.frame)
    } else if let Some(input) = profile
        .as_deref()
        .and_then(|profile| profile.scripted_camera_input(eval_dt(&time, eval.as_deref())))
    {
        *manual_look_active_last_frame = false;
        input
    } else {
        let mouse_delta = sources.mouse_motion.delta;
        let look_active =
            sources.mouse_look.captured || sources.mouse_buttons.pressed(MouseButton::Right);
        manual_camera_input(mouse_delta, look_active, &mut manual_look_active_last_frame)
    };

    step_camera_control(&mut state, input, &tuning, eval_dt(&time, eval.as_deref()));
}

pub(crate) fn follow_camera(
    time: Res<Time>,
    eval: Option<Res<EvalRun>>,
    profile: Option<Res<PlayProfileRun>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scene: CameraScene,
) {
    let Ok((player_transform, player_velocity)) = scene.player.single() else {
        return;
    };
    let Ok((mut camera_transform, follow, mut follow_state, mut obstruction_memory)) =
        scene.camera.single_mut()
    else {
        return;
    };
    let previous_camera_position = camera_transform.translation;
    let previous_camera_rotation = camera_transform.rotation;

    let dt = eval_dt(&time, eval.as_deref());
    let movement_input = if let Some(run) = eval.as_deref() {
        scripted_input(run.scenario, run.frame)
    } else if let Some(input) = profile
        .as_deref()
        .and_then(PlayProfileRun::scripted_flight_input)
    {
        input
    } else {
        keyboard_flight_input(&keyboard)
    };
    let desired_follow_direction = movement_input_stable_follow_direction(
        player_velocity.0,
        *player_transform.forward(),
        follow_state.direction,
        movement_input.planar_axis(),
    );
    let follow_direction =
        update_follow_direction_state(&mut follow_state, desired_follow_direction, follow, dt);
    let frame = step_camera_with_direction_and_input(
        camera_transform.translation,
        camera_transform.rotation,
        player_transform.translation,
        follow_direction,
        follow,
        scene.camera_control.orbit,
        scene.camera_control.input_active,
        dt,
    );
    let pre_floor_position = frame.position;
    let initial_camera_floor_y =
        camera_floor_y(&scene.route, frame.position, previous_camera_position);
    let frame =
        lift_camera_above_floor(frame, initial_camera_floor_y, CAMERA_MIN_SURFACE_CLEARANCE);
    let floor_lifted = frame.position.distance(pre_floor_position) > 0.0001;
    let camera_initializing = !obstruction_memory.diagnostics_initialized;
    obstruction_memory
        .state
        .set_intentional_camera_motion(scene.camera_control.input_active || camera_initializing);
    let obstruction_step = resolve_camera_obstruction_handoff(
        frame,
        previous_camera_position,
        previous_camera_rotation,
        player_transform.translation,
        scene.obstacles.iter().map(|obstacle| obstacle.0),
        CAMERA_OBSTRUCTION_CLEARANCE,
        dt,
        &mut obstruction_memory.state,
        |frame| {
            let camera_floor_y =
                camera_floor_y(&scene.route, frame.position, previous_camera_position);
            lift_camera_above_floor(frame, camera_floor_y, CAMERA_MIN_SURFACE_CLEARANCE)
        },
    );
    let frame = obstruction_step.frame;
    let orbit_alignment_degrees = camera_orbit_alignment_degrees(
        frame.position,
        frame.look_target,
        follow_direction,
        scene.camera_control.orbit,
    );

    let (diagnostics_previous_position, diagnostics_previous_rotation) =
        if obstruction_memory.diagnostics_initialized {
            (previous_camera_position, previous_camera_rotation)
        } else {
            (frame.position, frame.rotation)
        };
    obstruction_memory.diagnostics_initialized = true;

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
    scene.camera_diagnostics.obstruction_adjustment_m = obstruction_step.obstruction_adjustment_m;
    scene.camera_diagnostics.obstruction_hits = obstruction_step.obstruction_hits;
    scene.camera_diagnostics.correction_source = if scene.camera_control.input_active {
        CameraCorrectionSource::Input
    } else if obstruction_step.obstruction_hits > 0 {
        CameraCorrectionSource::Obstruction
    } else if floor_lifted {
        CameraCorrectionSource::Floor
    } else if obstruction_step.distance_clamped {
        CameraCorrectionSource::Distance
    } else {
        CameraCorrectionSource::Follow
    };
    scene.camera_diagnostics.continuity_offset_limited = obstruction_step.continuity_offset_limited;
    scene.camera_diagnostics.continuity_rotation_limited =
        obstruction_step.continuity_rotation_limited;

    camera_transform.translation = frame.position;
    camera_transform.rotation = frame.rotation;
}

pub(crate) fn direct_plateau_vista_camera(
    eval: Option<Res<EvalRun>>,
    route: Res<SkyRoute>,
    mut diagnostics: ResMut<CameraDiagnostics>,
    mut camera: Query<&mut Transform, CameraFollowFilter>,
    mut previous_pose: Local<Option<(Vec3, Quat)>>,
) {
    let Some(run) = eval.as_deref() else {
        *previous_pose = None;
        return;
    };
    if !matches!(
        run.scenario.name,
        GREAT_SKY_PLATEAU_VISTAS | ISLAND_SURFACE_REVIEW | ISLAND_HERO_GALLERY
    ) {
        *previous_pose = None;
        return;
    }
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let (position, rotation) = if run.scenario.name == ISLAND_HERO_GALLERY {
        let Some(pose) = run.island_review_pose() else {
            return;
        };
        let rotation = Transform::from_translation(pose.camera_position)
            .looking_at(pose.camera_target, Vec3::Y)
            .rotation;
        (pose.camera_position, rotation)
    } else if run.scenario.name == ISLAND_SURFACE_REVIEW {
        let pose = island_surface_review_camera_pose(run.frame);
        (pose.position, pose.rotation())
    } else {
        let Some(plateau) = route.island_named("great sky plateau") else {
            return;
        };
        let Some(broken_edge) = plateau.plateau_region_position(IslandPlateauRegion::BrokenEdge)
        else {
            return;
        };
        let arrival_position = plateau.center + Vec3::new(95.0, 30.0, 100.0);
        let arrival_target = plateau.center + Vec3::new(-18.0, 7.5, 5.0);
        let broken_edge_offset = IslandPlateauRegion::BrokenEdge.sample_offset();
        let broken_edge_angle = broken_edge_offset.y.atan2(broken_edge_offset.x);
        let broken_edge_contour = plateau.footprint_contour_point(broken_edge_angle, false);
        let waterfall_lip = Vec3::new(
            broken_edge_contour.x,
            plateau.terrain_surface_y_at(Vec3::new(
                broken_edge_contour.x,
                broken_edge.y,
                broken_edge_contour.y,
            )),
            broken_edge_contour.y,
        );
        let outward = (waterfall_lip - plateau.center)
            .with_y(0.0)
            .normalize_or(Vec3::X);
        let tangent = Vec3::new(-outward.z, 0.0, outward.x);
        let waterfall_position = waterfall_lip + outward * 165.0 + tangent * 35.0 + Vec3::Y * 24.0;
        let waterfall_target = waterfall_lip + outward * 4.0 - Vec3::Y * (plateau.thickness * 0.22);
        let arrival_rotation = Transform::from_translation(arrival_position)
            .looking_at(arrival_target, Vec3::Y)
            .rotation;
        let waterfall_rotation = Transform::from_translation(waterfall_position)
            .looking_at(waterfall_target, Vec3::Y)
            .rotation;
        let transition = ((run
            .frame
            .saturating_sub(PLATEAU_VISTA_TRANSITION_START_FRAME))
            as f32
            / (PLATEAU_VISTA_TRANSITION_END_FRAME - PLATEAU_VISTA_TRANSITION_START_FRAME) as f32)
            .clamp(0.0, 1.0);
        let transition = transition * transition * (3.0 - 2.0 * transition);

        (
            arrival_position.lerp(waterfall_position, transition),
            arrival_rotation.slerp(waterfall_rotation, transition),
        )
    };

    if let Some((previous_position, previous_rotation)) = *previous_pose {
        diagnostics.step_distance_m = previous_position.distance(position);
        diagnostics.rotation_delta_degrees = previous_rotation.angle_between(rotation).to_degrees();
    } else {
        diagnostics.step_distance_m = 0.0;
        diagnostics.rotation_delta_degrees = 0.0;
    }
    let view_direction = rotation * Vec3::NEG_Z;
    diagnostics.orbit_alignment_degrees = 0.0;
    diagnostics.follow_direction =
        Vec3::new(view_direction.x, 0.0, view_direction.z).normalize_or(Vec3::NEG_Z);
    diagnostics.follow_direction_error_degrees = 0.0;
    diagnostics.obstruction_adjustment_m = 0.0;
    diagnostics.obstruction_hits = 0;
    diagnostics.correction_source = CameraCorrectionSource::Scripted;
    diagnostics.continuity_offset_limited = false;
    diagnostics.continuity_rotation_limited = false;

    camera_transform.translation = position;
    camera_transform.rotation = rotation;
    *previous_pose = Some((position, rotation));
}

fn island_surface_review_camera_pose(frame: u32) -> DirectCameraPose {
    if frame < SURFACE_REVIEW_SECOND_TRANSITION_START_FRAME {
        ISLAND_SURFACE_REVIEW_CAMERA_POSES[0].interpolate(
            ISLAND_SURFACE_REVIEW_CAMERA_POSES[1],
            smoothstep_frame_progress(
                frame,
                SURFACE_REVIEW_FIRST_TRANSITION_START_FRAME,
                SURFACE_REVIEW_FIRST_TRANSITION_END_FRAME,
            ),
        )
    } else {
        ISLAND_SURFACE_REVIEW_CAMERA_POSES[1].interpolate(
            ISLAND_SURFACE_REVIEW_CAMERA_POSES[2],
            smoothstep_frame_progress(
                frame,
                SURFACE_REVIEW_SECOND_TRANSITION_START_FRAME,
                SURFACE_REVIEW_SECOND_TRANSITION_END_FRAME,
            ),
        )
    }
}

fn smoothstep_frame_progress(frame: u32, start_frame: u32, end_frame: u32) -> f32 {
    let progress = (frame.saturating_sub(start_frame)) as f32 / (end_frame - start_frame) as f32;
    let progress = progress.clamp(0.0, 1.0);
    progress * progress * (3.0 - 2.0 * progress)
}

fn eval_dt(time: &Time, eval: Option<&EvalRun>) -> f32 {
    eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt)
}

fn camera_floor_y(route: &SkyRoute, position: Vec3, previous_camera_position: Vec3) -> f32 {
    let ground = route.ground_at(position);
    let approaching_island_from_below = ground.island_name.is_some()
        && previous_camera_position.y < ground.floor_y
        && position.y < ground.floor_y;
    if approaching_island_from_below {
        world_terrain_floor_y_at(position)
    } else {
        ground.floor_y
    }
}

fn manual_camera_input(
    mouse_delta: Vec2,
    look_active: bool,
    look_active_last_frame: &mut bool,
) -> CameraInput {
    let activated_this_frame = look_active && !*look_active_last_frame;
    *look_active_last_frame = look_active;
    let stale_capture_delta =
        activated_this_frame && mouse_delta.length() > CAMERA_CAPTURE_STALE_DELTA_THRESHOLD_PX;
    CameraInput {
        mouse_delta: if look_active && mouse_delta.is_finite() && !stale_capture_delta {
            mouse_delta
        } else {
            Vec2::ZERO
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_mouse_delta_after_look_capture_quarantines_only_implausible_spikes() {
        let mut active_last_frame = false;
        let stale_ui_delta = Vec2::new(120.0, -45.0);

        assert_eq!(
            manual_camera_input(stale_ui_delta, true, &mut active_last_frame),
            CameraInput::default()
        );
        assert_eq!(
            manual_camera_input(Vec2::new(4.0, -2.0), true, &mut active_last_frame),
            CameraInput {
                mouse_delta: Vec2::new(4.0, -2.0)
            }
        );

        let mut fresh_capture = false;
        assert_eq!(
            manual_camera_input(Vec2::new(4.0, -2.0), true, &mut fresh_capture),
            CameraInput {
                mouse_delta: Vec2::new(4.0, -2.0)
            },
            "ordinary click-drag motion must be responsive on the capture frame"
        );
    }

    #[test]
    fn inactive_look_resets_capture_history_before_resume() {
        let mut active_last_frame = true;

        assert_eq!(
            manual_camera_input(Vec2::new(18.0, -6.0), false, &mut active_last_frame),
            CameraInput::default()
        );
        assert!(!active_last_frame);
        assert_eq!(
            manual_camera_input(Vec2::new(120.0, -45.0), true, &mut active_last_frame),
            CameraInput::default(),
            "resume must quarantine mouse motion accumulated while look was inactive"
        );
    }

    #[test]
    fn camera_floor_does_not_capture_an_island_from_below() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let mut position = island.center;
        let island_floor_y = route.ground_at(position).floor_y;
        position.y = island_floor_y - 1.0;

        assert_eq!(
            camera_floor_y(&route, position, position),
            world_terrain_floor_y_at(position)
        );
    }

    #[test]
    fn island_surface_review_camera_holds_requested_checkpoint_poses() {
        for (frame, expected_position, expected_target) in [
            (
                60,
                Vec3::new(-306.2, 729.7, -2694.5),
                Vec3::new(-221.2, 695.7, -2629.5),
            ),
            (
                180,
                Vec3::new(-150.0, 720.0, -2482.0),
                Vec3::new(-225.0, 692.0, -2566.0),
            ),
            (
                300,
                Vec3::new(205.0, 725.0, -2482.0),
                Vec3::new(-20.0, 675.0, -2605.0),
            ),
        ] {
            let pose = island_surface_review_camera_pose(frame);
            assert_eq!(pose.position, expected_position);
            assert_eq!(pose.target, expected_target);
        }

        assert_eq!(
            island_surface_review_camera_pose(75),
            ISLAND_SURFACE_REVIEW_CAMERA_POSES[0]
        );
        assert_eq!(
            island_surface_review_camera_pose(165),
            ISLAND_SURFACE_REVIEW_CAMERA_POSES[1]
        );
        assert_eq!(
            island_surface_review_camera_pose(195),
            ISLAND_SURFACE_REVIEW_CAMERA_POSES[1]
        );
        assert_eq!(
            island_surface_review_camera_pose(285),
            ISLAND_SURFACE_REVIEW_CAMERA_POSES[2]
        );
    }

    #[test]
    fn island_surface_review_camera_transitions_stay_within_motion_bounds() {
        let initial_pose = island_surface_review_camera_pose(0);
        let mut previous_position = initial_pose.position;
        let mut previous_rotation = initial_pose.rotation();
        let mut max_step_distance_m = 0.0_f32;
        let mut max_rotation_delta_degrees = 0.0_f32;

        for frame in 1..=360 {
            let pose = island_surface_review_camera_pose(frame);
            let rotation = pose.rotation();
            max_step_distance_m =
                max_step_distance_m.max(previous_position.distance(pose.position));
            max_rotation_delta_degrees = max_rotation_delta_degrees
                .max(previous_rotation.angle_between(rotation).to_degrees());
            previous_position = pose.position;
            previous_rotation = rotation;
        }

        assert!(max_step_distance_m > 0.0);
        assert!(
            max_step_distance_m <= 6.0,
            "surface review camera step reached {max_step_distance_m}m"
        );
        assert!(
            max_rotation_delta_degrees <= 3.5,
            "surface review camera rotation reached {max_rotation_delta_degrees}deg"
        );
    }
}
