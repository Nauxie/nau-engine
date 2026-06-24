use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::system::SystemParam;
use bevy::gltf::GltfAssetLabel;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::input::mouse::MouseMotion;
use bevy::light::{
    AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, DirectionalLightShadowMap,
    VolumetricFog, VolumetricLight,
};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::window::{CompositeAlphaMode, CursorGrabMode, CursorOptions, PrimaryWindow};
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend,
};
use nau_engine::asset_pipeline::{
    VISUAL_ASSET_SPECS, VisualAssetLoadState, VisualAssetPipelineMetrics, VisualAssetSpec,
    visual_asset_pipeline_metrics_with_load_states,
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
    AERIAL_POWER_UP_ROUTE, AerialPowerUp, GAMEPLAY_LIFT_ROUTE, LiftField, LiftRouteNode, WindField,
    WindFieldKind, active_lift_fields_at, apply_aerial_power_up, apply_lift_fields,
    readable_lift_fields_at, visible_fields_at,
};
use nau_engine::eval::{
    EvalAccumulator, EvalArtifacts, EvalObjectiveProgress, EvalSample, EvalScenario,
    SCENARIO_NAMES, scenario_named, scripted_camera_input, scripted_input,
};
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning, Velocity,
    face_horizontal_velocity, step_flight,
};
use nau_engine::world::{
    LodBand, START_POSITION, SkyIsland, SkyRoute, StreamActivation, is_recovery_branch_island,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const PLAYER_START: Vec3 = START_POSITION;
const WORLD_RADIUS: f32 = 920.0;
const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;
const EVAL_FRAME_TIME_WARMUP_FRAMES: u32 = 5;
const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;
const PROCEDURAL_TEXTURE_SIZE: u32 = 64;

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
    let screenshot_eval = eval
        .as_deref()
        .is_some_and(|options| options.capture_screenshot);

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.50, 0.68, 0.92)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.62, 0.68, 0.78),
            brightness: 360.0,
            ..default()
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(FlightTuning::default())
        .insert_resource(CameraControlTuning::default())
        .insert_resource(CameraControlState::default())
        .insert_resource(CameraDiagnostics::default())
        .insert_resource(CinematicWeather::default())
        .insert_resource(VisualAssetDiagnostics::default())
        .insert_resource(IslandStreamDiagnostics::default())
        .insert_resource(RouteObjectiveTracker::default())
        .insert_resource(PowerUpCollectionState::default())
        .insert_resource(MouseLookState::default())
        .insert_resource(DebugVisuals {
            enabled: !screenshot_eval,
        })
        .insert_resource(SkyRoute::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window(eval.as_deref())),
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
            (
                update_island_stream_visibility,
                update_cinematic_weather,
                update_weather_drift,
                update_updraft_guides,
                update_power_up_guides,
                update_route_objectives,
                update_visual_asset_diagnostics,
                update_debug_readout,
                draw_debug_gizmos,
            )
                .chain()
                .in_set(GameSet::Diagnostics),
        );

    if let Some(eval_options) = eval {
        let eval_run = match EvalRun::new(*eval_options) {
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
                (
                    collect_eval_frame_time,
                    collect_eval_metrics,
                    finish_eval_frame,
                )
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
        composite_alpha_mode: CompositeAlphaMode::Opaque,
        transparent: false,
        visible: !hidden_metric_eval,
        focused: !hidden_metric_eval,
        ..default()
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct DebugReadout;

#[derive(Component)]
struct CinematicSun;

#[derive(Resource, Clone, Copy, Debug)]
struct CinematicWeather {
    cycle_seconds: f32,
    haze_floor_m: f32,
    haze_ceiling_m: f32,
}

impl Default for CinematicWeather {
    fn default() -> Self {
        Self {
            cycle_seconds: 96.0,
            haze_floor_m: 240.0,
            haze_ceiling_m: WORLD_RADIUS,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
struct RouteObjectiveTracker {
    target_island_name: Option<&'static str>,
    completed_count: usize,
    total_count: usize,
    current_label: &'static str,
    current_distance_m: f32,
    complete: bool,
}

#[derive(Component, Clone, Copy, Debug)]
struct WeatherDrift {
    origin: Vec3,
    axis: Vec3,
    amplitude: f32,
    bob: f32,
    speed: f32,
    phase: f32,
    spin_speed: f32,
    base_rotation: Quat,
}

#[derive(Component, Clone, Copy, Debug)]
struct UpdraftGuide {
    center: Vec3,
    radius: f32,
    height_offset: f32,
    phase: f32,
    angular_speed: f32,
}

#[derive(Resource, Clone, Debug, Default)]
struct PowerUpCollectionState {
    collected: HashSet<&'static str>,
    activations_this_frame: usize,
    total_activations: usize,
    effect_timer_secs: f32,
}

impl PowerUpCollectionState {
    fn begin_frame(&mut self, dt: f32) {
        self.activations_this_frame = 0;
        self.effect_timer_secs = (self.effect_timer_secs - dt.max(0.0)).max(0.0);
    }

    fn collect(&mut self, power_up: AerialPowerUp) -> bool {
        if !self.collected.insert(power_up.name) {
            return false;
        }

        self.activations_this_frame += 1;
        self.total_activations += 1;
        self.effect_timer_secs = self.effect_timer_secs.max(power_up.effect_duration_secs);
        true
    }

    fn is_collected(&self, power_up: AerialPowerUp) -> bool {
        self.collected.contains(power_up.name)
    }

    fn collected_count(&self) -> usize {
        self.collected.len()
    }

    fn visible_count(&self) -> usize {
        AERIAL_POWER_UP_ROUTE
            .len()
            .saturating_sub(self.collected.len())
    }

    fn active_effects(&self) -> usize {
        usize::from(self.effect_timer_secs > 0.0)
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct AerialPowerUpVisual {
    power_up: AerialPowerUp,
    offset: Vec3,
    scale: f32,
    phase: f32,
    angular_speed: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct IslandLodVisual;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum IslandVisualLayer {
    Terrain,
    Detail,
    Beacon,
    Impostor,
}

impl IslandVisualLayer {
    fn is_resident_in(self, activation: StreamActivation, band: LodBand) -> bool {
        match self {
            Self::Terrain => activation.is_active(),
            Self::Detail => activation.is_active() && band == LodBand::Near,
            Self::Beacon => true,
            Self::Impostor => !activation.is_active() || band != LodBand::Near,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct IslandLodVisualCounts {
    visible_terrain_count: usize,
    hidden_terrain_count: usize,
    visible_detail_count: usize,
    hidden_detail_count: usize,
    visible_beacon_count: usize,
    visible_impostor_count: usize,
    hidden_impostor_count: usize,
}

impl IslandLodVisualCounts {
    fn record(&mut self, layer: IslandVisualLayer, hidden: bool) {
        match (layer, hidden) {
            (IslandVisualLayer::Terrain, false) => self.visible_terrain_count += 1,
            (IslandVisualLayer::Terrain, true) => self.hidden_terrain_count += 1,
            (IslandVisualLayer::Detail, false) => self.visible_detail_count += 1,
            (IslandVisualLayer::Detail, true) => self.hidden_detail_count += 1,
            (IslandVisualLayer::Beacon, false) => self.visible_beacon_count += 1,
            (IslandVisualLayer::Beacon, true) => {}
            (IslandVisualLayer::Impostor, false) => self.visible_impostor_count += 1,
            (IslandVisualLayer::Impostor, true) => self.hidden_impostor_count += 1,
        }
    }

    fn resident_count(self) -> usize {
        self.visible_terrain_count
            + self.visible_detail_count
            + self.visible_beacon_count
            + self.visible_impostor_count
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
struct IslandStreamDiagnostics {
    counts: IslandLodVisualCounts,
    visibility_changes_this_frame: usize,
    max_visibility_changes_per_frame: usize,
    total_visibility_changes: usize,
    initialized: bool,
}

#[derive(Resource, Debug)]
struct VisualAssetRegistry {
    slots: Vec<VisualAssetSlot>,
}

#[derive(Debug)]
struct VisualAssetSlot {
    spec: VisualAssetSpec,
    scene_handle: Option<Handle<Scene>>,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
struct VisualAssetDiagnostics {
    metrics: VisualAssetPipelineMetrics,
}

#[derive(Component, Clone, Copy, Debug)]
struct CameraObstacle(CameraObstruction);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct IslandVisualKey {
    island_name: &'static str,
    layer: IslandVisualLayer,
    index: usize,
}

#[derive(Clone)]
struct IslandVisualEntry {
    key: IslandVisualKey,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    name: &'static str,
}

#[derive(Resource, Default)]
struct IslandVisualCatalog {
    entries: Vec<IslandVisualEntry>,
}

#[derive(Resource, Default)]
struct IslandStreamState {
    spawned: HashMap<IslandVisualKey, Entity>,
}

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
    power_ups: ResMut<'w, PowerUpCollectionState>,
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
    stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    route_objectives: Res<'w, RouteObjectiveTracker>,
    power_ups: Res<'w, PowerUpCollectionState>,
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
    stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    route_objectives: Res<'w, RouteObjectiveTracker>,
    power_ups: Res<'w, PowerUpCollectionState>,
    wind_fields: Query<'w, 's, &'static WindField>,
    lift_fields: Query<'w, 's, &'static LiftField>,
    weather_clouds: Query<'w, 's, Entity, With<WeatherDrift>>,
    all_entities: Query<'w, 's, Entity>,
}

struct PlayerKinematics<'a> {
    transform: &'a mut Transform,
    velocity: &'a mut Velocity,
    controller: &'a mut FlightController,
}

struct PlayerStepContext<'a> {
    tuning: &'a FlightTuning,
    route: &'a SkyRoute,
    lift_fields: &'a [LiftField],
    power_ups: &'a mut PowerUpCollectionState,
}

#[derive(Clone, Debug)]
struct EvalOptions {
    scenario: EvalScenario,
    output_dir: PathBuf,
    capture_screenshot: bool,
}

#[derive(Clone, Debug)]
enum CliAction {
    Run { eval: Option<Box<EvalOptions>> },
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
    pending_screenshot_exit_success: Option<bool>,
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
            pending_screenshot_exit_success: None,
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

        Some(Box::new(EvalOptions {
            scenario,
            output_dir,
            capture_screenshot,
        }))
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
    mut images: ResMut<Assets<Image>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(prepare_visual_asset_registry(&asset_server));

    let suit_material = textured_material(
        &mut images,
        &mut materials,
        [38, 48, 62, 255],
        [24, 30, 42, 255],
        [78, 90, 104, 255],
        3,
        0.82,
        0.32,
    );
    let skin_material = textured_material(
        &mut images,
        &mut materials,
        [206, 145, 100, 255],
        [172, 106, 72, 255],
        [232, 176, 130, 255],
        5,
        0.64,
        0.24,
    );
    let accent_material = emissive_material(
        &mut images,
        &mut materials,
        [238, 156, 36, 255],
        [174, 92, 22, 255],
        [255, 220, 94, 255],
        7,
        LinearRgba::rgb(3.8, 1.7, 0.35),
    );
    let glider_material = textured_material(
        &mut images,
        &mut materials,
        [166, 88, 44, 255],
        [98, 48, 30, 255],
        [222, 156, 72, 255],
        11,
        0.86,
        0.28,
    );
    let island_grass_material = textured_material(
        &mut images,
        &mut materials,
        [54, 128, 70, 255],
        [28, 92, 48, 255],
        [128, 174, 78, 255],
        17,
        0.94,
        0.2,
    );
    let island_meadow_material = textured_material(
        &mut images,
        &mut materials,
        [96, 138, 70, 255],
        [56, 104, 54, 255],
        [166, 172, 90, 255],
        19,
        0.92,
        0.21,
    );
    let island_clay_material = textured_material(
        &mut images,
        &mut materials,
        [126, 104, 76, 255],
        [80, 70, 60, 255],
        [162, 138, 96, 255],
        23,
        0.98,
        0.18,
    );
    let island_alpine_material = textured_material(
        &mut images,
        &mut materials,
        [52, 110, 118, 255],
        [30, 80, 94, 255],
        [142, 176, 164, 255],
        29,
        0.9,
        0.22,
    );
    let island_highland_material = textured_material(
        &mut images,
        &mut materials,
        [132, 132, 92, 255],
        [86, 96, 70, 255],
        [178, 166, 112, 255],
        31,
        0.94,
        0.2,
    );
    let target_grass_material = textured_material(
        &mut images,
        &mut materials,
        [70, 150, 94, 255],
        [34, 100, 62, 255],
        [156, 198, 112, 255],
        37,
        0.9,
        0.24,
    );
    let island_rock_material = textured_material(
        &mut images,
        &mut materials,
        [92, 86, 80, 255],
        [48, 48, 48, 255],
        [140, 128, 112, 255],
        41,
        0.98,
        0.16,
    );
    let island_under_material = textured_material(
        &mut images,
        &mut materials,
        [54, 50, 44, 255],
        [26, 24, 22, 255],
        [88, 78, 64, 255],
        43,
        1.0,
        0.12,
    );
    let target_marker_material = emissive_material(
        &mut images,
        &mut materials,
        [242, 190, 48, 255],
        [170, 112, 24, 255],
        [255, 235, 120, 255],
        47,
        LinearRgba::rgb(4.8, 3.2, 0.7),
    );
    let trunk_material = textured_material(
        &mut images,
        &mut materials,
        [82, 48, 28, 255],
        [46, 28, 18, 255],
        [132, 84, 48, 255],
        53,
        0.96,
        0.16,
    );
    let foliage_material = textured_material(
        &mut images,
        &mut materials,
        [28, 106, 54, 255],
        [14, 70, 38, 255],
        [86, 150, 76, 255],
        59,
        0.88,
        0.22,
    );
    let ground_cover_material = ground_cover_material(&mut images, &mut materials);
    let flower_material = emissive_material(
        &mut images,
        &mut materials,
        [210, 50, 96, 255],
        [124, 28, 80, 255],
        [255, 126, 162, 255],
        61,
        LinearRgba::rgb(1.2, 0.25, 0.45),
    );
    let water_material = water_surface_material(&mut images, &mut materials);
    let path_material = textured_material(
        &mut images,
        &mut materials,
        [118, 102, 76, 255],
        [72, 64, 54, 255],
        [166, 146, 104, 255],
        67,
        0.98,
        0.18,
    );
    let ground_material = textured_material(
        &mut images,
        &mut materials,
        [42, 94, 52, 255],
        [24, 60, 40, 255],
        [92, 130, 68, 255],
        71,
        0.96,
        0.18,
    );
    let pillar_material = textured_material(
        &mut images,
        &mut materials,
        [106, 94, 74, 255],
        [66, 58, 52, 255],
        [152, 134, 100, 255],
        73,
        0.98,
        0.16,
    );
    let cloud_material = cloud_surface_material(&mut materials);
    let cloud_veil_material = cloud_veil_material(&mut materials);
    let updraft_column_material = updraft_column_material(&mut materials);
    let updraft_marker_material = emissive_material(
        &mut images,
        &mut materials,
        [62, 198, 244, 210],
        [20, 118, 184, 210],
        [178, 246, 255, 240],
        83,
        LinearRgba::rgb(0.5, 3.2, 5.8),
    );
    let power_up_material = emissive_material(
        &mut images,
        &mut materials,
        [255, 210, 70, 230],
        [210, 82, 34, 220],
        [255, 246, 150, 255],
        89,
        LinearRgba::rgb(5.6, 2.4, 0.5),
    );
    let torso_mesh = meshes.add(Capsule3d::new(0.4, 1.0));
    let head_mesh = meshes.add(Sphere::new(0.3));
    let arm_mesh = meshes.add(Cuboid::new(0.2, 0.82, 0.2));
    let leg_mesh = meshes.add(Cuboid::new(0.24, 0.9, 0.24));
    let wing_mesh = meshes.add(Cuboid::new(2.15, 0.05, 0.75));

    commands.spawn((
        DirectionalLight {
            illuminance: 48_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, -0.55, 0.0)),
        VolumetricLight,
        CinematicSun,
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 20.0,
            maximum_distance: 340.0,
            ..default()
        }
        .build(),
    ));

    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(WORLD_RADIUS * 2.0, WORLD_RADIUS * 2.0),
            ),
        ),
        MeshMaterial3d(ground_material),
        Transform::default(),
    ));

    let mut island_visual_catalog = IslandVisualCatalog::default();

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

        queue_sky_island(
            &mut island_visual_catalog.entries,
            &mut meshes,
            top_material,
            island_rock_material.clone(),
            island_under_material.clone(),
            target_marker_material.clone(),
            updraft_marker_material.clone(),
            trunk_material.clone(),
            foliage_material.clone(),
            ground_cover_material.clone(),
            flower_material.clone(),
            water_material.clone(),
            path_material.clone(),
            index,
            *island,
        );
    }

    let island_stream_state =
        spawn_initial_island_visuals(&mut commands, &island_visual_catalog, PLAYER_START);
    commands.insert_resource(island_visual_catalog);
    commands.insert_resource(island_stream_state);

    for (index, x) in (-5..=5).enumerate() {
        let height = 5.0 + (index as f32 % 4.0) * 4.0;
        let z = if index % 2 == 0 { -28.0 } else { 34.0 };

        let center = Vec3::new(x as f32 * 20.0, height * 0.5, z);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(5.0, height, 5.0))),
            MeshMaterial3d(pillar_material.clone()),
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
        WindField::crosswind(
            Vec3::new(34.0, 10.0, -8.0),
            Vec3::new(18.0, 8.0, 10.0),
            Vec3::new(-1.0, 0.0, 0.35),
            7.0,
        ),
        Name::new("Visual crosswind volume"),
    ));
    for lift in GAMEPLAY_LIFT_ROUTE {
        commands.spawn((
            lift.visual_field(),
            Name::new(format!("{} visual", lift.name)),
        ));
        commands.spawn((lift.lift_field(), Name::new(lift.name)));
        spawn_updraft_guide(
            &mut commands,
            &mut meshes,
            updraft_column_material.clone(),
            updraft_marker_material.clone(),
            lift,
        );
    }
    spawn_power_up_guides(&mut commands, &mut meshes, power_up_material);

    spawn_weather_layers(
        &mut commands,
        &mut meshes,
        cloud_material,
        cloud_veil_material,
        route.islands(),
    );

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
                end: WORLD_RADIUS,
            },
        },
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

