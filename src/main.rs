use bevy::asset::RenderAssetUsages;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseMotion;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend,
};
use nau_engine::camera::{
    CameraControlState, CameraControlTuning, CameraInput, CameraObstruction, FollowCamera,
    FollowCameraState, apply_camera_input, avoid_camera_obstructions, camera_distance,
    camera_orbit_alignment_degrees, camera_pitch_degrees, camera_surface_clearance,
    camera_target_angle_degrees, camera_view_yaw_degrees, lift_camera_above_floor,
    step_camera_with_direction, update_follow_direction_state,
};
use nau_engine::diagnostics::frame_ms;
use nau_engine::environment::{
    LiftField, WindField, WindFieldKind, active_lift_fields_at, apply_lift_fields,
    visible_fields_at,
};
use nau_engine::eval::{
    EvalAccumulator, EvalArtifacts, EvalSample, EvalScenario, SCENARIO_NAMES, scenario_named,
    scripted_camera_input, scripted_input,
};
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning, Velocity,
    face_horizontal_velocity, step_flight,
};
use nau_engine::world::{START_POSITION, SkyIsland, SkyRoute};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const PLAYER_START: Vec3 = START_POSITION;
const WORLD_RADIUS: f32 = 920.0;
const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;
const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;

fn main() -> AppExit {
    let cli = match CliAction::from_env() {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            return AppExit::from_code(2);
        }
    };

    let CliAction::Run { eval } = cli else {
        println!("{}", usage());
        return AppExit::Success;
    };

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.55, 0.72, 0.9)))
        .insert_resource(FlightTuning::default())
        .insert_resource(CameraControlTuning::default())
        .insert_resource(CameraControlState::default())
        .insert_resource(CameraDiagnostics::default())
        .insert_resource(MouseLookState::default())
        .insert_resource(DebugVisuals::default())
        .insert_resource(SkyRoute::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window(eval.as_ref())),
            ..default()
        }))
        .configure_sets(
            Update,
            (
                GameSet::Movement,
                GameSet::Camera,
                GameSet::Diagnostics,
                GameSet::Eval,
            )
                .chain(),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update_mouse_look_capture,
                update_camera_control,
                animate_character,
                follow_camera,
            )
                .chain()
                .in_set(GameSet::Camera),
        )
        .add_systems(
            Update,
            (update_debug_readout, draw_debug_gizmos).in_set(GameSet::Diagnostics),
        );

    if let Some(eval_options) = eval {
        let eval_run = match EvalRun::new(eval_options) {
            Ok(eval_run) => eval_run,
            Err(error) => {
                eprintln!("failed to prepare eval output: {error}");
                return AppExit::from_code(2);
            }
        };

        app.insert_resource(eval_run)
            .add_systems(Update, eval_fly_player.in_set(GameSet::Movement))
            .add_systems(
                Update,
                (collect_eval_metrics, finish_eval_frame)
                    .chain()
                    .in_set(GameSet::Eval),
            );
    } else {
        app.add_systems(
            Update,
            (toggle_debug_visuals, fly_player).in_set(GameSet::Movement),
        );
    }

    app.run()
}

