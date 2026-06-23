use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend,
};
use nau_engine::camera::{FollowCamera, camera_distance, camera_pitch_degrees, step_camera};
use nau_engine::diagnostics::frame_ms;
use nau_engine::environment::{WindField, WindFieldKind, visible_fields_at};
use nau_engine::eval::{
    EvalAccumulator, EvalArtifacts, EvalSample, EvalScenario, SCENARIO_NAMES, scenario_named,
    scripted_input,
};
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightState, FlightTuning, Velocity,
    face_horizontal_velocity, step_flight,
};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const PLAYER_START: Vec3 = Vec3::new(0.0, 1.2, 0.0);
const WORLD_RADIUS: f32 = 360.0;
const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;

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
        .insert_resource(DebugVisuals::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The NAU Engine Flight Sandbox".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
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
            (animate_character, follow_camera).in_set(GameSet::Camera),
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

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DebugReadout;

#[derive(Resource)]
struct DebugVisuals {
    enabled: bool,
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
    accumulator: EvalAccumulator,
    frame: u32,
    finalized: bool,
    screenshot_wait_frames: u32,
    io_error: Option<String>,
}

impl EvalRun {
    fn new(options: EvalOptions) -> std::io::Result<Self> {
        fs::create_dir_all(&options.output_dir)?;

        let samples_path = options.output_dir.join("samples.ndjson");
        let summary_path = options.output_dir.join("summary.json");
        let screenshot_path = options
            .capture_screenshot
            .then(|| options.output_dir.join("final.png"));

        remove_existing_file(&summary_path)?;
        if let Some(path) = &screenshot_path {
            remove_existing_file(path)?;
        }
        File::create(&samples_path)?;

        Ok(Self {
            scenario: options.scenario,
            samples_path,
            summary_path,
            screenshot_path,
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

type CameraFollowFilter = (With<Camera3d>, Without<Player>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let suit_material = materials.add(Color::srgb(0.18, 0.23, 0.3));
    let skin_material = materials.add(Color::srgb(0.82, 0.58, 0.42));
    let accent_material = materials.add(Color::srgb(0.96, 0.64, 0.16));
    let glider_material = materials.add(Color::srgb(0.78, 0.42, 0.18));
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

    for (index, x) in (-5..=5).enumerate() {
        let height = 5.0 + (index as f32 % 4.0) * 4.0;
        let z = if index % 2 == 0 { -28.0 } else { 34.0 };

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(5.0, height, 5.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.42, 0.38, 0.31))),
            Transform::from_xyz(x as f32 * 20.0, height * 0.5, z),
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

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 7.0, -13.0).looking_at(PLAYER_START + Vec3::Y, Vec3::Y),
        FollowCamera::default(),
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

fn toggle_debug_visuals(keyboard: Res<ButtonInput<KeyCode>>, mut visuals: ResMut<DebugVisuals>) {
    if keyboard.just_pressed(KeyCode::F1) {
        visuals.enabled = !visuals.enabled;
    }
}

fn fly_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    tuning: Res<FlightTuning>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
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
        &tuning,
        &mut transform,
        &mut velocity,
        &mut controller,
    );
}

fn eval_fly_player(
    run: Res<EvalRun>,
    tuning: Res<FlightTuning>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    if run.finalized {
        return;
    }

    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
    };

    step_player(
        run.scenario.fixed_dt,
        scripted_input(run.scenario, run.frame),
        &tuning,
        &mut transform,
        &mut velocity,
        &mut controller,
    );
}

fn step_player(
    dt: f32,
    input: FlightInput,
    tuning: &FlightTuning,
    transform: &mut Transform,
    velocity: &mut Velocity,
    controller: &mut FlightController,
) {
    let next = step_flight(
        FlightState::new(transform.translation, velocity.0, *controller),
        input,
        Facing::new(*transform.forward(), *transform.right()),
        tuning,
        dt,
    );

    transform.translation = next.position;
    velocity.0 = next.velocity;
    *controller = next.controller;
    transform.rotation =
        face_horizontal_velocity(transform.rotation, velocity.0, tuning.turn_rate, dt);
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

fn follow_camera(
    time: Res<Time>,
    eval: Option<Res<EvalRun>>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    mut camera: Query<(&mut Transform, &FollowCamera), CameraFollowFilter>,
) {
    let Ok((player_transform, velocity)) = player.single() else {
        return;
    };
    let Ok((mut camera_transform, follow)) = camera.single_mut() else {
        return;
    };

    let frame = step_camera(
        camera_transform.translation,
        camera_transform.rotation,
        player_transform.translation,
        *player_transform.forward(),
        velocity.0,
        follow,
        eval_dt(&time, eval.as_deref()),
    );

    camera_transform.translation = frame.position;
    camera_transform.rotation = frame.rotation;
}

fn eval_dt(time: &Time, eval: Option<&EvalRun>) -> f32 {
    eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt)
}

fn update_debug_readout(
    time: Res<Time>,
    visuals: Res<DebugVisuals>,
    player: Query<(&Transform, &Velocity, &FlightController), With<Player>>,
    camera: Query<&Transform, CameraFollowFilter>,
    wind_fields: Query<&WindField>,
    mut readout: Query<&mut Text, With<DebugReadout>>,
) {
    let Ok((transform, velocity, controller)) = player.single() else {
        return;
    };
    let Ok(mut text) = readout.single_mut() else {
        return;
    };
    let (distance, pitch) = camera
        .single()
        .map(|camera_transform| {
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_pitch_degrees(camera_transform.rotation),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields = visible_fields_at(transform.translation, wind_fields.iter().copied());
    let wind_field_count = wind_fields.iter().count();

    **text = format!(
        "frame {:>4.1} ms\nmode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\ncamera pitch {:>5.1} deg\ncamera distance {:>5.1} m\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nvisual wind fields {} / {}\nlaunch cooldown {:>4.1}s\nlaunch ready {}\ndebug visuals {} (F1)\nWASD steer  Space glider  E launch  Shift dive",
        frame_ms(time.delta_secs()),
        controller.mode.label(),
        velocity.0.length(),
        transform.translation.y,
        pitch,
        distance,
        velocity.0.x,
        velocity.0.y,
        velocity.0.z,
        visible_wind_fields,
        wind_field_count,
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
    player: Query<(&Transform, &Velocity, &FlightController), With<Player>>,
    camera: Query<&Transform, CameraFollowFilter>,
    wind_fields: Query<&WindField>,
    all_entities: Query<Entity>,
) {
    if run.finalized || !run.scenario.should_sample(run.frame) {
        return;
    }

    let Ok((transform, velocity, controller)) = player.single() else {
        return;
    };
    let (camera_distance_m, camera_pitch_degrees) = camera
        .single()
        .map(|camera_transform| {
            (
                camera_distance(camera_transform.translation, transform.translation),
                camera_pitch_degrees(camera_transform.rotation),
            )
        })
        .unwrap_or_default();
    let visible_wind_fields = visible_fields_at(transform.translation, wind_fields.iter().copied());
    let sample = EvalSample::new(
        run.frame,
        run.scenario.fixed_dt,
        transform.translation,
        velocity.0,
        controller.mode,
        camera_distance_m,
        camera_pitch_degrees,
        visible_wind_fields,
        wind_fields.iter().count(),
        all_entities.iter().count(),
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

fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    visuals: Res<DebugVisuals>,
    player: Query<(&Transform, &Velocity), With<Player>>,
    camera: Query<&Transform, CameraFollowFilter>,
    wind_fields: Query<&WindField>,
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