fn prepare_visual_asset_registry(asset_server: &AssetServer) -> VisualAssetRegistry {
    let slots = VISUAL_ASSET_SPECS
        .iter()
        .copied()
        .map(|spec| VisualAssetSlot {
            spec,
            scene_handle: visual_asset_path_exists(spec.gltf_scene_path).then(|| {
                asset_server.load(GltfAssetLabel::Scene(0).from_asset(spec.gltf_scene_path))
            }),
        })
        .collect();

    VisualAssetRegistry { slots }
}

fn visual_asset_path_exists(asset_path: &str) -> bool {
    Path::new("assets").join(asset_path).is_file()
}

#[allow(clippy::too_many_arguments)]
fn textured_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    perceptual_roughness: f32,
    reflectance: f32,
) -> Handle<StandardMaterial> {
    let material_seed = seed.wrapping_add(1_337);
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        metallic_roughness_texture: Some(
            images.add(procedural_material_map(material_seed, perceptual_roughness)),
        ),
        occlusion_texture: Some(
            images.add(procedural_occlusion_map(material_seed.wrapping_add(23))),
        ),
        depth_map: Some(images.add(procedural_depth_map(
            material_seed.wrapping_add(47),
            ImageFilterMode::Nearest,
        ))),
        parallax_depth_scale: 0.012,
        max_parallax_layer_count: 8.0,
        perceptual_roughness,
        reflectance,
        ..default()
    })
}