fn primary_window(eval: Option<&EvalOptions>) -> Window {
    let hidden_metric_eval = eval.is_some_and(|options| !options.capture_screenshot);

    Window {
        title: "The NAU Engine Flight Sandbox".into(),
        resolution: (1280, 720).into(),
        visible: !hidden_metric_eval,
        focused: !hidden_metric_eval,
        ..default()
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DebugReadout;

#[derive(Component, Clone, Copy, Debug)]
struct CameraObstacle(CameraObstruction);

#[derive(Resource, Clone, Copy, Debug, Default)]
struct CameraDiagnostics {
    step_distance_m: f32,
    rotation_delta_degrees: f32,
    orbit_alignment_degrees: f32,
    obstruction_adjustment_m: f32,
    obstruction_hits: usize,
}

#[derive(Resource)]
struct DebugVisuals {
    enabled: bool,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
struct MouseLookState {
    captured: bool,
}

impl Default for DebugVisuals {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum GameSet {
    Movement,
    Camera,
    Diagnostics,
    Eval,
}

#[derive(SystemParam)]
struct MovementWorld<'w, 's> {
    route: Res<'w, SkyRoute>,
    lift_fields: Query<'w, 's, &'static LiftField>,
}

#[derive(SystemParam)]
struct CameraScene<'w, 's> {
    route: Res<'w, SkyRoute>,
    camera_control: Res<'w, CameraControlState>,
    camera_diagnostics: ResMut<'w, CameraDiagnostics>,
    player: Query<'w, 's, &'static Transform, With<Player>>,
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

#[derive(SystemParam)]
struct DebugScene<'w, 's> {
    route: Res<'w, SkyRoute>,
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
    camera: Query<'w, 's, &'static Transform, CameraFollowFilter>,
    camera_control: Res<'w, CameraControlState>,
    camera_diagnostics: Res<'w, CameraDiagnostics>,
    mouse_look: Res<'w, MouseLookState>,
    wind_fields: Query<'w, 's, &'static WindField>,
    lift_fields: Query<'w, 's, &'static LiftField>,
}

#[derive(SystemParam)]
struct EvalScene<'w, 's> {
    route: Res<'w, SkyRoute>,
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
    camera: Query<'w, 's, &'static Transform, CameraFollowFilter>,
    camera_diagnostics: Res<'w, CameraDiagnostics>,
    wind_fields: Query<'w, 's, &'static WindField>,
    lift_fields: Query<'w, 's, &'static LiftField>,
    all_entities: Query<'w, 's, Entity>,
}

struct PlayerKinematics<'a> {
    transform: &'a mut Transform,
    velocity: &'a mut Velocity,
    controller: &'a mut FlightController,
}

#[derive(Clone, Debug)]
struct EvalOptions {
    scenario: EvalScenario,
    output_dir: PathBuf,
    capture_screenshot: bool,
}

#[derive(Clone, Debug)]
enum CliAction {
    Run { eval: Option<EvalOptions> },
    Help,
}

impl CliAction {
    fn from_env() -> Result<Self, String> {
        parse_cli_args(env::args().skip(1))
    }
}

#[derive(Resource, Debug)]
struct EvalRun {
    scenario: EvalScenario,
    samples_path: PathBuf,
    summary_path: PathBuf,
    screenshot_path: Option<PathBuf>,
    checkpoint_captures: Vec<EvalCheckpointCapture>,
    accumulator: EvalAccumulator,
    frame: u32,
    finalized: bool,
    screenshot_wait_frames: u32,
    io_error: Option<String>,
}

#[derive(Debug)]
struct EvalCheckpointCapture {
    frame: u32,
    path: PathBuf,
    captured: bool,
}

impl EvalRun {
    fn new(options: EvalOptions) -> std::io::Result<Self> {
        fs::create_dir_all(&options.output_dir)?;

        let samples_path = options.output_dir.join("samples.ndjson");
        let summary_path = options.output_dir.join("summary.json");
        let screenshot_path = options
            .capture_screenshot
            .then(|| options.output_dir.join("final.png"));
        let mut checkpoint_captures = Vec::new();

        remove_existing_file(&summary_path)?;
        if let Some(path) = &screenshot_path {
            remove_existing_file(path)?;
        }
        if options.capture_screenshot {
            let checkpoint_dir = options.output_dir.join("checkpoints");
            remove_existing_dir(&checkpoint_dir)?;
            fs::create_dir_all(&checkpoint_dir)?;
            checkpoint_captures = options
                .scenario
                .checkpoints
                .iter()
                .map(|checkpoint| EvalCheckpointCapture {
                    frame: checkpoint.frame,
                    path: checkpoint_dir
                        .join(format!("{:04}_{}.png", checkpoint.frame, checkpoint.name)),
                    captured: false,
                })
                .collect();
        }
        File::create(&samples_path)?;

        Ok(Self {
            scenario: options.scenario,
            samples_path,
            summary_path,
            screenshot_path,
            checkpoint_captures,
            accumulator: EvalAccumulator::default(),
            frame: 0,
            finalized: false,
            screenshot_wait_frames: 0,
            io_error: None,
        })
    }

    fn record_sample(&mut self, sample: EvalSample) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new().append(true).open(&self.samples_path)?;
        writeln!(file, "{}", sample.to_json())?;
        self.accumulator.observe(sample);
        Ok(())
    }

    fn write_summary(&self) -> Result<bool, std::io::Error> {
        let artifacts = EvalArtifacts {
            summary_json: path_string(&self.summary_path),
            samples_ndjson: path_string(&self.samples_path),
            screenshot_png: self.screenshot_path.as_deref().map(path_string),
            checkpoint_screenshots: self
                .checkpoint_captures
                .iter()
                .map(|checkpoint| path_string(&checkpoint.path))
                .collect(),
        };
        let summary = self.accumulator.summary(self.scenario, artifacts);
        let passed = summary.passed;

        fs::write(&self.summary_path, summary.to_json())?;
        Ok(passed)
    }
}

fn parse_cli_args(args: impl IntoIterator<Item = String>) -> Result<CliAction, String> {
    let mut eval_name = None;
    let mut eval_output = None;
    let mut capture_screenshot = true;
    let mut saw_eval = false;
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            return Ok(CliAction::Help);
        } else if arg == "--eval" {
            saw_eval = true;
            eval_name = Some(
                args.next()
                    .ok_or_else(|| "--eval requires a scenario name".to_string())?,
            );
        } else if let Some(value) = arg.strip_prefix("--eval=") {
            saw_eval = true;
            eval_name = Some(value.to_string());
        } else if arg == "--eval-output" {
            eval_output =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--eval-output requires a path".to_string()
                })?));
        } else if let Some(value) = arg.strip_prefix("--eval-output=") {
            eval_output = Some(PathBuf::from(value));
        } else if arg == "--eval-no-screenshot" {
            capture_screenshot = false;
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }

    let eval = if saw_eval {
        let name = eval_name.unwrap_or_else(|| "baseline_route".to_string());
        let scenario = scenario_named(&name).ok_or_else(|| {
            format!(
                "unknown eval scenario: {name}. available scenarios: {}",
                SCENARIO_NAMES.join(", ")
            )
        })?;
        let output_dir = eval_output.unwrap_or_else(|| PathBuf::from("target/eval").join(name));

        Some(EvalOptions {
            scenario,
            output_dir,
            capture_screenshot,
        })
    } else {
        None
    };

    Ok(CliAction::Run { eval })
}

