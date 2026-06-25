mod authored_assets;
mod content_export;
mod debug_visuals;
mod eval_runtime;
mod generated_content;
use authored_assets::*;
use bevy::camera::{CameraOutputMode, ClearColorConfig, Exposure};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseMotion;
use bevy::light::{
    AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, DirectionalLightShadowMap,
    VolumetricFog, VolumetricLight,
};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::window::{CompositeAlphaMode, CursorGrabMode, CursorOptions, PrimaryWindow};
#[cfg(test)]
use content_export::mesh_uv0;
use content_export::{
    export_terrain_inspection, export_visual_content_inspection, terrain_export_json_number,
    terrain_export_json_string, terrain_export_json_vec3,
};
use debug_visuals::*;
use eval_runtime::{
    CliAction, EvalCheckpointCapture, EvalMovementBasis, EvalOptions, EvalRun, path_string, usage,
};
#[cfg(test)]
use eval_runtime::{parse_cli_args, remove_existing_dir};
use generated_content::*;
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend, wing_airflow_strength,
};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::camera::{
    CameraControlState, CameraControlTuning, CameraInput, CameraObstruction, FollowCamera,
    FollowCameraState, apply_camera_input, avoid_camera_obstructions, camera_distance,
    camera_orbit_alignment_degrees, camera_pitch_degrees, camera_surface_clearance,
    camera_target_angle_degrees, camera_view_yaw_degrees, lift_camera_above_floor,
    movement_input_stable_follow_direction, step_camera_with_direction,
    update_follow_direction_state,
};
use nau_engine::diagnostics::frame_ms;
use nau_engine::environment::{
    AERIAL_POWER_UP_ROUTE, AerialPowerUp, GAMEPLAY_LIFT_ROUTE, LiftField, LiftRouteNode, WindField,
    active_lift_fields_at, apply_aerial_power_up, apply_lift_fields, readable_lift_fields_at,
    visible_fields_at, wind_sway_motion,
};
#[cfg(test)]
use nau_engine::eval::scenario_named;
use nau_engine::eval::{
    EvalMovementMetrics, EvalObjectiveProgress, EvalSample, EvalScenario, scripted_camera_input,
    scripted_input,
};
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning, Velocity,
    body_forward, body_roll_degrees, body_yaw_error_degrees, desired_heading_alignment_speed,
    desired_planar_movement_direction, face_flight_direction, lateral_response_speed, step_flight,
};
use nau_engine::world::{
    LodBand, RouteObjectiveKind, START_POSITION, SkyIsland, SkyRoute, StreamActivation,
    TERRAIN_VISUAL_FOOTING_OFFSET_M, is_recovery_branch_island,
};
#[cfg(test)]
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

const PLAYER_START: Vec3 = START_POSITION;
const WORLD_RADIUS: f32 = 920.0;
const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;
const EVAL_FRAME_TIME_WARMUP_FRAMES: u32 = 5;
const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;
const ATTACHED_PLAYER_VISUAL_OFFSET_Y: f32 = -TERRAIN_VISUAL_FOOTING_OFFSET_M;
const INITIAL_SKY_CLEAR_COLOR: Color = Color::srgb(0.50, 0.68, 0.92);
fn main() -> AppExit {
    let cli = match CliAction::from_env() {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{}", usage());
            return AppExit::from_code(2);
        }
    };

    let eval = match cli {
        CliAction::Run { eval } => eval,
        CliAction::ExportTerrain { output_dir } => {
            return match export_terrain_inspection(&output_dir) {
                Ok(report) => {
                    println!(
                        "exported {} islands / {} meshes to {}",
                        report.island_count,
                        report.mesh_count,
                        path_string(&report.manifest_path)
                    );
                    AppExit::Success
                }
                Err(error) => {
                    eprintln!("terrain export failed: {error}");
                    AppExit::from_code(1)
                }
            };
        }
        CliAction::ExportVisualContent { output_dir } => {
            return match export_visual_content_inspection(&output_dir) {
                Ok(report) => {
                    println!(
                        "exported {} visual-content meshes to {}",
                        report.mesh_count,
                        path_string(&report.manifest_path)
                    );
                    AppExit::Success
                }
                Err(error) => {
                    eprintln!("visual content export failed: {error}");
                    AppExit::from_code(1)
                }
            };
        }
        CliAction::Help => {
            println!("{}", usage());
            return AppExit::Success;
        }
    };
    let screenshot_eval = eval
        .as_deref()
        .is_some_and(|options| options.capture_screenshot);

    let mut app = App::new();
    app.insert_resource(ClearColor(INITIAL_SKY_CLEAR_COLOR))
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
                link_ready_authored_animations,
                update_authored_player_animation,
                update_glider_airflow_trails,
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
                update_wind_responsive_visuals,
                update_updraft_guides,
                update_updraft_ribbons,
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
            .insert_resource(EvalMovementBasis::default())
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

fn authored_player_scene_transform() -> Transform {
    Transform::from_xyz(0.0, ATTACHED_PLAYER_VISUAL_OFFSET_Y, 0.0)
}

fn authored_glider_scene_transform() -> Transform {
    Transform::from_xyz(0.0, 1.35 + ATTACHED_PLAYER_VISUAL_OFFSET_Y, -0.45)
}

fn grounded_visual_foot_gap_m(player_y: f32, ground_floor_y: f32, mode: FlightMode) -> f32 {
    if mode != FlightMode::Grounded {
        return 0.0;
    }

    let visual_foot_y = player_y + authored_player_scene_transform().translation.y;
    let terrain_visual_y = ground_floor_y - TERRAIN_VISUAL_FOOTING_OFFSET_M;
    visual_foot_y - terrain_visual_y
}

#[derive(Component)]
pub(crate) struct Player;

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

#[derive(Clone, Copy, Debug)]
struct WindVisualMotion {
    phase: f32,
    amplitude_m: f32,
    bend_radians: f32,
    gust_speed: f32,
    wind_direction: Vec3,
}

#[derive(Component, Clone, Copy, Debug)]
struct WindResponsiveVisual {
    base_translation: Vec3,
    base_rotation: Quat,
    base_scale: Vec3,
    motion: WindVisualMotion,
}

#[derive(Component, Clone, Copy, Debug)]
struct GliderAirflowTrail {
    side: Side,
    base_translation: Vec3,
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

#[derive(Component, Clone, Copy, Debug)]
struct UpdraftRibbon {
    spin_speed: f32,
    base_rotation: Quat,
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

    fn hidden_count(self) -> usize {
        self.hidden_terrain_count + self.hidden_detail_count + self.hidden_impostor_count
    }

    fn catalog_count(self) -> usize {
        self.resident_count() + self.hidden_count()
    }