fn emissive_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    emissive: LinearRgba,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        emissive,
        emissive_exposure_weight: 0.15,
        perceptual_roughness: 0.7,
        reflectance: 0.38,
        ..default()
    })
}

fn water_surface_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.58, 0.86, 0.76),
        base_color_texture: Some(images.add(procedural_surface_texture(
            [54, 154, 210, 210],
            [22, 92, 156, 210],
            [160, 220, 244, 210],
            79,
        ))),
        metallic_roughness_texture: Some(images.add(procedural_material_map(1_079, 0.22))),
        depth_map: Some(images.add(procedural_depth_map(1_113, ImageFilterMode::Linear))),
        parallax_depth_scale: 0.018,
        max_parallax_layer_count: 10.0,
        alpha_mode: AlphaMode::Blend,
        double_sided: true,
        perceptual_roughness: 0.18,
        reflectance: 0.82,
        clearcoat: 0.85,
        clearcoat_perceptual_roughness: 0.06,
        diffuse_transmission: 0.18,
        specular_transmission: 0.08,
        thickness: 0.08,
        ior: 1.33,
        ..default()
    })
}

fn cloud_surface_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.86, 0.91, 0.96, 0.38),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        reflectance: 0.12,
        diffuse_transmission: 0.18,
        ..default()
    })
}

fn cloud_veil_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.76, 0.84, 0.96, 0.24),
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 1.0,
        reflectance: 0.06,
        diffuse_transmission: 0.34,
        ..default()
    })
}

fn updraft_column_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.18, 0.74, 1.0, 0.16),
        emissive: LinearRgba::rgb(0.08, 0.65, 1.4),
        emissive_exposure_weight: 0.25,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.32,
        reflectance: 0.2,
        ..default()
    })
}

fn ground_cover_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(images.add(procedural_surface_texture(
            [62, 138, 62, 255],
            [26, 92, 46, 255],
            [212, 178, 86, 255],
            97,
        ))),
        metallic_roughness_texture: Some(images.add(procedural_material_map(1_397, 0.94))),
        alpha_mode: AlphaMode::Opaque,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 0.94,
        reflectance: 0.2,
        ..default()
    })
}

fn procedural_surface_texture(
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
) -> Image {
    let size = PROCEDURAL_TEXTURE_SIZE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let noise = texture_noise(x, y, seed);
            let vein = (x.wrapping_mul(5) + y.wrapping_mul(3) + seed).is_multiple_of(31);
            let check = (x / 16 + y / 16 + seed).is_multiple_of(2);
            let mut color = if noise < 74 {
                secondary
            } else if noise > 216 {
                accent
            } else {
                primary
            };

            if check {
                color = mix_rgba(color, primary, 178);
            }
            if vein {
                color = mix_rgba(color, accent, 112);
            }

            data.extend_from_slice(&color);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

fn procedural_material_map(seed: u32, roughness: f32) -> Image {
    let size = PROCEDURAL_TEXTURE_SIZE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let noise = texture_noise(x, y, seed) as f32 / 255.0;
            let pore = texture_noise(x / 2, y / 2, seed.wrapping_add(9)) as f32 / 255.0;
            let roughness_value =
                (roughness * (0.82 + noise * 0.28) + pore * 0.08).clamp(0.08, 1.0);
            data.extend_from_slice(&[0, (roughness_value * 255.0) as u8, 0, 255]);
        }
    }

    procedural_data_texture(data, ImageFilterMode::Linear)
}

fn procedural_occlusion_map(seed: u32) -> Image {
    let size = PROCEDURAL_TEXTURE_SIZE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let noise = texture_noise(x, y, seed) as u16;
            let large = texture_noise(x / 4, y / 4, seed.wrapping_add(17)) as u16;
            let occlusion = (190 + noise / 5 + large / 7).min(255) as u8;
            data.extend_from_slice(&[occlusion, occlusion, occlusion, 255]);
        }
    }

    procedural_data_texture(data, ImageFilterMode::Linear)
}

fn procedural_depth_map(seed: u32, filter: ImageFilterMode) -> Image {
    let size = PROCEDURAL_TEXTURE_SIZE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let fine = texture_noise(x, y, seed) as u16;
            let broad = texture_noise(x / 4, y / 4, seed.wrapping_add(31)) as u16;
            let ridge = if (x.wrapping_mul(7) + y.wrapping_mul(11) + seed).is_multiple_of(37) {
                18
            } else {
                0
            };
            let depth = (64 + fine / 3 + broad / 4 + ridge).min(255) as u8;
            data.extend_from_slice(&[depth, depth, depth, 255]);
        }
    }

    procedural_data_texture(data, filter)
}

fn procedural_data_texture(data: Vec<u8>, filter: ImageFilterMode) -> Image {
    let size = PROCEDURAL_TEXTURE_SIZE;
    let anisotropy_clamp = if filter == ImageFilterMode::Linear {
        8
    } else {
        1
    };
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        anisotropy_clamp,
        ..default()
    });
    image
}

fn texture_noise(x: u32, y: u32, seed: u32) -> u8 {
    let mut value = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(2_654_435_761));
    value ^= value >> 13;
    value = value.wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 0xff) as u8
}

fn mix_rgba(source: [u8; 4], target: [u8; 4], target_weight: u16) -> [u8; 4] {
    let source_weight = 255 - target_weight;
    [
        ((source[0] as u16 * source_weight + target[0] as u16 * target_weight) / 255) as u8,
        ((source[1] as u16 * source_weight + target[1] as u16 * target_weight) / 255) as u8,
        ((source[2] as u16 * source_weight + target[2] as u16 * target_weight) / 255) as u8,
        ((source[3] as u16 * source_weight + target[3] as u16 * target_weight) / 255) as u8,
    ]
}

fn mix_color(source: Color, target: Color, target_weight: f32) -> Color {
    source.mix(&target, target_weight.clamp(0.0, 1.0))
}

fn spawn_updraft_guide(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    column_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    lift: LiftRouteNode,
) {
    let radius = lift.half_extents.x.min(lift.half_extents.z);
    let height = lift.half_extents.y * 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius, height))),
        MeshMaterial3d(column_material),
        Transform::from_translation(lift.center),
        Name::new(format!("{} visible column", lift.name)),
    ));

    let marker_mesh = meshes.add(Sphere::new(0.72));
    let ring_radius = radius * 0.82;
    let ring_levels = [-0.78, -0.34, 0.1, 0.54, 0.9];
    let markers_per_ring = 9;

    for (level_index, level) in ring_levels.into_iter().enumerate() {
        for marker_index in 0..markers_per_ring {
            let phase = marker_index as f32 / markers_per_ring as f32 * std::f32::consts::TAU
                + level_index as f32 * 0.46;
            let guide = UpdraftGuide {
                center: lift.center,
                radius: ring_radius,
                height_offset: level * lift.half_extents.y,
                phase,
                angular_speed: 0.35 + level_index as f32 * 0.04,
            };
            commands.spawn((
                Mesh3d(marker_mesh.clone()),
                MeshMaterial3d(marker_material.clone()),
                Transform::from_translation(updraft_guide_position(&guide, 0.0)),
                guide,
                Name::new(format!("{} guide mote", lift.name)),
            ));
        }
    }
}