fn usage() -> String {
    format!(
        "Usage:\n  cargo run\n  cargo run -- --eval <scenario> [--eval-output <dir>] [--eval-no-screenshot]\n\nScenarios: {}",
        SCENARIO_NAMES.join(", ")
    )
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn remove_existing_file(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn remove_existing_dir(path: &Path) -> std::io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

type CameraFollowFilter = (With<Camera3d>, Without<Player>);

fn setup(
    mut commands: Commands,
    route: Res<SkyRoute>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let suit_material = materials.add(Color::srgb(0.18, 0.23, 0.3));
    let skin_material = materials.add(Color::srgb(0.82, 0.58, 0.42));
    let accent_material = materials.add(Color::srgb(0.96, 0.64, 0.16));
    let glider_material = materials.add(Color::srgb(0.78, 0.42, 0.18));
    let island_grass_material = materials.add(Color::srgb(0.26, 0.58, 0.32));
    let island_meadow_material = materials.add(Color::srgb(0.42, 0.58, 0.32));
    let island_clay_material = materials.add(Color::srgb(0.52, 0.44, 0.30));
    let island_alpine_material = materials.add(Color::srgb(0.24, 0.48, 0.52));
    let island_highland_material = materials.add(Color::srgb(0.56, 0.54, 0.40));
    let target_grass_material = materials.add(Color::srgb(0.34, 0.62, 0.42));
    let island_rock_material = materials.add(Color::srgb(0.38, 0.34, 0.3));
    let island_under_material = materials.add(Color::srgb(0.22, 0.2, 0.18));
    let target_marker_material = materials.add(Color::srgb(0.95, 0.74, 0.22));
    let trunk_material = materials.add(Color::srgb(0.32, 0.2, 0.12));
    let foliage_material = materials.add(Color::srgb(0.12, 0.44, 0.24));
    let flower_material = materials.add(Color::srgb(0.82, 0.22, 0.44));
    let water_material = materials.add(Color::srgba(0.18, 0.54, 0.82, 0.82));
    let path_material = materials.add(Color::srgb(0.47, 0.41, 0.32));
    let torso_mesh = meshes.add(Capsule3d::new(0.4, 1.0));
    let head_mesh = meshes.add(Sphere::new(0.3));
    let arm_mesh = meshes.add(Cuboid::new(0.2, 0.82, 0.2));
    let leg_mesh = meshes.add(Cuboid::new(0.24, 0.9, 0.24));
    let wing_mesh = meshes.add(Cuboid::new(2.15, 0.05, 0.75));

    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.55, 0.0)),
    ));

    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(WORLD_RADIUS * 2.0, WORLD_RADIUS * 2.0),
            ),
        ),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.44, 0.25))),
        Transform::default(),
    ));

    for (index, island) in route.islands().iter().enumerate() {
        let top_material = if island.is_target {
            target_grass_material.clone()
        } else {
            match index % 5 {
                0 => island_grass_material.clone(),
                1 => island_meadow_material.clone(),
                2 => island_clay_material.clone(),
                3 => island_alpine_material.clone(),
                _ => island_highland_material.clone(),
            }
        };

        spawn_sky_island(
            &mut commands,
            &mut meshes,
            top_material,
            island_rock_material.clone(),
            island_under_material.clone(),
            target_marker_material.clone(),
            trunk_material.clone(),
            foliage_material.clone(),
            flower_material.clone(),
            water_material.clone(),
            path_material.clone(),
            index,
            *island,
        );
    }

    for (index, x) in (-5..=5).enumerate() {
        let height = 5.0 + (index as f32 % 4.0) * 4.0;
        let z = if index % 2 == 0 { -28.0 } else { 34.0 };

        let center = Vec3::new(x as f32 * 20.0, height * 0.5, z);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(5.0, height, 5.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.42, 0.38, 0.31))),
            Transform::from_translation(center),
            CameraObstacle(CameraObstruction::new(
                center,
                Vec3::new(2.5, height * 0.5, 2.5),
            )),
        ));
    }

    commands.spawn((
        WindField::crosswind(
            Vec3::new(0.0, 5.0, 20.0),
            Vec3::new(20.0, 4.0, 8.0),
            Vec3::X,
            10.0,
        ),
        Name::new("Visual wind ribbon"),
    ));
    commands.spawn((
        WindField::updraft(Vec3::new(-28.0, 14.0, 24.0), Vec3::new(9.0, 14.0, 9.0), 8.0),
        Name::new("Visual updraft column"),
    ));
    commands.spawn((
        WindField::crosswind(
            Vec3::new(34.0, 10.0, -8.0),
            Vec3::new(18.0, 8.0, 10.0),
            Vec3::new(-1.0, 0.0, 0.35),
            7.0,
        ),
        Name::new("Visual crosswind volume"),
    ));
    commands.spawn((
        WindField::updraft(
            Vec3::new(24.0, 74.0, -430.0),
            Vec3::new(18.0, 30.0, 18.0),
            10.0,
        ),
        Name::new("Distant visual updraft column"),
    ));
    commands.spawn((
        LiftField::updraft(
            Vec3::new(38.0, 68.0, -112.0),
            Vec3::new(20.0, 34.0, 22.0),
            28.0,
            20.0,
        ),
        Name::new("Gameplay updraft lift"),
    ));
    commands.spawn((
        LiftField::updraft(
            Vec3::new(24.0, 74.0, -430.0),
            Vec3::new(26.0, 42.0, 26.0),
            24.0,
            22.0,
        ),
        Name::new("Distant gameplay updraft lift"),
    ));

    commands
        .spawn((
            Transform::from_translation(PLAYER_START),
            Player,
            Velocity::default(),
            FlightController::default(),
            AnimationState::default(),
            Visibility::Inherited,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(torso_mesh.clone()),
                MeshMaterial3d(suit_material.clone()),
                Transform::from_xyz(0.0, 0.95, 0.0),
                Visibility::Inherited,
                CharacterPart::new(
                    CharacterPartRole::Torso,
                    Vec3::new(0.0, 0.95, 0.0),
                    Quat::IDENTITY,
                ),
            ));

            parent.spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(skin_material),
                Transform::from_xyz(0.0, 1.78, 0.0),
                Visibility::Inherited,
                CharacterPart::new(
                    CharacterPartRole::Head,
                    Vec3::new(0.0, 1.78, 0.0),
                    Quat::IDENTITY,
                ),
            ));

            for side in [Side::Left, Side::Right] {
                let sign = side.sign();
                let arm_translation = Vec3::new(sign * 0.58, 1.05, 0.0);
                let arm_rotation = Quat::from_rotation_z(sign * 0.18);
                let leg_translation = Vec3::new(sign * 0.22, 0.28, 0.0);
                let leg_rotation = Quat::from_rotation_z(sign * 0.08);

                parent.spawn((
                    Mesh3d(arm_mesh.clone()),
                    MeshMaterial3d(suit_material.clone()),
                    Transform {
                        translation: arm_translation,
                        rotation: arm_rotation,
                        ..default()
                    },
                    Visibility::Inherited,
                    CharacterPart::new(CharacterPartRole::Arm(side), arm_translation, arm_rotation),
                ));

                parent.spawn((
                    Mesh3d(leg_mesh.clone()),
                    MeshMaterial3d(suit_material.clone()),
                    Transform {
                        translation: leg_translation,
                        rotation: leg_rotation,
                        ..default()
                    },
                    Visibility::Inherited,
                    CharacterPart::new(CharacterPartRole::Leg(side), leg_translation, leg_rotation),
                ));

                let wing_translation = Vec3::new(sign * 1.02, 1.45, -0.46);
                let wing_rotation =
                    Quat::from_rotation_z(sign * 0.16) * Quat::from_rotation_x(-0.08);

                parent.spawn((
                    Mesh3d(wing_mesh.clone()),
                    MeshMaterial3d(glider_material.clone()),
                    Transform {
                        translation: wing_translation,
                        rotation: wing_rotation,
                        ..default()
                    },
                    Visibility::Hidden,
                    CharacterPart::new(
                        CharacterPartRole::Wing(side),
                        wing_translation,
                        wing_rotation,
                    ),
                ));
            }

            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.18, 0.18, 0.38))),
                MeshMaterial3d(accent_material),
                Transform::from_xyz(0.0, 1.15, -0.28),
            ));
        });

    let follow_camera = FollowCamera::default();
    let initial_camera_direction = Vec3::NEG_Z;
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(
            PLAYER_START - initial_camera_direction * follow_camera.distance
                + Vec3::Y * follow_camera.height,
        )
        .looking_at(
            PLAYER_START
                + Vec3::Y * follow_camera.look_height
                + initial_camera_direction * follow_camera.look_ahead,
            Vec3::Y,
        ),
        follow_camera,
        FollowCameraState::default(),
    ));

    commands.spawn((
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(14.0),
            ..default()
        },
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        DebugReadout,
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn_sky_island(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    top_material: Handle<StandardMaterial>,
    rock_material: Handle<StandardMaterial>,
    under_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    trunk_material: Handle<StandardMaterial>,
    foliage_material: Handle<StandardMaterial>,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let top_thickness = 0.55;
    let top_y = island.mesh_top_y();

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.0, top_thickness))),
        MeshMaterial3d(top_material.clone()),
        Transform {
            translation: Vec3::new(
                island.center.x,
                top_y - top_thickness * 0.5,
                island.center.z,
            ),
            scale: Vec3::new(island.half_extents.x, 1.0, island.half_extents.y),
            ..default()
        },
        island,
        Name::new(island.name),
    ));

    commands.spawn((
        Mesh3d(meshes.add(island_terrain_mesh(island_index, island))),
        MeshMaterial3d(top_material),
        Transform::default(),
        Name::new("island terrain surface"),
    ));

    let rock_body_center = Vec3::new(
        island.center.x,
        top_y - top_thickness - island.thickness * 0.5,
        island.center.z,
    );
    let rock_body_half_extents = Vec3::new(
        island.half_extents.x * 0.78,
        island.thickness * 0.5,
        island.half_extents.y * 0.78,
    );
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.0, island.thickness))),
        MeshMaterial3d(rock_material),
        Transform {
            translation: rock_body_center,
            scale: Vec3::new(rock_body_half_extents.x, 1.0, rock_body_half_extents.z),
            ..default()
        },
        CameraObstacle(CameraObstruction::new(
            rock_body_center,
            rock_body_half_extents,
        )),
        Name::new("island rock body"),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.0, island.thickness * 0.7))),
        MeshMaterial3d(under_material.clone()),
        Transform {
            translation: Vec3::new(
                island.center.x,
                top_y - top_thickness - island.thickness * 1.08,
                island.center.z,
            ),
            scale: Vec3::new(
                island.half_extents.x * 0.42,
                1.0,
                island.half_extents.y * 0.42,
            ),
            ..default()
        },
        Name::new("island shadow base"),
    ));

    let ridge_width = island.half_extents.x * 0.32;
    let ridge_center = Vec3::new(
        island.center.x + island.half_extents.x * 0.28,
        top_y + 0.1,
        island.center.z - island.half_extents.y * 0.24,
    );
    let ridge_half_extents = Vec3::new(ridge_width * 0.5, 0.375, island.half_extents.y * 0.09);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(ridge_width, 0.75, island.half_extents.y * 0.18))),
        MeshMaterial3d(under_material),
        Transform::from_translation(ridge_center),
        CameraObstacle(CameraObstruction::new(ridge_center, ridge_half_extents)),
        Name::new("island ridge"),
    ));

    if island.is_target {
        let marker_center = Vec3::new(island.center.x, island.floor_y() + 1.8, island.center.z);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.2, 6.0, 2.2))),
            MeshMaterial3d(marker_material),
            Transform::from_translation(marker_center),
            CameraObstacle(CameraObstruction::new(
                marker_center,
                Vec3::new(1.1, 3.0, 1.1),
            )),
            Name::new("landing target marker"),
        ));
    }

    spawn_sky_island_details(
        commands,
        meshes,
        trunk_material,
        foliage_material,
        flower_material,
        water_material,
        path_material,
        island_index,
        island,
    );
}