    fn resident_fraction(self) -> f32 {
        self.resident_count() as f32 / self.catalog_count().max(1) as f32
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
struct IslandStreamDiagnostics {
    counts: IslandLodVisualCounts,
    visibility_changes_this_frame: usize,
    max_visibility_changes_per_frame: usize,
    total_visibility_changes: usize,
    spawned_visuals_this_frame: usize,
    despawned_visuals_this_frame: usize,
    max_spawned_visuals_per_frame: usize,
    max_despawned_visuals_per_frame: usize,
    total_spawned_visuals: usize,
    total_despawned_visuals: usize,
    initialized: bool,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
struct IslandContentDiagnostics {
    island_terrain_surface_count: usize,
    min_island_terrain_mesh_vertices: usize,
    min_island_terrain_color_bands: usize,
    min_island_terrain_material_weight_bands: usize,
    min_island_terrain_material_channels: usize,
    min_island_terrain_material_regions: usize,
    min_island_terrain_texture_detail_bands: usize,
    min_island_terrain_relief_range_cm: usize,
    min_island_cliff_color_bands: usize,
    min_island_impostor_mesh_vertices: usize,
    min_island_impostor_color_bands: usize,
    procedural_island_body_count: usize,
    primitive_island_body_count: usize,
    min_island_body_silhouette_segments: usize,
    max_island_body_silhouette_segments: usize,
    total_island_body_silhouette_segments: usize,
    min_island_body_mesh_vertices: usize,
    max_island_body_mesh_vertices: usize,
    generated_ground_cover_patch_count: usize,
    min_ground_cover_blade_count: usize,
    min_ground_cover_mesh_vertices: usize,
    generated_tree_trunk_count: usize,
    generated_tree_canopy_count: usize,
    min_tree_trunk_mesh_vertices: usize,
    min_tree_canopy_mesh_vertices: usize,
    detail_biome_palette_mask: u32,
    generated_rock_count: usize,
    min_rock_mesh_vertices: usize,
    generated_weather_cloud_count: usize,
    generated_weather_cloud_bank_count: usize,
    min_weather_cloud_bank_depth_cm: usize,
    min_weather_cloud_lobe_count: usize,
    max_weather_cloud_lobe_count: usize,
    min_weather_cloud_mesh_vertices: usize,
}

impl IslandContentDiagnostics {
    fn record_island_terrain_surface(
        &mut self,
        mesh_vertices: usize,
        color_bands: usize,
        material_weight_bands: usize,
        material_channels: usize,
        material_regions: usize,
        relief_range_m: f32,
    ) {
        let relief_range_cm = (relief_range_m.max(0.0) * 100.0).round() as usize;
        if self.island_terrain_surface_count == 0 {
            self.min_island_terrain_mesh_vertices = mesh_vertices;
            self.min_island_terrain_color_bands = color_bands;
            self.min_island_terrain_material_weight_bands = material_weight_bands;
            self.min_island_terrain_material_channels = material_channels;
            self.min_island_terrain_material_regions = material_regions;
            self.min_island_terrain_relief_range_cm = relief_range_cm;
        } else {
            self.min_island_terrain_mesh_vertices =
                self.min_island_terrain_mesh_vertices.min(mesh_vertices);
            self.min_island_terrain_color_bands =
                self.min_island_terrain_color_bands.min(color_bands);
            self.min_island_terrain_material_weight_bands = self
                .min_island_terrain_material_weight_bands
                .min(material_weight_bands);
            self.min_island_terrain_material_channels = self
                .min_island_terrain_material_channels
                .min(material_channels);
            self.min_island_terrain_material_regions = self
                .min_island_terrain_material_regions
                .min(material_regions);
            self.min_island_terrain_relief_range_cm =
                self.min_island_terrain_relief_range_cm.min(relief_range_cm);
        }
        self.island_terrain_surface_count += 1;
    }

    fn record_terrain_material_texture_detail(&mut self, detail_bands: usize) {
        if self.min_island_terrain_texture_detail_bands == 0 {
            self.min_island_terrain_texture_detail_bands = detail_bands;
        } else {
            self.min_island_terrain_texture_detail_bands = self
                .min_island_terrain_texture_detail_bands
                .min(detail_bands);
        }
    }

    fn record_island_cliff_detail(&mut self, color_bands: usize) {
        if self.min_island_cliff_color_bands == 0 {
            self.min_island_cliff_color_bands = color_bands;
        } else {
            self.min_island_cliff_color_bands = self.min_island_cliff_color_bands.min(color_bands);
        }
    }

    fn min_island_terrain_relief_range_m(self) -> f32 {
        self.min_island_terrain_relief_range_cm as f32 / 100.0
    }

    fn record_island_impostor(&mut self, mesh_vertices: usize, color_bands: usize) {
        if self.min_island_impostor_mesh_vertices == 0 {
            self.min_island_impostor_mesh_vertices = mesh_vertices;
            self.min_island_impostor_color_bands = color_bands;
        } else {
            self.min_island_impostor_mesh_vertices =
                self.min_island_impostor_mesh_vertices.min(mesh_vertices);
            self.min_island_impostor_color_bands =
                self.min_island_impostor_color_bands.min(color_bands);
        }
    }

    fn record_procedural_island_body(&mut self, silhouette_segments: usize, mesh_vertices: usize) {
        if self.procedural_island_body_count == 0 {
            self.min_island_body_silhouette_segments = silhouette_segments;
            self.min_island_body_mesh_vertices = mesh_vertices;
        } else {
            self.min_island_body_silhouette_segments = self
                .min_island_body_silhouette_segments
                .min(silhouette_segments);
            self.min_island_body_mesh_vertices =
                self.min_island_body_mesh_vertices.min(mesh_vertices);
        }
        self.procedural_island_body_count += 1;
        self.max_island_body_silhouette_segments = self
            .max_island_body_silhouette_segments
            .max(silhouette_segments);
        self.total_island_body_silhouette_segments += silhouette_segments;
        self.max_island_body_mesh_vertices = self.max_island_body_mesh_vertices.max(mesh_vertices);
    }

    fn average_island_body_silhouette_segments(self) -> f32 {
        if self.procedural_island_body_count == 0 {
            0.0
        } else {
            self.total_island_body_silhouette_segments as f32
                / self.procedural_island_body_count as f32
        }
    }

    fn record_generated_ground_cover(
        &mut self,
        patch_count: usize,
        blade_count: usize,
        mesh_vertices: usize,
    ) {
        if self.generated_ground_cover_patch_count == 0 {
            self.min_ground_cover_blade_count = blade_count;
            self.min_ground_cover_mesh_vertices = mesh_vertices;
        } else {
            self.min_ground_cover_blade_count = self.min_ground_cover_blade_count.min(blade_count);
            self.min_ground_cover_mesh_vertices =
                self.min_ground_cover_mesh_vertices.min(mesh_vertices);
        }
        self.generated_ground_cover_patch_count += patch_count;
    }

    fn record_generated_tree_trunk(&mut self, mesh_vertices: usize) {
        if self.generated_tree_trunk_count == 0 {
            self.min_tree_trunk_mesh_vertices = mesh_vertices;
        } else {
            self.min_tree_trunk_mesh_vertices =
                self.min_tree_trunk_mesh_vertices.min(mesh_vertices);
        }
        self.generated_tree_trunk_count += 1;
    }

    fn record_generated_tree_canopy(&mut self, mesh_vertices: usize) {
        if self.generated_tree_canopy_count == 0 {
            self.min_tree_canopy_mesh_vertices = mesh_vertices;
        } else {
            self.min_tree_canopy_mesh_vertices =
                self.min_tree_canopy_mesh_vertices.min(mesh_vertices);
        }
        self.generated_tree_canopy_count += 1;
    }

    fn record_detail_biome_palette(&mut self, palette_index: usize) {
        self.detail_biome_palette_mask |= 1_u32 << (palette_index % TERRAIN_BIOME_PALETTE_COUNT);
    }

    fn detail_biome_palette_count(self) -> usize {
        self.detail_biome_palette_mask.count_ones() as usize
    }

    fn record_generated_rock(&mut self, mesh_vertices: usize) {
        if self.generated_rock_count == 0 {
            self.min_rock_mesh_vertices = mesh_vertices;
        } else {
            self.min_rock_mesh_vertices = self.min_rock_mesh_vertices.min(mesh_vertices);
        }
        self.generated_rock_count += 1;
    }

    fn record_generated_weather_cloud(
        &mut self,
        lobe_count: usize,
        mesh_vertices: usize,
        depth_m: f32,
        is_bank: bool,
    ) {
        if self.generated_weather_cloud_count == 0 {
            self.min_weather_cloud_lobe_count = lobe_count;
            self.min_weather_cloud_mesh_vertices = mesh_vertices;
        } else {
            self.min_weather_cloud_lobe_count = self.min_weather_cloud_lobe_count.min(lobe_count);
            self.min_weather_cloud_mesh_vertices =
                self.min_weather_cloud_mesh_vertices.min(mesh_vertices);
        }
        self.generated_weather_cloud_count += 1;
        self.max_weather_cloud_lobe_count = self.max_weather_cloud_lobe_count.max(lobe_count);
        if is_bank {
            let depth_cm = (depth_m.max(0.0) * 100.0).round() as usize;
            if self.generated_weather_cloud_bank_count == 0 {
                self.min_weather_cloud_bank_depth_cm = depth_cm;
            } else {
                self.min_weather_cloud_bank_depth_cm =
                    self.min_weather_cloud_bank_depth_cm.min(depth_cm);
            }
            self.generated_weather_cloud_bank_count += 1;
        }
    }

    fn min_weather_cloud_bank_depth_m(self) -> f32 {
        self.min_weather_cloud_bank_depth_cm as f32 / 100.0
    }
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
    wind_motion: Option<WindVisualMotion>,
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
    follow_direction: Vec3,
    follow_direction_error_degrees: f32,
    obstruction_adjustment_m: f32,
    obstruction_hits: usize,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
struct MouseLookState {
    captured: bool,
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
    content_diagnostics: Res<'w, IslandContentDiagnostics>,
    asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    route_objectives: Res<'w, RouteObjectiveTracker>,
    power_ups: Res<'w, PowerUpCollectionState>,
    wind_fields: Query<'w, 's, &'static WindField>,
    lift_fields: Query<'w, 's, &'static LiftField>,
    wind_responsive_visuals: Query<'w, 's, (&'static WindResponsiveVisual, &'static Transform)>,
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
    camera_projection:
        Query<'w, 's, (&'static Camera, &'static GlobalTransform), CameraFollowFilter>,
    camera_diagnostics: Res<'w, CameraDiagnostics>,
    stream_diagnostics: Res<'w, IslandStreamDiagnostics>,
    content_diagnostics: Res<'w, IslandContentDiagnostics>,
    asset_diagnostics: Res<'w, VisualAssetDiagnostics>,
    route_objectives: Res<'w, RouteObjectiveTracker>,
    power_ups: Res<'w, PowerUpCollectionState>,
    wind_fields: Query<'w, 's, &'static WindField>,
    lift_fields: Query<'w, 's, &'static LiftField>,
    weather_clouds: Query<'w, 's, &'static Transform, With<WeatherDrift>>,
    wind_responsive_visuals: Query<'w, 's, (&'static WindResponsiveVisual, &'static Transform)>,
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

type CameraFollowFilter = (With<Camera3d>, Without<Player>);
type GeneratedPlayerPlaceholderFilter = (
    With<GeneratedPlayerPlaceholder>,
    Without<CharacterPart>,
    Without<AuthoredVisualScene>,
);
type GeneratedCharacterPartAnimationFilter = (Without<AuthoredVisualScene>, Without<Player>);

fn setup(
    mut commands: Commands,
    route: Res<SkyRoute>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    asset_server: Res<AssetServer>,
) {
    let mut visual_asset_registry = prepare_visual_asset_registry(&asset_server);
    let player_scene_handle = visual_asset_registry.scene_handle(VisualAssetKind::PlayerCharacter);
    let glider_scene_handle = visual_asset_registry.scene_handle(VisualAssetKind::Glider);
    let authored_world_fixture_scene_handles =
        authored_world_fixture_scene_handles(&visual_asset_registry);
    let mut player_scene_entity = None;
    let mut glider_scene_entity = None;
    let mut authored_world_fixture_scene_entities =
        Vec::with_capacity(authored_world_fixture_scene_handles.len());

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
    let glider_airflow_material = glider_airflow_material(&mut materials);
    let (island_grass_material, island_grass_texture_detail_bands) = terrain_surface_material(
        &mut images,
        &mut materials,
        [54, 128, 70, 255],
        [28, 92, 48, 255],
        [128, 174, 78, 255],
        17,
        0.94,
        0.2,
    );
    let (island_meadow_material, island_meadow_texture_detail_bands) = terrain_surface_material(
        &mut images,
        &mut materials,
        [96, 138, 70, 255],
        [56, 104, 54, 255],
        [166, 172, 90, 255],
        19,
        0.92,
        0.21,
    );
    let (island_clay_material, island_clay_texture_detail_bands) = terrain_surface_material(
        &mut images,
        &mut materials,
        [126, 104, 76, 255],
        [80, 70, 60, 255],
        [162, 138, 96, 255],
        23,
        0.98,
        0.18,
    );
    let (island_alpine_material, island_alpine_texture_detail_bands) = terrain_surface_material(
        &mut images,
        &mut materials,
        [52, 110, 118, 255],
        [30, 80, 94, 255],
        [142, 176, 164, 255],
        29,
        0.9,
        0.22,
    );
    let (island_highland_material, island_highland_texture_detail_bands) = terrain_surface_material(
        &mut images,
        &mut materials,
        [132, 132, 92, 255],
        [86, 96, 70, 255],
        [178, 166, 112, 255],
        31,
        0.94,
        0.2,
    );
    let (target_grass_material, target_grass_texture_detail_bands) = terrain_surface_material(
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
    let biome_detail_material_sets = (0..TERRAIN_BIOME_PALETTE_COUNT)
        .map(|index| biome_detail_materials(&mut images, &mut materials, index))
        .collect::<Vec<_>>();
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
    let updraft_ribbon_material = updraft_ribbon_material(&mut materials);
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
    let glider_airflow_mesh = meshes.add(glider_airflow_trail_mesh());

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
    let mut island_content_diagnostics = IslandContentDiagnostics::default();
    let terrain_texture_detail_bands = [
        island_grass_texture_detail_bands,
        island_meadow_texture_detail_bands,
        island_clay_texture_detail_bands,
        island_alpine_texture_detail_bands,
        island_highland_texture_detail_bands,
        target_grass_texture_detail_bands,
    ]
    .into_iter()
    .min()
    .unwrap_or(0);
    island_content_diagnostics.record_terrain_material_texture_detail(terrain_texture_detail_bands);

    for (index, island) in route.islands().iter().enumerate() {
        let top_material = if island.is_target {
            target_grass_material.clone()
        } else {
            match index % TERRAIN_BIOME_PALETTE_COUNT {
                0 => island_grass_material.clone(),
                1 => island_meadow_material.clone(),
                2 => island_clay_material.clone(),
                3 => island_alpine_material.clone(),
                _ => island_highland_material.clone(),
            }
        };

        queue_sky_island(
            &mut island_visual_catalog.entries,
            &mut island_content_diagnostics,
            &mut meshes,
            top_material,
            island_rock_material.clone(),
            island_under_material.clone(),
            target_marker_material.clone(),
            updraft_marker_material.clone(),
            biome_detail_material_sets[index % TERRAIN_BIOME_PALETTE_COUNT].clone(),
            flower_material.clone(),
            water_material.clone(),
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
            updraft_ribbon_material.clone(),
            updraft_marker_material.clone(),
            lift,
        );
    }
    spawn_power_up_guides(&mut commands, &mut meshes, power_up_material);

    spawn_weather_layers(
        &mut commands,
        &mut island_content_diagnostics,
        &mut meshes,
        cloud_material,
        cloud_veil_material,
        route.islands(),
    );
    commands.insert_resource(island_content_diagnostics);

    for (kind, label, scene_handle) in authored_world_fixture_scene_handles {
        let mut scene = commands.spawn((
            SceneRoot(scene_handle),
            authored_world_fixture_transform(kind, &route),
            Visibility::Inherited,
            AuthoredVisualScene {
                kind,
                role: AuthoredVisualSceneRole::WorldFixture,
            },
            VisibleAuthoredWorldFixture { kind },
            Name::new(format!("visible authored {label} fixture scene")),
        ));
        scene.observe(mark_authored_scene_ready);
        authored_world_fixture_scene_entities.push((kind, scene.id()));
    }

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
            if let Some(scene_handle) = player_scene_handle.clone() {
                let mut scene = parent.spawn((
                    SceneRoot(scene_handle),
                    authored_player_scene_transform(),
                    Visibility::Hidden,
                    AuthoredVisualScene {
                        kind: VisualAssetKind::PlayerCharacter,
                        role: AuthoredVisualSceneRole::PlayerRuntime,
                    },
                    Name::new("authored player character scene"),
                ));
                scene.observe(mark_authored_scene_ready);
                player_scene_entity = Some(scene.id());
            }

            if let Some(scene_handle) = glider_scene_handle.clone() {
                let mut scene = parent.spawn((
                    SceneRoot(scene_handle),
                    authored_glider_scene_transform(),
                    Visibility::Hidden,
                    AuthoredVisualScene {
                        kind: VisualAssetKind::Glider,
                        role: AuthoredVisualSceneRole::GliderRuntime,
                    },
                    Name::new("authored glider scene"),
                ));
                scene.observe(mark_authored_scene_ready);
                glider_scene_entity = Some(scene.id());
            }

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

                let trail_translation = Vec3::new(sign * 1.74, 1.38, 0.86);
                let trail_rotation =
                    Quat::from_rotation_z(sign * 0.08) * Quat::from_rotation_x(0.04);
                parent.spawn((
                    Mesh3d(glider_airflow_mesh.clone()),
                    MeshMaterial3d(glider_airflow_material.clone()),
                    Transform {
                        translation: trail_translation,
                        rotation: trail_rotation,
                        scale: Vec3::new(0.35, 1.0, 0.05),
                    },
                    Visibility::Hidden,
                    GliderAirflowTrail {
                        side,
                        base_translation: trail_translation,
                        base_rotation: trail_rotation,
                    },
                ));
            }

            parent.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.18, 0.18, 0.38))),
                MeshMaterial3d(accent_material),
                Transform::from_xyz(0.0, 1.15, -0.28),
                Visibility::Inherited,
                GeneratedPlayerPlaceholder,
            ));
        });

    if let Some(entity) = player_scene_entity {
        visual_asset_registry.mark_scene_spawned(VisualAssetKind::PlayerCharacter, entity);
    }
    if let Some(entity) = glider_scene_entity {
        visual_asset_registry.mark_scene_spawned(VisualAssetKind::Glider, entity);
    }
    for (kind, entity) in authored_world_fixture_scene_entities {
        visual_asset_registry.mark_scene_spawned(kind, entity);
    }
    commands.insert_resource(visual_asset_registry);

    let follow_camera = FollowCamera::default();
    let initial_camera_direction = Vec3::NEG_Z;
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(INITIAL_SKY_CLEAR_COLOR),
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::REPLACE),
                clear_color: ClearColorConfig::Custom(INITIAL_SKY_CLEAR_COLOR),
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