fn spawn_power_up_guides(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
) {
    let bar_mesh = meshes.add(Cuboid::new(5.0, 0.22, 0.22));
    let core_mesh = meshes.add(Sphere::new(1.1));
    let segments = 10;

    for (power_index, power_up) in AERIAL_POWER_UP_ROUTE.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(core_mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(power_up.center),
            AerialPowerUpVisual {
                power_up,
                offset: Vec3::ZERO,
                scale: 1.0,
                phase: power_index as f32 * 0.7,
                angular_speed: 0.75,
            },
            Name::new(format!("{} core", power_up.name)),
        ));

        for segment in 0..segments {
            let phase = segment as f32 / segments as f32 * std::f32::consts::TAU;
            let radius = power_up.radius_m * 0.58;
            let offset = Vec3::new(phase.cos() * radius, phase.sin() * radius, 0.0);
            commands.spawn((
                Mesh3d(bar_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: power_up.center + offset,
                    rotation: Quat::from_rotation_z(phase + std::f32::consts::FRAC_PI_2),
                    scale: Vec3::splat(1.0),
                },
                AerialPowerUpVisual {
                    power_up,
                    offset,
                    scale: 1.0 + power_index as f32 * 0.08,
                    phase,
                    angular_speed: 0.55 + power_index as f32 * 0.08,
                },
                Name::new(format!("{} ring segment", power_up.name)),
            ));
        }
    }
}

fn spawn_weather_layers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    cloud_material: Handle<StandardMaterial>,
    cloud_veil_material: Handle<StandardMaterial>,
    islands: &[SkyIsland],
) {
    let cloud_mesh = meshes.add(Sphere::new(1.0));
    let veil_mesh = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));

    for (index, island) in islands.iter().enumerate() {
        let phase = index as f32 * 0.73;
        let offset = Vec3::new(
            (phase * 2.1).sin() * island.half_extents.x * 0.75,
            42.0 + (index % 4) as f32 * 7.0,
            (phase * 1.7).cos() * island.half_extents.y * 0.85,
        );
        let origin = island.center + offset;
        let axis = Vec3::new(0.96, 0.0, 0.28).normalize();
        let scale = Vec3::new(
            island.half_extents.x * 0.45 + 18.0,
            2.6 + (index % 3) as f32 * 0.45,
            island.half_extents.y * 0.26 + 8.0,
        );

        commands.spawn((
            Mesh3d(cloud_mesh.clone()),
            MeshMaterial3d(cloud_material.clone()),
            Transform {
                translation: origin,
                scale,
                rotation: Quat::from_rotation_y(phase * 0.35),
            },
            WeatherDrift {
                origin,
                axis,
                amplitude: 5.5 + (index % 5) as f32 * 1.2,
                bob: 0.8 + (index % 3) as f32 * 0.25,
                speed: 0.07 + (index % 4) as f32 * 0.012,
                phase,
                spin_speed: 0.012 + (index % 4) as f32 * 0.004,
                base_rotation: Quat::from_rotation_y(phase * 0.35),
            },
            Name::new("drifting cloud bank"),
        ));

        if index % 2 == 0 {
            let veil_origin = island.center
                + Vec3::new(
                    (phase * 1.3).cos() * island.half_extents.x,
                    78.0 + (index % 3) as f32 * 8.0,
                    (phase * 1.9).sin() * island.half_extents.y,
                );
            let veil_rotation = Quat::from_euler(EulerRot::XYZ, -0.04, phase * 0.27, 0.06);
            commands.spawn((
                Mesh3d(veil_mesh.clone()),
                MeshMaterial3d(cloud_veil_material.clone()),
                Transform {
                    translation: veil_origin,
                    scale: Vec3::new(
                        island.half_extents.x * 1.35 + 36.0,
                        1.0,
                        island.half_extents.y * 0.42 + 18.0,
                    ),
                    rotation: veil_rotation,
                },
                WeatherDrift {
                    origin: veil_origin,
                    axis: Vec3::new(0.74, 0.0, -0.18).normalize(),
                    amplitude: 8.0 + (index % 5) as f32,
                    bob: 0.35,
                    speed: 0.025 + (index % 3) as f32 * 0.006,
                    phase,
                    spin_speed: 0.004,
                    base_rotation: veil_rotation,
                },
                Name::new("high cirrus veil"),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_island_visual(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    name: &'static str,
) {
    let key = IslandVisualKey {
        island_name: island.name,
        layer,
        index: *visual_index,
    };
    *visual_index += 1;

    entries.push(IslandVisualEntry {
        key,
        island,
        layer,
        mesh,
        material,
        transform,
        obstacle,
        name,
    });
}

#[allow(clippy::too_many_arguments)]
fn queue_sky_island(
    entries: &mut Vec<IslandVisualEntry>,
    meshes: &mut Assets<Mesh>,
    top_material: Handle<StandardMaterial>,
    rock_material: Handle<StandardMaterial>,
    under_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    branch_marker_material: Handle<StandardMaterial>,
    trunk_material: Handle<StandardMaterial>,
    foliage_material: Handle<StandardMaterial>,
    ground_cover_material: Handle<StandardMaterial>,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let top_thickness = 0.55;
    let top_y = island.mesh_top_y();
    let mut visual_index = 0;

    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(Cylinder::new(1.0, top_thickness)),
        top_material.clone(),
        Transform {
            translation: Vec3::new(
                island.center.x,
                top_y - top_thickness * 0.5,
                island.center.z,
            ),
            scale: Vec3::new(island.half_extents.x, 1.0, island.half_extents.y),
            ..default()
        },
        None,
        island.name,
    );

    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Impostor,
        meshes.add(island_impostor_mesh(island_index, island)),
        top_material.clone(),
        Transform::default(),
        None,
        "island distant impostor",
    );

    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(island_terrain_mesh(island_index, island)),
        top_material,
        Transform::default(),
        None,
        "island terrain surface",
    );

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
    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(Cylinder::new(1.0, island.thickness)),
        rock_material,
        Transform {
            translation: rock_body_center,
            scale: Vec3::new(rock_body_half_extents.x, 1.0, rock_body_half_extents.z),
            ..default()
        },
        Some(CameraObstacle(CameraObstruction::new(
            rock_body_center,
            rock_body_half_extents,
        ))),
        "island rock body",
    );

    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(Cylinder::new(1.0, island.thickness * 0.7)),
        under_material.clone(),
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
        None,
        "island shadow base",
    );

    let ridge_width = island.half_extents.x * 0.32;
    let ridge_surface = island_visual_surface_position(island, Vec2::new(0.28, -0.24));
    let ridge_center = ridge_surface + Vec3::Y * 0.375;
    let ridge_half_extents = Vec3::new(ridge_width * 0.5, 0.375, island.half_extents.y * 0.09);
    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(Cuboid::new(ridge_width, 0.75, island.half_extents.y * 0.18)),
        under_material,
        Transform::from_translation(ridge_center),
        Some(CameraObstacle(CameraObstruction::new(
            ridge_center,
            ridge_half_extents,
        ))),
        "island ridge",
    );

    if island.is_target {
        let marker_center = Vec3::new(
            island.center.x,
            island.mesh_top_y_at(island.center) + 1.8,
            island.center.z,
        );
        queue_island_visual(
            entries,
            &mut visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(Cuboid::new(2.2, 6.0, 2.2)),
            marker_material,
            Transform::from_translation(marker_center),
            Some(CameraObstacle(CameraObstruction::new(
                marker_center,
                Vec3::new(1.1, 3.0, 1.1),
            ))),
            "landing target marker",
        );
    }
    if is_recovery_branch_island(island.name) {
        queue_recovery_branch_marker(
            entries,
            &mut visual_index,
            meshes,
            branch_marker_material,
            island,
        );
    }

    queue_sky_island_details(
        entries,
        &mut visual_index,
        meshes,
        trunk_material,
        foliage_material,
        ground_cover_material,
        flower_material,
        water_material,
        path_material,
        island_index,
        island,
    );
}