fn island_terrain_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    const RINGS: usize = 8;
    const SEGMENTS: usize = 48;

    let top_y = island.mesh_top_y();
    let vertex_count = 1 + RINGS * SEGMENTS;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(SEGMENTS * 3 + (RINGS - 1) * SEGMENTS * 6);

    positions.push([
        island.center.x,
        top_y + island_terrain_height(island_index, 0.0, 0.0),
        island.center.z,
    ]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for ring in 1..=RINGS {
        let radius = ring as f32 / RINGS as f32;
        for segment in 0..SEGMENTS {
            let angle = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            let phase = island_index as f32 * 0.73;
            let edge_variation =
                1.0 + 0.035 * (angle * 5.0 + phase).sin() + 0.018 * (angle * 9.0).cos();
            let x = island.center.x + angle.cos() * island.half_extents.x * radius * edge_variation;
            let z = island.center.z + angle.sin() * island.half_extents.y * radius * edge_variation;
            let y = top_y + island_terrain_height(island_index, radius, angle);

            positions.push([x, y, z]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([
                0.5 + angle.cos() * radius * 0.5,
                0.5 + angle.sin() * radius * 0.5,
            ]);
        }
    }

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (1 + (ring - 1) * SEGMENTS + segment % SEGMENTS) as u32
    };

    for segment in 0..SEGMENTS {
        indices.extend([0, ring_index(1, segment + 1), ring_index(1, segment)]);
    }

    for ring in 1..RINGS {
        for segment in 0..SEGMENTS {
            let inner_current = ring_index(ring, segment);
            let inner_next = ring_index(ring, segment + 1);
            let outer_current = ring_index(ring + 1, segment);
            let outer_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                inner_current,
                inner_next,
                outer_current,
                inner_next,
                outer_next,
                outer_current,
            ]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn island_terrain_height(island_index: usize, radius: f32, angle: f32) -> f32 {
    let phase = island_index as f32 * 0.83;
    let ridges = (angle * 3.0 + phase).sin() * 0.035 + (angle * 7.0 - phase * 0.5).cos() * 0.018;
    let dome = (1.0 - radius).powi(2) * 0.045;
    let edge_softening = radius.powf(2.4) * 0.04;

    (0.028 + dome + ridges * (1.0 - radius * 0.45) - edge_softening).clamp(0.008, 0.065)
}

#[allow(clippy::too_many_arguments)]
fn spawn_sky_island_details(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    trunk_material: Handle<StandardMaterial>,
    foliage_material: Handle<StandardMaterial>,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let floor_y = island.floor_y();
    let detail_phase = island_index as f32 * 0.77;
    let tree_offsets = [
        Vec2::new(-0.42, -0.24),
        Vec2::new(0.34, -0.36),
        Vec2::new(0.24, 0.32),
    ];

    for (index, offset) in tree_offsets.into_iter().enumerate() {
        if island.is_target && index == 1 {
            continue;
        }
        let sway = (detail_phase + index as f32).sin() * 0.08;
        let x = island.center.x + island.half_extents.x * (offset.x + sway);
        let z = island.center.z + island.half_extents.y * offset.y;
        let trunk_height = 2.1 + index as f32 * 0.25;
        let trunk_center = Vec3::new(x, floor_y + trunk_height * 0.5, z);
        let canopy_radius = 1.05 + index as f32 * 0.08;
        let canopy_center = Vec3::new(x, floor_y + trunk_height + 0.72, z);

        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.22, trunk_height))),
            MeshMaterial3d(trunk_material.clone()),
            Transform::from_translation(trunk_center),
            CameraObstacle(CameraObstruction::new(
                trunk_center,
                Vec3::new(0.22, trunk_height * 0.5, 0.22),
            )),
            Name::new("island tree trunk"),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(canopy_radius))),
            MeshMaterial3d(foliage_material.clone()),
            Transform::from_translation(canopy_center),
            CameraObstacle(CameraObstruction::new(
                canopy_center,
                Vec3::splat(canopy_radius),
            )),
            Name::new("island tree canopy"),
        ));
    }

    for index in 0..5 {
        let angle = detail_phase + index as f32 * 1.37;
        let radius = if index % 2 == 0 { 0.52 } else { 0.72 };
        let x = island.center.x + angle.cos() * island.half_extents.x * radius;
        let z = island.center.z + angle.sin() * island.half_extents.y * radius;
        let stone_scale = 0.45 + index as f32 * 0.08;

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(stone_scale))),
            MeshMaterial3d(path_material.clone()),
            Transform::from_xyz(x, floor_y + stone_scale * 0.45, z),
            Name::new("island stone scatter"),
        ));
    }

    let pond_offset = if island.is_target {
        Vec2::new(-0.34, 0.18)
    } else {
        Vec2::new(0.18, 0.28)
    };
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.0, 0.08))),
        MeshMaterial3d(water_material),
        Transform {
            translation: Vec3::new(
                island.center.x + island.half_extents.x * pond_offset.x,
                floor_y + 0.04,
                island.center.z + island.half_extents.y * pond_offset.y,
            ),
            scale: Vec3::new(
                island.half_extents.x * 0.12,
                1.0,
                island.half_extents.y * 0.08,
            ),
            ..default()
        },
        Name::new("island pond"),
    ));

    if !island.is_target && island.name != "launch mesa" {
        let beacon_height = 1.4 + (island_index % 3) as f32 * 0.35;
        let beacon_center = Vec3::new(
            island.center.x - island.half_extents.x * 0.18,
            floor_y + beacon_height * 0.5,
            island.center.z + island.half_extents.y * 0.22,
        );
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.42, beacon_height))),
            MeshMaterial3d(flower_material.clone()),
            Transform::from_translation(beacon_center),
            Name::new("route cairn"),
        ));
    }

    if island.is_target {
        let ring_size = 8.0;
        for (translation, scale) in [
            (
                Vec3::new(0.0, 0.05, ring_size * 0.5),
                Vec3::new(ring_size, 0.1, 0.35),
            ),
            (
                Vec3::new(0.0, 0.05, -ring_size * 0.5),
                Vec3::new(ring_size, 0.1, 0.35),
            ),
            (
                Vec3::new(ring_size * 0.5, 0.05, 0.0),
                Vec3::new(0.35, 0.1, ring_size),
            ),
            (
                Vec3::new(-ring_size * 0.5, 0.05, 0.0),
                Vec3::new(0.35, 0.1, ring_size),
            ),
        ] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(scale.x, scale.y, scale.z))),
                MeshMaterial3d(flower_material.clone()),
                Transform::from_translation(island.center + translation),
                Name::new("landing garden ring"),
            ));
        }
    } else if island.name == "launch mesa" {
        let beacon_center = Vec3::new(
            island.center.x - island.half_extents.x * 0.42,
            floor_y + 1.6,
            island.center.z + island.half_extents.y * 0.38,
        );
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.7, 3.2))),
            MeshMaterial3d(flower_material),
            Transform::from_translation(beacon_center),
            CameraObstacle(CameraObstruction::new(
                beacon_center,
                Vec3::new(0.7, 1.6, 0.7),
            )),
            Name::new("launch beacon"),
        ));

        let launch_tree_height = 4.4;
        let launch_tree_center =
            Vec3::new(island.center.x, floor_y + launch_tree_height * 0.5, 8.0);
        let launch_canopy_radius = 1.55;
        let launch_canopy_center =
            Vec3::new(island.center.x, floor_y + launch_tree_height + 0.85, 8.0);
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.35, launch_tree_height))),
            MeshMaterial3d(trunk_material),
            Transform::from_translation(launch_tree_center),
            CameraObstacle(CameraObstruction::new(
                launch_tree_center,
                Vec3::new(0.35, launch_tree_height * 0.5, 0.35),
            )),
            Name::new("launch camera tree trunk"),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(launch_canopy_radius))),
            MeshMaterial3d(foliage_material),
            Transform::from_translation(launch_canopy_center),
            CameraObstacle(CameraObstruction::new(
                launch_canopy_center,
                Vec3::splat(launch_canopy_radius),
            )),
            Name::new("launch camera tree canopy"),
        ));
    }
}