fn spawn_updraft_guide(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    column_material: Handle<StandardMaterial>,
    ribbon_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    lift: LiftRouteNode,
) {
    let radius = lift.half_extents.x.min(lift.half_extents.z);
    let height = lift.half_extents.y * 2.0;
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(radius * 0.34, height))),
        MeshMaterial3d(column_material),
        Transform::from_translation(lift.center),
        Name::new(format!("{} atmospheric lift haze", lift.name)),
    ));

    for ribbon_index in 0..3 {
        let phase = ribbon_index as f32 / 3.0 * std::f32::consts::TAU;
        let base_rotation = Quat::from_rotation_y(phase * 0.35);
        commands.spawn((
            Mesh3d(meshes.add(updraft_ribbon_mesh(radius, height, phase))),
            MeshMaterial3d(ribbon_material.clone()),
            Transform {
                translation: lift.center,
                rotation: base_rotation,
                ..default()
            },
            UpdraftRibbon {
                spin_speed: 0.035 + ribbon_index as f32 * 0.012,
                base_rotation,
            },
            Name::new(format!("{} spiral airflow ribbon", lift.name)),
        ));
    }

    let marker_mesh = meshes.add(Sphere::new(0.32));
    let ring_radius = radius * 0.5;
    let ring_levels = [-0.78, -0.34, 0.1, 0.54, 0.9];
    let markers_per_ring = 7;

    for (level_index, level) in ring_levels.into_iter().enumerate() {
        for marker_index in 0..markers_per_ring {
            let phase = marker_index as f32 / markers_per_ring as f32 * std::f32::consts::TAU
                + level_index as f32 * 0.46;
            let guide = UpdraftGuide {
                center: lift.center,
                radius: ring_radius,
                height_offset: level * lift.half_extents.y,
                phase,
                angular_speed: 0.26 + level_index as f32 * 0.035,
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
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    cloud_material: Handle<StandardMaterial>,
    cloud_veil_material: Handle<StandardMaterial>,
    islands: &[SkyIsland],
) {
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
            3.8 + (index % 3) as f32 * 0.55,
            island.half_extents.y * 0.26 + 8.0,
        );
        let cloud_mesh_data = cloud_cluster_mesh(2_000 + index as u32 * 37, CLOUD_BANK_LOBES);
        let cloud_depth_m = mesh_y_range(&cloud_mesh_data) * scale.y;
        content_diagnostics.record_generated_weather_cloud(
            CLOUD_BANK_LOBES,
            cloud_mesh_data.count_vertices(),
            cloud_depth_m,
            true,
        );
        let cloud_mesh = meshes.add(cloud_mesh_data);

        commands.spawn((
            Mesh3d(cloud_mesh),
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
            let veil_anchor = island.center
                + Vec3::new(
                    (phase * 1.3).cos() * island.half_extents.x,
                    78.0 + (index % 3) as f32 * 8.0,
                    (phase * 1.9).sin() * island.half_extents.y,
                );
            for puff_index in 0..3 {
                let puff_phase = phase + puff_index as f32 * 1.17;
                let layer_offset = Vec3::new(
                    (puff_phase * 0.9).cos() * (14.0 + puff_index as f32 * 5.0),
                    (puff_index as f32 - 1.0) * 1.7,
                    (puff_phase * 1.1).sin() * (9.0 + puff_index as f32 * 3.5),
                );
                let veil_origin = veil_anchor + layer_offset;
                let veil_rotation = Quat::from_euler(EulerRot::XYZ, -0.04, puff_phase * 0.27, 0.06);
                let veil_mesh_data = cloud_cluster_mesh(
                    3_000 + index as u32 * 53 + puff_index as u32 * 11,
                    CLOUD_VEIL_LOBES,
                );
                let veil_scale = Vec3::new(
                    island.half_extents.x * 0.36 + 14.0 + puff_index as f32 * 4.0,
                    0.52 + puff_index as f32 * 0.12,
                    island.half_extents.y * 0.13 + 6.0 + puff_index as f32 * 1.8,
                );
                let veil_depth_m = mesh_y_range(&veil_mesh_data) * veil_scale.y;
                content_diagnostics.record_generated_weather_cloud(
                    CLOUD_VEIL_LOBES,
                    veil_mesh_data.count_vertices(),
                    veil_depth_m,
                    false,
                );
                let veil_mesh = meshes.add(veil_mesh_data);

                commands.spawn((
                    Mesh3d(veil_mesh),
                    MeshMaterial3d(cloud_veil_material.clone()),
                    Transform {
                        translation: veil_origin,
                        scale: veil_scale,
                        rotation: veil_rotation,
                    },
                    WeatherDrift {
                        origin: veil_origin,
                        axis: Vec3::new(0.74, 0.0, -0.18).normalize(),
                        amplitude: 5.0 + (index % 5) as f32 * 0.7 + puff_index as f32 * 0.6,
                        bob: 0.22 + puff_index as f32 * 0.05,
                        speed: 0.021 + (index % 3) as f32 * 0.005,
                        phase: puff_phase,
                        spin_speed: 0.003 + puff_index as f32 * 0.001,
                        base_rotation: veil_rotation,
                    },
                    Name::new("high cirrus puff"),
                ));
            }
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
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        layer,
        mesh,
        material,
        transform,
        obstacle,
        None,
        name,
    );
}

#[allow(clippy::too_many_arguments)]
fn queue_wind_island_visual(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    wind_motion: WindVisualMotion,
    name: &'static str,
) {
    queue_island_visual_with_motion(
        entries,
        visual_index,
        island,
        layer,
        mesh,
        material,
        transform,
        obstacle,
        Some(wind_motion),
        name,
    );
}

#[allow(clippy::too_many_arguments)]
fn queue_island_visual_with_motion(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    island: SkyIsland,
    layer: IslandVisualLayer,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    obstacle: Option<CameraObstacle>,
    wind_motion: Option<WindVisualMotion>,
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
        wind_motion,
        name,
    });
}

#[allow(clippy::too_many_arguments)]
fn queue_sky_island(
    entries: &mut Vec<IslandVisualEntry>,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    top_material: Handle<StandardMaterial>,
    rock_material: Handle<StandardMaterial>,
    under_material: Handle<StandardMaterial>,
    marker_material: Handle<StandardMaterial>,
    branch_marker_material: Handle<StandardMaterial>,
    detail_materials: IslandDetailMaterials,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let top_y = island.mesh_top_y();
    let mut visual_index = 0;

    let impostor_mesh = island_impostor_mesh(island_index, island);
    content_diagnostics.record_island_impostor(
        impostor_mesh.count_vertices(),
        mesh_vertex_color_band_count(&impostor_mesh),
    );
    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Impostor,
        meshes.add(impostor_mesh),
        top_material.clone(),
        Transform::default(),
        None,
        "island distant impostor",
    );

    let terrain_mesh = island_terrain_mesh(island_index, island);
    content_diagnostics.record_island_terrain_surface(
        terrain_mesh.count_vertices(),
        mesh_vertex_color_band_count(&terrain_mesh),
        mesh_terrain_material_weight_band_count(&terrain_mesh),
        mesh_terrain_material_channel_count(&terrain_mesh),
        mesh_terrain_material_region_count(&terrain_mesh),
        mesh_y_range(&terrain_mesh),
    );
    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(terrain_mesh),
        top_material,
        Transform::default(),
        None,
        "island terrain surface",
    );

    let rock_body_center = Vec3::new(
        island.center.x,
        top_y - island.thickness * 0.54,
        island.center.z,
    );
    let rock_body_half_extents = Vec3::new(
        island.half_extents.x * 0.78,
        island.thickness * 0.5,
        island.half_extents.y * 0.78,
    );
    let cliff_mesh = island_cliff_mesh(island_index, island);
    let cliff_vertex_count = cliff_mesh.count_vertices();
    content_diagnostics.record_island_cliff_detail(mesh_vertex_color_band_count(&cliff_mesh));
    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(cliff_mesh),
        rock_material,
        Transform::default(),
        Some(CameraObstacle(CameraObstruction::new(
            rock_body_center,
            rock_body_half_extents,
        ))),
        "island procedural cliff body",
    );

    let underside_mesh = island_underside_mesh(island_index, island);
    let underside_vertex_count = underside_mesh.count_vertices();
    content_diagnostics.record_island_cliff_detail(mesh_vertex_color_band_count(&underside_mesh));
    queue_island_visual(
        entries,
        &mut visual_index,
        island,
        IslandVisualLayer::Terrain,
        meshes.add(underside_mesh),
        under_material.clone(),
        Transform::default(),
        None,
        "island tapered underside",
    );
    content_diagnostics.record_procedural_island_body(
        ISLAND_BODY_SEGMENTS,
        cliff_vertex_count + underside_vertex_count,
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
        content_diagnostics,
        meshes,
        detail_materials,
        flower_material,
        water_material,
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

fn wind_visual_motion(
    island_index: usize,
    phase_offset: f32,
    amplitude_m: f32,
    bend_radians: f32,
    gust_speed: f32,
) -> WindVisualMotion {
    let island_phase = island_index as f32 * 0.91;
    let wind_direction =
        Vec3::new(0.9, 0.0, -0.34 + (island_phase * 0.8).sin() * 0.22).normalize_or_zero();

    WindVisualMotion {
        phase: island_phase + phase_offset,
        amplitude_m,
        bend_radians,
        gust_speed,
        wind_direction,
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_sky_island_details(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    detail_materials: IslandDetailMaterials,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let detail_phase = island_index as f32 * 0.77;
    content_diagnostics.record_detail_biome_palette(island_index);
    let ground_cover_mesh = island_ground_cover_mesh(island_index, island);
    content_diagnostics.record_generated_ground_cover(
        GROUND_COVER_PATCHES,
        GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH,
        ground_cover_mesh.count_vertices(),
    );
    queue_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Detail,
        meshes.add(ground_cover_mesh),
        detail_materials.ground_cover.clone(),
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
        let trunk_mesh = tree_trunk_mesh(
            0.22,
            trunk_height,
            5_000 + island_index as u32 * 97 + index as u32 * 13,
        );
        content_diagnostics.record_generated_tree_trunk(trunk_mesh.count_vertices());
        let canopy_mesh = tree_canopy_mesh(
            canopy_radius,
            6_000 + island_index as u32 * 101 + index as u32 * 17,
        );
        content_diagnostics.record_generated_tree_canopy(canopy_mesh.count_vertices());

        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(trunk_mesh),
            detail_materials.trunk.clone(),
            Transform::from_translation(trunk_center),
            Some(CameraObstacle(CameraObstruction::new(
                trunk_center,
                Vec3::new(0.22, trunk_height * 0.5, 0.22),
            ))),
            wind_visual_motion(island_index, index as f32 * 0.61, 0.025, 0.018, 0.9),
            "island tree trunk",
        );
        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(canopy_mesh),
            detail_materials.foliage.clone(),
            Transform::from_translation(canopy_center),
            Some(CameraObstacle(CameraObstruction::new(
                canopy_center,
                Vec3::splat(canopy_radius),
            ))),
            wind_visual_motion(island_index, index as f32 * 0.83 + 1.7, 0.22, 0.075, 1.35),
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
        let rock_mesh = rock_scatter_mesh(
            stone_scale,
            9_000 + island_index as u32 * 131 + index as u32 * 19,
        );
        content_diagnostics.record_generated_rock(rock_mesh.count_vertices());

        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(rock_mesh),
            detail_materials.stone.clone(),
            Transform::from_xyz(x, surface_y + stone_scale * 0.5, z),
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
    queue_wind_island_visual(
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
        wind_visual_motion(island_index, 3.4, 0.035, 0.018, 1.1),
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
        let launch_trunk_mesh =
            tree_trunk_mesh(0.35, launch_tree_height, 7_000 + island_index as u32 * 97);
        content_diagnostics.record_generated_tree_trunk(launch_trunk_mesh.count_vertices());
        let launch_canopy_mesh =
            tree_canopy_mesh(launch_canopy_radius, 8_000 + island_index as u32 * 101);
        content_diagnostics.record_generated_tree_canopy(launch_canopy_mesh.count_vertices());

        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(launch_trunk_mesh),
            detail_materials.trunk,
            Transform::from_translation(launch_tree_center),
            Some(CameraObstacle(CameraObstruction::new(
                launch_tree_center,
                Vec3::new(0.35, launch_tree_height * 0.5, 0.35),
            ))),
            wind_visual_motion(island_index, 4.2, 0.035, 0.02, 0.9),
            "launch camera tree trunk",
        );
        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(launch_canopy_mesh),
            detail_materials.foliage,
            Transform::from_translation(launch_canopy_center),
            Some(CameraObstacle(CameraObstruction::new(
                launch_canopy_center,
                Vec3::splat(launch_canopy_radius),
            ))),
            wind_visual_motion(island_index, 5.1, 0.28, 0.09, 1.25),
            "launch camera tree canopy",
        );
    }
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
    if let Some(motion) = entry.wind_motion {
        entity.insert(WindResponsiveVisual {
            base_translation: entry.transform.translation,
            base_rotation: entry.transform.rotation,
            base_scale: entry.transform.scale,
            motion,
        });
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
    let mut spawned_visuals = 0;
    let mut despawned_visual_count = 0;

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
                    spawned_visuals += 1;
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
            despawned_visual_count += 1;
        }
    }

    let stream_changes = spawned_visuals + despawned_visual_count;
    diagnostics.counts = counts;
    diagnostics.visibility_changes_this_frame = stream_changes;
    diagnostics.max_visibility_changes_per_frame = diagnostics
        .max_visibility_changes_per_frame
        .max(stream_changes);
    diagnostics.total_visibility_changes += stream_changes;
    diagnostics.spawned_visuals_this_frame = spawned_visuals;
    diagnostics.despawned_visuals_this_frame = despawned_visual_count;
    diagnostics.max_spawned_visuals_per_frame = diagnostics
        .max_spawned_visuals_per_frame
        .max(spawned_visuals);
    diagnostics.max_despawned_visuals_per_frame = diagnostics
        .max_despawned_visuals_per_frame
        .max(despawned_visual_count);
    diagnostics.total_spawned_visuals += spawned_visuals;
    diagnostics.total_despawned_visuals += despawned_visual_count;
    diagnostics.initialized = true;
}