fn queue_recovery_branch_marker(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    meshes: &mut Assets<Mesh>,
    marker_material: Handle<StandardMaterial>,
    island: SkyIsland,
) {
    let mast_height = 5.6;
    let mast_surface = island_visual_surface_position(island, Vec2::new(-0.08, 0.08));
    let mast_center = mast_surface + Vec3::Y * (mast_height * 0.5);
    queue_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Beacon,
        meshes.add(Cylinder::new(0.42, mast_height)),
        marker_material.clone(),
        Transform::from_translation(mast_center),
        None,
        "recovery branch mast",
    );

    let ring_size = 7.2;
    for (offset, scale) in [
        (
            Vec3::new(0.0, 0.09, ring_size * 0.5),
            Vec3::new(ring_size, 0.12, 0.34),
        ),
        (
            Vec3::new(0.0, 0.09, -ring_size * 0.5),
            Vec3::new(ring_size, 0.12, 0.34),
        ),
        (
            Vec3::new(ring_size * 0.5, 0.09, 0.0),
            Vec3::new(0.34, 0.12, ring_size),
        ),
        (
            Vec3::new(-ring_size * 0.5, 0.09, 0.0),
            Vec3::new(0.34, 0.12, ring_size),
        ),
    ] {
        let surface_y = island.mesh_top_y_at(island.center + Vec3::new(offset.x, 0.0, offset.z));
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(Cuboid::new(scale.x, scale.y, scale.z)),
            marker_material.clone(),
            Transform::from_xyz(
                island.center.x + offset.x,
                surface_y + offset.y,
                island.center.z + offset.z,
            ),
            None,
            "recovery branch ring",
        );
    }
}

fn island_visual_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let x = island.center.x + island.half_extents.x * normalized_offset.x;
    let z = island.center.z + island.half_extents.y * normalized_offset.y;

    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
}

fn island_terrain_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    const RINGS: usize = 12;
    const SEGMENTS: usize = 48;

    let vertex_count = 1 + RINGS * SEGMENTS;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(SEGMENTS * 3 + (RINGS - 1) * SEGMENTS * 6);

    positions.push([
        island.center.x,
        island.mesh_top_y_at(island.center),
        island.center.z,
    ]);
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
            let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));

            positions.push([x, y, z]);
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

    let normals = smooth_normals_from_triangles(&positions, &indices);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn smooth_normals_from_triangles(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![Vec3::ZERO; positions.len()];

    for triangle in indices.chunks_exact(3) {
        let a_index = triangle[0] as usize;
        let b_index = triangle[1] as usize;
        let c_index = triangle[2] as usize;
        let a = Vec3::from_array(positions[a_index]);
        let b = Vec3::from_array(positions[b_index]);
        let c = Vec3::from_array(positions[c_index]);
        let mut face_normal = (b - a).cross(c - a).normalize_or_zero();

        if face_normal.y < 0.0 {
            face_normal = -face_normal;
        }
        if face_normal.length_squared() <= f32::EPSILON {
            face_normal = Vec3::Y;
        }

        normals[a_index] += face_normal;
        normals[b_index] += face_normal;
        normals[c_index] += face_normal;
    }

    normals
        .into_iter()
        .map(|normal| {
            if normal.length_squared() <= f32::EPSILON {
                Vec3::Y.to_array()
            } else {
                normal.normalize().to_array()
            }
        })
        .collect()
}

fn island_impostor_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    const SEGMENTS: usize = 20;

    let top_center_y = island.mesh_top_y() - 0.16;
    let lower_center_y = top_center_y - island.thickness * 0.42;
    let bottom_y = top_center_y - island.thickness * 0.92;
    let phase = island_index as f32 * 0.71;
    let top_ring_start = 1;
    let lower_ring_start = top_ring_start + SEGMENTS;
    let bottom_index = lower_ring_start + SEGMENTS;
    let mut positions = Vec::with_capacity(bottom_index + 1);
    let mut normals = Vec::with_capacity(bottom_index + 1);
    let mut uvs = Vec::with_capacity(bottom_index + 1);
    let mut indices = Vec::with_capacity(SEGMENTS * 12);

    positions.push([island.center.x, top_center_y, island.center.z]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for segment in 0..SEGMENTS {
        let angle = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let edge_variation =
            1.0 + 0.09 * (angle * 3.0 + phase).sin() + 0.045 * (angle * 7.0 - phase).cos();
        let radius_x = island.half_extents.x * 0.9 * edge_variation;
        let radius_z = island.half_extents.y * 0.9 * edge_variation;
        let x = island.center.x + angle.cos() * radius_x;
        let z = island.center.z + angle.sin() * radius_z;
        let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) - 0.18;

        positions.push([x, y, z]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5 + angle.cos() * 0.45, 0.5 + angle.sin() * 0.45]);
    }

    for segment in 0..SEGMENTS {
        let angle = segment as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
        let edge_variation = 1.0 + 0.08 * (angle * 4.0 + phase).sin() - 0.035 * (angle * 8.0).cos();
        let radius_x = island.half_extents.x * 0.66 * edge_variation;
        let radius_z = island.half_extents.y * 0.66 * edge_variation;
        let x = island.center.x + angle.cos() * radius_x;
        let z = island.center.z + angle.sin() * radius_z;
        let y = lower_center_y - 0.9 * (angle * 5.0 + phase).sin().abs();

        positions.push([x, y, z]);
        normals.push([angle.cos() * 0.55, 0.24, angle.sin() * 0.55]);
        uvs.push([0.5 + angle.cos() * 0.34, 0.82 + angle.sin() * 0.1]);
    }

    positions.push([island.center.x, bottom_y, island.center.z]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 1.0]);

    for segment in 0..SEGMENTS {
        let next = (segment + 1) % SEGMENTS;
        let top_current = (top_ring_start + segment) as u32;
        let top_next = (top_ring_start + next) as u32;
        let lower_current = (lower_ring_start + segment) as u32;
        let lower_next = (lower_ring_start + next) as u32;
        let bottom = bottom_index as u32;

        indices.extend([0, top_next, top_current]);
        indices.extend([top_current, top_next, lower_current]);
        indices.extend([top_next, lower_next, lower_current]);
        indices.extend([lower_current, lower_next, bottom]);
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

#[allow(clippy::too_many_arguments)]
fn queue_sky_island_details(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    meshes: &mut Assets<Mesh>,
    trunk_material: Handle<StandardMaterial>,
    foliage_material: Handle<StandardMaterial>,
    ground_cover_material: Handle<StandardMaterial>,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    path_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let detail_phase = island_index as f32 * 0.77;
    queue_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Detail,
        meshes.add(island_ground_cover_mesh(island_index, island)),
        ground_cover_material,
        Transform::default(),
        None,
        "island ground cover",
    );

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
        let surface = island_visual_surface_position(island, Vec2::new(offset.x + sway, offset.y));
        let trunk_height = 2.1 + index as f32 * 0.25;
        let trunk_center = surface + Vec3::Y * (trunk_height * 0.5);
        let canopy_radius = 1.05 + index as f32 * 0.08;
        let canopy_center = surface + Vec3::Y * (trunk_height + 0.72);

        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(Cylinder::new(0.22, trunk_height)),
            trunk_material.clone(),
            Transform::from_translation(trunk_center),
            Some(CameraObstacle(CameraObstruction::new(
                trunk_center,
                Vec3::new(0.22, trunk_height * 0.5, 0.22),
            ))),
            "island tree trunk",
        );
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(Sphere::new(canopy_radius)),
            foliage_material.clone(),
            Transform::from_translation(canopy_center),
            Some(CameraObstacle(CameraObstruction::new(
                canopy_center,
                Vec3::splat(canopy_radius),
            ))),
            "island tree canopy",
        );
    }

    for index in 0..5 {
        let angle = detail_phase + index as f32 * 1.37;
        let radius = if index % 2 == 0 { 0.52 } else { 0.72 };
        let x = island.center.x + angle.cos() * island.half_extents.x * radius;
        let z = island.center.z + angle.sin() * island.half_extents.y * radius;
        let stone_scale = 0.45 + index as f32 * 0.08;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));

        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(Sphere::new(stone_scale)),
            path_material.clone(),
            Transform::from_xyz(x, surface_y + stone_scale * 0.45, z),
            None,
            "island stone scatter",
        );
    }

    let pond_offset = if island.is_target {
        Vec2::new(-0.34, 0.18)
    } else {
        Vec2::new(0.18, 0.28)
    };
    let pond_surface = island_visual_surface_position(island, pond_offset);
    queue_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Detail,
        meshes.add(Cylinder::new(1.0, 0.08)),
        water_material,
        Transform {
            translation: pond_surface + Vec3::Y * 0.04,
            scale: Vec3::new(
                island.half_extents.x * 0.12,
                1.0,
                island.half_extents.y * 0.08,
            ),
            ..default()
        },
        None,
        "island pond",
    );

    if !island.is_target && island.name != "launch mesa" {
        let beacon_height = 3.8 + (island_index % 3) as f32 * 0.7;
        let beacon_surface = island_visual_surface_position(island, Vec2::new(-0.18, 0.22));
        let beacon_center = beacon_surface + Vec3::Y * (beacon_height * 0.5);
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(Cylinder::new(0.34, beacon_height)),
            flower_material.clone(),
            Transform::from_translation(beacon_center),
            None,
            "route cairn",
        );
    }

    if island.is_target {
        let ring_size = 8.0;
        for (offset, scale) in [
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
            let surface_y =
                island.mesh_top_y_at(island.center + Vec3::new(offset.x, 0.0, offset.z));
            queue_island_visual(
                entries,
                visual_index,
                island,
                IslandVisualLayer::Beacon,
                meshes.add(Cuboid::new(scale.x, scale.y, scale.z)),
                flower_material.clone(),
                Transform::from_xyz(
                    island.center.x + offset.x,
                    surface_y + offset.y,
                    island.center.z + offset.z,
                ),
                None,
                "landing garden ring",
            );
        }
    } else if island.name == "launch mesa" {
        let beacon_surface = island_visual_surface_position(island, Vec2::new(-0.42, 0.38));
        let beacon_center = beacon_surface + Vec3::Y * 1.6;
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(Cylinder::new(0.7, 3.2)),
            flower_material,
            Transform::from_translation(beacon_center),
            Some(CameraObstacle(CameraObstruction::new(
                beacon_center,
                Vec3::new(0.7, 1.6, 0.7),
            ))),
            "launch beacon",
        );

        let launch_tree_height = 4.4;
        let launch_tree_surface_y =
            island.mesh_top_y_at(Vec3::new(island.center.x, island.center.y, 8.0));
        let launch_tree_center = Vec3::new(
            island.center.x,
            launch_tree_surface_y + launch_tree_height * 0.5,
            8.0,
        );
        let launch_canopy_radius = 1.55;
        let launch_canopy_center = Vec3::new(
            island.center.x,
            launch_tree_surface_y + launch_tree_height + 0.85,
            8.0,
        );
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(Cylinder::new(0.35, launch_tree_height)),
            trunk_material,
            Transform::from_translation(launch_tree_center),
            Some(CameraObstacle(CameraObstruction::new(
                launch_tree_center,
                Vec3::new(0.35, launch_tree_height * 0.5, 0.35),
            ))),
            "launch camera tree trunk",
        );
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(Sphere::new(launch_canopy_radius)),
            foliage_material,
            Transform::from_translation(launch_canopy_center),
            Some(CameraObstacle(CameraObstruction::new(
                launch_canopy_center,
                Vec3::splat(launch_canopy_radius),
            ))),
            "launch camera tree canopy",
        );
    }
}