fn toggle_debug_visuals(keyboard: Res<ButtonInput<KeyCode>>, mut visuals: ResMut<DebugVisuals>) {
    if keyboard.just_pressed(KeyCode::F1) {
        visuals.enabled = !visuals.enabled;
    }
}

fn fly_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    tuning: Res<FlightTuning>,
    world: MovementWorld,
    camera: Query<&Transform, CameraFollowFilter>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
    };
    let facing = movement_facing(camera.single().ok(), &transform);
    let mut kinematics = PlayerKinematics {
        transform: &mut transform,
        velocity: &mut velocity,
        controller: &mut controller,
    };

    step_player(
        time.delta_secs(),
        FlightInput {
            forward: keyboard.pressed(KeyCode::KeyW),
            backward: keyboard.pressed(KeyCode::KeyS),
            left: keyboard.pressed(KeyCode::KeyA),
            right: keyboard.pressed(KeyCode::KeyD),
            glide: keyboard.pressed(KeyCode::Space),
            dive: keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
            launch: keyboard.just_pressed(KeyCode::KeyE),
        },
        facing,
        &tuning,
        &world.route,
        world.lift_fields.iter().copied(),
        &mut kinematics,
    );
}

fn eval_fly_player(
    run: Res<EvalRun>,
    tuning: Res<FlightTuning>,
    world: MovementWorld,
    camera: Query<&Transform, CameraFollowFilter>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    if run.finalized {
        return;
    }

    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
    };
    let facing = movement_facing(camera.single().ok(), &transform);
    let mut kinematics = PlayerKinematics {
        transform: &mut transform,
        velocity: &mut velocity,
        controller: &mut controller,
    };

    step_player(
        run.scenario.fixed_dt,
        scripted_input(run.scenario, run.frame),
        facing,
        &tuning,
        &world.route,
        world.lift_fields.iter().copied(),
        &mut kinematics,
    );
}