fn update_cinematic_weather(
    time: Res<Time>,
    weather: Res<CinematicWeather>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<CinematicSun>>,
    mut camera_fx: Query<
        (
            &mut Camera,
            &mut Exposure,
            &mut DistanceFog,
            &mut VolumetricFog,
        ),
        With<Camera3d>,
    >,
) {
    let cycle = (time.elapsed_secs() / weather.cycle_seconds * std::f32::consts::TAU).sin();
    let warm = (cycle * 0.5 + 0.5).clamp(0.0, 1.0);
    let storm = ((time.elapsed_secs() * 0.037).sin() * 0.5 + 0.5).powf(2.2) * 0.34;
    let cool_light = Color::srgb(0.78, 0.84, 1.0);
    let warm_light = Color::srgb(1.0, 0.82, 0.55);
    let sky_clear = Color::srgb(0.46, 0.66, 0.92);
    let sky_weather = Color::srgb(0.38, 0.48, 0.64);

    let sky_color = mix_color(
        mix_color(sky_weather, sky_clear, warm),
        Color::srgb(0.56, 0.70, 0.88),
        0.18,
    );
    clear_color.0 = sky_color;
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

    for (mut camera, mut exposure, mut fog, mut volumetric_fog) in &mut camera_fx {
        camera.clear_color = ClearColorConfig::Custom(sky_color);
        camera.output_mode = CameraOutputMode::Write {
            blend_state: Some(BlendState::REPLACE),
            clear_color: ClearColorConfig::Custom(sky_color),
        };
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

fn update_wind_responsive_visuals(
    time: Res<Time>,
    mut visuals: Query<(&WindResponsiveVisual, &mut Transform)>,
) {
    let elapsed = time.elapsed_secs();

    for (visual, mut transform) in &mut visuals {
        let motion = wind_sway_motion(
            elapsed,
            visual.motion.phase,
            visual.motion.amplitude_m,
            visual.motion.bend_radians,
            visual.motion.gust_speed,
            visual.motion.wind_direction,
        );
        transform.translation = visual.base_translation + motion.offset;
        transform.rotation = visual.base_rotation * motion.rotation;
        transform.scale = visual.base_scale * motion.scale;
    }
}

fn update_updraft_guides(time: Res<Time>, mut guides: Query<(&UpdraftGuide, &mut Transform)>) {
    let elapsed = time.elapsed_secs();

    for (guide, mut transform) in &mut guides {
        transform.translation = updraft_guide_position(guide, elapsed);
        transform.rotation = Quat::from_rotation_y(guide.phase + elapsed * guide.angular_speed);
    }
}

fn update_updraft_ribbons(time: Res<Time>, mut ribbons: Query<(&UpdraftRibbon, &mut Transform)>) {
    let elapsed = time.elapsed_secs();

    for (ribbon, mut transform) in &mut ribbons {
        transform.rotation =
            ribbon.base_rotation * Quat::from_rotation_y(elapsed * ribbon.spin_speed);
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
        keyboard_flight_input(&keyboard),
        facing,
        &mut context,
        &mut kinematics,
    );
}

fn keyboard_flight_input(keyboard: &ButtonInput<KeyCode>) -> FlightInput {
    FlightInput {
        forward: keyboard.pressed(KeyCode::KeyW),
        backward: keyboard.pressed(KeyCode::KeyS),
        left: keyboard.pressed(KeyCode::KeyA),
        right: keyboard.pressed(KeyCode::KeyD),
        glide: keyboard.pressed(KeyCode::Space),
        dive: keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight),
        launch: keyboard.just_pressed(KeyCode::KeyE),
    }
}

fn eval_fly_player(
    run: Res<EvalRun>,
    tuning: Res<FlightTuning>,
    mut world: MovementWorld,
    camera: Query<&Transform, CameraFollowFilter>,
    mut movement_basis: ResMut<EvalMovementBasis>,
    mut player: Query<(&mut Transform, &mut Velocity, &mut FlightController), With<Player>>,
) {
    if run.finalized {
        return;
    }

    let Ok((mut transform, mut velocity, mut controller)) = player.single_mut() else {
        return;
    };
    let facing = movement_facing(camera.single().ok(), &transform);
    *movement_basis = EvalMovementBasis {
        frame: run.frame,
        facing: Some(facing),
    };
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
    player.transform.rotation = face_flight_direction(
        player.transform.rotation,
        player.velocity.0,
        input,
        facing,
        *player.controller,
        &tuning,
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
    visual_assets: Res<VisualAssetRegistry>,
    mut player: Query<
        (
            &Transform,
            &Velocity,
            &FlightController,
            &mut AnimationState,
        ),
        With<Player>,
    >,
    mut parts: Query<
        (&CharacterPart, &mut Transform, &mut Visibility),
        GeneratedCharacterPartAnimationFilter,
    >,
    mut authored_scenes: Query<(&AuthoredVisualScene, &mut Visibility), Without<CharacterPart>>,
    mut generated_placeholders: Query<&mut Visibility, GeneratedPlayerPlaceholderFilter>,
) {
    let Ok((transform, velocity, controller, mut animation)) = player.single_mut() else {
        return;
    };

    let dt = eval_dt(&time, eval.as_deref());
    animation.phase = advance_phase(animation.phase, velocity.0.length(), dt);
    let pose_velocity = character_pose_velocity(velocity.0, transform.rotation);
    let blend = pose_blend(dt);
    let authored_player_ready = visual_assets.scene_ready(VisualAssetKind::PlayerCharacter);
    let authored_glider_ready = visual_assets.scene_ready(VisualAssetKind::Glider);

    for (part, mut transform, mut visibility) in &mut parts {
        let pose = part_pose(part, controller.mode, pose_velocity, animation.phase);
        transform.translation = transform.translation.lerp(pose.translation, blend);
        transform.rotation = transform.rotation.slerp(pose.rotation, blend);

        let replaced_by_authored_scene = match part.role {
            CharacterPartRole::Wing(_) => authored_glider_ready,
            _ => authored_player_ready,
        };

        *visibility = if replaced_by_authored_scene {
            Visibility::Hidden
        } else {
            match pose.visibility {
                PartVisibility::Inherited => Visibility::Inherited,
                PartVisibility::Hidden => Visibility::Hidden,
                PartVisibility::Visible => Visibility::Visible,
            }
        };
    }

    for (scene, mut visibility) in &mut authored_scenes {
        let visible = match scene.role {
            AuthoredVisualSceneRole::PlayerRuntime => authored_player_ready,
            AuthoredVisualSceneRole::GliderRuntime => {
                authored_glider_ready && controller.mode == FlightMode::Gliding
            }
            AuthoredVisualSceneRole::WorldFixture => visual_assets.scene_ready(scene.kind),
        };
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    for mut visibility in &mut generated_placeholders {
        *visibility = if authored_player_ready {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

fn character_pose_velocity(world_velocity: Vec3, player_rotation: Quat) -> Vec3 {
    let forward = body_forward(player_rotation);
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    Vec3::new(
        world_velocity.dot(right),
        world_velocity.y,
        -world_velocity.dot(forward),
    )
}

fn update_glider_airflow_trails(
    time: Res<Time>,
    visual_assets: Res<VisualAssetRegistry>,
    player: Query<(&Velocity, &FlightController), With<Player>>,
    mut trails: Query<(&GliderAirflowTrail, &mut Transform, &mut Visibility)>,
) {
    let Ok((velocity, controller)) = player.single() else {
        return;
    };

    let airflow = wing_airflow_strength(controller.mode, velocity.0);
    let visible = airflow > 0.04 && !visual_assets.scene_ready(VisualAssetKind::Glider);
    let pulse = (time.elapsed_secs() * 9.0).sin() * 0.04 * airflow;

    for (trail, mut transform, mut visibility) in &mut trails {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        let sign = trail.side.sign();
        transform.translation =
            trail.base_translation + Vec3::new(sign * airflow * 0.08, pulse, airflow * 0.55);
        transform.rotation = trail.base_rotation
            * Quat::from_rotation_y(sign * airflow * 0.14)
            * Quat::from_rotation_x(-airflow * 0.07);
        transform.scale = Vec3::new(0.24 + airflow * 0.38, 1.0, 0.12 + airflow * 2.2);
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

fn follow_camera(
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

fn wind_responsive_visual_metrics<'a>(
    visuals: impl Iterator<Item = (&'a WindResponsiveVisual, &'a Transform)>,
) -> (usize, f32) {
    visuals.fold((0, 0.0_f32), |(count, max_offset), (visual, transform)| {
        (
            count + 1,
            max_offset.max(transform.translation.distance(visual.base_translation)),
        )
    })
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
    let content_metrics = *scene.content_diagnostics;
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
    let (environment_motion_visuals, max_environment_motion_offset_m) =
        wind_responsive_visual_metrics(scene.wind_responsive_visuals.iter());
    let body_roll = body_roll_degrees(transform.rotation);

    **text = format!(
        "frame {:>4.1} ms\nmode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\ntarget {:>5.1} m {}\nobjective {}/{} {} {:>5.1} m {}\ncamera pitch {:>5.1} deg\ncamera distance {:>5.1} m\ncamera frame {:>5.1} deg\ncamera motion {:>4.1} m / {:>4.1} deg\ncamera orbit {:>5.1} deg\ncamera obstruction {:>4.1} m / {}\nmouse yaw {:>5.1} deg\nmouse pitch {:>5.1} deg\nmouse {}\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\nbody bank/roll {:>5.1} / {:>5.1} deg\npower ups visible/collected/active {} / {} / {}\nvisual assets {} gltf {} ready {} placeholders {} missing {} stream {}\nasset load queued/loading/loaded/deferred/failed {} / {} / {} / {} / {}\nasset preload deps/ready {} / {} always/stream {} / {}\nasset scene spawned/ready {} / {}\nauthored world fixtures {}\nasset anim clips ready/declared {} / {} players {} graphs {}\nasset residency always/window/near/far/weather {} / {} / {} / {} / {}\nvisual wind fields {} / {}\nlift fields {} / {}\nsky islands {}\nisland terrain surfaces {} vertices {} color bands {} material bands/channels/regions/texture {} / {} / {} / {} relief {:>4.2} m cliff bands {}\nisland body proc/prim {} / {} silhouette min/avg {} / {:>4.1} vertices min/max {} / {}\nground cover patches {} blades {} vertices {}\ngenerated trees trunk/canopy {} / {} vertices {} / {} biome palettes {}\ngenerated rocks {} vertices {}\ngenerated clouds {} banks {} depth {:>4.1} m lobes min/max {} / {} vertices {}\nstream chunk [{}, {}] active {} / {}\nlod near/mid/far {} / {} / {}\nstream terrain visible/hidden {} / {}\nstream impostor visible/hidden {} / {}\nlod detail visible/hidden {} / {}\nenvironment motion {} / {:>4.2} m\nstream residency {} / {} {:>4.1}% hidden {}\nstream spawn/despawn {} / {} max {} / {} total {} / {}\nstream entity changes {} max {} total {}\nroute beacons {}\nlaunch cooldown {:>4.1}s\nlaunch ready {}\ndebug visuals {} (F1)\nWASD camera-relative  Click mouse lock  Esc release  Space glider  E launch  Shift dive",
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
        controller.bank_degrees,
        body_roll,
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
        asset_metrics.deferred_scene_count,
        asset_metrics.failed_scene_count,
        asset_metrics.dependency_loaded_scene_count,
        asset_metrics.preload_ready_scene_count,
        asset_metrics.always_preload_ready_slot_count,
        asset_metrics.streaming_preload_ready_slot_count,
        asset_metrics.spawned_scene_count,
        asset_metrics.ready_scene_count,
        scene.asset_diagnostics.visible_world_fixture_count,
        asset_metrics.ready_animation_clip_count,
        asset_metrics.declared_animation_clip_count,
        asset_metrics.animation_player_count,
        asset_metrics.animation_graph_count,
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
        content_metrics.island_terrain_surface_count,
        content_metrics.min_island_terrain_mesh_vertices,
        content_metrics.min_island_terrain_color_bands,
        content_metrics.min_island_terrain_material_weight_bands,
        content_metrics.min_island_terrain_material_channels,
        content_metrics.min_island_terrain_material_regions,
        content_metrics.min_island_terrain_texture_detail_bands,
        content_metrics.min_island_terrain_relief_range_m(),
        content_metrics.min_island_cliff_color_bands,
        content_metrics.procedural_island_body_count,
        content_metrics.primitive_island_body_count,
        content_metrics.min_island_body_silhouette_segments,
        content_metrics.average_island_body_silhouette_segments(),
        content_metrics.min_island_body_mesh_vertices,
        content_metrics.max_island_body_mesh_vertices,
        content_metrics.generated_ground_cover_patch_count,
        content_metrics.min_ground_cover_blade_count,
        content_metrics.min_ground_cover_mesh_vertices,
        content_metrics.generated_tree_trunk_count,
        content_metrics.generated_tree_canopy_count,
        content_metrics.min_tree_trunk_mesh_vertices,
        content_metrics.min_tree_canopy_mesh_vertices,
        content_metrics.detail_biome_palette_count(),
        content_metrics.generated_rock_count,
        content_metrics.min_rock_mesh_vertices,
        content_metrics.generated_weather_cloud_count,
        content_metrics.generated_weather_cloud_bank_count,
        content_metrics.min_weather_cloud_bank_depth_m(),
        content_metrics.min_weather_cloud_lobe_count,
        content_metrics.max_weather_cloud_lobe_count,
        content_metrics.min_weather_cloud_mesh_vertices,
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
        environment_motion_visuals,
        max_environment_motion_offset_m,
        lod_visuals.resident_count(),
        lod_visuals.catalog_count(),
        lod_visuals.resident_fraction() * 100.0,
        lod_visuals.hidden_count(),
        scene.stream_diagnostics.spawned_visuals_this_frame,
        scene.stream_diagnostics.despawned_visuals_this_frame,
        scene.stream_diagnostics.max_spawned_visuals_per_frame,
        scene.stream_diagnostics.max_despawned_visuals_per_frame,
        scene.stream_diagnostics.total_spawned_visuals,
        scene.stream_diagnostics.total_despawned_visuals,
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
    movement_basis: Res<EvalMovementBasis>,
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
        camera_world_yaw,
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
                camera_view_yaw_degrees(
                    camera_transform.rotation,
                    scene.camera_diagnostics.follow_direction,
                ),
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
    let player_ground = scene.route.ground_at(transform.translation);
    let visual_foot_gap_m = grounded_visual_foot_gap_m(
        transform.translation.y,
        player_ground.floor_y,
        controller.mode,
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
    let content_metrics = *scene.content_diagnostics;
    let (environment_motion_visuals, max_environment_motion_offset_m) =
        wind_responsive_visual_metrics(scene.wind_responsive_visuals.iter());
    let movement_input = scripted_input(run.scenario, run.frame);
    let movement_axis = movement_input.planar_axis();
    let movement_facing = if movement_basis.frame == run.frame {
        movement_basis
            .facing
            .unwrap_or_else(|| movement_facing(scene.camera.single().ok(), transform))
    } else {
        movement_facing(scene.camera.single().ok(), transform)
    };
    let desired_movement_direction =
        if movement_input.forward || movement_input.left || movement_input.right {
            desired_planar_movement_direction(movement_input, movement_facing)
        } else {
            None
        };
    let desired_body_yaw_error_degrees = desired_movement_direction
        .map(|direction| body_yaw_error_degrees(transform.rotation, direction))
        .unwrap_or(f32::NAN);
    let desired_heading_alignment_mps = desired_movement_direction
        .map(|direction| desired_heading_alignment_speed(velocity.0, direction))
        .unwrap_or(f32::NAN);
    let lateral_axis_active = movement_input.has_lateral_axis();
    let lateral_input_active = lateral_axis_active && controller.mode != FlightMode::Grounded;
    let lateral_response_mps = if lateral_axis_active {
        lateral_response_speed(velocity.0, movement_input, movement_facing)
    } else {
        0.0
    };
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
        environment_motion_visuals,
        max_environment_motion_offset_m,
        lod_visuals.resident_count(),
        scene.stream_diagnostics.visibility_changes_this_frame,
        scene.stream_diagnostics.max_visibility_changes_per_frame,
        scene.stream_diagnostics.total_visibility_changes,
        lod_visuals.catalog_count(),
        lod_visuals.hidden_count(),
        lod_visuals.resident_fraction(),
        scene.stream_diagnostics.spawned_visuals_this_frame,
        scene.stream_diagnostics.despawned_visuals_this_frame,
        scene.stream_diagnostics.max_spawned_visuals_per_frame,
        scene.stream_diagnostics.max_despawned_visuals_per_frame,
        scene.stream_diagnostics.total_spawned_visuals,
        scene.stream_diagnostics.total_despawned_visuals,
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
        asset_metrics.dependency_loaded_scene_count,
        asset_metrics.preload_ready_scene_count,
        asset_metrics.failed_scene_count,
        asset_metrics.spawned_scene_count,
        asset_metrics.ready_scene_count,
        asset_metrics.always_slot_count,
        asset_metrics.stream_window_slot_count,
        asset_metrics.near_lod_slot_count,
        asset_metrics.far_lod_slot_count,
        asset_metrics.weather_slot_count,
        asset_metrics.always_preload_ready_slot_count,
        asset_metrics.streaming_preload_ready_slot_count,
        asset_metrics.declared_animation_clip_count,
        asset_metrics.ready_animation_clip_count,
        asset_metrics.animation_player_count,
        asset_metrics.animation_graph_count,
        AERIAL_POWER_UP_ROUTE.len(),
        scene.power_ups.visible_count(),
        scene.power_ups.collected_count(),
        scene.power_ups.active_effects(),
        scene.power_ups.total_activations,
    )
    .with_visible_authored_world_fixture_count(scene.asset_diagnostics.visible_world_fixture_count)
    .with_deferred_visual_asset_scene_count(asset_metrics.deferred_scene_count)
    .with_camera_follow_metrics(scene.camera_diagnostics.follow_direction_error_degrees)
    .with_camera_world_yaw_metrics(camera_world_yaw)
    .with_visual_foot_gap(visual_foot_gap_m)
    .with_content_metrics(
        content_metrics.island_terrain_surface_count,
        content_metrics.min_island_terrain_mesh_vertices,
        content_metrics.min_island_terrain_color_bands,
        content_metrics.min_island_terrain_relief_range_m(),
        content_metrics.min_island_cliff_color_bands,
        content_metrics.procedural_island_body_count,
        content_metrics.primitive_island_body_count,
        content_metrics.min_island_body_silhouette_segments,
        content_metrics.average_island_body_silhouette_segments(),
        content_metrics.min_island_body_mesh_vertices,
        content_metrics.max_island_body_mesh_vertices,
    )
    .with_island_impostor_metrics(
        content_metrics.min_island_impostor_mesh_vertices,
        content_metrics.min_island_impostor_color_bands,
    )
    .with_terrain_material_metrics(
        content_metrics.min_island_terrain_material_weight_bands,
        content_metrics.min_island_terrain_material_channels,
        content_metrics.min_island_terrain_material_regions,
        content_metrics.min_island_terrain_texture_detail_bands,
    )
    .with_generated_visual_shape_metrics(
        content_metrics.generated_ground_cover_patch_count,
        content_metrics.min_ground_cover_blade_count,
        content_metrics.min_ground_cover_mesh_vertices,
        content_metrics.generated_tree_trunk_count,
        content_metrics.generated_tree_canopy_count,
        content_metrics.min_tree_trunk_mesh_vertices,
        content_metrics.min_tree_canopy_mesh_vertices,
        content_metrics.detail_biome_palette_count(),
        content_metrics.generated_rock_count,
        content_metrics.min_rock_mesh_vertices,
        content_metrics.generated_weather_cloud_count,
        content_metrics.generated_weather_cloud_bank_count,
        content_metrics.min_weather_cloud_bank_depth_m(),
        content_metrics.min_weather_cloud_lobe_count,
        content_metrics.max_weather_cloud_lobe_count,
        content_metrics.min_weather_cloud_mesh_vertices,
    )
    .with_movement_metrics(EvalMovementMetrics {
        desired_body_yaw_error_degrees,
        body_roll_degrees: body_roll_degrees(transform.rotation),
        desired_heading_alignment_mps,
        lateral_response_mps,
        lateral_input_active,
        movement_axis,
    });

    if let Err(error) = run.record_sample(sample) {
        run.io_error = Some(format!("failed to write eval sample: {error}"));
    }
}

fn finish_eval_frame(
    mut commands: Commands,
    mut run: ResMut<EvalRun>,
    scene: EvalScene,
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

    if let Err(error) = capture_due_checkpoint_screenshots(&mut commands, &mut run, &scene) {
        run.io_error = Some(format!(
            "failed to write checkpoint marker metadata: {error}"
        ));
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

fn capture_due_checkpoint_screenshots(
    commands: &mut Commands,
    run: &mut EvalRun,
    scene: &EvalScene,
) -> std::io::Result<()> {
    let frame = run.frame;
    let scenario = run.scenario;
    for checkpoint in run
        .checkpoint_captures
        .iter_mut()
        .filter(|checkpoint| !checkpoint.captured && checkpoint.frame == frame)
    {
        if !checkpoint.marker_metadata_written {
            write_checkpoint_marker_metadata(
                &checkpoint.marker_metadata_path,
                scenario,
                checkpoint,
                scene,
            )?;
            checkpoint.marker_metadata_written = true;
        }
        let screenshot_path = checkpoint.path.clone();
        checkpoint.captured = true;
        commands.spawn(Screenshot::primary_window()).observe(
            move |captured: On<ScreenshotCaptured>| {
                save_to_disk(screenshot_path.clone())(captured);
            },
        );
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
struct SemanticRouteMarker {
    kind: &'static str,
    label: &'static str,
    world_position: Vec3,
    current_objective: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticMarkerVisibility {
    Visible,
    Occluded,
    Offscreen,
    BehindCamera,
}

impl SemanticMarkerVisibility {
    fn label(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::Occluded => "occluded",
            Self::Offscreen => "offscreen",
            Self::BehindCamera => "behind_camera",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SemanticMarkerOcclusion {
    island_name: &'static str,
    distance_m: f32,
}

#[derive(Clone, Copy, Debug)]
struct SemanticSceneSample {
    kind: &'static str,
    label: &'static str,
    expected_material: &'static str,
    world_position: Vec3,
}

fn write_checkpoint_marker_metadata(
    path: &Path,
    scenario: EvalScenario,
    checkpoint: &EvalCheckpointCapture,
    scene: &EvalScene,
) -> std::io::Result<()> {
    let markers = semantic_route_markers(scene);
    let scene_samples = semantic_scene_samples(scene);
    let expected_objective_count = scene
        .route
        .route_objectives(scenario.target_island_name)
        .len();
    let (
        viewport_size,
        marker_json,
        visible_count,
        in_viewport_marker_count,
        occluded_marker_count,
        current_objective_visible,
        scene_sample_json,
        in_viewport_scene_sample_count,
        occluded_scene_sample_count,
        visible_scene_sample_count,
        visible_scene_material_count,
    ) = match scene.camera_projection.single() {
        Ok((camera, camera_transform)) => {
            let (
                viewport_size,
                marker_json,
                visible_count,
                in_viewport_marker_count,
                occluded_marker_count,
                current_objective_visible,
            ) = checkpoint_marker_projection_json(
                camera,
                camera_transform,
                &markers,
                scene.route.islands(),
            );
            let (
                scene_sample_json,
                visible_scene_sample_count,
                in_viewport_scene_sample_count,
                occluded_scene_sample_count,
                visible_scene_material_count,
            ) = checkpoint_scene_sample_projection_json(
                camera,
                camera_transform,
                &scene_samples,
                scene.route.islands(),
            );
            (
                viewport_size,
                marker_json,
                visible_count,
                in_viewport_marker_count,
                occluded_marker_count,
                current_objective_visible,
                scene_sample_json,
                in_viewport_scene_sample_count,
                occluded_scene_sample_count,
                visible_scene_sample_count,
                visible_scene_material_count,
            )
        }
        Err(_) => (None, Vec::new(), 0, 0, 0, false, Vec::new(), 0, 0, 0, 0),
    };
    let viewport_json = viewport_size
        .map(|size| {
            format!(
                "{{\"width\": {}, \"height\": {}}}",
                terrain_export_json_number(size.x),
                terrain_export_json_number(size.y)
            )
        })
        .unwrap_or_else(|| "null".to_string());
    let passed = markers.len() >= expected_objective_count
        && visible_count > 0
        && scene_samples.len() >= 4
        && visible_scene_sample_count > 0
        && viewport_size.is_some();
    let target_island = scenario
        .target_island_name
        .map(terrain_export_json_string)
        .unwrap_or_else(|| "null".to_string());
    let json = format!(
        "{{\n  \"passed\": {},\n  \"scenario\": {},\n  \"target_island\": {},\n  \"frame\": {},\n  \"checkpoint\": {},\n  \"screenshot\": {},\n  \"viewport\": {},\n  \"semantic_marker_count\": {},\n  \"expected_objective_marker_count\": {},\n  \"in_viewport_semantic_marker_count\": {},\n  \"occluded_semantic_marker_count\": {},\n  \"visible_semantic_marker_count\": {},\n  \"current_objective_visible\": {},\n  \"semantic_scene_sample_count\": {},\n  \"in_viewport_semantic_scene_sample_count\": {},\n  \"occluded_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_sample_count\": {},\n  \"visible_semantic_scene_material_count\": {},\n  \"markers\": [\n{}\n  ],\n  \"scene_samples\": [\n{}\n  ]\n}}\n",
        passed,
        terrain_export_json_string(scenario.name),
        target_island,
        checkpoint.frame,
        terrain_export_json_string(checkpoint.name),
        terrain_export_json_string(&path_string(&checkpoint.path)),
        viewport_json,
        markers.len(),
        expected_objective_count,
        in_viewport_marker_count,
        occluded_marker_count,
        visible_count,
        current_objective_visible,
        scene_samples.len(),
        in_viewport_scene_sample_count,
        occluded_scene_sample_count,
        visible_scene_sample_count,
        visible_scene_material_count,
        marker_json
            .into_iter()
            .map(|entry| format!("    {entry}"))
            .collect::<Vec<_>>()
            .join(",\n"),
        scene_sample_json
            .into_iter()
            .map(|entry| format!("    {entry}"))
            .collect::<Vec<_>>()
            .join(",\n"),
    );

    fs::write(path, json)
}

fn semantic_route_markers(scene: &EvalScene) -> Vec<SemanticRouteMarker> {
    let mut markers = Vec::new();
    let current_label =
        (!scene.route_objectives.complete).then_some(scene.route_objectives.current_label);

    for objective in scene
        .route
        .route_objectives(scene.route_objectives.target_island_name)
    {
        let kind = match objective.kind {
            RouteObjectiveKind::FlyThrough => "objective_updraft",
            RouteObjectiveKind::Land => "objective_landing",
        };
        markers.push(SemanticRouteMarker {
            kind,
            label: objective.label,
            world_position: objective.position,
            current_objective: current_label == Some(objective.label),
        });
    }

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        if island.is_target {
            let ring_size = 8.0;
            for offset in [
                Vec3::new(0.0, 0.05, ring_size * 0.5),
                Vec3::new(0.0, 0.05, -ring_size * 0.5),
                Vec3::new(ring_size * 0.5, 0.05, 0.0),
                Vec3::new(-ring_size * 0.5, 0.05, 0.0),
            ] {
                let surface_y =
                    island.mesh_top_y_at(island.center + Vec3::new(offset.x, 0.0, offset.z));
                markers.push(SemanticRouteMarker {
                    kind: "landing_marker",
                    label: island.name,
                    world_position: Vec3::new(
                        island.center.x + offset.x,
                        surface_y + offset.y,
                        island.center.z + offset.z,
                    ),
                    current_objective: current_label == Some(island.name),
                });
            }
        } else if island.name == "launch mesa" {
            markers.push(SemanticRouteMarker {
                kind: "launch_beacon",
                label: island.name,
                world_position: island_visual_surface_position(island, Vec2::new(-0.42, 0.38))
                    + Vec3::Y * 1.6,
                current_objective: false,
            });
        } else {
            let beacon_height = 3.8 + (island_index % 3) as f32 * 0.7;
            markers.push(SemanticRouteMarker {
                kind: "route_cairn",
                label: island.name,
                world_position: island_visual_surface_position(island, Vec2::new(-0.18, 0.22))
                    + Vec3::Y * (beacon_height * 0.5),
                current_objective: false,
            });
        }
    }

    for power_up in AERIAL_POWER_UP_ROUTE {
        if scene.power_ups.is_collected(power_up) {
            continue;
        }
        markers.push(SemanticRouteMarker {
            kind: "aerial_power_up",
            label: power_up.name,
            world_position: power_up.center,
            current_objective: false,
        });
    }

    markers
}

fn semantic_scene_samples(scene: &EvalScene) -> Vec<SemanticSceneSample> {
    let mut samples = Vec::new();

    for (island_index, island) in scene.route.islands().iter().copied().enumerate() {
        samples.push(SemanticSceneSample {
            kind: "terrain_surface",
            label: island.name,
            expected_material: "terrain",
            world_position: island_visual_surface_position(island, Vec2::new(0.16, -0.14))
                + Vec3::Y * 0.08,
        });
        samples.push(SemanticSceneSample {
            kind: "distant_island",
            label: island.name,
            expected_material: "distant_island",
            world_position: island_visual_surface_position(island, Vec2::new(0.0, 0.0))
                + Vec3::Y * 1.2,
        });

        for (sample_index, canopy_position) in tree_canopy_sample_positions(island_index, island)
            .into_iter()
            .enumerate()
        {
            if sample_index == 1 && island.is_target {
                continue;
            }
            samples.push(SemanticSceneSample {
                kind: "tree_canopy",
                label: island.name,
                expected_material: "foliage",
                world_position: canopy_position,
            });
        }
    }

    for cloud_transform in scene.weather_clouds.iter().take(18) {
        samples.push(SemanticSceneSample {
            kind: "weather_cloud",
            label: "weather cloud",
            expected_material: "cloud",
            world_position: cloud_transform.translation,
        });
    }

    samples
}

fn tree_canopy_sample_positions(island_index: usize, island: SkyIsland) -> Vec<Vec3> {
    if island.name == "launch mesa" {
        let launch_tree_height = 4.4;
        let launch_tree_surface_y =
            island.mesh_top_y_at(Vec3::new(island.center.x, island.center.y, 8.0));
        return vec![Vec3::new(
            island.center.x,
            launch_tree_surface_y + launch_tree_height + 0.85,
            8.0,
        )];
    }

    let detail_phase = island_index as f32 * 0.77;
    [
        Vec2::new(-0.42, -0.24),
        Vec2::new(0.34, -0.36),
        Vec2::new(0.24, 0.32),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, offset)| {
        let sway = (detail_phase + index as f32).sin() * 0.08;
        let surface = island_visual_surface_position(island, Vec2::new(offset.x + sway, offset.y));
        let trunk_height = 2.1 + index as f32 * 0.25;
        surface + Vec3::Y * (trunk_height + 0.72)
    })
    .collect()
}

fn checkpoint_marker_projection_json(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    markers: &[SemanticRouteMarker],
    islands: &[SkyIsland],
) -> (Option<Vec2>, Vec<String>, usize, usize, usize, bool) {
    let viewport_size = camera.logical_viewport_size();
    let camera_position = camera_transform.translation();
    let mut visible_count = 0usize;
    let mut in_viewport_count = 0usize;
    let mut occluded_count = 0usize;
    let mut current_objective_visible = false;
    let entries = markers
        .iter()
        .map(|marker| {
            let projected = camera
                .world_to_viewport_with_depth(camera_transform, marker.world_position)
                .ok();
            let in_viewport = projected
                .zip(viewport_size)
                .is_some_and(|(screen, viewport)| {
                    screen.x >= 0.0
                        && screen.y >= 0.0
                        && screen.x <= viewport.x
                        && screen.y <= viewport.y
                        && screen.z.is_finite()
                        && screen.z > 0.0
                });
            let behind_camera = projected.is_some_and(|screen| {
                screen.z.is_finite() && screen.z <= 0.0
            });
            let occlusion = in_viewport
                .then(|| marker_occlusion_between(camera_position, marker.world_position, islands))
                .flatten();
            let visibility = if behind_camera {
                SemanticMarkerVisibility::BehindCamera
            } else if !in_viewport {
                SemanticMarkerVisibility::Offscreen
            } else if occlusion.is_some() {
                SemanticMarkerVisibility::Occluded
            } else {
                SemanticMarkerVisibility::Visible
            };
            if in_viewport {
                in_viewport_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Occluded {
                occluded_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Visible {
                visible_count += 1;
                current_objective_visible |= marker.current_objective;
            }

            let screen_json = projected
                .map(|screen| {
                    format!(
                        "{{\"x\": {}, \"y\": {}, \"depth_m\": {}}}",
                        terrain_export_json_number(screen.x),
                        terrain_export_json_number(screen.y),
                        terrain_export_json_number(screen.z)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let occluder_json = occlusion
                .map(|occlusion| {
                    format!(
                        "{{\"kind\": \"sky_island\", \"label\": {}, \"distance_m\": {}}}",
                        terrain_export_json_string(occlusion.island_name),
                        terrain_export_json_number(occlusion.distance_m)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let camera_distance_m = marker.world_position.distance(camera_position);

            format!(
                "{{\"kind\": {}, \"label\": {}, \"current_objective\": {}, \"world\": {}, \"screen\": {}, \"in_viewport\": {}, \"visibility\": {}, \"occluder\": {}, \"camera_distance_m\": {}}}",
                terrain_export_json_string(marker.kind),
                terrain_export_json_string(marker.label),
                marker.current_objective,
                terrain_export_json_vec3(marker.world_position),
                screen_json,
                in_viewport,
                terrain_export_json_string(visibility.label()),
                occluder_json,
                terrain_export_json_number(camera_distance_m)
            )
        })
        .collect();

    (
        viewport_size,
        entries,
        visible_count,
        in_viewport_count,
        occluded_count,
        current_objective_visible,
    )
}

fn marker_occlusion_between(
    camera_position: Vec3,
    marker_position: Vec3,
    islands: &[SkyIsland],
) -> Option<SemanticMarkerOcclusion> {
    let mut nearest = None;
    for island in islands {
        let Some(distance_m) =
            island_segment_occlusion_distance(camera_position, marker_position, *island)
        else {
            continue;
        };
        if nearest
            .as_ref()
            .is_none_or(|occlusion: &SemanticMarkerOcclusion| distance_m < occlusion.distance_m)
        {
            nearest = Some(SemanticMarkerOcclusion {
                island_name: island.name,
                distance_m,
            });
        }
    }
    nearest
}

fn island_segment_occlusion_distance(
    camera_position: Vec3,
    marker_position: Vec3,
    island: SkyIsland,
) -> Option<f32> {
    let segment = marker_position - camera_position;
    let length = segment.length();
    if length <= 0.01 {
        return None;
    }
    let direction = segment / length;
    let max_distance = length - 2.0;
    if max_distance <= 1.0 {
        return None;
    }
    let steps = ((length / 6.0).ceil() as usize).clamp(12, 96);

    for step in 1..steps {
        let distance_m = length * step as f32 / steps as f32;
        if distance_m >= max_distance {
            break;
        }
        let point = camera_position + direction * distance_m;
        if island_blocks_marker_ray(island, point) {
            return Some(distance_m);
        }
    }

    None
}

fn island_blocks_marker_ray(island: SkyIsland, point: Vec3) -> bool {
    let dx = (point.x - island.center.x) / island.half_extents.x.max(0.001);
    let dz = (point.z - island.center.z) / island.half_extents.y.max(0.001);
    if dx * dx + dz * dz > 1.10 {
        return false;
    }

    let top_y = island.mesh_top_y_at(point) + 0.9;
    let bottom_y = island.center.y - island.thickness * 1.15;
    point.y >= bottom_y && point.y <= top_y
}

fn checkpoint_scene_sample_projection_json(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    samples: &[SemanticSceneSample],
    scene_islands: &[SkyIsland],
) -> (Vec<String>, usize, usize, usize, usize) {
    let viewport_size = camera.logical_viewport_size();
    let camera_position = camera_transform.translation();
    let mut visible_count = 0usize;
    let mut in_viewport_count = 0usize;
    let mut occluded_count = 0usize;
    let mut visible_materials = HashSet::new();
    let entries = samples
        .iter()
        .map(|sample| {
            let projected = camera
                .world_to_viewport_with_depth(camera_transform, sample.world_position)
                .ok();
            let in_viewport = projected
                .zip(viewport_size)
                .is_some_and(|(screen, viewport)| {
                    screen.x >= 0.0
                        && screen.y >= 0.0
                        && screen.x <= viewport.x
                        && screen.y <= viewport.y
                        && screen.z.is_finite()
                        && screen.z > 0.0
                });
            let behind_camera = projected.is_some_and(|screen| {
                screen.z.is_finite() && screen.z <= 0.0
            });
            let occlusion = in_viewport
                .then(|| {
                    marker_occlusion_between(camera_position, sample.world_position, scene_islands)
                })
                .flatten();
            let visibility = if behind_camera {
                SemanticMarkerVisibility::BehindCamera
            } else if !in_viewport {
                SemanticMarkerVisibility::Offscreen
            } else if occlusion.is_some() {
                SemanticMarkerVisibility::Occluded
            } else {
                SemanticMarkerVisibility::Visible
            };
            if in_viewport {
                in_viewport_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Occluded {
                occluded_count += 1;
            }
            if visibility == SemanticMarkerVisibility::Visible {
                visible_count += 1;
                visible_materials.insert(sample.expected_material);
            }

            let screen_json = projected
                .map(|screen| {
                    format!(
                        "{{\"x\": {}, \"y\": {}, \"depth_m\": {}}}",
                        terrain_export_json_number(screen.x),
                        terrain_export_json_number(screen.y),
                        terrain_export_json_number(screen.z)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let occluder_json = occlusion
                .map(|occlusion| {
                    format!(
                        "{{\"kind\": \"sky_island\", \"label\": {}, \"distance_m\": {}}}",
                        terrain_export_json_string(occlusion.island_name),
                        terrain_export_json_number(occlusion.distance_m)
                    )
                })
                .unwrap_or_else(|| "null".to_string());
            let camera_distance_m = sample.world_position.distance(camera_position);

            format!(
                "{{\"kind\": {}, \"label\": {}, \"expected_material\": {}, \"world\": {}, \"screen\": {}, \"in_viewport\": {}, \"visibility\": {}, \"occluder\": {}, \"camera_distance_m\": {}}}",
                terrain_export_json_string(sample.kind),
                terrain_export_json_string(sample.label),
                terrain_export_json_string(sample.expected_material),
                terrain_export_json_vec3(sample.world_position),
                screen_json,
                in_viewport,
                terrain_export_json_string(visibility.label()),
                occluder_json,
                terrain_export_json_number(camera_distance_m)
            )
        })
        .collect();

    (
        entries,
        visible_count,
        in_viewport_count,
        occluded_count,
        visible_materials.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::{Indices, VertexAttributeValues};

    fn test_island() -> SkyIsland {
        SkyIsland::new(
            "test island",
            Vec3::new(12.0, 40.0, -8.0),
            Vec2::new(22.0, 15.0),
            12.0,
            false,
        )
    }

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(values)) => values,
            _ => panic!("mesh should expose Float32x3 positions"),
        }
    }

    fn u32_indices(mesh: &Mesh) -> &[u32] {
        match mesh.indices() {
            Some(Indices::U32(values)) => values,
            _ => panic!("mesh should expose U32 indices"),
        }
    }

    fn colors(mesh: &Mesh) -> &[[f32; 4]] {
        match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(values)) => values,
            _ => panic!("mesh should expose Float32x4 vertex colors"),
        }
    }

    fn triangle_normal_y(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
        let a = Vec3::from_array(positions[indices[0] as usize]);
        let b = Vec3::from_array(positions[indices[1] as usize]);
        let c = Vec3::from_array(positions[indices[2] as usize]);
        (b - a).cross(c - a).y
    }

    #[test]
    fn character_pose_velocity_uses_body_local_lateral_axis() {
        let rotation = Transform::from_translation(Vec3::ZERO)
            .looking_to(Vec3::X, Vec3::Y)
            .rotation;
        let pose_velocity = character_pose_velocity(Vec3::NEG_Z * 14.0, rotation);

        assert!(pose_velocity.x < -13.9);
        assert!(pose_velocity.z.abs() < 0.001);
    }

    fn normalized_radius(island: SkyIsland, position: [f32; 3]) -> f32 {
        Vec2::new(
            (position[0] - island.center.x) / island.half_extents.x,
            (position[2] - island.center.z) / island.half_extents.y,
        )
        .length()
    }

    #[test]
    fn marker_occlusion_detects_island_between_camera_and_marker() {
        let island = SkyIsland::new(
            "blocking island",
            Vec3::new(0.0, 40.0, -40.0),
            Vec2::new(22.0, 16.0),
            14.0,
            false,
        );
        let occlusion = marker_occlusion_between(
            Vec3::new(0.0, 40.0, 0.0),
            Vec3::new(0.0, 40.0, -96.0),
            &[island],
        )
        .expect("island should block the marker ray");

        assert_eq!(occlusion.island_name, "blocking island");
        assert!(occlusion.distance_m > 20.0);
        assert!(occlusion.distance_m < 70.0);
    }

    #[test]
    fn marker_occlusion_ignores_clear_high_ray() {
        let island = SkyIsland::new(
            "low island",
            Vec3::new(0.0, 40.0, -40.0),
            Vec2::new(22.0, 16.0),
            14.0,
            false,
        );

        assert!(
            marker_occlusion_between(
                Vec3::new(0.0, 72.0, 0.0),
                Vec3::new(0.0, 72.0, -96.0),
                &[island],
            )
            .is_none()
        );
    }

    #[test]
    fn authored_player_clip_selection_tracks_flight_state() {
        assert_eq!(
            authored_player_clip_for_state(FlightMode::Grounded, 0.2),
            AuthoredPlayerClip::Idle
        );
        assert_eq!(
            authored_player_clip_for_state(FlightMode::Grounded, 4.0),
            AuthoredPlayerClip::Jog
        );
        assert_eq!(
            authored_player_clip_for_state(FlightMode::Launching, 18.0),
            AuthoredPlayerClip::Launch
        );
        assert_eq!(
            authored_player_clip_for_state(FlightMode::Gliding, 40.0),
            AuthoredPlayerClip::Glide
        );
        assert_eq!(
            authored_player_clip_for_state(FlightMode::Airborne, 16.0),
            AuthoredPlayerClip::AirBrake
        );
        assert_eq!(
            authored_player_clip_for_state(FlightMode::Airborne, 4.0),
            AuthoredPlayerClip::Land
        );
    }

    #[test]
    fn authored_player_clip_indices_match_declared_gltf_order() {
        assert_eq!(AuthoredPlayerClip::Idle.index(), 0);
        assert_eq!(AuthoredPlayerClip::Jog.index(), 1);
        assert_eq!(AuthoredPlayerClip::Launch.index(), 2);
        assert_eq!(AuthoredPlayerClip::Glide.index(), 3);
        assert_eq!(AuthoredPlayerClip::AirBrake.index(), 4);
        assert_eq!(AuthoredPlayerClip::Land.index(), 5);
    }

    #[test]
    fn attached_authored_visuals_share_terrain_footing_offset() {
        assert_eq!(
            authored_player_scene_transform().translation.y,
            -TERRAIN_VISUAL_FOOTING_OFFSET_M
        );
        assert_eq!(
            authored_glider_scene_transform().translation.y,
            1.35 - TERRAIN_VISUAL_FOOTING_OFFSET_M
        );
        assert_eq!(
            grounded_visual_foot_gap_m(28.0, 28.0, FlightMode::Grounded),
            0.0
        );
    }

    #[test]
    fn named_animation_clip_resolution_reports_missing_clips() {
        let mut named_animations = HashMap::new();
        named_animations.insert("Idle_Loop".to_string(), Handle::<AnimationClip>::default());
        named_animations.insert("Glide_Loop".to_string(), Handle::<AnimationClip>::default());

        let resolution = resolve_named_animation_clip_handles(
            &["Idle_Loop", "Jog_Fwd_Loop", "Glide_Loop"],
            &named_animations,
        );

        assert_eq!(resolution.ready_clip_count(), 2);
        assert_eq!(resolution.expected_clip_count, 3);
        assert_eq!(resolution.missing_clip_names, vec!["Jog_Fwd_Loop"]);
        assert!(!resolution.is_complete());
    }

    #[test]
    fn parse_cli_args_accepts_terrain_export() {
        let action = parse_cli_args(
            ["--export-terrain", "target/terrain_export"]
                .into_iter()
                .map(str::to_string),
        )
        .expect("terrain export args should parse");

        match action {
            CliAction::ExportTerrain { output_dir } => {
                assert_eq!(output_dir, PathBuf::from("target/terrain_export"));
            }
            _ => panic!("expected terrain export action"),
        }
    }

    #[test]
    fn parse_cli_args_accepts_visual_content_export() {
        let action = parse_cli_args(
            ["--export-visual-content", "target/visual_content_export"]
                .into_iter()
                .map(str::to_string),
        )
        .expect("visual content export args should parse");

        match action {
            CliAction::ExportVisualContent { output_dir } => {
                assert_eq!(output_dir, PathBuf::from("target/visual_content_export"));
            }
            _ => panic!("expected visual content export action"),
        }
    }

    #[test]
    fn parse_cli_args_rejects_eval_and_terrain_export_together() {
        let error = parse_cli_args(
            [
                "--eval",
                "baseline_route",
                "--export-terrain",
                "target/terrain_export",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect_err("eval and terrain export should be mutually exclusive");

        assert!(error.contains("cannot be combined"));
    }

    #[test]
    fn parse_cli_args_rejects_eval_and_visual_content_export_together() {
        let error = parse_cli_args(
            [
                "--eval",
                "baseline_route",
                "--export-visual-content",
                "target/visual_content_export",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect_err("eval and visual content export should be mutually exclusive");

        assert!(error.contains("cannot be combined"));
    }

    #[test]
    fn parse_cli_args_rejects_both_export_paths_together() {
        let error = parse_cli_args(
            [
                "--export-terrain",
                "target/terrain_export",
                "--export-visual-content",
                "target/visual_content_export",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .expect_err("export paths should be mutually exclusive");

        assert!(error.contains("cannot be combined"));
    }

    #[test]
    fn metric_only_eval_window_is_hidden_and_unfocused() {
        let scenario = scenario_named("baseline_route").expect("baseline scenario should exist");
        let options = EvalOptions {
            scenario,
            output_dir: PathBuf::from("target/eval/test_hidden_window"),
            capture_screenshot: false,
        };

        let window = primary_window(Some(&options));

        assert!(!window.visible);
        assert!(!window.focused);
        assert!(!window.transparent);
        assert_eq!(window.composite_alpha_mode, CompositeAlphaMode::Opaque);
    }

    #[test]
    fn screenshot_eval_window_remains_visible_for_capture() {
        let scenario = scenario_named("baseline_route").expect("baseline scenario should exist");
        let options = EvalOptions {
            scenario,
            output_dir: PathBuf::from("target/eval/test_visible_window"),
            capture_screenshot: true,
        };

        let window = primary_window(Some(&options));

        assert!(window.visible);
        assert!(window.focused);
        assert!(!window.transparent);
        assert_eq!(window.composite_alpha_mode, CompositeAlphaMode::Opaque);
    }

    #[test]
    fn terrain_export_writes_manifest_meshes_and_weight_sidecars() {
        let output_dir = std::env::temp_dir().join(format!(
            "nau-terrain-export-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        remove_existing_dir(&output_dir).expect("stale terrain export dir should be removable");

        let report = export_terrain_inspection(&output_dir).expect("terrain export should succeed");
        let manifest =
            fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
        let launch_terrain = output_dir.join("islands/00_launch_mesa_terrain.obj");
        let launch_impostor = output_dir.join("islands/00_launch_mesa_impostor.obj");
        let launch_weights = output_dir.join("islands/00_launch_mesa_terrain_material_weights.csv");
        let weights =
            fs::read_to_string(&launch_weights).expect("material weights csv should be readable");

        assert_eq!(report.island_count, SkyRoute::default().islands().len());
        assert_eq!(report.mesh_count, report.island_count * 4);
        assert!(report.total_vertex_count > report.island_count * (2305 + 140));
        assert!(report.total_triangle_count > report.island_count * 4000);
        assert!(report.min_terrain_mesh_vertices >= 2305);
        assert!(report.min_terrain_color_bands >= ISLAND_TERRAIN_COLOR_BANDS);
        assert!(report.min_terrain_material_weight_bands >= ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS);
        assert!(report.min_terrain_material_channels >= ISLAND_TERRAIN_MATERIAL_CHANNELS);
        assert!(report.min_terrain_material_regions >= ISLAND_TERRAIN_MATERIAL_REGIONS);
        assert!(report.min_terrain_texture_detail_bands >= ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS);
        assert!(report.min_terrain_texture_edge_promille >= ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE);
        assert!(report.min_terrain_relief_range_m >= 0.8);
        assert!(report.min_cliff_color_bands >= ISLAND_CLIFF_STRATA_BANDS / 2);
        assert!(report.min_impostor_mesh_vertices >= 2 + ISLAND_IMPOSTOR_SEGMENTS * 3);
        assert!(report.min_impostor_color_bands >= ISLAND_IMPOSTOR_COLOR_BANDS);
        assert!(launch_terrain.exists());
        assert!(launch_impostor.exists());
        assert!(launch_weights.exists());
        assert!(weights.starts_with("vertex,lush_highland,exposed_edge\n"));
        assert!(weights.lines().count() > 2000);
        assert!(manifest.contains("\"schema\": \"nau_terrain_export.v1\""));
        assert!(manifest.contains(
            "\"material_weights_csv\": \"islands/00_launch_mesa_terrain_material_weights.csv\""
        ));
        assert!(manifest.contains("\"terrain_material_weight_bands\": 36"));
        assert!(manifest.contains("\"terrain_material_regions\": 4"));
        assert!(manifest.contains("\"terrain_texture_detail_bands\": 47"));
        assert!(manifest.contains(&format!(
            "\"terrain_texture_edge_promille\": {}",
            report.min_terrain_texture_edge_promille
        )));
        assert!(manifest.contains("\"impostor_mesh_vertices\": 146"));
        assert!(manifest.contains("\"impostor_color_bands\": 21"));
        assert!(
            manifest.contains("\"impostor\": {\"obj\": \"islands/00_launch_mesa_impostor.obj\"")
        );

        remove_existing_dir(&output_dir).expect("terrain export test dir should be removable");
    }

    #[test]
    fn visual_content_export_writes_manifest_meshes_and_shape_metrics() {
        let output_dir = std::env::temp_dir().join(format!(
            "nau-visual-content-export-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        remove_existing_dir(&output_dir)
            .expect("stale visual content export dir should be removable");

        let report = export_visual_content_inspection(&output_dir)
            .expect("visual content export should succeed");
        let manifest =
            fs::read_to_string(&report.manifest_path).expect("manifest should be readable");
        let launch_ground_cover = output_dir.join("visuals/00_launch_mesa_ground_cover.obj");
        let launch_tree_trunk = output_dir.join("visuals/00_launch_mesa_launch_tree_trunk.obj");
        let launch_cloud = output_dir.join("visuals/00_launch_mesa_bank_0.obj");

        assert_eq!(
            report.ground_cover_count,
            SkyRoute::default().islands().len()
        );
        assert_eq!(
            report.ground_cover_patch_total,
            SkyRoute::default().islands().len() * GROUND_COVER_PATCHES
        );
        assert_eq!(
            report.ground_cover_blade_total,
            SkyRoute::default().islands().len()
                * GROUND_COVER_PATCHES
                * GROUND_COVER_BLADES_PER_PATCH
        );
        assert_eq!(report.tree_trunk_count, 36);
        assert_eq!(report.tree_canopy_count, 36);
        assert_eq!(report.weather_cloud_count, 30);
        assert_eq!(
            report.weather_cloud_bank_count,
            SkyRoute::default().islands().len()
        );
        assert_eq!(
            report.mesh_count,
            report.ground_cover_count + report.tree_trunk_count * 2 + report.weather_cloud_count
        );
        assert!(report.total_vertex_count > 40_000);
        assert!(report.total_triangle_count > 50_000);
        assert!(report.min_ground_cover_mesh_vertices >= 1100);
        assert!(report.min_ground_cover_blade_count >= 220);
        assert!(report.min_ground_cover_blade_height_range_m >= 0.7);
        assert!(report.min_tree_trunk_mesh_vertices >= 60);
        assert!(report.min_tree_trunk_taper_ratio >= 1.35);
        assert!(report.min_tree_branch_reach_ratio >= 1.8);
        assert!(report.min_tree_canopy_mesh_vertices >= 400);
        assert!(report.min_tree_canopy_lobe_count >= 6);
        assert!(report.min_tree_canopy_detail_card_count >= 12);
        assert!(report.min_tree_canopy_vertical_to_horizontal_ratio >= 0.45);
        assert!(report.min_weather_cloud_mesh_vertices >= 560);
        assert!(report.min_weather_cloud_lobe_count >= 6);
        assert!(report.min_weather_cloud_wisp_card_count >= 14);
        assert!(report.min_weather_cloud_bank_depth_m >= 4.0);
        assert!(report.min_weather_cloud_bank_lobe_count >= 10);
        assert_eq!(
            report.terrain_biome_palette_count,
            TERRAIN_BIOME_PALETTE_COUNT
        );
        assert_eq!(report.foliage_palette_count, TERRAIN_BIOME_PALETTE_COUNT);
        assert!(report.stone_palette_count >= TERRAIN_BIOME_PALETTE_COUNT - 1);
        assert!(launch_ground_cover.exists());
        assert!(launch_tree_trunk.exists());
        assert!(launch_cloud.exists());
        assert!(manifest.contains("\"schema\": \"nau_visual_content_export.v1\""));
        assert!(manifest.contains("\"ground_cover_blade_height_range_m\""));
        assert!(manifest.contains("\"tree_branch_reach_ratio\""));
        assert!(manifest.contains("\"weather_cloud_wisp_card_count\""));
        assert!(manifest.contains("\"terrain_biome_palette_count\": 5"));

        remove_existing_dir(&output_dir)
            .expect("visual content export test dir should be removable");
    }

    #[test]
    fn tree_trunk_mesh_is_tapered_instead_of_a_straight_cylinder() {
        let mesh = tree_trunk_mesh(0.3, 4.0, 123);
        let positions = positions(&mesh);
        let bottom_ring = &positions[..TREE_TRUNK_SEGMENTS];
        let top_ring = &positions[2 * TREE_TRUNK_SEGMENTS..3 * TREE_TRUNK_SEGMENTS];
        let branch_vertices_start = TREE_TRUNK_SEGMENTS * 3 + 2;
        let top_center = top_ring
            .iter()
            .map(|position| Vec2::new(position[0], position[2]))
            .sum::<Vec2>()
            / TREE_TRUNK_SEGMENTS as f32;
        let average_bottom_radius = bottom_ring
            .iter()
            .map(|position| Vec2::new(position[0], position[2]).length())
            .sum::<f32>()
            / TREE_TRUNK_SEGMENTS as f32;
        let average_top_radius = top_ring
            .iter()
            .map(|position| (Vec2::new(position[0], position[2]) - top_center).length())
            .sum::<f32>()
            / TREE_TRUNK_SEGMENTS as f32;
        let max_branch_reach = positions[branch_vertices_start..]
            .iter()
            .map(|position| Vec2::new(position[0], position[2]).length())
            .fold(0.0, f32::max);

        assert_eq!(
            mesh.count_vertices(),
            TREE_TRUNK_SEGMENTS * 3 + 2 + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2
        );
        assert!(
            average_bottom_radius > average_top_radius * 1.45,
            "tree trunks should taper enough to stop reading as plain cylinders"
        );
        assert!(
            max_branch_reach > average_bottom_radius * 1.8,
            "tree trunks should include visible branch mass instead of only a tapered stick"
        );
    }

    #[test]
    fn tree_trunk_cap_winding_matches_declared_normals() {
        let mesh = tree_trunk_mesh(0.3, 4.0, 123);
        let positions = positions(&mesh);
        let indices = u32_indices(&mesh);
        let cap_start = TREE_TRUNK_SEGMENTS * 12;

        for segment in 0..TREE_TRUNK_SEGMENTS {
            let bottom = &indices[cap_start + segment * 6..cap_start + segment * 6 + 3];
            let top = &indices[cap_start + segment * 6 + 3..cap_start + segment * 6 + 6];

            assert!(
                triangle_normal_y(positions, bottom) < 0.0,
                "bottom cap triangles should face downward"
            );
            assert!(
                triangle_normal_y(positions, top) > 0.0,
                "top cap triangles should face upward"
            );
        }
    }

    #[test]
    fn tree_canopy_mesh_uses_overlapping_lobes_instead_of_one_sphere() {
        let mesh = tree_canopy_mesh(1.4, 42);
        let positions = positions(&mesh);
        let single_lobe_vertices =
            (TREE_CANOPY_LATITUDE_SEGMENTS + 1) * (TREE_CANOPY_LONGITUDE_SEGMENTS + 1);
        let expected_card_vertices = TREE_CANOPY_CARD_COUNT * DETAIL_CARD_VERTICES;
        let min_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let horizontal_span = positions
            .iter()
            .map(|position| Vec2::new(position[0], position[2]).length())
            .fold(0.0, f32::max);

        assert!(mesh.count_vertices() > single_lobe_vertices * 3);
        assert!(mesh.count_vertices() >= single_lobe_vertices + expected_card_vertices);
        assert!(max_y - min_y > 1.9);
        assert!(horizontal_span > 1.45);
    }

    #[test]
    fn cloud_cluster_mesh_uses_multiple_lobes_for_depth() {
        let mesh = cloud_cluster_mesh(99, CLOUD_BANK_LOBES);
        let positions = positions(&mesh);
        let lobe_vertices = (5 + 1) * (10 + 1);
        let card_vertices = CLOUD_WISP_CARDS_PER_LOBE * DETAIL_CARD_VERTICES;
        let min_x = positions
            .iter()
            .map(|position| position[0])
            .fold(f32::INFINITY, f32::min);
        let max_x = positions
            .iter()
            .map(|position| position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_z = positions
            .iter()
            .map(|position| position[2])
            .fold(f32::INFINITY, f32::min);
        let max_z = positions
            .iter()
            .map(|position| position[2])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);

        assert_eq!(
            mesh.count_vertices(),
            CLOUD_BANK_LOBES * (lobe_vertices + card_vertices)
        );
        assert!(
            max_x - min_x > 1.2,
            "cloud clusters should have lateral lobe structure"
        );
        assert!(
            max_y - min_y > 1.0,
            "cloud clusters should stack lobes vertically instead of staying wafer-flat"
        );
        assert!(
            max_z - min_z > 0.8,
            "cloud clusters should have visible depth, not one flat blob"
        );
    }

    #[test]
    fn ground_cover_mesh_uses_dense_curved_blades() {
        let mesh = island_ground_cover_mesh(2, test_island());
        let positions = positions(&mesh);
        let indices = u32_indices(&mesh);
        let blade_count = GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH;
        let min_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);

        assert_eq!(
            mesh.count_vertices(),
            blade_count * VERTICES_PER_GROUND_BLADE
        );
        assert_eq!(indices.len(), blade_count * INDICES_PER_GROUND_BLADE);
        assert!(
            max_y - min_y > 1.0,
            "ground cover should have enough varied height to read as dense vegetation"
        );
    }

    #[test]
    fn rock_scatter_mesh_has_flattened_irregular_silhouette() {
        let mesh = rock_scatter_mesh(0.7, 1234);
        let positions = positions(&mesh);
        let indices = u32_indices(&mesh);
        let radial_lengths: Vec<f32> = positions[1..positions.len() - 1]
            .iter()
            .map(|position| Vec2::new(position[0], position[2]).length())
            .collect();
        let min_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::INFINITY, f32::min);
        let max_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_radius = radial_lengths.iter().copied().fold(f32::INFINITY, f32::min);
        let max_radius = radial_lengths
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let top_cap_start = ROCK_MESH_SEGMENTS * 3 + (ROCK_MESH_RINGS - 1) * ROCK_MESH_SEGMENTS * 6;

        assert_eq!(
            mesh.count_vertices(),
            ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2
        );
        assert!(
            max_radius - min_radius > 0.5,
            "rock scatter should have a jagged profile instead of one repeated radius"
        );
        assert!(
            max_radius > (max_y - min_y) * 0.95,
            "rock scatter should be squat and grounded rather than a sphere"
        );
        assert!(
            triangle_normal_y(positions, &indices[0..3]) < 0.0,
            "rock bottom cap should face downward"
        );
        assert!(
            triangle_normal_y(positions, &indices[top_cap_start..top_cap_start + 3]) > 0.0,
            "rock top cap should face upward"
        );
    }

    #[test]
    fn terrain_mesh_uses_high_resolution_irregular_silhouette() {
        let island = test_island();
        let mesh = island_terrain_mesh(2, island);
        let positions = positions(&mesh);
        let colors = colors(&mesh);
        let outer_ring_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;
        let outer_ring = &positions[outer_ring_start..outer_ring_start + ISLAND_BODY_SEGMENTS];
        let min_radius = outer_ring
            .iter()
            .map(|position| normalized_radius(island, *position))
            .fold(f32::INFINITY, f32::min);
        let max_radius = outer_ring
            .iter()
            .map(|position| normalized_radius(island, *position))
            .fold(f32::NEG_INFINITY, f32::max);

        assert_eq!(
            mesh.count_vertices(),
            1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS
        );
        assert!(
            max_radius <= 1.001,
            "playable terrain must stay inside the route collision footprint"
        );
        assert!(
            max_radius - min_radius > 0.10,
            "outer ring should not read as a perfect cylinder"
        );
        assert_eq!(colors.len(), positions.len());
        assert!(
            mesh_vertex_color_band_count(&mesh) >= ISLAND_TERRAIN_COLOR_BANDS,
            "terrain mesh should carry vertex-color biome/detail variation"
        );
        assert!(
            mesh_terrain_material_weight_band_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS,
            "terrain mesh should carry material-weight variation for future PBR blends"
        );
        assert!(
            mesh_terrain_material_channel_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_CHANNELS,
            "terrain mesh should expose base, lush, and edge material channels"
        );
        assert!(
            mesh_terrain_material_region_count(&mesh) >= ISLAND_TERRAIN_MATERIAL_REGIONS,
            "terrain mesh should expose distinct meadow, transition, highland, and edge regions"
        );
        let uvs = mesh_uv0(&mesh).expect("terrain mesh should expose material uvs");
        let min_u = uvs.iter().map(|uv| uv[0]).fold(f32::INFINITY, f32::min);
        let max_u = uvs.iter().map(|uv| uv[0]).fold(f32::NEG_INFINITY, f32::max);
        let min_v = uvs.iter().map(|uv| uv[1]).fold(f32::INFINITY, f32::min);
        let max_v = uvs.iter().map(|uv| uv[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!(
            max_u - min_u >= 3.0 && max_v - min_v >= 2.0,
            "terrain albedo should tile across large islands instead of stretching one texture over the whole surface"
        );
        assert!(
            mesh_y_range(&mesh) >= 0.8,
            "terrain mesh should have enough relief range to avoid flat plateaus"
        );
    }

    #[test]
    fn island_impostor_mesh_uses_layered_color_and_silhouette() {
        let island = test_island();
        let mesh = island_impostor_mesh(4, island);
        let positions = positions(&mesh);
        let colors = colors(&mesh);
        let top_ring = &positions[1..1 + ISLAND_IMPOSTOR_SEGMENTS];
        let min_radius = top_ring
            .iter()
            .map(|position| normalized_radius(island, *position))
            .fold(f32::INFINITY, f32::min);
        let max_radius = top_ring
            .iter()
            .map(|position| normalized_radius(island, *position))
            .fold(f32::NEG_INFINITY, f32::max);

        assert_eq!(mesh.count_vertices(), 2 + ISLAND_IMPOSTOR_SEGMENTS * 3);
        assert_eq!(colors.len(), positions.len());
        assert!(
            max_radius - min_radius > 0.08,
            "distant impostor should keep an irregular island silhouette"
        );
        assert!(
            mesh_y_range(&mesh) >= island.thickness * 0.85,
            "distant impostor should include a readable underside mass"
        );
        assert!(
            mesh_vertex_color_band_count(&mesh) >= ISLAND_IMPOSTOR_COLOR_BANDS,
            "distant impostor should carry terrain, cliff, and underside color variation"
        );
    }

    #[test]
    fn cliff_and_underside_meshes_replace_cylinder_body_resolution() {
        let island = test_island();
        let cliff_mesh = island_cliff_mesh(3, island);
        let underside_mesh = island_underside_mesh(3, island);
        let underside_positions = positions(&underside_mesh);
        let underside_top_radius = normalized_radius(island, underside_positions[0]);
        let underside_tip = *underside_positions.last().expect("bottom tip exists");

        assert_eq!(
            cliff_mesh.count_vertices(),
            (ISLAND_CLIFF_RINGS + 1) * ISLAND_BODY_SEGMENTS
        );
        assert_eq!(
            underside_mesh.count_vertices(),
            (ISLAND_UNDERSIDE_RINGS + 1) * ISLAND_BODY_SEGMENTS + 1
        );
        assert!(underside_top_radius > 0.55);
        assert!(normalized_radius(island, underside_tip) < 0.01);
        assert!(underside_tip[1] < island.mesh_top_y() - island.thickness * 1.5);
        assert!(
            mesh_vertex_color_band_count(&cliff_mesh) >= ISLAND_CLIFF_STRATA_BANDS,
            "cliff mesh should carry visible strata color bands"
        );
        assert!(
            mesh_vertex_color_band_count(&underside_mesh) >= ISLAND_CLIFF_STRATA_BANDS / 2,
            "underside mesh should not be one flat rock color"
        );
    }

    #[test]
    fn cliff_and_underside_share_their_transition_ring() {
        let island = test_island();
        let cliff_mesh = island_cliff_mesh(3, island);
        let underside_mesh = island_underside_mesh(3, island);
        let cliff_positions = positions(&cliff_mesh);
        let underside_positions = positions(&underside_mesh);
        let cliff_bottom_start = ISLAND_CLIFF_RINGS * ISLAND_BODY_SEGMENTS;

        for segment in 0..ISLAND_BODY_SEGMENTS {
            let cliff = Vec3::from_array(cliff_positions[cliff_bottom_start + segment]);
            let underside = Vec3::from_array(underside_positions[segment]);
            assert!(
                cliff.distance(underside) < 0.001,
                "cliff and underside should not leave a visible body seam"
            );
        }
    }

    #[test]
    fn content_diagnostics_tracks_procedural_body_complexity() {
        let mut diagnostics = IslandContentDiagnostics::default();

        diagnostics.record_procedural_island_body(ISLAND_BODY_SEGMENTS, 833);
        diagnostics.record_procedural_island_body(ISLAND_BODY_SEGMENTS, 821);
        diagnostics.record_island_terrain_surface(2305, 9, 16, 3, 4, 1.12);
        diagnostics.record_island_terrain_surface(2305, 7, 12, 3, 4, 0.92);
        diagnostics.record_terrain_material_texture_detail(72);
        diagnostics.record_terrain_material_texture_detail(64);
        diagnostics.record_island_cliff_detail(11);
        diagnostics.record_island_cliff_detail(10);
        diagnostics.record_island_impostor(146, 22);
        diagnostics.record_island_impostor(144, 19);

        assert_eq!(diagnostics.procedural_island_body_count, 2);
        assert_eq!(diagnostics.island_terrain_surface_count, 2);
        assert_eq!(diagnostics.min_island_terrain_mesh_vertices, 2305);
        assert_eq!(diagnostics.min_island_terrain_color_bands, 7);
        assert_eq!(diagnostics.min_island_terrain_material_weight_bands, 12);
        assert_eq!(diagnostics.min_island_terrain_material_channels, 3);
        assert_eq!(diagnostics.min_island_terrain_material_regions, 4);
        assert_eq!(diagnostics.min_island_terrain_texture_detail_bands, 64);
        assert_eq!(diagnostics.min_island_terrain_relief_range_m(), 0.92);
        assert_eq!(diagnostics.min_island_cliff_color_bands, 10);
        assert_eq!(diagnostics.min_island_impostor_mesh_vertices, 144);
        assert_eq!(diagnostics.min_island_impostor_color_bands, 19);
        assert_eq!(diagnostics.primitive_island_body_count, 0);
        assert_eq!(
            diagnostics.min_island_body_silhouette_segments,
            ISLAND_BODY_SEGMENTS
        );
        assert_eq!(
            diagnostics.average_island_body_silhouette_segments(),
            ISLAND_BODY_SEGMENTS as f32
        );
        assert_eq!(diagnostics.min_island_body_mesh_vertices, 821);
        assert_eq!(diagnostics.max_island_body_mesh_vertices, 833);
    }

    #[test]
    fn terrain_surface_texture_has_sharp_material_detail() {
        let data = procedural_terrain_surface_texture_data(
            [54, 128, 70, 255],
            [28, 92, 48, 255],
            [128, 174, 78, 255],
            17,
            TERRAIN_TEXTURE_SIZE,
        );

        assert_eq!(
            data.len(),
            (TERRAIN_TEXTURE_SIZE * TERRAIN_TEXTURE_SIZE * 4) as usize
        );
        assert!(
            texture_detail_band_count(&data) >= ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS,
            "terrain texture should carry enough high-frequency color bins to avoid blurry flat fills"
        );
        assert!(
            texture_edge_promille(&data, TERRAIN_TEXTURE_SIZE)
                >= ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE,
            "terrain texture should carry enough local edge contrast to avoid smeared low-frequency fills"
        );
    }

    #[test]
    fn terrain_biome_palettes_vary_base_hues() {
        let palette_keys = (0..5)
            .map(|index| {
                let grass = terrain_biome_palette(index).grass;
                [
                    (grass.x * 31.0).round() as u8,
                    (grass.y * 31.0).round() as u8,
                    (grass.z * 31.0).round() as u8,
                ]
            })
            .collect::<HashSet<_>>();

        assert_eq!(
            palette_keys.len(),
            5,
            "terrain palettes should give repeated island materials distinct base hues"
        );
    }

    #[test]
    fn terrain_vertex_colors_use_biome_palette_variation() {
        let color_keys = (0..5)
            .map(|index| {
                let color = island_terrain_vertex_color(index, 0.56, 1.2, 0.24);
                [
                    (color[0] * 31.0).round() as u8,
                    (color[1] * 31.0).round() as u8,
                    (color[2] * 31.0).round() as u8,
                ]
            })
            .collect::<HashSet<_>>();

        assert!(
            color_keys.len() >= 4,
            "same-region terrain samples should not collapse into one shared island palette"
        );
    }

    #[test]
    fn biome_detail_color_sets_vary_vegetation_and_stone_hues() {
        let foliage_keys = (0..TERRAIN_BIOME_PALETTE_COUNT)
            .map(|index| biome_detail_color_set(index).foliage_primary)
            .collect::<HashSet<_>>();
        let stone_keys = (0..TERRAIN_BIOME_PALETTE_COUNT)
            .map(|index| biome_detail_color_set(index).stone_primary)
            .collect::<HashSet<_>>();

        assert_eq!(
            foliage_keys.len(),
            TERRAIN_BIOME_PALETTE_COUNT,
            "generated tree canopies should inherit per-island biome identity"
        );
        assert!(
            stone_keys.len() >= TERRAIN_BIOME_PALETTE_COUNT - 1,
            "stone scatter should vary with the island biome instead of sharing one material"
        );
    }

    #[test]
    fn content_diagnostics_tracks_generated_tree_and_cloud_complexity() {
        let mut diagnostics = IslandContentDiagnostics::default();

        diagnostics.record_detail_biome_palette(0);
        diagnostics.record_detail_biome_palette(2);
        diagnostics.record_detail_biome_palette(2);
        diagnostics.record_generated_ground_cover(44, 220, 1100);
        diagnostics.record_generated_ground_cover(44, 220, 1100);
        diagnostics.record_generated_tree_trunk(26);
        diagnostics.record_generated_tree_trunk(30);
        diagnostics.record_generated_tree_canopy(226);
        diagnostics.record_generated_tree_canopy(240);
        diagnostics.record_generated_rock(74);
        diagnostics.record_generated_rock(80);
        diagnostics.record_generated_weather_cloud(7, 315, 4.2, true);
        diagnostics.record_generated_weather_cloud(4, 180, 0.8, false);

        assert_eq!(diagnostics.generated_ground_cover_patch_count, 88);
        assert_eq!(diagnostics.min_ground_cover_blade_count, 220);
        assert_eq!(diagnostics.min_ground_cover_mesh_vertices, 1100);
        assert_eq!(diagnostics.generated_tree_trunk_count, 2);
        assert_eq!(diagnostics.generated_tree_canopy_count, 2);
        assert_eq!(diagnostics.min_tree_trunk_mesh_vertices, 26);
        assert_eq!(diagnostics.min_tree_canopy_mesh_vertices, 226);
        assert_eq!(diagnostics.detail_biome_palette_count(), 2);
        assert_eq!(diagnostics.generated_rock_count, 2);
        assert_eq!(diagnostics.min_rock_mesh_vertices, 74);
        assert_eq!(diagnostics.generated_weather_cloud_count, 2);
        assert_eq!(diagnostics.generated_weather_cloud_bank_count, 1);
        assert_eq!(diagnostics.min_weather_cloud_bank_depth_m(), 4.2);
        assert_eq!(diagnostics.min_weather_cloud_lobe_count, 4);
        assert_eq!(diagnostics.max_weather_cloud_lobe_count, 7);
        assert_eq!(diagnostics.min_weather_cloud_mesh_vertices, 180);
    }
}