fn island_ground_cover_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    const PATCHES: usize = 34;
    const BLADES_PER_PATCH: usize = 3;
    const VERTICES_PER_BLADE: usize = 3;
    const INDICES_PER_BLADE: usize = 3;

    let blade_count = PATCHES * BLADES_PER_PATCH;
    let mut positions = Vec::with_capacity(blade_count * VERTICES_PER_BLADE);
    let mut normals = Vec::with_capacity(blade_count * VERTICES_PER_BLADE);
    let mut uvs = Vec::with_capacity(blade_count * VERTICES_PER_BLADE);
    let mut indices = Vec::with_capacity(blade_count * INDICES_PER_BLADE);
    let seed = island_index as u32 * 41 + 503;

    for patch in 0..PATCHES {
        let base_angle = random_unit(seed, patch as u32, 3) * std::f32::consts::TAU;
        let radius = random_unit(seed, patch as u32, 11).sqrt() * 0.86;
        let jitter = Vec2::new(
            (random_unit(seed, patch as u32, 17) - 0.5) * 0.06,
            (random_unit(seed, patch as u32, 23) - 0.5) * 0.06,
        );
        let normalized_offset = Vec2::new(base_angle.cos(), base_angle.sin()) * radius + jitter;
        let x = island.center.x + normalized_offset.x * island.half_extents.x;
        let z = island.center.z + normalized_offset.y * island.half_extents.y;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) + 0.08;

        for blade in 0..BLADES_PER_PATCH {
            let blade_phase =
                base_angle + blade as f32 * std::f32::consts::TAU / BLADES_PER_PATCH as f32;
            let width = 0.18 + random_unit(seed, patch as u32, 31 + blade as u32) * 0.16;
            let height = 0.78 + random_unit(seed, patch as u32, 43 + blade as u32) * 0.72;
            let lean = Vec3::new(blade_phase.cos(), 0.0, blade_phase.sin())
                * (0.1 + random_unit(seed, patch as u32, 53 + blade as u32) * 0.24);
            push_ground_cover_blade(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                Vec3::new(x, surface_y, z),
                blade_phase,
                width,
                height,
                lean,
                patch,
            );
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

#[allow(clippy::too_many_arguments)]
fn push_ground_cover_blade(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    origin: Vec3,
    angle: f32,
    width: f32,
    height: f32,
    lean: Vec3,
    patch: usize,
) {
    let right = Vec3::new(angle.cos(), 0.0, angle.sin());
    let side = right * (width * 0.5);
    let tip = origin + Vec3::Y * height + lean;
    let blade_normal = Vec3::new(right.z * 0.35, 0.8, -right.x * 0.35).normalize();
    let start = positions.len() as u32;

    positions.extend([
        (origin - side).to_array(),
        (origin + side).to_array(),
        tip.to_array(),
    ]);
    normals.extend([blade_normal.to_array(); VERTICES_PER_GROUND_BLADE]);
    let uv_offset = if patch.is_multiple_of(2) { 0.0 } else { 0.5 };
    uvs.extend([
        [uv_offset, 1.0],
        [uv_offset + 0.42, 1.0],
        [uv_offset + 0.21, 0.0],
    ]);
    indices.extend([start, start + 1, start + 2]);
}

const VERTICES_PER_GROUND_BLADE: usize = 3;

fn random_unit(seed: u32, x: u32, salt: u32) -> f32 {
    texture_noise(x.wrapping_mul(17).wrapping_add(salt), salt, seed) as f32 / 255.0
}

fn island_visual_is_resident(entry: &IslandVisualEntry, player_position: Vec3) -> bool {
    let activation = entry.island.stream_activation(player_position);
    let band = entry.island.lod_band(player_position);

    entry.layer.is_resident_in(activation, band)
}

fn spawn_initial_island_visuals(
    commands: &mut Commands,
    catalog: &IslandVisualCatalog,
    player_position: Vec3,
) -> IslandStreamState {
    let mut state = IslandStreamState::default();

    for entry in catalog
        .entries
        .iter()
        .filter(|entry| island_visual_is_resident(entry, player_position))
    {
        let entity = spawn_island_visual_entry(commands, entry);
        state.spawned.insert(entry.key, entity);
    }

    state
}

fn spawn_island_visual_entry(commands: &mut Commands, entry: &IslandVisualEntry) -> Entity {
    let mut entity = commands.spawn((
        Mesh3d(entry.mesh.clone()),
        MeshMaterial3d(entry.material.clone()),
        entry.transform,
        IslandLodVisual,
        Name::new(entry.name),
    ));
    if let Some(obstacle) = entry.obstacle {
        entity.insert(obstacle);
    }

    entity.id()
}

fn update_island_stream_visibility(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    catalog: Res<IslandVisualCatalog>,
    mut stream_state: ResMut<IslandStreamState>,
    mut diagnostics: ResMut<IslandStreamDiagnostics>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let mut counts = IslandLodVisualCounts::default();
    let mut desired_keys = HashSet::new();
    let mut stream_changes = 0;

    for entry in &catalog.entries {
        let resident = island_visual_is_resident(entry, player_transform.translation);
        counts.record(entry.layer, !resident);

        if resident {
            desired_keys.insert(entry.key);
            if let std::collections::hash_map::Entry::Vacant(slot) =
                stream_state.spawned.entry(entry.key)
            {
                let entity = spawn_island_visual_entry(&mut commands, entry);
                slot.insert(entity);
                if diagnostics.initialized {
                    stream_changes += 1;
                }
            }
        }
    }

    let despawned_visuals = stream_state
        .spawned
        .iter()
        .filter_map(|(key, entity)| (!desired_keys.contains(key)).then_some((*key, *entity)))
        .collect::<Vec<_>>();

    for (key, entity) in despawned_visuals {
        commands.entity(entity).despawn();
        stream_state.spawned.remove(&key);
        if diagnostics.initialized {
            stream_changes += 1;
        }
    }

    diagnostics.counts = counts;
    diagnostics.visibility_changes_this_frame = stream_changes;
    diagnostics.max_visibility_changes_per_frame = diagnostics
        .max_visibility_changes_per_frame
        .max(stream_changes);
    diagnostics.total_visibility_changes += stream_changes;
    diagnostics.initialized = true;
}

fn update_cinematic_weather(
    time: Res<Time>,
    weather: Res<CinematicWeather>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<CinematicSun>>,
    mut camera_fx: Query<(&mut Exposure, &mut DistanceFog, &mut VolumetricFog), With<Camera3d>>,
) {
    let cycle = (time.elapsed_secs() / weather.cycle_seconds * std::f32::consts::TAU).sin();
    let warm = (cycle * 0.5 + 0.5).clamp(0.0, 1.0);
    let storm = ((time.elapsed_secs() * 0.037).sin() * 0.5 + 0.5).powf(2.2) * 0.34;
    let cool_light = Color::srgb(0.78, 0.84, 1.0);
    let warm_light = Color::srgb(1.0, 0.82, 0.55);
    let sky_clear = Color::srgb(0.46, 0.66, 0.92);
    let sky_weather = Color::srgb(0.38, 0.48, 0.64);

    clear_color.0 = mix_color(
        mix_color(sky_weather, sky_clear, warm),
        Color::srgb(0.56, 0.70, 0.88),
        0.18,
    );
    ambient.color = mix_color(
        Color::srgb(0.48, 0.56, 0.72),
        Color::srgb(0.72, 0.68, 0.60),
        warm,
    );
    ambient.brightness = 260.0 + warm * 170.0 - storm * 80.0;

    for (mut light, mut transform) in &mut sun {
        light.color = mix_color(cool_light, warm_light, warm);
        light.illuminance = 34_000.0 + warm * 24_000.0 - storm * 7_000.0;
        let elevation = -0.62 - warm * 0.34;
        let yaw = -0.62 + cycle * 0.18;
        transform.rotation = Quat::from_euler(EulerRot::XYZ, elevation, yaw, 0.0);
    }

    for (mut exposure, mut fog, mut volumetric_fog) in &mut camera_fx {
        exposure.ev100 = 12.35 + warm * 0.42 - storm * 0.2;
        fog.color = mix_color(
            Color::srgba(0.44, 0.52, 0.66, 0.58),
            Color::srgba(0.60, 0.74, 0.92, 0.42),
            warm,
        );
        fog.directional_light_color = mix_color(
            Color::srgba(0.72, 0.78, 1.0, 0.36),
            Color::srgba(1.0, 0.78, 0.46, 0.58),
            warm,
        );
        fog.directional_light_exponent = 12.0 + warm * 14.0;
        fog.falloff = FogFalloff::Linear {
            start: weather.haze_floor_m - storm * 70.0,
            end: weather.haze_ceiling_m - storm * 150.0,
        };
        volumetric_fog.ambient_color = mix_color(
            Color::srgb(0.48, 0.56, 0.70),
            Color::srgb(0.76, 0.70, 0.60),
            warm,
        );
        volumetric_fog.ambient_intensity = 0.028 + warm * 0.022 + storm * 0.012;
        volumetric_fog.jitter = 0.42;
        volumetric_fog.step_count = 56;
    }
}

fn update_weather_drift(time: Res<Time>, mut clouds: Query<(&WeatherDrift, &mut Transform)>) {
    let elapsed = time.elapsed_secs();

    for (drift, mut transform) in &mut clouds {
        let sway = (elapsed * drift.speed + drift.phase).sin();
        let bob = (elapsed * drift.speed * 0.7 + drift.phase * 1.9).cos();
        transform.translation =
            drift.origin + drift.axis * sway * drift.amplitude + Vec3::Y * bob * drift.bob;
        transform.rotation =
            drift.base_rotation * Quat::from_rotation_y(elapsed * drift.spin_speed + sway * 0.08);
    }
}

fn update_updraft_guides(time: Res<Time>, mut guides: Query<(&UpdraftGuide, &mut Transform)>) {
    let elapsed = time.elapsed_secs();

    for (guide, mut transform) in &mut guides {
        transform.translation = updraft_guide_position(guide, elapsed);
        transform.rotation = Quat::from_rotation_y(guide.phase + elapsed * guide.angular_speed);
    }
}

fn update_power_up_guides(
    time: Res<Time>,
    collection: Res<PowerUpCollectionState>,
    mut guides: Query<(&AerialPowerUpVisual, &mut Transform, &mut Visibility)>,
) {
    let elapsed = time.elapsed_secs();

    for (guide, mut transform, mut visibility) in &mut guides {
        if collection.is_collected(guide.power_up) {
            *visibility = Visibility::Hidden;
            continue;
        }

        *visibility = Visibility::Inherited;
        let spin = guide.phase + elapsed * guide.angular_speed;
        let pulse = 1.0 + 0.08 * (elapsed * 3.4 + guide.phase).sin();
        transform.translation =
            guide.power_up.center + Quat::from_rotation_z(spin * 0.18).mul_vec3(guide.offset);
        transform.rotation = Quat::from_rotation_z(spin + std::f32::consts::FRAC_PI_2);
        transform.scale = Vec3::splat(guide.scale * pulse);
    }
}

fn update_route_objectives(
    eval: Option<Res<EvalRun>>,
    route: Res<SkyRoute>,
    player: Query<(&Transform, &FlightController), With<Player>>,
    mut tracker: ResMut<RouteObjectiveTracker>,
) {
    let Ok((transform, controller)) = player.single() else {
        return;
    };
    let target_island_name = eval
        .as_deref()
        .and_then(|run| run.scenario.target_island_name);

    if tracker.target_island_name != target_island_name {
        *tracker = RouteObjectiveTracker {
            target_island_name,
            ..default()
        };
    }

    let objectives = route.route_objectives(target_island_name);
    tracker.total_count = objectives.len();
    tracker.completed_count = tracker.completed_count.min(objectives.len());

    while let Some(objective) = objectives.get(tracker.completed_count).copied() {
        if !objective.is_complete(&route, transform.translation, controller.mode) {
            break;
        }
        tracker.completed_count += 1;
    }

    if let Some(objective) = objectives.get(tracker.completed_count).copied() {
        tracker.current_label = objective.label;
        tracker.current_distance_m = objective.horizontal_distance(transform.translation);
        tracker.complete = false;
    } else {
        tracker.current_label = "complete";
        tracker.current_distance_m = 0.0;
        tracker.complete = !objectives.is_empty();
    }
}

fn updraft_guide_position(guide: &UpdraftGuide, elapsed: f32) -> Vec3 {
    let angle = guide.phase + elapsed * guide.angular_speed;
    let bob = (elapsed * 1.4 + guide.phase).sin() * 0.35;
    guide.center
        + Vec3::new(
            angle.cos() * guide.radius,
            guide.height_offset + bob,
            angle.sin() * guide.radius,
        )
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
    mut world: MovementWorld,
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
    let dt = time.delta_secs();
    let lift_fields = world.lift_fields.iter().copied().collect::<Vec<_>>();
    world.power_ups.begin_frame(dt);
    let mut context = PlayerStepContext {
        tuning: &tuning,
        route: &world.route,
        lift_fields: &lift_fields,
        power_ups: &mut world.power_ups,
    };

    step_player(
        dt,
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
        &mut context,
        &mut kinematics,
    );
}

fn eval_fly_player(
    run: Res<EvalRun>,
    tuning: Res<FlightTuning>,
    mut world: MovementWorld,
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
    let dt = run.scenario.fixed_dt;
    let lift_fields = world.lift_fields.iter().copied().collect::<Vec<_>>();
    world.power_ups.begin_frame(dt);
    let mut context = PlayerStepContext {
        tuning: &tuning,
        route: &world.route,
        lift_fields: &lift_fields,
        power_ups: &mut world.power_ups,
    };

    step_player(
        dt,
        scripted_input(run.scenario, run.frame),
        facing,
        &mut context,
        &mut kinematics,
    );
}

fn step_player(
    dt: f32,
    input: FlightInput,
    facing: Facing,
    context: &mut PlayerStepContext,
    player: &mut PlayerKinematics,
) {
    let mut tuning = *context.tuning;
    let was_grounded = context.route.is_grounded_at(player.transform.translation);
    tuning.floor_y = context
        .route
        .ground_at(player.transform.translation)
        .floor_y;
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
        context.lift_fields.iter().copied(),
        dt,
        next.controller.mode != FlightMode::Grounded,
    );
    next.velocity = lift.velocity;
    collect_aerial_power_ups(&mut next, context.power_ups);
    let next = context
        .route
        .resolve_ground_contact_after_step(next, was_grounded);

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

fn collect_aerial_power_ups(state: &mut FlightState, collection: &mut PowerUpCollectionState) {
    if state.controller.mode == FlightMode::Grounded {
        return;
    }

    for power_up in AERIAL_POWER_UP_ROUTE {
        if !collection.is_collected(power_up) && power_up.contains(state.position) {
            state.velocity = apply_aerial_power_up(state.velocity, power_up);
            collection.collect(power_up);
        }
    }
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

fn update_visual_asset_diagnostics(
    asset_server: Res<AssetServer>,
    registry: Res<VisualAssetRegistry>,
    mut diagnostics: ResMut<VisualAssetDiagnostics>,
) {
    diagnostics.metrics =
        visual_asset_pipeline_metrics_with_load_states(&VISUAL_ASSET_SPECS, |spec| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.gltf_scene_path == spec.gltf_scene_path)
                .map_or(VisualAssetLoadState::Missing, |slot| {
                    visual_asset_load_state(&asset_server, slot)
                })
        });
}