fn step_player(
    dt: f32,
    input: FlightInput,
    facing: Facing,
    tuning: &FlightTuning,
    route: &SkyRoute,
    lift_fields: impl IntoIterator<Item = LiftField>,
    player: &mut PlayerKinematics,
) {
    let mut tuning = *tuning;
    let was_grounded = route.is_grounded_at(player.transform.translation);
    tuning.floor_y = route.ground_at(player.transform.translation).floor_y;
    let next = step_flight(
        FlightState::new(
            player.transform.translation,
            player.velocity.0,
            *player.controller,
        ),
        input,
        facing,
        &tuning,
        dt,
    );
    let mut next = next;
    let lift = apply_lift_fields(
        next.position,
        next.velocity,
        lift_fields,
        dt,
        next.controller.mode != FlightMode::Grounded,
    );
    next.velocity = lift.velocity;
    let next = route.resolve_ground_contact_after_step(next, was_grounded);

    player.transform.translation = next.position;
    player.velocity.0 = next.velocity;
    *player.controller = next.controller;
    player.transform.rotation = face_horizontal_velocity(
        player.transform.rotation,
        player.velocity.0,
        tuning.turn_rate,
        dt,
    );
}

fn movement_facing(camera: Option<&Transform>, player_transform: &Transform) -> Facing {
    camera.map_or_else(
        || Facing::new(*player_transform.forward(), *player_transform.right()),
        |camera_transform| Facing::new(*camera_transform.forward(), *camera_transform.right()),
    )
}

fn animate_character(
    time: Res<Time>,
    eval: Option<Res<EvalRun>>,
    mut player: Query<(&Velocity, &FlightController, &mut AnimationState), With<Player>>,
    mut parts: Query<(&CharacterPart, &mut Transform, &mut Visibility)>,
) {
    let Ok((velocity, controller, mut animation)) = player.single_mut() else {
        return;
    };

    let dt = eval_dt(&time, eval.as_deref());
    animation.phase = advance_phase(animation.phase, velocity.0.length(), dt);
    let blend = pose_blend(dt);

    for (part, mut transform, mut visibility) in &mut parts {
        let pose = part_pose(part, controller.mode, velocity.0, animation.phase);
        transform.translation = transform.translation.lerp(pose.translation, blend);
        transform.rotation = transform.rotation.slerp(pose.rotation, blend);

        *visibility = match pose.visibility {
            PartVisibility::Inherited => Visibility::Inherited,
            PartVisibility::Hidden => Visibility::Hidden,
            PartVisibility::Visible => Visibility::Visible,
        };
    }
}

fn update_mouse_look_capture(
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

fn update_camera_control(
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

fn follow_camera(time: Res<Time>, eval: Option<Res<EvalRun>>, mut scene: CameraScene) {
    let Ok(player_transform) = scene.player.single() else {
        return;
    };
    let Ok((mut camera_transform, follow, mut follow_state)) = scene.camera.single_mut() else {
        return;
    };
    let previous_camera_position = camera_transform.translation;
    let previous_camera_rotation = camera_transform.rotation;

    let dt = eval_dt(&time, eval.as_deref());
    let desired_follow_direction = follow_state.direction;
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

    scene.camera_diagnostics.step_distance_m = previous_camera_position.distance(frame.position);
    scene.camera_diagnostics.rotation_delta_degrees = previous_camera_rotation
        .angle_between(frame.rotation)
        .to_degrees();
    scene.camera_diagnostics.orbit_alignment_degrees = orbit_alignment_degrees;
    scene.camera_diagnostics.obstruction_adjustment_m = obstruction_resolution.adjusted_distance_m;
    scene.camera_diagnostics.obstruction_hits = obstruction_resolution.hit_count;

    camera_transform.translation = frame.position;
    camera_transform.rotation = frame.rotation;
}

fn eval_dt(time: &Time, eval: Option<&EvalRun>) -> f32 {
    eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt)
}

fn update_debug_readout(
    time: Res<Time>,
    visuals: Res<DebugVisuals>,
    scene: DebugScene,
    mut readout: Query<&mut Text, With<DebugReadout>>,
) {
    let Ok((transform, velocity, controller)) = scene.player.single() else {
        return;
    };
    let Ok(mut text) = readout.single_mut() else {
        return;
    };
    let player_focus = transform.translation + Vec3::Y * CAMERA_PLAYER_FOCUS_HEIGHT;
    let (distance, pitch, framing_angle) = scene
        .camera
        .single()
        .map(|camera_transform| {
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_pitch_degrees(camera_transform.rotation),
                camera_target_angle_degrees(
                    camera_transform.translation,
                    camera_transform.rotation,
                    player_focus,
                ),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields =
        visible_fields_at(transform.translation, scene.wind_fields.iter().copied());
    let wind_field_count = scene.wind_fields.iter().count();
    let active_lift_fields =
        active_lift_fields_at(transform.translation, scene.lift_fields.iter().copied());
    let lift_field_count = scene.lift_fields.iter().count();
    let target_distance = scene.route.target_distance(transform.translation);
    let on_target = scene
        .route
        .on_landing_target(transform.translation, controller.mode);
    let camera_yaw = scene.camera_control.orbit.yaw_degrees();
    let camera_pitch_offset = scene.camera_control.orbit.pitch_degrees();
    let mouse_lock = if scene.mouse_look.captured {
        "locked"
    } else {
        "free"
    };

    **text = format!(
        "frame {:>4.1} ms\nmode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\ntarget {:>5.1} m {}\ncamera pitch {:>5.1} deg\ncamera distance {:>5.1} m\ncamera frame {:>5.1} deg\ncamera motion {:>4.1} m / {:>4.1} deg\ncamera orbit {:>5.1} deg\ncamera obstruction {:>4.1} m / {}\nmouse yaw {:>5.1} deg\nmouse pitch {:>5.1} deg\nmouse {}\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nvisual wind fields {} / {}\nlift fields {} / {}\nsky islands {}\nlaunch cooldown {:>4.1}s\nlaunch ready {}\ndebug visuals {} (F1)\nWASD camera-relative  Click mouse lock  Esc release  Space glider  E launch  Shift dive",
        frame_ms(time.delta_secs()),
        controller.mode.label(),
        velocity.0.length(),
        transform.translation.y,
        target_distance,
        if on_target { "landed" } else { "out" },
        pitch,
        distance,
        framing_angle,
        scene.camera_diagnostics.step_distance_m,
        scene.camera_diagnostics.rotation_delta_degrees,
        scene.camera_diagnostics.orbit_alignment_degrees,
        scene.camera_diagnostics.obstruction_adjustment_m,
        scene.camera_diagnostics.obstruction_hits,
        camera_yaw,
        camera_pitch_offset,
        mouse_lock,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z,
        visible_wind_fields,
        wind_field_count,
        active_lift_fields,
        lift_field_count,
        scene.route.islands().len(),
        controller.launch_cooldown_remaining,
        if controller.launch_available {
            "yes"
        } else {
            "no"
        },
        if visuals.enabled { "on" } else { "off" }
    );
}

fn collect_eval_metrics(
    mut run: ResMut<EvalRun>,
    camera_control: Res<CameraControlState>,
    scene: EvalScene,
) {
    if run.finalized || !run.scenario.should_sample(run.frame) {
        return;
    }

    let Ok((transform, velocity, controller)) = scene.player.single() else {
        return;
    };
    let (
        camera_distance_m,
        camera_surface_clearance_m,
        camera_player_angle_degrees,
        camera_pitch_degrees,
        camera_view_yaw,
    ) = scene
        .camera
        .single()
        .map(|camera_transform| {
            let camera_floor_y = scene.route.ground_at(camera_transform.translation).floor_y;
            let player_focus = transform.translation + Vec3::Y * CAMERA_PLAYER_FOCUS_HEIGHT;
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_surface_clearance(camera_transform.translation, camera_floor_y),
                camera_target_angle_degrees(
                    camera_transform.translation,
                    camera_transform.rotation,
                    player_focus,
                ),
                camera_pitch_degrees(camera_transform.rotation),
                camera_view_yaw_degrees(camera_transform.rotation, Vec3::NEG_Z),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields =
        visible_fields_at(transform.translation, scene.wind_fields.iter().copied());
    let active_lift_fields =
        active_lift_fields_at(transform.translation, scene.lift_fields.iter().copied());
    let sample = EvalSample::new(
        run.frame,
        run.scenario.fixed_dt,
        transform.translation,
        velocity.0,
        controller.mode,
        camera_distance_m,
        camera_surface_clearance_m,
        camera_player_angle_degrees,
        camera_pitch_degrees,
        camera_control.orbit.yaw_degrees(),
        camera_control.orbit.pitch_degrees(),
        scene.camera_diagnostics.step_distance_m,
        scene.camera_diagnostics.rotation_delta_degrees,
        scene.camera_diagnostics.orbit_alignment_degrees,
        camera_view_yaw,
        scene.camera_diagnostics.obstruction_adjustment_m,
        scene.camera_diagnostics.obstruction_hits,
        visible_wind_fields,
        scene.wind_fields.iter().count(),
        active_lift_fields,
        scene.lift_fields.iter().count(),
        scene.route.target_distance(transform.translation),
        scene
            .route
            .on_landing_target(transform.translation, controller.mode),
        scene.route.islands().len(),
        scene.all_entities.iter().count(),
    );

    if let Err(error) = run.record_sample(sample) {
        run.io_error = Some(format!("failed to write eval sample: {error}"));
    }
}

fn finish_eval_frame(
    mut commands: Commands,
    mut run: ResMut<EvalRun>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if let Some(error) = run.io_error.clone() {
        eprintln!("{error}");
        run.finalized = true;
        app_exit.write(AppExit::error());
        return;
    }

    if run.finalized {
        run.screenshot_wait_frames += 1;
        if run.screenshot_path.is_some()
            && run.screenshot_wait_frames > EVAL_SCREENSHOT_TIMEOUT_FRAMES
        {
            eprintln!(
                "eval screenshot did not finish within {} frames",
                EVAL_SCREENSHOT_TIMEOUT_FRAMES
            );
            app_exit.write(AppExit::error());
        }
        return;
    }

    capture_due_checkpoint_screenshots(&mut commands, &mut run);

    if run.frame < run.scenario.frame_count {
        run.frame += 1;
        return;
    }

    let passed = match run.write_summary() {
        Ok(passed) => passed,
        Err(error) => {
            eprintln!("failed to write eval summary: {error}");
            run.finalized = true;
            app_exit.write(AppExit::error());
            return;
        }
    };

    run.finalized = true;
    eprintln!("eval summary: {}", path_string(&run.summary_path));
    let exit = if passed {
        AppExit::Success
    } else {
        AppExit::error()
    };

    if let Some(screenshot_path) = run.screenshot_path.clone() {
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>, mut app_exit_writer: MessageWriter<AppExit>| {
                save_to_disk(screenshot_path.clone())(captured);
                app_exit_writer.write(exit.clone());
            },
        );
    } else {
        app_exit.write(exit);
    }
}

fn capture_due_checkpoint_screenshots(commands: &mut Commands, run: &mut EvalRun) {
    let frame = run.frame;
    for checkpoint in run
        .checkpoint_captures
        .iter_mut()
        .filter(|checkpoint| !checkpoint.captured && checkpoint.frame == frame)
    {
        let screenshot_path = checkpoint.path.clone();
        checkpoint.captured = true;
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>| {
                save_to_disk(screenshot_path.clone())(captured);
            },
        );
    }
}

fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    visuals: Res<DebugVisuals>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    camera: Query<&Transform, CameraFollowFilter>,
    wind_fields: Query<&WindField>,
    lift_fields: Query<&LiftField>,
) {
    if !visuals.enabled {
        return;
    }

    let Ok((player_transform, velocity)) = player.single() else {
        return;
    };

    let origin = player_transform.translation + Vec3::Y * 1.4;
    draw_vector(
        &mut gizmos,
        origin,
        capped_vector(velocity.0, 0.16, 7.0),
        Color::srgb(0.0, 0.85, 1.0),
    );
    draw_vector(
        &mut gizmos,
        origin,
        *player_transform.forward() * 3.0,
        Color::srgb(1.0, 0.68, 0.16),
    );
    draw_vector(
        &mut gizmos,
        origin,
        *player_transform.right() * 2.0,
        Color::srgb(0.55, 0.6, 0.62),
    );

    if let Ok(camera_transform) = camera.single() {
        gizmos.line(
            camera_transform.translation,
            origin,
            Color::srgb(1.0, 1.0, 1.0),
        );
    }

    for field in &wind_fields {
        draw_wind_field(&mut gizmos, *field);
    }
    for field in &lift_fields {
        draw_lift_field(&mut gizmos, *field);
    }
}

fn draw_wind_field(gizmos: &mut Gizmos, field: WindField) {
    const STREAM_COUNT: usize = 16;

    let color = wind_field_color(field.kind);
    draw_wire_box(gizmos, field.center, field.half_extents, color);

    for index in 0..STREAM_COUNT {
        let start = field.stream_origin(index, STREAM_COUNT);
        let stream = capped_vector(field.flow_vector(), 0.65, 7.5);
        draw_vector(gizmos, start, stream, color);
        gizmos.line(start - stream * 0.35, start, color);
    }
}

fn draw_lift_field(gizmos: &mut Gizmos, field: LiftField) {
    const STREAM_COUNT: usize = 12;
    let color = Color::srgb(1.0, 0.82, 0.18);
    draw_wire_box(gizmos, field.center, field.half_extents, color);

    for index in 0..STREAM_COUNT {
        let t = if STREAM_COUNT <= 1 {
            0.0
        } else {
            index as f32 / (STREAM_COUNT - 1) as f32
        };
        let angle = t * std::f32::consts::TAU;
        let radius = if index % 2 == 0 { 0.35 } else { 0.72 };
        let start = field.center - Vec3::Y * field.half_extents.y
            + Vec3::new(
                angle.cos() * field.half_extents.x * radius,
                0.0,
                angle.sin() * field.half_extents.z * radius,
            );
        draw_vector(
            gizmos,
            start,
            Vec3::Y * field.lift_accel.min(field.max_upward_speed).max(2.0) * 0.32,
            color,
        );
    }
}

fn draw_vector(gizmos: &mut Gizmos, start: Vec3, vector: Vec3, color: Color) {
    if vector.length_squared() > 0.0001 {
        gizmos.arrow(start, start + vector, color);
    }
}

fn capped_vector(vector: Vec3, scale: f32, max_length: f32) -> Vec3 {
    let scaled = vector * scale;
    let max_length_squared = max_length * max_length;

    if scaled.length_squared() <= max_length_squared {
        scaled
    } else {
        scaled.normalize() * max_length
    }
}

fn draw_wire_box(gizmos: &mut Gizmos, center: Vec3, half_extents: Vec3, color: Color) {
    const EDGES: [(usize, usize); 12] = [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    let min = center - half_extents;
    let max = center + half_extents;
    let corners = [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ];

    for (start, end) in EDGES {
        gizmos.line(corners[start], corners[end], color);
    }
}

fn wind_field_color(kind: WindFieldKind) -> Color {
    match kind {
        WindFieldKind::Crosswind => Color::srgb(0.0, 0.82, 1.0),
        WindFieldKind::Updraft => Color::srgb(0.25, 1.0, 0.45),
    }
}