fn visual_asset_load_state(
    asset_server: &AssetServer,
    slot: &VisualAssetSlot,
) -> VisualAssetLoadState {
    let Some(scene_handle) = &slot.scene_handle else {
        return VisualAssetLoadState::Missing;
    };

    match asset_server.load_state(scene_handle) {
        LoadState::NotLoaded => VisualAssetLoadState::Queued,
        LoadState::Loading => VisualAssetLoadState::Loading,
        LoadState::Loaded => VisualAssetLoadState::Loaded,
        LoadState::Failed(_) => VisualAssetLoadState::Failed,
    }
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
    let streaming_lod = scene.route.streaming_lod_stats(transform.translation);
    let lod_visuals = scene.stream_diagnostics.counts;
    let asset_metrics = scene.asset_diagnostics.metrics;
    let camera_yaw = scene.camera_control.orbit.yaw_degrees();
    let camera_pitch_offset = scene.camera_control.orbit.pitch_degrees();
    let mouse_lock = if scene.mouse_look.captured {
        "locked"
    } else {
        "free"
    };
    let objective_step =
        (scene.route_objectives.completed_count + 1).min(scene.route_objectives.total_count);
    let objective_state = if scene.route_objectives.complete {
        "done"
    } else {
        "go"
    };

    **text = format!(
        "frame {:>4.1} ms\nmode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\ntarget {:>5.1} m {}\nobjective {}/{} {} {:>5.1} m {}\ncamera pitch {:>5.1} deg\ncamera distance {:>5.1} m\ncamera frame {:>5.1} deg\ncamera motion {:>4.1} m / {:>4.1} deg\ncamera orbit {:>5.1} deg\ncamera obstruction {:>4.1} m / {}\nmouse yaw {:>5.1} deg\nmouse pitch {:>5.1} deg\nmouse {}\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\npower ups visible/collected/active {} / {} / {}\nvisual assets {} gltf {} ready {} placeholders {} missing {} stream {}\nasset load queued/loading/loaded/failed {} / {} / {} / {}\nasset residency always/window/near/far/weather {} / {} / {} / {} / {}\nvisual wind fields {} / {}\nlift fields {} / {}\nsky islands {}\nstream chunk [{}, {}] active {} / {}\nlod near/mid/far {} / {} / {}\nstream terrain visible/hidden {} / {}\nstream impostor visible/hidden {} / {}\nlod detail visible/hidden {} / {}\nresident island visuals {}\nstream entity changes {} max {} total {}\nroute beacons {}\nlaunch cooldown {:>4.1}s\nlaunch ready {}\ndebug visuals {} (F1)\nWASD camera-relative  Click mouse lock  Esc release  Space glider  E launch  Shift dive",
        frame_ms(time.delta_secs()),
        controller.mode.label(),
        velocity.0.length(),
        transform.translation.y,
        target_distance,
        if on_target { "landed" } else { "out" },
        objective_step,
        scene.route_objectives.total_count,
        scene.route_objectives.current_label,
        scene.route_objectives.current_distance_m,
        objective_state,
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
        scene.power_ups.visible_count(),
        scene.power_ups.collected_count(),
        scene.power_ups.active_effects(),
        asset_metrics.slot_count,
        asset_metrics.gltf_scene_slot_count,
        asset_metrics.ready_slot_count,
        asset_metrics.placeholder_slot_count,
        asset_metrics.missing_slot_count,
        asset_metrics.streaming_slot_count,
        asset_metrics.queued_scene_count,
        asset_metrics.loading_scene_count,
        asset_metrics.loaded_scene_count,
        asset_metrics.failed_scene_count,
        asset_metrics.always_slot_count,
        asset_metrics.stream_window_slot_count,
        asset_metrics.near_lod_slot_count,
        asset_metrics.far_lod_slot_count,
        asset_metrics.weather_slot_count,
        visible_wind_fields,
        wind_field_count,
        active_lift_fields,
        lift_field_count,
        scene.route.islands().len(),
        streaming_lod.player_chunk.x,
        streaming_lod.player_chunk.z,
        streaming_lod.active_island_count,
        streaming_lod.active_chunk_count,
        streaming_lod.near_lod_islands,
        streaming_lod.mid_lod_islands,
        streaming_lod.far_lod_islands,
        lod_visuals.visible_terrain_count,
        lod_visuals.hidden_terrain_count,
        lod_visuals.visible_impostor_count,
        lod_visuals.hidden_impostor_count,
        lod_visuals.visible_detail_count,
        lod_visuals.hidden_detail_count,
        lod_visuals.resident_count(),
        scene.stream_diagnostics.visibility_changes_this_frame,
        scene.stream_diagnostics.max_visibility_changes_per_frame,
        scene.stream_diagnostics.total_visibility_changes,
        lod_visuals.visible_beacon_count,
        controller.launch_cooldown_remaining,
        if controller.launch_available {
            "yes"
        } else {
            "no"
        },
        if visuals.enabled { "on" } else { "off" }
    );
}

fn collect_eval_frame_time(time: Res<Time>, mut run: ResMut<EvalRun>) {
    if !run.finalized && run.frame >= EVAL_FRAME_TIME_WARMUP_FRAMES {
        run.accumulator
            .observe_frame_time_ms(frame_ms(time.delta_secs()));
    }
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
    let readable_lift_fields = readable_lift_fields_at(
        transform.translation,
        scene.lift_fields.iter().copied(),
        scene.wind_fields.iter().copied(),
    );
    let scenario_target = run.scenario.target_island_name;
    let target_distance_m = scene
        .route
        .target_distance_to(transform.translation, scenario_target);
    let on_landing_target = scene.route.on_landing_target_named(
        transform.translation,
        controller.mode,
        scenario_target,
    );
    let objective = EvalObjectiveProgress::new(
        scene.route_objectives.completed_count,
        scene.route_objectives.total_count,
        scene.route_objectives.current_label,
        scene.route_objectives.current_distance_m,
        scene.route_objectives.complete,
    );
    let streaming_lod = scene.route.streaming_lod_stats(transform.translation);
    let lod_visuals = scene.stream_diagnostics.counts;
    let asset_metrics = scene.asset_diagnostics.metrics;
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
        readable_lift_fields,
        scene.lift_fields.iter().count(),
        target_distance_m,
        on_landing_target,
        objective,
        scene.route.islands().len(),
        streaming_lod.active_chunk_count,
        streaming_lod.active_island_count,
        streaming_lod.near_lod_islands,
        streaming_lod.mid_lod_islands,
        streaming_lod.far_lod_islands,
        lod_visuals.visible_terrain_count,
        lod_visuals.hidden_terrain_count,
        lod_visuals.visible_impostor_count,
        lod_visuals.hidden_impostor_count,
        lod_visuals.visible_detail_count,
        lod_visuals.hidden_detail_count,
        lod_visuals.visible_beacon_count,
        scene.weather_clouds.iter().count(),
        lod_visuals.resident_count(),
        scene.stream_diagnostics.visibility_changes_this_frame,
        scene.stream_diagnostics.max_visibility_changes_per_frame,
        scene.stream_diagnostics.total_visibility_changes,
        scene.all_entities.iter().count(),
        asset_metrics.slot_count,
        asset_metrics.gltf_scene_slot_count,
        asset_metrics.ready_slot_count,
        asset_metrics.placeholder_slot_count,
        asset_metrics.streaming_slot_count,
        asset_metrics.missing_slot_count,
        asset_metrics.queued_scene_count,
        asset_metrics.loading_scene_count,
        asset_metrics.loaded_scene_count,
        asset_metrics.failed_scene_count,
        asset_metrics.always_slot_count,
        asset_metrics.stream_window_slot_count,
        asset_metrics.near_lod_slot_count,
        asset_metrics.far_lod_slot_count,
        asset_metrics.weather_slot_count,
        AERIAL_POWER_UP_ROUTE.len(),
        scene.power_ups.visible_count(),
        scene.power_ups.collected_count(),
        scene.power_ups.active_effects(),
        scene.power_ups.total_activations,
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
        if let Some(exit_success) = run.pending_screenshot_exit_success {
            if run
                .screenshot_path
                .as_deref()
                .is_some_and(screenshot_file_ready)
            {
                run.pending_screenshot_exit_success = None;
                let exit = if exit_success {
                    AppExit::Success
                } else {
                    AppExit::error()
                };
                app_exit.write(exit);
                return;
            }

            run.screenshot_wait_frames += 1;
            if run.screenshot_wait_frames > EVAL_SCREENSHOT_TIMEOUT_FRAMES {
                run.pending_screenshot_exit_success = None;
                eprintln!(
                    "eval screenshot did not finish within {} frames",
                    EVAL_SCREENSHOT_TIMEOUT_FRAMES
                );
                app_exit.write(AppExit::error());
            }
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

    if let Some(screenshot_path) = run.screenshot_path.clone() {
        run.screenshot_wait_frames = 0;
        run.pending_screenshot_exit_success = Some(passed);
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>| {
                save_to_disk(screenshot_path.clone())(captured);
            },
        );
    } else if passed {
        app_exit.write(AppExit::Success);
    } else {
        app_exit.write(AppExit::error());
    }
}

fn screenshot_file_ready(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if metadata.len() == 0 {
        return false;
    }

    image::ImageReader::open(path)
        .and_then(|reader| reader.with_guessed_format())
        .ok()
        .and_then(|reader| reader.decode().ok())
        .is_some_and(|image| image.width() > 0 && image.height() > 0)
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
