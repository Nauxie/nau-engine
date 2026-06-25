use bevy::animation::graph::AnimationNodeIndex;
use bevy::asset::{LoadState, RenderAssetUsages};
use bevy::camera::{CameraOutputMode, ClearColorConfig, Exposure};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::ecs::system::SystemParam;
use bevy::gltf::{Gltf, GltfAssetLabel};
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::input::mouse::MouseMotion;
use bevy::light::{
    AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, DirectionalLightShadowMap,
    VolumetricFog, VolumetricLight,
};
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::render_resource::{BlendState, Extent3d, TextureDimension, TextureFormat};
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use bevy::scene::SceneInstanceReady;
use bevy::window::{CompositeAlphaMode, CursorGrabMode, CursorOptions, PrimaryWindow};
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend, wing_airflow_strength,
};
use nau_engine::asset_pipeline::{
    VISUAL_ASSET_SPECS, VisualAssetAnimationState, VisualAssetKind, VisualAssetLoadState,
    VisualAssetPipelineMetrics, VisualAssetPreloadState, VisualAssetSceneState, VisualAssetSpec,
    visual_asset_pipeline_metrics_with_preload_states,
};
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
    WindFieldKind, active_lift_fields_at, apply_aerial_power_up, apply_lift_fields,
    readable_lift_fields_at, visible_fields_at, wind_sway_motion,
};
use nau_engine::eval::{
    EvalAccumulator, EvalArtifacts, EvalObjectiveProgress, EvalSample, EvalScenario,
    SCENARIO_NAMES, scenario_named, scripted_camera_input, scripted_input,
};
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning, Velocity,
    body_yaw_error_degrees, desired_heading_alignment_speed, desired_planar_movement_direction,
    face_flight_direction, lateral_response_speed, step_flight,
};
use nau_engine::world::{
    LodBand, RouteObjectiveKind, START_POSITION, SkyIsland, SkyRoute, StreamActivation,
    TERRAIN_VISUAL_FOOTING_OFFSET_M, is_recovery_branch_island,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

const PLAYER_START: Vec3 = START_POSITION;
const WORLD_RADIUS: f32 = 920.0;
const EVAL_SCREENSHOT_TIMEOUT_FRAMES: u32 = 180;
const EVAL_FRAME_TIME_WARMUP_FRAMES: u32 = 5;
const CAMERA_MIN_SURFACE_CLEARANCE: f32 = 2.2;
const CAMERA_OBSTRUCTION_CLEARANCE: f32 = 0.45;
const CAMERA_PLAYER_FOCUS_HEIGHT: f32 = 1.4;
const ATTACHED_PLAYER_VISUAL_OFFSET_Y: f32 = -TERRAIN_VISUAL_FOOTING_OFFSET_M;
const PROCEDURAL_TEXTURE_SIZE: u32 = 64;
const TERRAIN_TEXTURE_SIZE: u32 = 128;
const TERRAIN_UV_TILES_PER_METER: f32 = 1.0 / 12.0;
const TERRAIN_BIOME_PALETTE_COUNT: usize = 5;
const INITIAL_SKY_CLEAR_COLOR: Color = Color::srgb(0.50, 0.68, 0.92);
const TREE_CANOPY_LATITUDE_SEGMENTS: usize = 6;
const TREE_CANOPY_LONGITUDE_SEGMENTS: usize = 12;
const TREE_CANOPY_CARD_COUNT: usize = 12;
const TREE_TRUNK_SEGMENTS: usize = 8;
const TREE_BRANCH_COUNT: usize = 3;
const TREE_BRANCH_SEGMENTS: usize = 6;
const ROCK_MESH_SEGMENTS: usize = 12;
const ROCK_MESH_RINGS: usize = 6;
const CLOUD_BANK_LOBES: usize = 14;
const CLOUD_VEIL_LOBES: usize = 7;
const CLOUD_WISP_CARDS_PER_LOBE: usize = 2;
const GROUND_COVER_PATCHES: usize = 44;
const GROUND_COVER_BLADES_PER_PATCH: usize = 5;
#[cfg(test)]
const DETAIL_CARD_VERTICES: usize = 8;
const VERTICES_PER_GROUND_BLADE: usize = 5;
const INDICES_PER_GROUND_BLADE: usize = 9;
const AUTHORED_WORLD_FIXTURE_KINDS: &[VisualAssetKind] = &[
    VisualAssetKind::IslandTerrain,
    VisualAssetKind::IslandFoliage,
    VisualAssetKind::IslandRock,
    VisualAssetKind::IslandWater,
    VisualAssetKind::RouteMarker,
    VisualAssetKind::WeatherLayer,
    VisualAssetKind::DistantImpostor,
];
#[cfg(test)]
const ISLAND_TERRAIN_COLOR_BANDS: usize = 5;
#[cfg(test)]
const ISLAND_TERRAIN_MATERIAL_WEIGHT_BANDS: usize = 12;
#[cfg(test)]
const ISLAND_TERRAIN_MATERIAL_CHANNELS: usize = 3;
#[cfg(test)]
const ISLAND_TERRAIN_MATERIAL_REGIONS: usize = 4;
#[cfg(test)]
const ISLAND_TERRAIN_TEXTURE_DETAIL_BANDS: usize = 44;
#[cfg(test)]
const ISLAND_TERRAIN_TEXTURE_EDGE_PROMILLE: usize = 240;
#[cfg(test)]
const ISLAND_IMPOSTOR_COLOR_BANDS: usize = 18;
const ISLAND_CLIFF_STRATA_BANDS: usize = 9;

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

#[derive(Resource, Debug)]
struct VisualAssetRegistry {
    slots: Vec<VisualAssetSlot>,
}

impl VisualAssetRegistry {
    fn scene_handle(&self, kind: VisualAssetKind) -> Option<Handle<Scene>> {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == kind)
            .and_then(|slot| slot.scene_handle.clone())
    }

    fn mark_scene_spawned(&mut self, kind: VisualAssetKind, entity: Entity) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.scene_entity = Some(entity);
        }
    }

    fn mark_scene_ready(&mut self, kind: VisualAssetKind) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.scene_ready = true;
        }
    }

    fn scene_ready(&self, kind: VisualAssetKind) -> bool {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == kind)
            .is_some_and(|slot| slot.scene_ready)
    }

    fn mark_animation_player_linked(
        &mut self,
        kind: VisualAssetKind,
        entity: Entity,
        ready_clip_count: usize,
    ) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.animation_player_entity = Some(entity);
            slot.ready_animation_clip_count =
                ready_clip_count.min(slot.spec.animation_clip_names.len());
        }
    }

    fn mark_animation_graph_ready(
        &mut self,
        kind: VisualAssetKind,
        entity: Entity,
        ready_clip_count: usize,
    ) {
        self.mark_animation_player_linked(kind, entity, ready_clip_count);
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.animation_graph_ready = true;
        }
    }

    fn pending_animation_links(&self) -> Vec<PendingAuthoredAnimationLink> {
        self.slots
            .iter()
            .filter(|slot| {
                slot.scene_ready
                    && !slot.animation_graph_ready
                    && !slot.spec.animation_clip_names.is_empty()
            })
            .filter_map(|slot| {
                Some(PendingAuthoredAnimationLink {
                    kind: slot.spec.kind,
                    spec: slot.spec,
                    scene_entity: slot.scene_entity?,
                    gltf_handle: slot.gltf_handle.clone()?,
                })
            })
            .collect()
    }

    fn scene_state_for(&self, spec: &VisualAssetSpec) -> VisualAssetSceneState {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == spec.kind)
            .map_or(VisualAssetSceneState::NotSpawned, |slot| {
                if slot.scene_ready {
                    VisualAssetSceneState::Ready
                } else if slot.scene_entity.is_some() {
                    VisualAssetSceneState::Spawned
                } else {
                    VisualAssetSceneState::NotSpawned
                }
            })
    }

    fn animation_state_for(&self, spec: &VisualAssetSpec) -> VisualAssetAnimationState {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == spec.kind)
            .map_or(VisualAssetAnimationState::default(), |slot| {
                VisualAssetAnimationState {
                    ready_clip_count: slot.ready_animation_clip_count,
                    animation_player_linked: slot.animation_player_entity.is_some(),
                    animation_graph_ready: slot.animation_graph_ready,
                }
            })
    }
}

#[derive(Debug)]
struct VisualAssetSlot {
    spec: VisualAssetSpec,
    gltf_handle: Option<Handle<Gltf>>,
    scene_handle: Option<Handle<Scene>>,
    scene_entity: Option<Entity>,
    scene_ready: bool,
    animation_player_entity: Option<Entity>,
    ready_animation_clip_count: usize,
    animation_graph_ready: bool,
}

#[derive(Clone)]
struct PendingAuthoredAnimationLink {
    kind: VisualAssetKind,
    spec: VisualAssetSpec,
    scene_entity: Entity,
    gltf_handle: Handle<Gltf>,
}

#[derive(Debug)]
struct NamedAnimationClipResolution {
    clips: Vec<Handle<AnimationClip>>,
    expected_clip_count: usize,
    missing_clip_names: Vec<&'static str>,
}

impl NamedAnimationClipResolution {
    fn ready_clip_count(&self) -> usize {
        self.clips.len()
    }

    fn is_complete(&self) -> bool {
        self.ready_clip_count() == self.expected_clip_count && self.missing_clip_names.is_empty()
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
struct VisualAssetDiagnostics {
    metrics: VisualAssetPipelineMetrics,
    visible_world_fixture_count: usize,
}

#[derive(Component, Clone, Copy, Debug)]
struct AuthoredVisualScene {
    kind: VisualAssetKind,
    role: AuthoredVisualSceneRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthoredVisualSceneRole {
    PlayerRuntime,
    GliderRuntime,
    WorldFixture,
}

#[derive(Component, Clone, Copy, Debug)]
struct VisibleAuthoredWorldFixture {
    kind: VisualAssetKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthoredPlayerClip {
    Idle,
    Jog,
    Launch,
    Glide,
    AirBrake,
    Land,
}

impl AuthoredPlayerClip {
    fn index(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Jog => 1,
            Self::Launch => 2,
            Self::Glide => 3,
            Self::AirBrake => 4,
            Self::Land => 5,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct AuthoredPlayerAnimation {
    nodes: [AnimationNodeIndex; 6],
    current: AuthoredPlayerClip,
}

impl AuthoredPlayerAnimation {
    fn new(nodes: [AnimationNodeIndex; 6], current: AuthoredPlayerClip) -> Self {
        Self { nodes, current }
    }

    fn node(self, clip: AuthoredPlayerClip) -> AnimationNodeIndex {
        self.nodes[clip.index()]
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct GeneratedPlayerPlaceholder;

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

#[derive(Clone, Debug)]
struct EvalOptions {
    scenario: EvalScenario,
    output_dir: PathBuf,
    capture_screenshot: bool,
}

#[derive(Clone, Debug)]
enum CliAction {
    Run { eval: Option<Box<EvalOptions>> },
    ExportTerrain { output_dir: PathBuf },
    ExportVisualContent { output_dir: PathBuf },
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

#[derive(Resource, Clone, Copy, Debug, Default)]
struct EvalMovementBasis {
    frame: u32,
    facing: Option<Facing>,
}

#[derive(Debug)]
struct EvalCheckpointCapture {
    frame: u32,
    name: &'static str,
    path: PathBuf,
    marker_metadata_path: PathBuf,
    captured: bool,
    marker_metadata_written: bool,
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
                    name: checkpoint.name,
                    path: checkpoint_dir
                        .join(format!("{:04}_{}.png", checkpoint.frame, checkpoint.name)),
                    marker_metadata_path: checkpoint_dir.join(format!(
                        "{:04}_{}.markers.json",
                        checkpoint.frame, checkpoint.name
                    )),
                    captured: false,
                    marker_metadata_written: false,
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
            checkpoint_marker_metadata: self
                .checkpoint_captures
                .iter()
                .map(|checkpoint| path_string(&checkpoint.marker_metadata_path))
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
    let mut export_terrain_output = None;
    let mut export_visual_content_output = None;
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
        } else if arg == "--export-terrain" {
            export_terrain_output =
                Some(PathBuf::from(args.next().ok_or_else(|| {
                    "--export-terrain requires an output directory".to_string()
                })?));
        } else if let Some(value) = arg.strip_prefix("--export-terrain=") {
            export_terrain_output = Some(PathBuf::from(value));
        } else if arg == "--export-visual-content" {
            export_visual_content_output = Some(PathBuf::from(args.next().ok_or_else(|| {
                "--export-visual-content requires an output directory".to_string()
            })?));
        } else if let Some(value) = arg.strip_prefix("--export-visual-content=") {
            export_visual_content_output = Some(PathBuf::from(value));
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }

    if export_terrain_output.is_some() && export_visual_content_output.is_some() {
        return Err("--export-terrain cannot be combined with --export-visual-content".to_string());
    }

    if let Some(output_dir) = export_terrain_output {
        if saw_eval {
            return Err("--export-terrain cannot be combined with --eval".to_string());
        }
        return Ok(CliAction::ExportTerrain { output_dir });
    }
    if let Some(output_dir) = export_visual_content_output {
        if saw_eval {
            return Err("--export-visual-content cannot be combined with --eval".to_string());
        }
        return Ok(CliAction::ExportVisualContent { output_dir });
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
        "Usage:\n  cargo run\n  cargo run -- --eval <scenario> [--eval-output <dir>] [--eval-no-screenshot]\n  cargo run -- --export-terrain <dir>\n  cargo run -- --export-visual-content <dir>\n\nScenarios: {}",
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

#[derive(Debug)]
struct TerrainExportReport {
    manifest_path: PathBuf,
    island_count: usize,
    mesh_count: usize,
    total_vertex_count: usize,
    total_triangle_count: usize,
    min_terrain_mesh_vertices: usize,
    min_terrain_color_bands: usize,
    min_terrain_material_weight_bands: usize,
    min_terrain_material_channels: usize,
    min_terrain_material_regions: usize,
    min_terrain_texture_detail_bands: usize,
    min_terrain_texture_edge_promille: usize,
    min_terrain_relief_range_m: f32,
    min_cliff_color_bands: usize,
    min_impostor_mesh_vertices: usize,
    min_impostor_color_bands: usize,
    islands: Vec<TerrainExportIslandSummary>,
}

#[derive(Debug)]
struct TerrainExportIslandSummary {
    index: usize,
    island: SkyIsland,
    slug: String,
    terrain: TerrainExportMeshSummary,
    cliff: TerrainExportMeshSummary,
    underside: TerrainExportMeshSummary,
    impostor: TerrainExportMeshSummary,
}

#[derive(Debug)]
struct TerrainExportMeshSummary {
    obj_path: PathBuf,
    material_weights_path: Option<PathBuf>,
    vertex_count: usize,
    triangle_count: usize,
    color_bands: usize,
    material_weight_bands: usize,
    material_channels: usize,
    material_regions: usize,
    relief_range_m: f32,
}

#[derive(Debug)]
struct VisualContentExportReport {
    manifest_path: PathBuf,
    mesh_count: usize,
    total_vertex_count: usize,
    total_triangle_count: usize,
    ground_cover_count: usize,
    ground_cover_patch_total: usize,
    ground_cover_blade_total: usize,
    tree_trunk_count: usize,
    tree_canopy_count: usize,
    weather_cloud_count: usize,
    weather_cloud_bank_count: usize,
    min_ground_cover_mesh_vertices: usize,
    min_ground_cover_blade_count: usize,
    min_ground_cover_blade_height_range_m: f32,
    min_tree_trunk_mesh_vertices: usize,
    min_tree_trunk_taper_ratio: f32,
    min_tree_branch_reach_ratio: f32,
    min_tree_canopy_mesh_vertices: usize,
    min_tree_canopy_lobe_count: usize,
    min_tree_canopy_detail_card_count: usize,
    min_tree_canopy_vertical_to_horizontal_ratio: f32,
    min_weather_cloud_mesh_vertices: usize,
    min_weather_cloud_lobe_count: usize,
    min_weather_cloud_wisp_card_count: usize,
    min_weather_cloud_bank_depth_m: f32,
    min_weather_cloud_bank_lobe_count: usize,
    terrain_biome_palette_count: usize,
    foliage_palette_count: usize,
    stone_palette_count: usize,
    ground_cover: Vec<VisualGroundCoverSummary>,
    trees: Vec<VisualTreeSummary>,
    clouds: Vec<VisualCloudSummary>,
    palettes: Vec<VisualPaletteSummary>,
}

#[derive(Debug)]
struct VisualMeshSummary {
    obj_path: PathBuf,
    vertex_count: usize,
    triangle_count: usize,
    horizontal_span_m: f32,
    vertical_span_m: f32,
    depth_span_m: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct GroundCoverBladeStats {
    blade_count: usize,
    min_height_m: f32,
    max_height_m: f32,
    height_range_m: f32,
}

#[derive(Debug)]
struct VisualGroundCoverSummary {
    island_name: &'static str,
    island_slug: String,
    mesh: VisualMeshSummary,
    patch_count: usize,
    blade_count: usize,
    min_blade_height_m: f32,
    max_blade_height_m: f32,
    blade_height_range_m: f32,
}

#[derive(Debug)]
struct VisualTreeSummary {
    island_name: &'static str,
    label: String,
    trunk: VisualMeshSummary,
    canopy: VisualMeshSummary,
    trunk_height_m: f32,
    canopy_radius_m: f32,
    trunk_taper_ratio: f32,
    branch_reach_ratio: f32,
    canopy_lobe_count: usize,
    canopy_detail_card_count: usize,
    canopy_vertical_to_horizontal_ratio: f32,
}

#[derive(Debug)]
struct VisualCloudSummary {
    island_name: &'static str,
    kind: &'static str,
    bank: bool,
    mesh: VisualMeshSummary,
    lobe_count: usize,
    wisp_card_count: usize,
    scaled_horizontal_span_m: f32,
    scaled_vertical_depth_m: f32,
    scaled_depth_span_m: f32,
}

#[derive(Debug)]
struct VisualPaletteSummary {
    index: usize,
    terrain_key: [u8; 3],
    foliage_key: [u8; 3],
    stone_key: [u8; 3],
}

impl TerrainExportReport {
    fn to_json(&self) -> String {
        let islands = self
            .islands
            .iter()
            .map(|island| island.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            concat!(
                "{{\n",
                "  \"schema\": \"nau_terrain_export.v1\",\n",
                "  \"island_count\": {},\n",
                "  \"mesh_count\": {},\n",
                "  \"total_vertex_count\": {},\n",
                "  \"total_triangle_count\": {},\n",
                "  \"minimums\": {{\n",
                "    \"terrain_mesh_vertices\": {},\n",
                "    \"terrain_color_bands\": {},\n",
                "    \"terrain_material_weight_bands\": {},\n",
                "    \"terrain_material_channels\": {},\n",
                "    \"terrain_material_regions\": {},\n",
                "    \"terrain_texture_detail_bands\": {},\n",
                "    \"terrain_texture_edge_promille\": {},\n",
                "    \"terrain_relief_range_m\": {},\n",
                "    \"cliff_color_bands\": {},\n",
                "    \"impostor_mesh_vertices\": {},\n",
                "    \"impostor_color_bands\": {}\n",
                "  }},\n",
                "  \"islands\": [\n",
                "{}\n",
                "  ]\n",
                "}}\n"
            ),
            self.island_count,
            self.mesh_count,
            self.total_vertex_count,
            self.total_triangle_count,
            self.min_terrain_mesh_vertices,
            self.min_terrain_color_bands,
            self.min_terrain_material_weight_bands,
            self.min_terrain_material_channels,
            self.min_terrain_material_regions,
            self.min_terrain_texture_detail_bands,
            self.min_terrain_texture_edge_promille,
            terrain_export_json_number(self.min_terrain_relief_range_m),
            self.min_cliff_color_bands,
            self.min_impostor_mesh_vertices,
            self.min_impostor_color_bands,
            islands
        )
    }
}

impl VisualContentExportReport {
    fn to_json(&self) -> String {
        let ground_cover = self
            .ground_cover
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let trees = self
            .trees
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let clouds = self
            .clouds
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");
        let palettes = self
            .palettes
            .iter()
            .map(|summary| summary.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            concat!(
                "{{\n",
                "  \"schema\": \"nau_visual_content_export.v1\",\n",
                "  \"mesh_count\": {},\n",
                "  \"total_vertex_count\": {},\n",
                "  \"total_triangle_count\": {},\n",
                "  \"counts\": {{\n",
                "    \"ground_cover_count\": {},\n",
                "    \"ground_cover_patch_total\": {},\n",
                "    \"ground_cover_blade_total\": {},\n",
                "    \"tree_trunk_count\": {},\n",
                "    \"tree_canopy_count\": {},\n",
                "    \"weather_cloud_count\": {},\n",
                "    \"weather_cloud_bank_count\": {}\n",
                "  }},\n",
                "  \"minimums\": {{\n",
                "    \"ground_cover_mesh_vertices\": {},\n",
                "    \"ground_cover_blade_count\": {},\n",
                "    \"ground_cover_blade_height_range_m\": {},\n",
                "    \"tree_trunk_mesh_vertices\": {},\n",
                "    \"tree_trunk_taper_ratio\": {},\n",
                "    \"tree_branch_reach_ratio\": {},\n",
                "    \"tree_canopy_mesh_vertices\": {},\n",
                "    \"tree_canopy_lobe_count\": {},\n",
                "    \"tree_canopy_detail_card_count\": {},\n",
                "    \"tree_canopy_vertical_to_horizontal_ratio\": {},\n",
                "    \"weather_cloud_mesh_vertices\": {},\n",
                "    \"weather_cloud_lobe_count\": {},\n",
                "    \"weather_cloud_wisp_card_count\": {},\n",
                "    \"weather_cloud_bank_depth_m\": {},\n",
                "    \"weather_cloud_bank_lobe_count\": {},\n",
                "    \"terrain_biome_palette_count\": {},\n",
                "    \"foliage_palette_count\": {},\n",
                "    \"stone_palette_count\": {}\n",
                "  }},\n",
                "  \"ground_cover\": [\n",
                "{}\n",
                "  ],\n",
                "  \"trees\": [\n",
                "{}\n",
                "  ],\n",
                "  \"clouds\": [\n",
                "{}\n",
                "  ],\n",
                "  \"palettes\": [\n",
                "{}\n",
                "  ]\n",
                "}}\n"
            ),
            self.mesh_count,
            self.total_vertex_count,
            self.total_triangle_count,
            self.ground_cover_count,
            self.ground_cover_patch_total,
            self.ground_cover_blade_total,
            self.tree_trunk_count,
            self.tree_canopy_count,
            self.weather_cloud_count,
            self.weather_cloud_bank_count,
            self.min_ground_cover_mesh_vertices,
            self.min_ground_cover_blade_count,
            terrain_export_json_number(self.min_ground_cover_blade_height_range_m),
            self.min_tree_trunk_mesh_vertices,
            terrain_export_json_number(self.min_tree_trunk_taper_ratio),
            terrain_export_json_number(self.min_tree_branch_reach_ratio),
            self.min_tree_canopy_mesh_vertices,
            self.min_tree_canopy_lobe_count,
            self.min_tree_canopy_detail_card_count,
            terrain_export_json_number(self.min_tree_canopy_vertical_to_horizontal_ratio),
            self.min_weather_cloud_mesh_vertices,
            self.min_weather_cloud_lobe_count,
            self.min_weather_cloud_wisp_card_count,
            terrain_export_json_number(self.min_weather_cloud_bank_depth_m),
            self.min_weather_cloud_bank_lobe_count,
            self.terrain_biome_palette_count,
            self.foliage_palette_count,
            self.stone_palette_count,
            ground_cover,
            trees,
            clouds,
            palettes
        )
    }
}

impl VisualMeshSummary {
    fn to_json(&self) -> String {
        format!(
            concat!(
                "{{\"obj\": {}, \"vertex_count\": {}, \"triangle_count\": {}, ",
                "\"horizontal_span_m\": {}, \"vertical_span_m\": {}, \"depth_span_m\": {}}}"
            ),
            terrain_export_json_string(&path_string(&self.obj_path)),
            self.vertex_count,
            self.triangle_count,
            terrain_export_json_number(self.horizontal_span_m),
            terrain_export_json_number(self.vertical_span_m),
            terrain_export_json_number(self.depth_span_m)
        )
    }
}

impl VisualGroundCoverSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"island_slug\": {},\n\
             {indent}  \"mesh\": {},\n\
             {indent}  \"patch_count\": {},\n\
             {indent}  \"blade_count\": {},\n\
             {indent}  \"min_blade_height_m\": {},\n\
             {indent}  \"max_blade_height_m\": {},\n\
             {indent}  \"blade_height_range_m\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(&self.island_slug),
            self.mesh.to_json(),
            self.patch_count,
            self.blade_count,
            terrain_export_json_number(self.min_blade_height_m),
            terrain_export_json_number(self.max_blade_height_m),
            terrain_export_json_number(self.blade_height_range_m)
        )
    }
}

impl VisualTreeSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"label\": {},\n\
             {indent}  \"trunk\": {},\n\
             {indent}  \"canopy\": {},\n\
             {indent}  \"trunk_height_m\": {},\n\
             {indent}  \"canopy_radius_m\": {},\n\
             {indent}  \"trunk_taper_ratio\": {},\n\
             {indent}  \"branch_reach_ratio\": {},\n\
             {indent}  \"canopy_lobe_count\": {},\n\
             {indent}  \"canopy_detail_card_count\": {},\n\
             {indent}  \"canopy_vertical_to_horizontal_ratio\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(&self.label),
            self.trunk.to_json(),
            self.canopy.to_json(),
            terrain_export_json_number(self.trunk_height_m),
            terrain_export_json_number(self.canopy_radius_m),
            terrain_export_json_number(self.trunk_taper_ratio),
            terrain_export_json_number(self.branch_reach_ratio),
            self.canopy_lobe_count,
            self.canopy_detail_card_count,
            terrain_export_json_number(self.canopy_vertical_to_horizontal_ratio)
        )
    }
}

impl VisualCloudSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island\": {},\n\
             {indent}  \"kind\": {},\n\
             {indent}  \"bank\": {},\n\
             {indent}  \"mesh\": {},\n\
             {indent}  \"lobe_count\": {},\n\
             {indent}  \"wisp_card_count\": {},\n\
             {indent}  \"scaled_horizontal_span_m\": {},\n\
             {indent}  \"scaled_vertical_depth_m\": {},\n\
             {indent}  \"scaled_depth_span_m\": {}\n\
             {indent}}}",
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(self.kind),
            self.bank,
            self.mesh.to_json(),
            self.lobe_count,
            self.wisp_card_count,
            terrain_export_json_number(self.scaled_horizontal_span_m),
            terrain_export_json_number(self.scaled_vertical_depth_m),
            terrain_export_json_number(self.scaled_depth_span_m)
        )
    }
}

impl VisualPaletteSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\"index\": {}, \"terrain_key\": {}, \"foliage_key\": {}, \"stone_key\": {}}}",
            self.index,
            visual_content_json_u8_triplet(self.terrain_key),
            visual_content_json_u8_triplet(self.foliage_key),
            visual_content_json_u8_triplet(self.stone_key)
        )
    }
}

impl TerrainExportIslandSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"index\": {},\n\
             {indent}  \"name\": {},\n\
             {indent}  \"slug\": {},\n\
             {indent}  \"center\": {},\n\
             {indent}  \"half_extents\": {},\n\
             {indent}  \"thickness_m\": {},\n\
             {indent}  \"target\": {},\n\
             {indent}  \"terrain\": {},\n\
             {indent}  \"cliff\": {},\n\
             {indent}  \"underside\": {},\n\
             {indent}  \"impostor\": {}\n\
             {indent}}}",
            self.index,
            terrain_export_json_string(self.island.name),
            terrain_export_json_string(&self.slug),
            terrain_export_json_vec3(self.island.center),
            terrain_export_json_vec2(self.island.half_extents),
            terrain_export_json_number(self.island.thickness),
            self.island.is_target,
            self.terrain.to_json(),
            self.cliff.to_json(),
            self.underside.to_json(),
            self.impostor.to_json(),
        )
    }
}

impl TerrainExportMeshSummary {
    fn to_json(&self) -> String {
        let material_weights_path = self
            .material_weights_path
            .as_deref()
            .map(|path| terrain_export_json_string(&path_string(path)))
            .unwrap_or_else(|| "null".to_string());

        format!(
            concat!(
                "{{\"obj\": {}, \"material_weights_csv\": {}, ",
                "\"vertex_count\": {}, \"triangle_count\": {}, ",
                "\"color_bands\": {}, \"material_weight_bands\": {}, ",
                "\"material_channels\": {}, \"material_regions\": {}, \"relief_range_m\": {}}}"
            ),
            terrain_export_json_string(&path_string(&self.obj_path)),
            material_weights_path,
            self.vertex_count,
            self.triangle_count,
            self.color_bands,
            self.material_weight_bands,
            self.material_channels,
            self.material_regions,
            terrain_export_json_number(self.relief_range_m)
        )
    }
}

fn export_terrain_inspection(output_dir: &Path) -> std::io::Result<TerrainExportReport> {
    fs::create_dir_all(output_dir)?;
    let islands_dir = output_dir.join("islands");
    remove_existing_dir(&islands_dir)?;
    fs::create_dir_all(&islands_dir)?;

    let route = SkyRoute::default();
    let mut islands = Vec::with_capacity(route.islands().len());

    for (index, island) in route.islands().iter().copied().enumerate() {
        let slug = terrain_export_slug(island.name);
        let prefix = format!("{index:02}_{slug}");
        let terrain_mesh = island_terrain_mesh(index, island);
        let cliff_mesh = island_cliff_mesh(index, island);
        let underside_mesh = island_underside_mesh(index, island);
        let impostor_mesh = island_impostor_mesh(index, island);

        let terrain_obj = PathBuf::from("islands").join(format!("{prefix}_terrain.obj"));
        let terrain_material_weights =
            PathBuf::from("islands").join(format!("{prefix}_terrain_material_weights.csv"));
        let cliff_obj = PathBuf::from("islands").join(format!("{prefix}_cliff.obj"));
        let underside_obj = PathBuf::from("islands").join(format!("{prefix}_underside.obj"));
        let impostor_obj = PathBuf::from("islands").join(format!("{prefix}_impostor.obj"));

        write_mesh_obj(&output_dir.join(&terrain_obj), &terrain_mesh, "terrain")?;
        write_terrain_material_weights_csv(
            &output_dir.join(&terrain_material_weights),
            &terrain_mesh,
        )?;
        write_mesh_obj(&output_dir.join(&cliff_obj), &cliff_mesh, "cliff")?;
        write_mesh_obj(
            &output_dir.join(&underside_obj),
            &underside_mesh,
            "underside",
        )?;
        write_mesh_obj(&output_dir.join(&impostor_obj), &impostor_mesh, "impostor")?;

        islands.push(TerrainExportIslandSummary {
            index,
            island,
            slug,
            terrain: terrain_export_mesh_summary(
                terrain_obj,
                Some(terrain_material_weights),
                &terrain_mesh,
            ),
            cliff: terrain_export_mesh_summary(cliff_obj, None, &cliff_mesh),
            underside: terrain_export_mesh_summary(underside_obj, None, &underside_mesh),
            impostor: terrain_export_mesh_summary(impostor_obj, None, &impostor_mesh),
        });
    }

    let island_count = islands.len();
    let mesh_count = island_count * 4;
    let total_vertex_count = islands
        .iter()
        .map(|island| {
            island.terrain.vertex_count
                + island.cliff.vertex_count
                + island.underside.vertex_count
                + island.impostor.vertex_count
        })
        .sum();
    let total_triangle_count = islands
        .iter()
        .map(|island| {
            island.terrain.triangle_count
                + island.cliff.triangle_count
                + island.underside.triangle_count
                + island.impostor.triangle_count
        })
        .sum();
    let min_terrain_mesh_vertices = islands
        .iter()
        .map(|island| island.terrain.vertex_count)
        .min()
        .unwrap_or(0);
    let min_terrain_color_bands = islands
        .iter()
        .map(|island| island.terrain.color_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_material_weight_bands = islands
        .iter()
        .map(|island| island.terrain.material_weight_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_material_channels = islands
        .iter()
        .map(|island| island.terrain.material_channels)
        .min()
        .unwrap_or(0);
    let min_terrain_material_regions = islands
        .iter()
        .map(|island| island.terrain.material_regions)
        .min()
        .unwrap_or(0);
    let min_terrain_texture_detail_bands = terrain_export_texture_detail_band_floor();
    let min_terrain_texture_edge_promille = terrain_export_texture_edge_promille_floor();
    let min_terrain_relief_range_m = islands
        .iter()
        .map(|island| island.terrain.relief_range_m)
        .min_by(f32::total_cmp)
        .unwrap_or(0.0);
    let min_cliff_color_bands = islands
        .iter()
        .flat_map(|island| [island.cliff.color_bands, island.underside.color_bands])
        .min()
        .unwrap_or(0);
    let min_impostor_mesh_vertices = islands
        .iter()
        .map(|island| island.impostor.vertex_count)
        .min()
        .unwrap_or(0);
    let min_impostor_color_bands = islands
        .iter()
        .map(|island| island.impostor.color_bands)
        .min()
        .unwrap_or(0);

    let manifest_path = output_dir.join("manifest.json");
    let report = TerrainExportReport {
        manifest_path,
        island_count,
        mesh_count,
        total_vertex_count,
        total_triangle_count,
        min_terrain_mesh_vertices,
        min_terrain_color_bands,
        min_terrain_material_weight_bands,
        min_terrain_material_channels,
        min_terrain_material_regions,
        min_terrain_texture_detail_bands,
        min_terrain_texture_edge_promille,
        min_terrain_relief_range_m,
        min_cliff_color_bands,
        min_impostor_mesh_vertices,
        min_impostor_color_bands,
        islands,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
}

fn export_visual_content_inspection(
    output_dir: &Path,
) -> std::io::Result<VisualContentExportReport> {
    fs::create_dir_all(output_dir)?;
    let visuals_dir = output_dir.join("visuals");
    remove_existing_dir(&visuals_dir)?;
    fs::create_dir_all(&visuals_dir)?;

    let route = SkyRoute::default();
    let mut ground_cover = Vec::with_capacity(route.islands().len());
    let mut trees = Vec::new();
    let mut clouds = Vec::new();

    for (island_index, island) in route.islands().iter().copied().enumerate() {
        let island_slug = terrain_export_slug(island.name);
        let ground_mesh = island_ground_cover_mesh(island_index, island);
        let ground_obj = PathBuf::from("visuals")
            .join(format!("{island_index:02}_{island_slug}_ground_cover.obj"));
        write_mesh_obj(&output_dir.join(&ground_obj), &ground_mesh, "ground cover")?;
        let blade_stats = ground_cover_blade_stats(&ground_mesh);
        ground_cover.push(VisualGroundCoverSummary {
            island_name: island.name,
            island_slug: island_slug.clone(),
            mesh: visual_content_mesh_summary(ground_obj, &ground_mesh),
            patch_count: GROUND_COVER_PATCHES,
            blade_count: blade_stats.blade_count,
            min_blade_height_m: blade_stats.min_height_m,
            max_blade_height_m: blade_stats.max_height_m,
            blade_height_range_m: blade_stats.height_range_m,
        });

        for tree in visual_content_tree_specs(island_index, island) {
            let tree_slug = terrain_export_slug(&tree.label);
            let trunk_mesh = tree_trunk_mesh(tree.trunk_radius_m, tree.trunk_height_m, tree.seed);
            let canopy_mesh = tree_canopy_mesh(tree.canopy_radius_m, tree.canopy_seed);
            let trunk_obj = PathBuf::from("visuals").join(format!(
                "{island_index:02}_{island_slug}_{tree_slug}_trunk.obj"
            ));
            let canopy_obj = PathBuf::from("visuals").join(format!(
                "{island_index:02}_{island_slug}_{tree_slug}_canopy.obj"
            ));
            write_mesh_obj(&output_dir.join(&trunk_obj), &trunk_mesh, "tree trunk")?;
            write_mesh_obj(&output_dir.join(&canopy_obj), &canopy_mesh, "tree canopy")?;

            let (trunk_taper_ratio, branch_reach_ratio) = tree_trunk_shape_metrics(&trunk_mesh);
            let trunk = visual_content_mesh_summary(trunk_obj, &trunk_mesh);
            let canopy = visual_content_mesh_summary(canopy_obj, &canopy_mesh);
            let canopy_horizontal_span = canopy.horizontal_span_m.max(canopy.depth_span_m);
            let canopy_vertical_to_horizontal_ratio =
                finite_ratio(canopy.vertical_span_m, canopy_horizontal_span);

            trees.push(VisualTreeSummary {
                island_name: island.name,
                label: tree.label,
                trunk,
                canopy,
                trunk_height_m: tree.trunk_height_m,
                canopy_radius_m: tree.canopy_radius_m,
                trunk_taper_ratio,
                branch_reach_ratio,
                canopy_lobe_count: tree_canopy_lobe_count(),
                canopy_detail_card_count: TREE_CANOPY_CARD_COUNT,
                canopy_vertical_to_horizontal_ratio,
            });
        }

        clouds.extend(visual_content_cloud_summaries(
            output_dir,
            island_index,
            island,
            &island_slug,
        )?);
    }

    let palettes = (0..TERRAIN_BIOME_PALETTE_COUNT)
        .map(visual_content_palette_summary)
        .collect::<Vec<_>>();
    let terrain_biome_palette_count = palettes
        .iter()
        .map(|palette| palette.terrain_key)
        .collect::<HashSet<_>>()
        .len();
    let foliage_palette_count = palettes
        .iter()
        .map(|palette| palette.foliage_key)
        .collect::<HashSet<_>>()
        .len();
    let stone_palette_count = palettes
        .iter()
        .map(|palette| palette.stone_key)
        .collect::<HashSet<_>>()
        .len();

    let mesh_count = ground_cover.len() + trees.len() * 2 + clouds.len();
    let total_vertex_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.vertex_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.vertex_count, summary.canopy.vertex_count]),
        )
        .chain(clouds.iter().map(|summary| summary.mesh.vertex_count))
        .sum();
    let total_triangle_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.triangle_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.triangle_count, summary.canopy.triangle_count]),
        )
        .chain(clouds.iter().map(|summary| summary.mesh.triangle_count))
        .sum();

    let manifest_path = output_dir.join("manifest.json");
    let report = VisualContentExportReport {
        manifest_path,
        mesh_count,
        total_vertex_count,
        total_triangle_count,
        ground_cover_count: ground_cover.len(),
        ground_cover_patch_total: ground_cover.iter().map(|summary| summary.patch_count).sum(),
        ground_cover_blade_total: ground_cover.iter().map(|summary| summary.blade_count).sum(),
        tree_trunk_count: trees.len(),
        tree_canopy_count: trees.len(),
        weather_cloud_count: clouds.len(),
        weather_cloud_bank_count: clouds.iter().filter(|summary| summary.bank).count(),
        min_ground_cover_mesh_vertices: ground_cover
            .iter()
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_ground_cover_blade_count: ground_cover
            .iter()
            .map(|summary| summary.blade_count)
            .min()
            .unwrap_or(0),
        min_ground_cover_blade_height_range_m: min_finite_f32(
            ground_cover
                .iter()
                .map(|summary| summary.blade_height_range_m),
        ),
        min_tree_trunk_mesh_vertices: trees
            .iter()
            .map(|summary| summary.trunk.vertex_count)
            .min()
            .unwrap_or(0),
        min_tree_trunk_taper_ratio: min_finite_f32(
            trees.iter().map(|summary| summary.trunk_taper_ratio),
        ),
        min_tree_branch_reach_ratio: min_finite_f32(
            trees.iter().map(|summary| summary.branch_reach_ratio),
        ),
        min_tree_canopy_mesh_vertices: trees
            .iter()
            .map(|summary| summary.canopy.vertex_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_lobe_count: trees
            .iter()
            .map(|summary| summary.canopy_lobe_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_detail_card_count: trees
            .iter()
            .map(|summary| summary.canopy_detail_card_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_vertical_to_horizontal_ratio: min_finite_f32(
            trees
                .iter()
                .map(|summary| summary.canopy_vertical_to_horizontal_ratio),
        ),
        min_weather_cloud_mesh_vertices: clouds
            .iter()
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_lobe_count: clouds
            .iter()
            .map(|summary| summary.lobe_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_wisp_card_count: clouds
            .iter()
            .map(|summary| summary.wisp_card_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_bank_depth_m: min_finite_f32(
            clouds
                .iter()
                .filter(|summary| summary.bank)
                .map(|summary| summary.scaled_vertical_depth_m),
        ),
        min_weather_cloud_bank_lobe_count: clouds
            .iter()
            .filter(|summary| summary.bank)
            .map(|summary| summary.lobe_count)
            .min()
            .unwrap_or(0),
        terrain_biome_palette_count,
        foliage_palette_count,
        stone_palette_count,
        ground_cover,
        trees,
        clouds,
        palettes,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
}

#[derive(Debug)]
struct VisualTreeSpec {
    label: String,
    trunk_radius_m: f32,
    trunk_height_m: f32,
    seed: u32,
    canopy_radius_m: f32,
    canopy_seed: u32,
}

fn visual_content_tree_specs(island_index: usize, island: SkyIsland) -> Vec<VisualTreeSpec> {
    let mut specs = Vec::new();

    for tree_index in 0..3 {
        if island.is_target && tree_index == 1 {
            continue;
        }
        specs.push(VisualTreeSpec {
            label: format!("detail tree {tree_index}"),
            trunk_radius_m: 0.22,
            trunk_height_m: 2.1 + tree_index as f32 * 0.25,
            seed: 5_000 + island_index as u32 * 97 + tree_index as u32 * 13,
            canopy_radius_m: 1.05 + tree_index as f32 * 0.08,
            canopy_seed: 6_000 + island_index as u32 * 101 + tree_index as u32 * 17,
        });
    }

    if island.name == "launch mesa" {
        specs.push(VisualTreeSpec {
            label: "launch tree".to_string(),
            trunk_radius_m: 0.35,
            trunk_height_m: 4.4,
            seed: 7_000 + island_index as u32 * 97,
            canopy_radius_m: 1.55,
            canopy_seed: 8_000 + island_index as u32 * 101,
        });
    }

    specs
}

fn visual_content_cloud_summaries(
    output_dir: &Path,
    island_index: usize,
    island: SkyIsland,
    island_slug: &str,
) -> std::io::Result<Vec<VisualCloudSummary>> {
    let mut clouds = Vec::new();
    let bank_scale = Vec3::new(
        island.half_extents.x * 0.45 + 18.0,
        3.8 + (island_index % 3) as f32 * 0.55,
        island.half_extents.y * 0.26 + 8.0,
    );
    clouds.push(write_visual_cloud_summary(
        output_dir,
        island.name,
        "bank",
        true,
        island_index,
        island_slug,
        0,
        CLOUD_BANK_LOBES,
        bank_scale,
        2_000 + island_index as u32 * 37,
    )?);

    if island_index.is_multiple_of(2) {
        for puff_index in 0..3 {
            let veil_scale = Vec3::new(
                island.half_extents.x * 0.36 + 14.0 + puff_index as f32 * 4.0,
                0.52 + puff_index as f32 * 0.12,
                island.half_extents.y * 0.13 + 6.0 + puff_index as f32 * 1.8,
            );
            clouds.push(write_visual_cloud_summary(
                output_dir,
                island.name,
                "veil",
                false,
                island_index,
                island_slug,
                puff_index + 1,
                CLOUD_VEIL_LOBES,
                veil_scale,
                3_000 + island_index as u32 * 53 + puff_index as u32 * 11,
            )?);
        }
    }

    Ok(clouds)
}

#[allow(clippy::too_many_arguments)]
fn write_visual_cloud_summary(
    output_dir: &Path,
    island_name: &'static str,
    kind: &'static str,
    bank: bool,
    island_index: usize,
    island_slug: &str,
    cloud_index: usize,
    lobe_count: usize,
    scale: Vec3,
    seed: u32,
) -> std::io::Result<VisualCloudSummary> {
    let mesh = cloud_cluster_mesh(seed, lobe_count);
    let obj_path = PathBuf::from("visuals").join(format!(
        "{island_index:02}_{island_slug}_{kind}_{cloud_index}.obj"
    ));
    write_mesh_obj(&output_dir.join(&obj_path), &mesh, kind)?;
    let mesh_summary = visual_content_mesh_summary(obj_path, &mesh);

    Ok(VisualCloudSummary {
        island_name,
        kind,
        bank,
        scaled_horizontal_span_m: mesh_summary.horizontal_span_m * scale.x,
        scaled_vertical_depth_m: mesh_summary.vertical_span_m * scale.y,
        scaled_depth_span_m: mesh_summary.depth_span_m * scale.z,
        mesh: mesh_summary,
        lobe_count,
        wisp_card_count: lobe_count * CLOUD_WISP_CARDS_PER_LOBE,
    })
}

fn visual_content_mesh_summary(obj_path: PathBuf, mesh: &Mesh) -> VisualMeshSummary {
    let (horizontal_span_m, vertical_span_m, depth_span_m) = mesh_bounds(mesh)
        .map_or((0.0, 0.0, 0.0), |(min, max)| {
            (max.x - min.x, max.y - min.y, max.z - min.z)
        });

    VisualMeshSummary {
        obj_path,
        vertex_count: mesh.count_vertices(),
        triangle_count: mesh_index_values(mesh).len() / 3,
        horizontal_span_m,
        vertical_span_m,
        depth_span_m,
    }
}

fn mesh_bounds(mesh: &Mesh) -> Option<(Vec3, Vec3)> {
    let positions = mesh_positions(mesh);
    let first = positions.first()?;
    let mut min = Vec3::from_array(*first);
    let mut max = min;

    for position in positions.iter().skip(1) {
        let position = Vec3::from_array(*position);
        min = min.min(position);
        max = max.max(position);
    }

    Some((min, max))
}

fn ground_cover_blade_stats(mesh: &Mesh) -> GroundCoverBladeStats {
    let positions = mesh_positions(mesh);
    let mut blade_count = 0usize;
    let mut min_height_m = f32::INFINITY;
    let mut max_height_m = 0.0f32;

    for blade in positions.chunks_exact(VERTICES_PER_GROUND_BLADE) {
        let base_y = blade[0][1].min(blade[1][1]);
        let tip_y = blade[4][1];
        let height = (tip_y - base_y).max(0.0);
        min_height_m = min_height_m.min(height);
        max_height_m = max_height_m.max(height);
        blade_count += 1;
    }

    if blade_count == 0 {
        return GroundCoverBladeStats::default();
    }

    GroundCoverBladeStats {
        blade_count,
        min_height_m,
        max_height_m,
        height_range_m: max_height_m - min_height_m,
    }
}

fn tree_trunk_shape_metrics(mesh: &Mesh) -> (f32, f32) {
    let positions = mesh_positions(mesh);
    let top_ring_start = TREE_TRUNK_SEGMENTS * 2;
    let branch_vertices_start = TREE_TRUNK_SEGMENTS * 3 + 2;
    if positions.len() <= branch_vertices_start {
        return (0.0, 0.0);
    }

    let bottom_radius = average_xz_radius(&positions[0..TREE_TRUNK_SEGMENTS]);
    let top_radius =
        average_xz_radius(&positions[top_ring_start..top_ring_start + TREE_TRUNK_SEGMENTS]);
    let branch_reach = positions[branch_vertices_start..]
        .iter()
        .map(|position| Vec2::new(position[0], position[2]).length())
        .fold(0.0, f32::max);

    let taper_ratio = finite_ratio(bottom_radius, top_radius);
    let branch_reach_ratio = finite_ratio(branch_reach, bottom_radius);

    (taper_ratio, branch_reach_ratio)
}

fn average_xz_radius(points: &[[f32; 3]]) -> f32 {
    if points.is_empty() {
        return 0.0;
    }
    let center = points
        .iter()
        .map(|position| Vec2::new(position[0], position[2]))
        .sum::<Vec2>()
        / points.len() as f32;

    points
        .iter()
        .map(|position| (Vec2::new(position[0], position[2]) - center).length())
        .sum::<f32>()
        / points.len() as f32
}

fn tree_canopy_lobe_count() -> usize {
    1 + 5
}

fn visual_content_palette_summary(index: usize) -> VisualPaletteSummary {
    let terrain = terrain_biome_palette(index);
    let detail = biome_detail_color_set(index);

    VisualPaletteSummary {
        index,
        terrain_key: visual_content_vec3_key(terrain.grass),
        foliage_key: visual_content_rgba_key(detail.foliage_primary),
        stone_key: visual_content_rgba_key(detail.stone_primary),
    }
}

fn visual_content_vec3_key(color: Vec3) -> [u8; 3] {
    [
        (color.x.clamp(0.0, 1.0) * 31.0).round() as u8,
        (color.y.clamp(0.0, 1.0) * 31.0).round() as u8,
        (color.z.clamp(0.0, 1.0) * 31.0).round() as u8,
    ]
}

fn visual_content_rgba_key(color: [u8; 4]) -> [u8; 3] {
    [color[0] / 8, color[1] / 8, color[2] / 8]
}

fn visual_content_json_u8_triplet(value: [u8; 3]) -> String {
    format!("[{}, {}, {}]", value[0], value[1], value[2])
}

fn min_finite_f32(values: impl Iterator<Item = f32>) -> f32 {
    values
        .filter(|value| value.is_finite())
        .min_by(f32::total_cmp)
        .unwrap_or(0.0)
}

fn finite_ratio(numerator: f32, denominator: f32) -> f32 {
    if denominator.abs() <= f32::EPSILON {
        return 0.0;
    }
    let ratio = numerator / denominator;
    if ratio.is_finite() { ratio } else { 0.0 }
}

fn terrain_export_mesh_summary(
    obj_path: PathBuf,
    material_weights_path: Option<PathBuf>,
    mesh: &Mesh,
) -> TerrainExportMeshSummary {
    TerrainExportMeshSummary {
        obj_path,
        material_weights_path,
        vertex_count: mesh.count_vertices(),
        triangle_count: mesh_index_values(mesh).len() / 3,
        color_bands: mesh_vertex_color_band_count(mesh),
        material_weight_bands: mesh_terrain_material_weight_band_count(mesh),
        material_channels: mesh_terrain_material_channel_count(mesh),
        material_regions: mesh_terrain_material_region_count(mesh),
        relief_range_m: mesh_y_range(mesh),
    }
}

fn terrain_export_texture_detail_band_floor() -> usize {
    terrain_export_texture_metric_floor(texture_detail_band_count)
}

fn terrain_export_texture_edge_promille_floor() -> usize {
    terrain_export_texture_metric_floor(|data| texture_edge_promille(data, TERRAIN_TEXTURE_SIZE))
}

fn terrain_export_texture_metric_floor(mut metric: impl FnMut(&[u8]) -> usize) -> usize {
    [
        (
            [54, 128, 70, 255],
            [28, 92, 48, 255],
            [128, 174, 78, 255],
            17,
        ),
        (
            [96, 138, 70, 255],
            [56, 104, 54, 255],
            [166, 172, 90, 255],
            19,
        ),
        (
            [126, 104, 76, 255],
            [80, 70, 60, 255],
            [162, 138, 96, 255],
            23,
        ),
        (
            [52, 110, 118, 255],
            [30, 80, 94, 255],
            [142, 176, 164, 255],
            29,
        ),
        (
            [132, 132, 92, 255],
            [86, 96, 70, 255],
            [178, 166, 112, 255],
            31,
        ),
        (
            [70, 150, 94, 255],
            [34, 100, 62, 255],
            [156, 198, 112, 255],
            37,
        ),
    ]
    .into_iter()
    .map(|(primary, secondary, accent, seed)| {
        let data = procedural_terrain_surface_texture_data(
            primary,
            secondary,
            accent,
            seed,
            TERRAIN_TEXTURE_SIZE,
        );
        metric(&data)
    })
    .min()
    .unwrap_or(0)
}

fn write_mesh_obj(path: &Path, mesh: &Mesh, object_name: &str) -> std::io::Result<()> {
    let positions = mesh_positions(mesh);
    let normals = mesh_normals(mesh).filter(|normals| normals.len() == positions.len());
    let uvs = mesh_uv0(mesh).filter(|uvs| uvs.len() == positions.len());
    let colors = mesh_colors(mesh).filter(|colors| colors.len() == positions.len());
    let indices = mesh_index_values(mesh);
    let mut file = File::create(path)?;

    writeln!(file, "# NAU terrain export")?;
    writeln!(file, "o {}", terrain_export_slug(object_name))?;
    for (index, position) in positions.iter().enumerate() {
        if let Some(colors) = colors {
            let color = colors[index];
            writeln!(
                file,
                "v {:.4} {:.4} {:.4} {:.4} {:.4} {:.4}",
                position[0], position[1], position[2], color[0], color[1], color[2]
            )?;
        } else {
            writeln!(
                file,
                "v {:.4} {:.4} {:.4}",
                position[0], position[1], position[2]
            )?;
        }
    }
    if let Some(uvs) = uvs {
        for uv in uvs {
            writeln!(file, "vt {:.4} {:.4}", uv[0], uv[1])?;
        }
    }
    if let Some(normals) = normals {
        for normal in normals {
            writeln!(
                file,
                "vn {:.4} {:.4} {:.4}",
                normal[0], normal[1], normal[2]
            )?;
        }
    }

    let has_uvs = uvs.is_some();
    let has_normals = normals.is_some();
    for triangle in indices.chunks_exact(3) {
        writeln!(
            file,
            "f {} {} {}",
            obj_face_index(triangle[0], has_uvs, has_normals),
            obj_face_index(triangle[1], has_uvs, has_normals),
            obj_face_index(triangle[2], has_uvs, has_normals)
        )?;
    }

    Ok(())
}

fn write_terrain_material_weights_csv(path: &Path, mesh: &Mesh) -> std::io::Result<()> {
    let Some(weights) = mesh_terrain_material_weights(mesh) else {
        return Ok(());
    };
    let mut file = File::create(path)?;
    writeln!(file, "vertex,lush_highland,exposed_edge")?;
    for (index, weight) in weights.iter().enumerate() {
        writeln!(file, "{index},{:.4},{:.4}", weight[0], weight[1])?;
    }
    Ok(())
}

fn obj_face_index(index: u32, has_uvs: bool, has_normals: bool) -> String {
    let obj_index = index + 1;
    match (has_uvs, has_normals) {
        (true, true) => format!("{obj_index}/{obj_index}/{obj_index}"),
        (true, false) => format!("{obj_index}/{obj_index}"),
        (false, true) => format!("{obj_index}//{obj_index}"),
        (false, false) => obj_index.to_string(),
    }
}

fn mesh_positions(mesh: &Mesh) -> &[[f32; 3]] {
    match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => &[],
    }
}

fn mesh_normals(mesh: &Mesh) -> Option<&[[f32; 3]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(values)) => Some(values),
        _ => None,
    }
}

fn mesh_uv0(mesh: &Mesh) -> Option<&[[f32; 2]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values),
        _ => None,
    }
}

fn mesh_colors(mesh: &Mesh) -> Option<&[[f32; 4]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(VertexAttributeValues::Float32x4(values)) => Some(values),
        _ => None,
    }
}

fn mesh_terrain_material_weights(mesh: &Mesh) -> Option<&[[f32; 2]]> {
    match mesh.attribute(Mesh::ATTRIBUTE_UV_1) {
        Some(VertexAttributeValues::Float32x2(values)) => Some(values),
        _ => None,
    }
}

fn mesh_index_values(mesh: &Mesh) -> Vec<u32> {
    match mesh.indices() {
        Some(Indices::U16(values)) => values.iter().map(|index| u32::from(*index)).collect(),
        Some(Indices::U32(values)) => values.clone(),
        None => (0..mesh.count_vertices() as u32).collect(),
    }
}

fn terrain_export_slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;

    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }

    if last_was_separator {
        slug.pop();
    }
    if slug.is_empty() {
        "unnamed".to_string()
    } else {
        slug
    }
}

fn terrain_export_json_vec3(value: Vec3) -> String {
    format!(
        "[{}, {}, {}]",
        terrain_export_json_number(value.x),
        terrain_export_json_number(value.y),
        terrain_export_json_number(value.z)
    )
}

fn terrain_export_json_vec2(value: Vec2) -> String {
    format!(
        "[{}, {}]",
        terrain_export_json_number(value.x),
        terrain_export_json_number(value.y)
    )
}

fn terrain_export_json_number(value: f32) -> String {
    if value.is_finite() {
        format!("{value:.4}")
    } else {
        "0.0000".to_string()
    }
}

fn terrain_export_json_string(value: &str) -> String {
    let mut output = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            value if value.is_control() => output.push_str(&format!("\\u{:04x}", value as u32)),
            value => output.push(value),
        }
    }
    output.push('"');
    output
}

type CameraFollowFilter = (With<Camera3d>, Without<Player>);
type GeneratedPlayerPlaceholderFilter = (
    With<GeneratedPlayerPlaceholder>,
    Without<CharacterPart>,
    Without<AuthoredVisualScene>,
);

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

fn prepare_visual_asset_registry(asset_server: &AssetServer) -> VisualAssetRegistry {
    let slots = VISUAL_ASSET_SPECS
        .iter()
        .copied()
        .map(|spec| VisualAssetSlot {
            spec,
            gltf_handle: visual_asset_path_exists(spec.gltf_scene_path)
                .then(|| asset_server.load(spec.gltf_scene_path)),
            scene_handle: visual_asset_path_exists(spec.gltf_scene_path).then(|| {
                asset_server.load(GltfAssetLabel::Scene(0).from_asset(spec.gltf_scene_path))
            }),
            scene_entity: None,
            scene_ready: false,
            animation_player_entity: None,
            ready_animation_clip_count: 0,
            animation_graph_ready: false,
        })
        .collect();

    VisualAssetRegistry { slots }
}

fn visual_asset_path_exists(asset_path: &str) -> bool {
    Path::new("assets").join(asset_path).is_file()
}

fn authored_world_fixture_scene_handles(
    registry: &VisualAssetRegistry,
) -> Vec<(VisualAssetKind, &'static str, Handle<Scene>)> {
    AUTHORED_WORLD_FIXTURE_KINDS
        .iter()
        .filter_map(|kind| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.kind == *kind)
                .and_then(|slot| {
                    slot.scene_handle
                        .clone()
                        .map(|scene_handle| (*kind, slot.spec.label, scene_handle))
                })
        })
        .collect()
}

fn authored_world_fixture_transform(kind: VisualAssetKind, route: &SkyRoute) -> Transform {
    let Some((island, normalized_offset, surface_offset_y, scale, yaw_radians)) =
        authored_world_fixture_layout(kind, route.islands())
    else {
        return Transform::from_xyz(-140.0, -80.0, 140.0);
    };
    let surface = island_visual_surface_position(island, normalized_offset);

    Transform {
        translation: surface + Vec3::Y * surface_offset_y,
        rotation: Quat::from_rotation_y(yaw_radians),
        scale: Vec3::splat(scale),
    }
}

fn authored_world_fixture_layout(
    kind: VisualAssetKind,
    islands: &[SkyIsland],
) -> Option<(SkyIsland, Vec2, f32, f32, f32)> {
    let (island_index, normalized_offset, surface_offset_y, scale, yaw_radians) = match kind {
        VisualAssetKind::IslandTerrain => (0, Vec2::new(0.34, -0.34), 0.08, 0.82, 0.35),
        VisualAssetKind::IslandFoliage => (0, Vec2::new(-0.42, -0.2), 0.02, 2.2, -0.2),
        VisualAssetKind::IslandRock => (1, Vec2::new(0.3, -0.26), 0.08, 1.8, 0.75),
        VisualAssetKind::IslandWater => (5, Vec2::new(-0.18, 0.24), 0.04, 1.35, -0.45),
        VisualAssetKind::RouteMarker => (3, Vec2::new(-0.08, 0.2), 1.58, 1.4, 0.2),
        VisualAssetKind::WeatherLayer => (4, Vec2::new(0.18, -0.18), 8.2, 4.5, -0.75),
        VisualAssetKind::DistantImpostor => (6, Vec2::new(0.0, 0.0), 4.0, 4.3, 0.55),
        VisualAssetKind::PlayerCharacter | VisualAssetKind::Glider => return None,
    };
    let island = islands
        .get(island_index.min(islands.len().saturating_sub(1)))
        .copied()?;

    Some((
        island,
        normalized_offset,
        surface_offset_y,
        scale,
        yaw_radians,
    ))
}

fn mark_authored_scene_ready(
    scene_ready: On<SceneInstanceReady>,
    authored_scenes: Query<&AuthoredVisualScene>,
    mut registry: ResMut<VisualAssetRegistry>,
) {
    let Ok(scene) = authored_scenes.get(scene_ready.entity) else {
        return;
    };

    registry.mark_scene_ready(scene.kind);
}

#[allow(clippy::too_many_arguments)]
fn link_ready_authored_animations(
    children: Query<&Children>,
    animation_player_entities: Query<Entity, With<AnimationPlayer>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    gltfs: Res<Assets<Gltf>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut registry: ResMut<VisualAssetRegistry>,
    mut commands: Commands,
) {
    let pending_links = registry.pending_animation_links();
    for pending in pending_links {
        link_ready_authored_animation(
            pending,
            &children,
            &animation_player_entities,
            &mut animation_players,
            &gltfs,
            &mut animation_graphs,
            &mut registry,
            &mut commands,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn link_ready_authored_animation(
    pending: PendingAuthoredAnimationLink,
    children: &Query<&Children>,
    animation_player_entities: &Query<Entity, With<AnimationPlayer>>,
    animation_players: &mut Query<&mut AnimationPlayer>,
    gltfs: &Assets<Gltf>,
    animation_graphs: &mut Assets<AnimationGraph>,
    registry: &mut VisualAssetRegistry,
    commands: &mut Commands,
) {
    let Some(animation_player_entity) =
        find_descendant_animation_player(pending.scene_entity, children, animation_player_entities)
    else {
        return;
    };

    let Some(gltf) = gltfs.get(&pending.gltf_handle) else {
        return;
    };
    let clip_resolution = resolve_named_animation_clips(pending.spec.animation_clip_names, gltf);
    registry.mark_animation_player_linked(
        pending.kind,
        animation_player_entity,
        clip_resolution.ready_clip_count(),
    );

    if !clip_resolution.is_complete() {
        return;
    }

    let Ok(mut animation_player) = animation_players.get_mut(animation_player_entity) else {
        return;
    };
    let (animation_graph, animation_nodes) = AnimationGraph::from_clips(clip_resolution.clips);
    let graph_handle = animation_graphs.add(animation_graph);
    let player_animation = if pending.kind == VisualAssetKind::PlayerCharacter {
        let Ok(nodes) = <[AnimationNodeIndex; 6]>::try_from(animation_nodes.as_slice()) else {
            return;
        };
        Some(AuthoredPlayerAnimation::new(
            nodes,
            AuthoredPlayerClip::Idle,
        ))
    } else {
        None
    };
    let mut transitions = AnimationTransitions::default();
    if let Some(idle_node) = animation_nodes.first().copied() {
        transitions
            .play(&mut animation_player, idle_node, Duration::ZERO)
            .repeat();
    }

    commands
        .entity(animation_player_entity)
        .insert((AnimationGraphHandle(graph_handle), transitions));
    if let Some(player_animation) = player_animation {
        commands
            .entity(animation_player_entity)
            .insert(player_animation);
    }
    registry.mark_animation_graph_ready(
        pending.kind,
        animation_player_entity,
        animation_nodes.len(),
    );
}

fn find_descendant_animation_player(
    scene_entity: Entity,
    children: &Query<&Children>,
    animation_player_entities: &Query<Entity, With<AnimationPlayer>>,
) -> Option<Entity> {
    children
        .iter_descendants(scene_entity)
        .find(|entity| animation_player_entities.get(*entity).is_ok())
}

fn resolve_named_animation_clips(
    animation_clip_names: &'static [&'static str],
    gltf: &Gltf,
) -> NamedAnimationClipResolution {
    let mut clips = Vec::with_capacity(animation_clip_names.len());
    let mut missing_clip_names = Vec::new();
    for clip_name in animation_clip_names {
        if let Some(clip) = gltf.named_animations.get(*clip_name) {
            clips.push(clip.clone());
        } else {
            missing_clip_names.push(*clip_name);
        }
    }

    NamedAnimationClipResolution {
        clips,
        expected_clip_count: animation_clip_names.len(),
        missing_clip_names,
    }
}

#[cfg(test)]
fn resolve_named_animation_clip_handles(
    animation_clip_names: &'static [&'static str],
    named_animations: &HashMap<String, Handle<AnimationClip>>,
) -> NamedAnimationClipResolution {
    let mut clips = Vec::with_capacity(animation_clip_names.len());
    let mut missing_clip_names = Vec::new();
    for clip_name in animation_clip_names {
        if let Some(clip) = named_animations.get(*clip_name) {
            clips.push(clip.clone());
        } else {
            missing_clip_names.push(*clip_name);
        }
    }

    NamedAnimationClipResolution {
        clips,
        expected_clip_count: animation_clip_names.len(),
        missing_clip_names,
    }
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

#[allow(clippy::too_many_arguments)]
fn terrain_surface_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    perceptual_roughness: f32,
    reflectance: f32,
) -> (Handle<StandardMaterial>, usize) {
    let material_seed = seed.wrapping_add(1_337);
    let surface_data = procedural_terrain_surface_texture_data(
        primary,
        secondary,
        accent,
        seed,
        TERRAIN_TEXTURE_SIZE,
    );
    let detail_bands = texture_detail_band_count(&surface_data);
    let base_color_texture = procedural_srgb_texture(
        surface_data,
        TERRAIN_TEXTURE_SIZE,
        ImageFilterMode::Linear,
        16,
    );

    (
        materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(images.add(base_color_texture)),
            metallic_roughness_texture: Some(images.add(procedural_material_map_with_size(
                material_seed,
                perceptual_roughness,
                TERRAIN_TEXTURE_SIZE,
            ))),
            occlusion_texture: Some(images.add(procedural_occlusion_map_with_size(
                material_seed.wrapping_add(23),
                TERRAIN_TEXTURE_SIZE,
            ))),
            depth_map: Some(images.add(procedural_depth_map_with_size(
                material_seed.wrapping_add(47),
                ImageFilterMode::Linear,
                TERRAIN_TEXTURE_SIZE,
            ))),
            parallax_depth_scale: 0.018,
            max_parallax_layer_count: 12.0,
            perceptual_roughness,
            reflectance,
            ..default()
        }),
        detail_bands,
    )
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

fn glider_airflow_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.58, 0.88, 1.0, 0.14),
        emissive: LinearRgba::rgb(0.035, 0.18, 0.42),
        emissive_exposure_weight: 0.12,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.72,
        reflectance: 0.1,
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
        base_color: Color::srgba(0.18, 0.74, 1.0, 0.006),
        emissive: LinearRgba::rgb(0.004, 0.025, 0.045),
        emissive_exposure_weight: 0.12,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.32,
        reflectance: 0.2,
        ..default()
    })
}

fn updraft_ribbon_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgba(0.44, 0.92, 1.0, 0.32),
        emissive: LinearRgba::rgb(0.06, 0.9, 1.8),
        emissive_exposure_weight: 0.2,
        alpha_mode: AlphaMode::Add,
        cull_mode: None,
        double_sided: true,
        unlit: true,
        perceptual_roughness: 0.4,
        reflectance: 0.18,
        ..default()
    })
}

fn ground_cover_material(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(
            images.add(procedural_surface_texture(primary, secondary, accent, seed)),
        ),
        metallic_roughness_texture: Some(
            images.add(procedural_material_map(seed.wrapping_add(1_300), 0.94)),
        ),
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
    procedural_srgb_texture(
        procedural_surface_texture_data(primary, secondary, accent, seed, PROCEDURAL_TEXTURE_SIZE),
        PROCEDURAL_TEXTURE_SIZE,
        ImageFilterMode::Linear,
        8,
    )
}

fn procedural_surface_texture_data(
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    size: u32,
) -> Vec<u8> {
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

    data
}

fn procedural_terrain_surface_texture_data(
    primary: [u8; 4],
    secondary: [u8; 4],
    accent: [u8; 4],
    seed: u32,
    size: u32,
) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let fine = texture_noise(x.wrapping_mul(5), y.wrapping_mul(5), seed);
            let grain = texture_noise(x.wrapping_mul(13), y.wrapping_mul(7), seed.wrapping_add(71));
            let broad = smooth_texture_noise(x, y, 22, seed.wrapping_add(19));
            let streak = smooth_texture_noise(
                x.wrapping_mul(2).wrapping_add(y / 2),
                y.wrapping_mul(5).wrapping_add(x / 2),
                12,
                seed.wrapping_add(137),
            );
            let secondary_weight = ((118i16 - broad as i16).max(0) as u16 * 126 / 118)
                .saturating_add((grain > 192) as u16 * 24)
                .min(150);
            let accent_weight = ((broad as i16 - 164).max(0) as u16 * 142 / 91)
                .saturating_add((fine > 222 && grain > 142) as u16 * 70)
                .min(172);
            let vein = (x.wrapping_mul(17) + y.wrapping_mul(29) + seed).is_multiple_of(53);
            let mineral_fleck = fine > 222 && grain > 142;
            let mut color = mix_rgba(primary, secondary, secondary_weight);
            color = mix_rgba(color, accent, accent_weight);

            if vein {
                color = mix_rgba(color, secondary, 104);
            }
            if mineral_fleck {
                color = mix_rgba(color, accent, 96);
            }

            let shade = fine as i16 / 4 + grain as i16 / 7 + streak as i16 / 9 - 82;
            data.extend_from_slice(&shade_rgba(color, shade));
        }
    }

    data
}

fn procedural_srgb_texture(
    data: Vec<u8>,
    size: u32,
    filter: ImageFilterMode,
    anisotropy_clamp: u16,
) -> Image {
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
        mag_filter: filter,
        min_filter: filter,
        mipmap_filter: filter,
        anisotropy_clamp,
        ..default()
    });
    image
}

fn procedural_material_map(seed: u32, roughness: f32) -> Image {
    procedural_material_map_with_size(seed, roughness, PROCEDURAL_TEXTURE_SIZE)
}

fn procedural_material_map_with_size(seed: u32, roughness: f32, size: u32) -> Image {
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

    procedural_data_texture_with_size(data, ImageFilterMode::Linear, size)
}

fn procedural_occlusion_map(seed: u32) -> Image {
    procedural_occlusion_map_with_size(seed, PROCEDURAL_TEXTURE_SIZE)
}

fn procedural_occlusion_map_with_size(seed: u32, size: u32) -> Image {
    let mut data = Vec::with_capacity((size * size * 4) as usize);

    for y in 0..size {
        for x in 0..size {
            let noise = texture_noise(x, y, seed) as u16;
            let large = texture_noise(x / 4, y / 4, seed.wrapping_add(17)) as u16;
            let occlusion = (190 + noise / 5 + large / 7).min(255) as u8;
            data.extend_from_slice(&[occlusion, occlusion, occlusion, 255]);
        }
    }

    procedural_data_texture_with_size(data, ImageFilterMode::Linear, size)
}

fn procedural_depth_map(seed: u32, filter: ImageFilterMode) -> Image {
    procedural_depth_map_with_size(seed, filter, PROCEDURAL_TEXTURE_SIZE)
}

fn procedural_depth_map_with_size(seed: u32, filter: ImageFilterMode, size: u32) -> Image {
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

    procedural_data_texture_with_size(data, filter, size)
}

fn procedural_data_texture_with_size(data: Vec<u8>, filter: ImageFilterMode, size: u32) -> Image {
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

fn texture_detail_band_count(data: &[u8]) -> usize {
    data.chunks_exact(4)
        .map(|pixel| [pixel[0] / 16, pixel[1] / 16, pixel[2] / 16])
        .collect::<HashSet<_>>()
        .len()
}

fn texture_edge_promille(data: &[u8], size: u32) -> usize {
    if size < 2 {
        return 0;
    }
    let stride = size as usize * 4;
    let mut edge_count = 0usize;
    let mut sample_count = 0usize;
    for y in 0..size as usize {
        for x in 0..size as usize {
            let offset = y * stride + x * 4;
            let luma = texture_luma(&data[offset..offset + 3]);
            if x + 1 < size as usize {
                let right = texture_luma(&data[offset + 4..offset + 7]);
                edge_count += usize::from(luma.abs_diff(right) >= 18);
                sample_count += 1;
            }
            if y + 1 < size as usize {
                let down_offset = offset + stride;
                let down = texture_luma(&data[down_offset..down_offset + 3]);
                edge_count += usize::from(luma.abs_diff(down) >= 18);
                sample_count += 1;
            }
        }
    }

    (edge_count * 1000).checked_div(sample_count).unwrap_or(0)
}

fn texture_luma(rgb: &[u8]) -> u8 {
    ((u16::from(rgb[0]) * 77 + u16::from(rgb[1]) * 150 + u16::from(rgb[2]) * 29) / 256) as u8
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

fn smooth_texture_noise(x: u32, y: u32, cell_size: u32, seed: u32) -> u8 {
    let cell_size = cell_size.max(1);
    let grid_x = x / cell_size;
    let grid_y = y / cell_size;
    let local_x = (x % cell_size) as f32 / cell_size as f32;
    let local_y = (y % cell_size) as f32 / cell_size as f32;
    let weight_x = local_x * local_x * (3.0 - 2.0 * local_x);
    let weight_y = local_y * local_y * (3.0 - 2.0 * local_y);

    let north_west = texture_noise(grid_x, grid_y, seed) as f32;
    let north_east = texture_noise(grid_x + 1, grid_y, seed) as f32;
    let south_west = texture_noise(grid_x, grid_y + 1, seed) as f32;
    let south_east = texture_noise(grid_x + 1, grid_y + 1, seed) as f32;
    let north = north_west + (north_east - north_west) * weight_x;
    let south = south_west + (south_east - south_west) * weight_x;

    (north + (south - north) * weight_y)
        .round()
        .clamp(0.0, 255.0) as u8
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

fn shade_rgba(source: [u8; 4], shade: i16) -> [u8; 4] {
    [
        (source[0] as i16 + shade).clamp(0, 255) as u8,
        (source[1] as i16 + shade).clamp(0, 255) as u8,
        (source[2] as i16 + shade).clamp(0, 255) as u8,
        source[3],
    ]
}

fn mix_color(source: Color, target: Color, target_weight: f32) -> Color {
    source.mix(&target, target_weight.clamp(0.0, 1.0))
}

fn rock_scatter_mesh(radius: f32, seed: u32) -> Mesh {
    let mut positions = Vec::with_capacity(ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2);
    let mut normals = Vec::with_capacity(ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2);
    let mut uvs = Vec::with_capacity(ROCK_MESH_RINGS * ROCK_MESH_SEGMENTS + 2);
    let mut indices =
        Vec::with_capacity((ROCK_MESH_RINGS - 1) * ROCK_MESH_SEGMENTS * 6 + ROCK_MESH_SEGMENTS * 6);
    let stretch = Vec2::new(
        0.88 + random_unit(seed, 5, 17) * 0.34,
        0.76 + random_unit(seed, 7, 23) * 0.28,
    );
    let ring_profiles = [
        (-0.46, 0.72),
        (-0.28, 1.04),
        (-0.06, 1.16),
        (0.18, 0.98),
        (0.38, 0.72),
        (0.54, 0.38),
    ];

    let bottom_center = positions.len() as u32;
    positions.push([0.0, radius * -0.5, 0.0]);
    normals.push(Vec3::NEG_Y.to_array());
    uvs.push([0.5, 0.0]);

    for (ring_index, (height_factor, ring_radius)) in ring_profiles.into_iter().enumerate() {
        let phase_offset = random_unit(seed, ring_index as u32, 31) * 0.2;
        for segment in 0..ROCK_MESH_SEGMENTS {
            let phase = segment as f32 / ROCK_MESH_SEGMENTS as f32 * std::f32::consts::TAU;
            let ridge = (phase * 3.0 + phase_offset).sin() * 0.09;
            let chip = (random_unit(seed, segment as u32, ring_index as u32 + 53) - 0.5) * 0.24;
            let radial = radius * ring_radius * (1.0 + ridge + chip);
            let x = phase.cos() * radial * stretch.x;
            let z = phase.sin() * radial * stretch.y;
            let y = radius * height_factor;
            let normal = Vec3::new(
                phase.cos() / stretch.x.max(0.1),
                0.28 + height_factor * 0.35,
                phase.sin() / stretch.y.max(0.1),
            )
            .normalize();

            positions.push([x, y, z]);
            normals.push(normal.to_array());
            uvs.push([
                segment as f32 / ROCK_MESH_SEGMENTS as f32,
                ring_index as f32 / (ROCK_MESH_RINGS - 1) as f32,
            ]);
        }
    }

    let top_center = positions.len() as u32;
    let top_offset = Vec2::new(
        (random_unit(seed, 97, 11) - 0.5) * radius * 0.16,
        (random_unit(seed, 101, 13) - 0.5) * radius * 0.16,
    );
    positions.push([top_offset.x, radius * 0.68, top_offset.y]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 1.0]);

    let first_ring = 1_u32;
    for segment in 0..ROCK_MESH_SEGMENTS {
        let next = ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
        indices.extend([
            bottom_center,
            first_ring + segment as u32,
            first_ring + next,
        ]);
    }

    for ring in 0..ROCK_MESH_RINGS - 1 {
        let start = 1 + (ring * ROCK_MESH_SEGMENTS) as u32;
        let next_start = 1 + ((ring + 1) * ROCK_MESH_SEGMENTS) as u32;
        for segment in 0..ROCK_MESH_SEGMENTS {
            let a = start + segment as u32;
            let b = start + ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
            let c = next_start + segment as u32;
            let d = next_start + ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let top_ring = 1 + ((ROCK_MESH_RINGS - 1) * ROCK_MESH_SEGMENTS) as u32;
    for segment in 0..ROCK_MESH_SEGMENTS {
        let next = ((segment + 1) % ROCK_MESH_SEGMENTS) as u32;
        indices.extend([top_center, top_ring + next, top_ring + segment as u32]);
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

fn tree_trunk_mesh(radius: f32, height: f32, seed: u32) -> Mesh {
    let trunk_vertices = TREE_TRUNK_SEGMENTS * 3 + 2;
    let branch_vertices = TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 2;
    let mut positions = Vec::with_capacity(trunk_vertices + branch_vertices);
    let mut normals = Vec::with_capacity(trunk_vertices + branch_vertices);
    let mut uvs = Vec::with_capacity(trunk_vertices + branch_vertices);
    let mut indices =
        Vec::with_capacity(TREE_TRUNK_SEGMENTS * 18 + TREE_BRANCH_COUNT * TREE_BRANCH_SEGMENTS * 6);
    let bend = Vec2::new(
        (random_unit(seed, 3, 11) - 0.5) * radius * 0.95,
        (random_unit(seed, 7, 17) - 0.5) * radius * 0.95,
    );
    let rings = [
        (-0.5, radius * 1.18, Vec2::ZERO),
        (0.0, radius * 0.96, bend * 0.42),
        (0.5, radius * 0.68, bend),
    ];

    for (ring_index, (height_factor, ring_radius, center_offset)) in rings.into_iter().enumerate() {
        for segment in 0..TREE_TRUNK_SEGMENTS {
            let phase = segment as f32 / TREE_TRUNK_SEGMENTS as f32 * std::f32::consts::TAU;
            let bark_noise = 0.9 + random_unit(seed, segment as u32, ring_index as u32) * 0.2;
            let x = center_offset.x + phase.cos() * ring_radius * bark_noise;
            let z = center_offset.y + phase.sin() * ring_radius * bark_noise;
            positions.push([x, height * height_factor, z]);
            normals.push(
                Vec3::new(phase.cos(), 0.16, phase.sin())
                    .normalize()
                    .to_array(),
            );
            uvs.push([
                segment as f32 / TREE_TRUNK_SEGMENTS as f32,
                ring_index as f32 / 2.0,
            ]);
        }
    }

    for ring in 0..2 {
        let start = (ring * TREE_TRUNK_SEGMENTS) as u32;
        let next = ((ring + 1) * TREE_TRUNK_SEGMENTS) as u32;
        for segment in 0..TREE_TRUNK_SEGMENTS {
            let a = start + segment as u32;
            let b = start + ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
            let c = next + segment as u32;
            let d = next + ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }

    let bottom_center = positions.len() as u32;
    positions.push([0.0, -height * 0.5, 0.0]);
    normals.push(Vec3::NEG_Y.to_array());
    uvs.push([0.5, 0.5]);
    let top_center = positions.len() as u32;
    positions.push([bend.x, height * 0.5, bend.y]);
    normals.push(Vec3::Y.to_array());
    uvs.push([0.5, 0.5]);

    for segment in 0..TREE_TRUNK_SEGMENTS {
        let next = ((segment + 1) % TREE_TRUNK_SEGMENTS) as u32;
        indices.extend([bottom_center, segment as u32, next]);
        let top_start = (2 * TREE_TRUNK_SEGMENTS) as u32;
        indices.extend([top_center, top_start + next, top_start + segment as u32]);
    }

    for branch in 0..TREE_BRANCH_COUNT {
        let height_factor = -0.08 + branch as f32 * 0.23;
        let branch_phase = branch as f32 / TREE_BRANCH_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, branch as u32, 89) * 0.55;
        let start = Vec3::new(
            bend.x * (height_factor + 0.5),
            height * height_factor,
            bend.y * (height_factor + 0.5),
        );
        let reach = radius * (2.6 + random_unit(seed, branch as u32, 97) * 0.85);
        let lift = height * (0.08 + random_unit(seed, branch as u32, 107) * 0.05);
        let end = start + Vec3::new(branch_phase.cos() * reach, lift, branch_phase.sin() * reach);
        append_tapered_limb(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            start,
            end,
            radius * 0.34,
            radius * 0.12,
        );
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

fn tree_canopy_mesh(radius: f32, seed: u32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    append_ellipsoid_lobe(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::ZERO,
        Vec3::new(radius * 1.08, radius * 0.82, radius),
        TREE_CANOPY_LATITUDE_SEGMENTS,
        TREE_CANOPY_LONGITUDE_SEGMENTS,
        seed,
        0.22,
    );

    for lobe in 0..5 {
        let phase =
            lobe as f32 / 5.0 * std::f32::consts::TAU + random_unit(seed, lobe as u32, 71) * 0.45;
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            Vec3::new(
                phase.cos() * radius * (0.30 + random_unit(seed, lobe as u32, 83) * 0.12),
                radius * (-0.02 + lobe as f32 * 0.035),
                phase.sin() * radius * (0.26 + random_unit(seed, lobe as u32, 97) * 0.10),
            ),
            Vec3::new(radius * 0.58, radius * 0.50, radius * 0.54),
            4,
            8,
            seed.wrapping_add(100 + lobe as u32 * 17),
            0.18,
        );
    }

    for card in 0..TREE_CANOPY_CARD_COUNT {
        let phase = card as f32 / TREE_CANOPY_CARD_COUNT as f32 * std::f32::consts::TAU
            + random_unit(seed, card as u32, 151) * 0.24;
        let outward = Vec3::new(phase.cos(), 0.0, phase.sin());
        let tangent = Vec3::new(-phase.sin(), 0.0, phase.cos()).normalize();
        let up = (Vec3::Y + outward * 0.16).normalize();
        let center = Vec3::new(
            outward.x * radius * (0.58 + random_unit(seed, card as u32, 163) * 0.22),
            radius * (-0.08 + random_unit(seed, card as u32, 167) * 0.34),
            outward.z * radius * (0.54 + random_unit(seed, card as u32, 173) * 0.20),
        );
        let half_width = radius * (0.20 + random_unit(seed, card as u32, 179) * 0.08);
        let half_height = radius * (0.28 + random_unit(seed, card as u32, 181) * 0.12);
        append_double_sided_detail_card(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            tangent,
            up,
            half_width,
            half_height,
        );
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
fn append_tapered_limb(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    start: Vec3,
    end: Vec3,
    base_radius: f32,
    tip_radius: f32,
) {
    let axis = (end - start).normalize_or_zero();
    if axis.length_squared() <= 0.0001 {
        return;
    }
    let side_seed = if axis.dot(Vec3::Y).abs() > 0.92 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let side = axis.cross(side_seed).normalize();
    let bitangent = side.cross(axis).normalize();
    let first = positions.len() as u32;

    for (ring, (center, radius)) in [(start, base_radius), (end, tip_radius)]
        .into_iter()
        .enumerate()
    {
        for segment in 0..TREE_BRANCH_SEGMENTS {
            let phase = segment as f32 / TREE_BRANCH_SEGMENTS as f32 * std::f32::consts::TAU;
            let radial = side * phase.cos() + bitangent * phase.sin();
            positions.push((center + radial * radius).to_array());
            normals.push(radial.normalize().to_array());
            uvs.push([segment as f32 / TREE_BRANCH_SEGMENTS as f32, ring as f32]);
        }
    }

    for segment in 0..TREE_BRANCH_SEGMENTS {
        let a = first + segment as u32;
        let b = first + ((segment + 1) % TREE_BRANCH_SEGMENTS) as u32;
        let c = first + TREE_BRANCH_SEGMENTS as u32 + segment as u32;
        let d = first + TREE_BRANCH_SEGMENTS as u32 + ((segment + 1) % TREE_BRANCH_SEGMENTS) as u32;
        indices.extend([a, c, b, b, c, d]);
    }
}

fn cloud_cluster_mesh(seed: u32, lobe_count: usize) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for lobe in 0..lobe_count {
        let phase = lobe as f32 / lobe_count as f32 * std::f32::consts::TAU
            + random_unit(seed, lobe as u32, 5) * 0.8;
        let layer = lobe % 4;
        let layer_height = match layer {
            0 => -0.34,
            1 => -0.08,
            2 => 0.18,
            _ => 0.40,
        };
        let layer_spread = match layer {
            0 => 0.72,
            1 => 0.56,
            2 => 0.40,
            _ => 0.24,
        };
        let radius =
            0.36 + random_unit(seed, lobe as u32, 19) * 0.27 + if layer == 0 { 0.08 } else { 0.0 };
        let center = Vec3::new(
            phase.cos() * (0.18 + layer_spread * random_unit(seed, lobe as u32, 29)),
            layer_height + (random_unit(seed, lobe as u32, 41) - 0.5) * 0.16,
            phase.sin() * (0.14 + layer_spread * random_unit(seed, lobe as u32, 53) * 0.82),
        );
        append_ellipsoid_lobe(
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
            center,
            Vec3::new(
                radius * (1.20 + layer as f32 * 0.04),
                radius * (0.54 + layer as f32 * 0.03),
                radius * (0.82 + layer as f32 * 0.05),
            ),
            5,
            10,
            seed.wrapping_add(lobe as u32 * 101),
            0.15,
        );

        for card in 0..CLOUD_WISP_CARDS_PER_LOBE {
            let card_phase = phase
                + card as f32 / CLOUD_WISP_CARDS_PER_LOBE as f32 * 1.9
                + random_unit(seed, lobe as u32, 211 + card as u32) * 0.45;
            let outward = Vec3::new(
                card_phase.cos(),
                0.10 + layer as f32 * 0.025,
                card_phase.sin(),
            )
            .normalize();
            let tangent = Vec3::new(-card_phase.sin(), 0.0, card_phase.cos()).normalize();
            let up = (Vec3::Y * 0.78 + outward * 0.22).normalize();
            let card_center = center
                + outward
                    * radius
                    * (0.58 + random_unit(seed, lobe as u32, 223 + card as u32) * 0.22);
            let half_width =
                radius * (0.62 + random_unit(seed, lobe as u32, 229 + card as u32) * 0.22);
            let half_height =
                radius * (0.20 + random_unit(seed, lobe as u32, 233 + card as u32) * 0.10);
            append_double_sided_detail_card(
                &mut positions,
                &mut normals,
                &mut uvs,
                &mut indices,
                card_center,
                tangent,
                up,
                half_width,
                half_height,
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
fn append_double_sided_detail_card(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    tangent: Vec3,
    up: Vec3,
    half_width: f32,
    half_height: f32,
) {
    let tangent = tangent.normalize_or_zero();
    let up = up.normalize_or_zero();
    if tangent.length_squared() <= 0.0001 || up.length_squared() <= 0.0001 {
        return;
    }
    let normal = tangent.cross(up).normalize_or_zero();
    if normal.length_squared() <= 0.0001 {
        return;
    }

    let side = tangent * half_width;
    let vertical = up * half_height;
    let card_positions = [
        center - side,
        center + vertical,
        center + side,
        center - vertical,
    ];
    let card_uvs = [[0.0, 0.5], [0.5, 0.0], [1.0, 0.5], [0.5, 1.0]];
    let start = positions.len() as u32;

    for position in card_positions {
        positions.push(position.to_array());
        normals.push(normal.to_array());
    }
    uvs.extend(card_uvs);
    for position in card_positions {
        positions.push(position.to_array());
        normals.push((-normal).to_array());
    }
    uvs.extend(card_uvs);

    indices.extend([
        start,
        start + 1,
        start + 2,
        start,
        start + 2,
        start + 3,
        start + 6,
        start + 5,
        start + 4,
        start + 7,
        start + 6,
        start + 4,
    ]);
}

#[allow(clippy::too_many_arguments)]
fn append_ellipsoid_lobe(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    radii: Vec3,
    latitude_segments: usize,
    longitude_segments: usize,
    seed: u32,
    noise_strength: f32,
) {
    let start = positions.len() as u32;

    for lat in 0..=latitude_segments {
        let theta = lat as f32 / latitude_segments as f32 * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        for lon in 0..=longitude_segments {
            let phi = lon as f32 / longitude_segments as f32 * std::f32::consts::TAU;
            let unit = Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin());
            let noise =
                (random_unit(seed, lat as u32 * 31 + lon as u32, 83) - 0.5) * noise_strength;
            let position = center + unit * radii * (1.0 + noise);
            let normal =
                Vec3::new(unit.x / radii.x, unit.y / radii.y, unit.z / radii.z).normalize_or_zero();
            positions.push(position.to_array());
            normals.push(normal.to_array());
            uvs.push([
                lon as f32 / longitude_segments as f32,
                lat as f32 / latitude_segments as f32,
            ]);
        }
    }

    let stride = longitude_segments + 1;
    for lat in 0..latitude_segments {
        for lon in 0..longitude_segments {
            let a = start + (lat * stride + lon) as u32;
            let b = start + (lat * stride + lon + 1) as u32;
            let c = start + ((lat + 1) * stride + lon) as u32;
            let d = start + ((lat + 1) * stride + lon + 1) as u32;
            indices.extend([a, c, b, b, c, d]);
        }
    }
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

fn updraft_ribbon_mesh(radius: f32, height: f32, phase: f32) -> Mesh {
    const SEGMENTS: usize = 44;
    const STRANDS: f32 = 1.45;

    let width = (radius * 0.03).clamp(0.32, 0.65);
    let ribbon_radius = radius * 0.42;
    let mut positions = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut normals = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut uvs = Vec::with_capacity((SEGMENTS + 1) * 2);
    let mut indices = Vec::with_capacity(SEGMENTS * 6);

    for segment in 0..=SEGMENTS {
        let t = segment as f32 / SEGMENTS as f32;
        let angle = phase + t * std::f32::consts::TAU * STRANDS;
        let y = -height * 0.5 + t * height;
        let breathing = 1.0 + 0.08 * (angle * 2.0 + phase).sin();
        let radial = Vec3::new(angle.cos(), 0.0, angle.sin());
        let center = radial * ribbon_radius * breathing + Vec3::Y * y;
        let side = radial * width;
        let normal = Vec3::new(radial.x * 0.32, 0.78, radial.z * 0.32).normalize();

        positions.extend([(center - side).to_array(), (center + side).to_array()]);
        normals.extend([normal.to_array(), normal.to_array()]);
        uvs.extend([[0.0, t], [1.0, t]]);

        if segment < SEGMENTS {
            let start = (segment * 2) as u32;
            indices.extend([start, start + 1, start + 2, start + 1, start + 3, start + 2]);
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

fn glider_airflow_trail_mesh() -> Mesh {
    let positions = vec![
        [-0.5, 0.0, -0.5],
        [0.5, 0.0, -0.5],
        [-0.14, 0.0, 0.5],
        [0.14, 0.0, 0.5],
    ];
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let uvs = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    let indices = vec![0, 1, 2, 1, 3, 2];

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
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

fn island_visual_surface_position(island: SkyIsland, normalized_offset: Vec2) -> Vec3 {
    let x = island.center.x + island.half_extents.x * normalized_offset.x;
    let z = island.center.z + island.half_extents.y * normalized_offset.y;

    Vec3::new(x, island.mesh_top_y_at(Vec3::new(x, island.center.y, z)), z)
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

const ISLAND_TERRAIN_RINGS: usize = 24;
const ISLAND_BODY_SEGMENTS: usize = 96;
const ISLAND_IMPOSTOR_SEGMENTS: usize = 48;
const ISLAND_CLIFF_RINGS: usize = 8;
const ISLAND_UNDERSIDE_RINGS: usize = 7;

fn island_silhouette_scale(island_index: usize, angle: f32) -> f32 {
    let phase = island_index as f32 * 0.73;
    (1.0 + 0.09 * (angle * 3.0 + phase).sin()
        + 0.055 * (angle * 7.0 - phase * 0.4).cos()
        + 0.032 * (angle * 11.0 + phase * 1.7).sin())
    .clamp(0.82, 1.18)
}

fn island_playable_silhouette_scale(island_index: usize, angle: f32) -> f32 {
    island_silhouette_scale(island_index, angle).min(1.0)
}

fn island_polar_position(island: SkyIsland, angle: f32, radius_scale: f32, y: f32) -> [f32; 3] {
    [
        island.center.x + angle.cos() * island.half_extents.x * radius_scale,
        y,
        island.center.z + angle.sin() * island.half_extents.y * radius_scale,
    ]
}

fn mesh_y_range(mesh: &Mesh) -> f32 {
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return 0.0;
    };
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for position in positions {
        min_y = min_y.min(position[1]);
        max_y = max_y.max(position[1]);
    }
    if min_y.is_finite() && max_y.is_finite() {
        max_y - min_y
    } else {
        0.0
    }
}

fn mesh_vertex_color_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
    else {
        return 0;
    };
    let mut bands = HashSet::new();
    for color in colors {
        bands.insert([
            (color[0].clamp(0.0, 1.0) * 31.0).round() as u8,
            (color[1].clamp(0.0, 1.0) * 31.0).round() as u8,
            (color[2].clamp(0.0, 1.0) * 31.0).round() as u8,
        ]);
    }
    bands.len()
}

fn mesh_terrain_material_weight_band_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    let mut bands = HashSet::new();
    for weight in weights {
        bands.insert([
            (weight[0].clamp(0.0, 1.0) * 15.0).round() as u8,
            (weight[1].clamp(0.0, 1.0) * 15.0).round() as u8,
        ]);
    }
    bands.len()
}

fn mesh_terrain_material_channel_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    let base = weights
        .iter()
        .any(|weight| weight[0] < 0.18 && weight[1] < 0.18);
    let lush = weights.iter().any(|weight| weight[0] > 0.18);
    let exposed = weights.iter().any(|weight| weight[1] > 0.18);
    usize::from(base) + usize::from(lush) + usize::from(exposed)
}

fn terrain_material_region_id(weight: [f32; 2]) -> u8 {
    let lush_highland = weight[0].clamp(0.0, 1.0);
    let exposed_edge = weight[1].clamp(0.0, 1.0);

    if exposed_edge >= 0.48 {
        3
    } else if lush_highland >= 0.42 {
        2
    } else if lush_highland >= 0.24 || exposed_edge >= 0.10 {
        1
    } else {
        0
    }
}

fn mesh_terrain_material_region_count(mesh: &Mesh) -> usize {
    let Some(VertexAttributeValues::Float32x2(weights)) = mesh.attribute(Mesh::ATTRIBUTE_UV_1)
    else {
        return 0;
    };
    weights
        .iter()
        .map(|weight| terrain_material_region_id(*weight))
        .collect::<HashSet<_>>()
        .len()
}

fn color_array(color: Vec3) -> [f32; 4] {
    [
        color.x.clamp(0.0, 1.0),
        color.y.clamp(0.0, 1.0),
        color.z.clamp(0.0, 1.0),
        1.0,
    ]
}

#[derive(Clone, Copy, Debug)]
struct TerrainBiomePalette {
    grass: Vec3,
    moss: Vec3,
    meadow: Vec3,
    clay: Vec3,
    rock: Vec3,
    region_tints: [Vec3; 4],
}

fn terrain_biome_palette(island_index: usize) -> TerrainBiomePalette {
    match island_index % TERRAIN_BIOME_PALETTE_COUNT {
        1 => TerrainBiomePalette {
            grass: Vec3::new(0.30, 0.56, 0.24),
            moss: Vec3::new(0.20, 0.38, 0.24),
            meadow: Vec3::new(0.62, 0.56, 0.30),
            clay: Vec3::new(0.50, 0.38, 0.27),
            rock: Vec3::new(0.43, 0.42, 0.38),
            region_tints: [
                Vec3::new(0.26, 0.50, 0.22),
                Vec3::new(0.57, 0.53, 0.28),
                Vec3::new(0.18, 0.34, 0.25),
                Vec3::new(0.40, 0.36, 0.30),
            ],
        },
        2 => TerrainBiomePalette {
            grass: Vec3::new(0.36, 0.49, 0.24),
            moss: Vec3::new(0.25, 0.34, 0.25),
            meadow: Vec3::new(0.61, 0.45, 0.24),
            clay: Vec3::new(0.56, 0.32, 0.20),
            rock: Vec3::new(0.48, 0.39, 0.33),
            region_tints: [
                Vec3::new(0.34, 0.46, 0.22),
                Vec3::new(0.59, 0.42, 0.23),
                Vec3::new(0.24, 0.32, 0.24),
                Vec3::new(0.43, 0.32, 0.27),
            ],
        },
        3 => TerrainBiomePalette {
            grass: Vec3::new(0.18, 0.48, 0.42),
            moss: Vec3::new(0.12, 0.34, 0.38),
            meadow: Vec3::new(0.42, 0.52, 0.44),
            clay: Vec3::new(0.35, 0.36, 0.34),
            rock: Vec3::new(0.38, 0.44, 0.46),
            region_tints: [
                Vec3::new(0.16, 0.44, 0.36),
                Vec3::new(0.40, 0.50, 0.40),
                Vec3::new(0.10, 0.30, 0.36),
                Vec3::new(0.34, 0.40, 0.42),
            ],
        },
        4 => TerrainBiomePalette {
            grass: Vec3::new(0.42, 0.52, 0.25),
            moss: Vec3::new(0.30, 0.39, 0.23),
            meadow: Vec3::new(0.62, 0.55, 0.30),
            clay: Vec3::new(0.48, 0.39, 0.25),
            rock: Vec3::new(0.43, 0.40, 0.34),
            region_tints: [
                Vec3::new(0.36, 0.48, 0.23),
                Vec3::new(0.59, 0.52, 0.29),
                Vec3::new(0.28, 0.36, 0.22),
                Vec3::new(0.42, 0.36, 0.28),
            ],
        },
        _ => TerrainBiomePalette {
            grass: Vec3::new(0.22, 0.58, 0.29),
            moss: Vec3::new(0.15, 0.42, 0.32),
            meadow: Vec3::new(0.55, 0.52, 0.28),
            clay: Vec3::new(0.48, 0.36, 0.25),
            rock: Vec3::new(0.42, 0.40, 0.36),
            region_tints: [
                Vec3::new(0.19, 0.52, 0.24),
                Vec3::new(0.50, 0.49, 0.25),
                Vec3::new(0.14, 0.36, 0.30),
                Vec3::new(0.39, 0.34, 0.29),
            ],
        },
    }
}

#[derive(Clone, Copy, Debug)]
struct BiomeDetailColorSet {
    trunk_primary: [u8; 4],
    trunk_secondary: [u8; 4],
    trunk_accent: [u8; 4],
    foliage_primary: [u8; 4],
    foliage_secondary: [u8; 4],
    foliage_accent: [u8; 4],
    ground_primary: [u8; 4],
    ground_secondary: [u8; 4],
    ground_accent: [u8; 4],
    stone_primary: [u8; 4],
    stone_secondary: [u8; 4],
    stone_accent: [u8; 4],
}

#[derive(Clone)]
struct IslandDetailMaterials {
    trunk: Handle<StandardMaterial>,
    foliage: Handle<StandardMaterial>,
    ground_cover: Handle<StandardMaterial>,
    stone: Handle<StandardMaterial>,
}

fn rgba8(color: Vec3) -> [u8; 4] {
    [
        (color.x.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.y.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.z.clamp(0.0, 1.0) * 255.0).round() as u8,
        255,
    ]
}

fn biome_detail_color_set(island_index: usize) -> BiomeDetailColorSet {
    let palette = terrain_biome_palette(island_index);
    let bark_base = palette.clay.lerp(Vec3::new(0.25, 0.14, 0.08), 0.46);
    let foliage_base = palette.grass.lerp(palette.moss, 0.54);
    let ground_base = palette.grass.lerp(palette.meadow, 0.24);
    let stone_base = palette.rock.lerp(palette.clay, 0.28);

    BiomeDetailColorSet {
        trunk_primary: rgba8(bark_base),
        trunk_secondary: rgba8(bark_base * 0.58),
        trunk_accent: rgba8(bark_base.lerp(Vec3::new(0.72, 0.46, 0.26), 0.38)),
        foliage_primary: rgba8(foliage_base),
        foliage_secondary: rgba8(palette.moss * 0.72),
        foliage_accent: rgba8(foliage_base.lerp(palette.meadow, 0.34)),
        ground_primary: rgba8(ground_base),
        ground_secondary: rgba8(palette.moss.lerp(palette.rock, 0.16)),
        ground_accent: rgba8(palette.meadow.lerp(Vec3::new(0.92, 0.78, 0.38), 0.22)),
        stone_primary: rgba8(stone_base),
        stone_secondary: rgba8(palette.rock * 0.68),
        stone_accent: rgba8(stone_base.lerp(Vec3::splat(0.76), 0.22)),
    }
}

fn biome_detail_materials(
    images: &mut Assets<Image>,
    materials: &mut Assets<StandardMaterial>,
    island_index: usize,
) -> IslandDetailMaterials {
    let colors = biome_detail_color_set(island_index);
    let seed_base = 211 + island_index as u32 * 41;

    IslandDetailMaterials {
        trunk: textured_material(
            images,
            materials,
            colors.trunk_primary,
            colors.trunk_secondary,
            colors.trunk_accent,
            seed_base,
            0.96,
            0.16,
        ),
        foliage: textured_material(
            images,
            materials,
            colors.foliage_primary,
            colors.foliage_secondary,
            colors.foliage_accent,
            seed_base + 7,
            0.88,
            0.22,
        ),
        ground_cover: ground_cover_material(
            images,
            materials,
            colors.ground_primary,
            colors.ground_secondary,
            colors.ground_accent,
            seed_base + 13,
        ),
        stone: textured_material(
            images,
            materials,
            colors.stone_primary,
            colors.stone_secondary,
            colors.stone_accent,
            seed_base + 19,
            0.98,
            0.18,
        ),
    }
}

fn island_terrain_material_factors(
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> (f32, f32, f32, f32) {
    let phase = island_index as f32 * 0.49;
    let inner_meadow = ((0.42 - radius) / 0.42).clamp(0.0, 1.0);
    let exposed_edge = ((radius - 0.72) / 0.28).clamp(0.0, 1.0);
    let highland = ((relief_m + 0.18) / 0.82).clamp(0.0, 1.0);
    let dapple = (angle * 13.0 + phase).sin() * 0.025
        + (angle * 29.0 - phase * 0.6).cos() * 0.015
        + (radius * 31.0 + phase).sin() * 0.018;
    (inner_meadow, exposed_edge, highland, dapple)
}

fn island_terrain_material_weights(
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> [f32; 2] {
    let (inner_meadow, exposed_edge, highland, _) =
        island_terrain_material_factors(island_index, radius, angle, relief_m);
    [
        (highland * 0.72 + inner_meadow * 0.28).clamp(0.0, 1.0),
        exposed_edge.clamp(0.0, 1.0),
    ]
}

fn island_terrain_vertex_color(
    island_index: usize,
    radius: f32,
    angle: f32,
    relief_m: f32,
) -> [f32; 4] {
    let palette = terrain_biome_palette(island_index);
    let (inner_meadow, exposed_edge, highland, dapple) =
        island_terrain_material_factors(island_index, radius, angle, relief_m);
    let region = terrain_material_region_id(island_terrain_material_weights(
        island_index,
        radius,
        angle,
        relief_m,
    ));
    let color = palette
        .grass
        .lerp(palette.meadow, inner_meadow * 0.36)
        .lerp(palette.moss, highland * 0.42)
        .lerp(palette.clay, exposed_edge * 0.38)
        .lerp(palette.rock, exposed_edge.powf(1.7) * 0.48)
        .lerp(palette.region_tints[region as usize], 0.32)
        + Vec3::splat(dapple);
    color_array(color)
}

fn island_rock_vertex_color(island_index: usize, angle: f32, t: f32, underside: bool) -> [f32; 4] {
    let phase = island_index as f32 * 0.61;
    let band = ((t * ISLAND_CLIFF_STRATA_BANDS as f32 + phase * 0.13).floor() as usize)
        % ISLAND_CLIFF_STRATA_BANDS;
    let band_tint = band as f32 / (ISLAND_CLIFF_STRATA_BANDS - 1) as f32;
    let vertical_stain = (angle * 17.0 + phase + t * 4.0).sin().abs() * 0.08;
    let base = if underside {
        Vec3::new(0.25, 0.22, 0.18)
    } else {
        Vec3::new(0.38, 0.35, 0.3)
    };
    let warm = Vec3::new(0.48, 0.39, 0.29);
    let cool = Vec3::new(0.28, 0.3, 0.31);
    let color = base
        .lerp(warm, band_tint * 0.32)
        .lerp(cool, ((band % 3) as f32 / 2.0) * 0.22)
        - Vec3::splat(vertical_stain + if underside { 0.07 } else { 0.0 });
    color_array(color)
}

fn island_terrain_uv(island_index: usize, island: SkyIsland, x: f32, z: f32) -> [f32; 2] {
    let island_offset = island_index as f32 * 0.173;
    [
        (x - island.center.x) * TERRAIN_UV_TILES_PER_METER + 0.5 + island_offset,
        (z - island.center.z) * TERRAIN_UV_TILES_PER_METER + 0.5 + island_offset * 1.37,
    ]
}

fn island_terrain_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let vertex_count = 1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    let mut material_weights = Vec::with_capacity(vertex_count);
    let mut colors = Vec::with_capacity(vertex_count);
    let mut indices = Vec::with_capacity(
        ISLAND_BODY_SEGMENTS * 3 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS * 6,
    );

    let center_y = island.mesh_top_y_at(island.center);
    positions.push([island.center.x, center_y, island.center.z]);
    uvs.push(island_terrain_uv(
        island_index,
        island,
        island.center.x,
        island.center.z,
    ));
    colors.push(island_terrain_vertex_color(
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));
    material_weights.push(island_terrain_material_weights(
        island_index,
        0.0,
        0.0,
        center_y - island.mesh_top_y(),
    ));

    for ring in 1..=ISLAND_TERRAIN_RINGS {
        let radius = ring as f32 / ISLAND_TERRAIN_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_scale = island_playable_silhouette_scale(island_index, angle);
            let radius_scale = radius * (1.0 + radius.powf(1.35) * (edge_scale - 1.0));
            let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
            let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
            let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));

            positions.push([x, y, z]);
            uvs.push(island_terrain_uv(island_index, island, x, z));
            colors.push(island_terrain_vertex_color(
                island_index,
                radius,
                angle,
                y - island.mesh_top_y(),
            ));
            material_weights.push(island_terrain_material_weights(
                island_index,
                radius,
                angle,
                y - island.mesh_top_y(),
            ));
        }
    }

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (1 + (ring - 1) * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for segment in 0..ISLAND_BODY_SEGMENTS {
        indices.extend([0, ring_index(1, segment + 1), ring_index(1, segment)]);
    }

    for ring in 1..ISLAND_TERRAIN_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
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
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, material_weights)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn island_cliff_surface_position(
    island_index: usize,
    island: SkyIsland,
    angle: f32,
    t: f32,
) -> [f32; 3] {
    let phase = island_index as f32 * 0.73;
    let shelf_variation = 1.0
        + t * 0.035 * (angle * 5.0 + phase + t * 1.7).sin()
        + t * 0.025 * (angle * 13.0 - phase * 0.3 + t * 2.1).cos();
    let ledge_phase = (t * ISLAND_CLIFF_STRATA_BANDS as f32 + phase * 0.11).fract();
    let ledge_shelf = (1.0 - (ledge_phase - 0.5).abs() * 2.0).max(0.0).powf(2.2);
    let radius_scale = island_playable_silhouette_scale(island_index, angle)
        * (1.0 - t.powf(1.18) * 0.34)
        * shelf_variation
        * (1.0 + ledge_shelf * 0.028);
    let x = island.center.x + angle.cos() * island.half_extents.x * radius_scale;
    let z = island.center.z + angle.sin() * island.half_extents.y * radius_scale;
    let vertical_fracture = t
        * ((angle * 8.0 + phase).sin() * (0.45 + t) + (angle * 17.0 - phase).cos() * 0.22).abs()
        * island.thickness
        * 0.045;
    let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z))
        - 0.06
        - island.thickness * (t * 0.78)
        - ledge_shelf * island.thickness * 0.018
        - vertical_fracture;

    [x, y, z]
}

fn island_cliff_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let mut positions = Vec::with_capacity((ISLAND_CLIFF_RINGS + 1) * ISLAND_BODY_SEGMENTS);
    let mut uvs = Vec::with_capacity(positions.capacity());
    let mut colors = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(ISLAND_CLIFF_RINGS * ISLAND_BODY_SEGMENTS * 6);

    for ring in 0..=ISLAND_CLIFF_RINGS {
        let t = ring as f32 / ISLAND_CLIFF_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            positions.push(island_cliff_surface_position(
                island_index,
                island,
                angle,
                t,
            ));
            uvs.push([segment as f32 / ISLAND_BODY_SEGMENTS as f32 * 4.0, t]);
            colors.push(island_rock_vertex_color(island_index, angle, t, false));
        }
    }

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (ring * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_CLIFF_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let upper_current = ring_index(ring, segment);
            let upper_next = ring_index(ring, segment + 1);
            let lower_current = ring_index(ring + 1, segment);
            let lower_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                upper_current,
                upper_next,
                lower_current,
                upper_next,
                lower_next,
                lower_current,
            ]);
        }
    }

    let normals = smooth_normals_from_triangles_oriented(&positions, &indices, Vec3::Z, false);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn island_underside_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let ring_vertex_count = (ISLAND_UNDERSIDE_RINGS + 1) * ISLAND_BODY_SEGMENTS;
    let bottom_index = ring_vertex_count as u32;
    let mut positions = Vec::with_capacity(ring_vertex_count + 1);
    let mut uvs = Vec::with_capacity(ring_vertex_count + 1);
    let mut colors = Vec::with_capacity(ring_vertex_count + 1);
    let mut indices = Vec::with_capacity(
        ISLAND_UNDERSIDE_RINGS * ISLAND_BODY_SEGMENTS * 6 + ISLAND_BODY_SEGMENTS * 3,
    );
    let phase = island_index as f32 * 0.73;
    let top_y = island.mesh_top_y();

    for ring in 0..=ISLAND_UNDERSIDE_RINGS {
        let t = ring as f32 / ISLAND_UNDERSIDE_RINGS as f32;
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let angle = segment as f32 / ISLAND_BODY_SEGMENTS as f32 * std::f32::consts::TAU;
            if ring == 0 {
                positions.push(island_cliff_surface_position(
                    island_index,
                    island,
                    angle,
                    1.0,
                ));
                uvs.push([0.5 + angle.cos() * 0.34, 0.5 + angle.sin() * 0.34]);
                colors.push(island_rock_vertex_color(island_index, angle, t, true));
                continue;
            }

            let twist = 0.045 * (angle * 6.0 + phase + t * 2.4).sin();
            let radius_scale = island_playable_silhouette_scale(island_index, angle)
                * (0.66 * (1.0 - t).powf(1.35) + 0.18 * t)
                * (1.0 + twist);
            let y = top_y
                - island.thickness * (0.82 + t * 0.58)
                - island.thickness * 0.06 * (angle * 5.0 - phase).sin().abs();

            positions.push(island_polar_position(island, angle, radius_scale, y));
            uvs.push([
                0.5 + angle.cos() * (0.34 - t * 0.19),
                0.5 + angle.sin() * (0.34 - t * 0.19),
            ]);
            colors.push(island_rock_vertex_color(island_index, angle, t, true));
        }
    }

    positions.push([
        island.center.x,
        top_y - island.thickness * 1.58,
        island.center.z,
    ]);
    uvs.push([0.5, 0.5]);
    colors.push(island_rock_vertex_color(island_index, 0.0, 1.0, true));

    let ring_index = |ring: usize, segment: usize| -> u32 {
        (ring * ISLAND_BODY_SEGMENTS + segment % ISLAND_BODY_SEGMENTS) as u32
    };

    for ring in 0..ISLAND_UNDERSIDE_RINGS {
        for segment in 0..ISLAND_BODY_SEGMENTS {
            let upper_current = ring_index(ring, segment);
            let upper_next = ring_index(ring, segment + 1);
            let lower_current = ring_index(ring + 1, segment);
            let lower_next = ring_index(ring + 1, segment + 1);

            indices.extend([
                upper_current,
                upper_next,
                lower_current,
                upper_next,
                lower_next,
                lower_current,
            ]);
        }
    }
    for segment in 0..ISLAND_BODY_SEGMENTS {
        indices.extend([
            ring_index(ISLAND_UNDERSIDE_RINGS, segment),
            ring_index(ISLAND_UNDERSIDE_RINGS, segment + 1),
            bottom_index,
        ]);
    }

    let normals = smooth_normals_from_triangles_oriented(&positions, &indices, Vec3::NEG_Y, false);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn smooth_normals_from_triangles(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    smooth_normals_from_triangles_oriented(positions, indices, Vec3::Y, true)
}

fn smooth_normals_from_triangles_oriented(
    positions: &[[f32; 3]],
    indices: &[u32],
    fallback: Vec3,
    force_positive_y: bool,
) -> Vec<[f32; 3]> {
    let mut normals = vec![Vec3::ZERO; positions.len()];
    let fallback = fallback.normalize_or_zero();
    let fallback = if fallback.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        fallback
    };

    for triangle in indices.chunks_exact(3) {
        let a_index = triangle[0] as usize;
        let b_index = triangle[1] as usize;
        let c_index = triangle[2] as usize;
        let a = Vec3::from_array(positions[a_index]);
        let b = Vec3::from_array(positions[b_index]);
        let c = Vec3::from_array(positions[c_index]);
        let mut face_normal = (b - a).cross(c - a).normalize_or_zero();

        if force_positive_y && face_normal.y < 0.0 {
            face_normal = -face_normal;
        }
        if face_normal.length_squared() <= f32::EPSILON {
            face_normal = fallback;
        }

        normals[a_index] += face_normal;
        normals[b_index] += face_normal;
        normals[c_index] += face_normal;
    }

    normals
        .into_iter()
        .map(|normal| {
            if normal.length_squared() <= f32::EPSILON {
                fallback.to_array()
            } else {
                normal.normalize().to_array()
            }
        })
        .collect()
}

fn island_impostor_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let top_center_y = island.mesh_top_y() - 0.16;
    let shoulder_center_y = top_center_y - island.thickness * 0.30;
    let lower_center_y = top_center_y - island.thickness * 0.62;
    let bottom_y = top_center_y - island.thickness * 0.92;
    let phase = island_index as f32 * 0.71;
    let top_ring_start = 1;
    let shoulder_ring_start = top_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let lower_ring_start = shoulder_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let bottom_index = lower_ring_start + ISLAND_IMPOSTOR_SEGMENTS;
    let mut positions = Vec::with_capacity(bottom_index + 1);
    let mut uvs = Vec::with_capacity(bottom_index + 1);
    let mut colors = Vec::with_capacity(bottom_index + 1);
    let mut indices = Vec::with_capacity(ISLAND_IMPOSTOR_SEGMENTS * 18);

    positions.push([island.center.x, top_center_y, island.center.z]);
    uvs.push([0.5, 0.5]);
    colors.push(island_terrain_vertex_color(island_index, 0.0, 0.0, 0.0));

    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
        let edge_variation =
            1.0 + 0.09 * (angle * 3.0 + phase).sin() + 0.045 * (angle * 7.0 - phase).cos();
        let radius_x = island.half_extents.x * 0.9 * edge_variation;
        let radius_z = island.half_extents.y * 0.9 * edge_variation;
        let x = island.center.x + angle.cos() * radius_x;
        let z = island.center.z + angle.sin() * radius_z;
        let y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) - 0.18;

        positions.push([x, y, z]);
        uvs.push([0.5 + angle.cos() * 0.45, 0.5 + angle.sin() * 0.45]);
        colors.push(island_terrain_vertex_color(
            island_index,
            0.9,
            angle,
            y - island.mesh_top_y(),
        ));
    }

    for (ring, (center_y, radius_scale, t, underside)) in [
        (shoulder_center_y, 0.72, 0.34, false),
        (lower_center_y, 0.48, 0.78, true),
    ]
    .into_iter()
    .enumerate()
    {
        for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
            let angle = segment as f32 / ISLAND_IMPOSTOR_SEGMENTS as f32 * std::f32::consts::TAU;
            let edge_variation =
                1.0 + 0.08 * (angle * 4.0 + phase).sin() - 0.035 * (angle * 8.0).cos();
            let radius_x = island.half_extents.x * radius_scale * edge_variation;
            let radius_z = island.half_extents.y * radius_scale * edge_variation;
            let x = island.center.x + angle.cos() * radius_x;
            let z = island.center.z + angle.sin() * radius_z;
            let y = center_y - island.thickness * 0.05 * (angle * 5.0 + phase).sin().abs();

            positions.push([x, y, z]);
            uvs.push([
                0.5 + angle.cos() * (0.35 - ring as f32 * 0.11),
                0.78 + angle.sin() * 0.11 + ring as f32 * 0.14,
            ]);
            colors.push(island_rock_vertex_color(island_index, angle, t, underside));
        }
    }

    positions.push([island.center.x, bottom_y, island.center.z]);
    uvs.push([0.5, 1.0]);
    colors.push(island_rock_vertex_color(island_index, 0.0, 1.0, true));

    for segment in 0..ISLAND_IMPOSTOR_SEGMENTS {
        let next = (segment + 1) % ISLAND_IMPOSTOR_SEGMENTS;
        let top_current = (top_ring_start + segment) as u32;
        let top_next = (top_ring_start + next) as u32;
        let shoulder_current = (shoulder_ring_start + segment) as u32;
        let shoulder_next = (shoulder_ring_start + next) as u32;
        let lower_current = (lower_ring_start + segment) as u32;
        let lower_next = (lower_ring_start + next) as u32;
        let bottom = bottom_index as u32;

        indices.extend([0, top_next, top_current]);
        indices.extend([top_current, top_next, shoulder_current]);
        indices.extend([top_next, shoulder_next, shoulder_current]);
        indices.extend([shoulder_current, shoulder_next, lower_current]);
        indices.extend([shoulder_next, lower_next, lower_current]);
        indices.extend([lower_current, lower_next, bottom]);
    }

    let normals = smooth_normals_from_triangles(&positions, &indices);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
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

fn island_ground_cover_mesh(island_index: usize, island: SkyIsland) -> Mesh {
    let blade_count = GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH;
    let mut positions = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut normals = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut uvs = Vec::with_capacity(blade_count * VERTICES_PER_GROUND_BLADE);
    let mut indices = Vec::with_capacity(blade_count * INDICES_PER_GROUND_BLADE);
    let seed = island_index as u32 * 41 + 503;

    for patch in 0..GROUND_COVER_PATCHES {
        let base_angle = random_unit(seed, patch as u32, 3) * std::f32::consts::TAU;
        let radius = random_unit(seed, patch as u32, 11).sqrt() * 0.90;
        let jitter = Vec2::new(
            (random_unit(seed, patch as u32, 17) - 0.5) * 0.08,
            (random_unit(seed, patch as u32, 23) - 0.5) * 0.08,
        );
        let normalized_offset = Vec2::new(base_angle.cos(), base_angle.sin()) * radius + jitter;
        let x = island.center.x + normalized_offset.x * island.half_extents.x;
        let z = island.center.z + normalized_offset.y * island.half_extents.y;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z)) + 0.08;

        for blade in 0..GROUND_COVER_BLADES_PER_PATCH {
            let blade_phase = base_angle
                + blade as f32 * std::f32::consts::TAU / GROUND_COVER_BLADES_PER_PATCH as f32;
            let width = 0.14 + random_unit(seed, patch as u32, 31 + blade as u32) * 0.15;
            let height = 0.72 + random_unit(seed, patch as u32, 43 + blade as u32) * 0.86;
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
    let mid_side = right * (width * 0.26);
    let mid = origin + Vec3::Y * (height * 0.54) + lean * 0.42;
    let tip = origin + Vec3::Y * height + lean;
    let blade_normal = Vec3::new(right.z * 0.35, 0.8, -right.x * 0.35).normalize();
    let start = positions.len() as u32;

    positions.extend([
        (origin - side).to_array(),
        (origin + side).to_array(),
        (mid - mid_side).to_array(),
        (mid + mid_side).to_array(),
        tip.to_array(),
    ]);
    normals.extend([blade_normal.to_array(); VERTICES_PER_GROUND_BLADE]);
    let uv_offset = if patch.is_multiple_of(2) { 0.0 } else { 0.5 };
    uvs.extend([
        [uv_offset, 1.0],
        [uv_offset + 0.42, 1.0],
        [uv_offset + 0.10, 0.46],
        [uv_offset + 0.32, 0.46],
        [uv_offset + 0.21, 0.0],
    ]);
    indices.extend([
        start,
        start + 1,
        start + 2,
        start + 1,
        start + 3,
        start + 2,
        start + 2,
        start + 3,
        start + 4,
    ]);
}

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
        player.controller.mode,
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
    mut player: Query<(&Velocity, &FlightController, &mut AnimationState), With<Player>>,
    mut parts: Query<
        (&CharacterPart, &mut Transform, &mut Visibility),
        Without<AuthoredVisualScene>,
    >,
    mut authored_scenes: Query<(&AuthoredVisualScene, &mut Visibility), Without<CharacterPart>>,
    mut generated_placeholders: Query<&mut Visibility, GeneratedPlayerPlaceholderFilter>,
) {
    let Ok((velocity, controller, mut animation)) = player.single_mut() else {
        return;
    };

    let dt = eval_dt(&time, eval.as_deref());
    animation.phase = advance_phase(animation.phase, velocity.0.length(), dt);
    let blend = pose_blend(dt);
    let authored_player_ready = visual_assets.scene_ready(VisualAssetKind::PlayerCharacter);
    let authored_glider_ready = visual_assets.scene_ready(VisualAssetKind::Glider);

    for (part, mut transform, mut visibility) in &mut parts {
        let pose = part_pose(part, controller.mode, velocity.0, animation.phase);
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

fn authored_player_clip_for_state(mode: FlightMode, speed_mps: f32) -> AuthoredPlayerClip {
    match mode {
        FlightMode::Grounded if speed_mps > 0.8 => AuthoredPlayerClip::Jog,
        FlightMode::Grounded => AuthoredPlayerClip::Idle,
        FlightMode::Launching => AuthoredPlayerClip::Launch,
        FlightMode::Gliding => AuthoredPlayerClip::Glide,
        FlightMode::Airborne if speed_mps < 8.0 => AuthoredPlayerClip::Land,
        FlightMode::Airborne => AuthoredPlayerClip::AirBrake,
    }
}

fn update_authored_player_animation(
    player: Query<(&Velocity, &FlightController), With<Player>>,
    mut authored_players: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &mut AuthoredPlayerAnimation,
    )>,
) {
    let Ok((velocity, controller)) = player.single() else {
        return;
    };
    let desired = authored_player_clip_for_state(controller.mode, velocity.0.length());

    for (mut animation_player, mut transitions, mut authored_animation) in &mut authored_players {
        let desired_node = authored_animation.node(desired);
        if authored_animation.current == desired
            && animation_player.is_playing_animation(desired_node)
        {
            continue;
        }

        let transition_duration = if authored_animation.current == desired {
            Duration::ZERO
        } else {
            Duration::from_millis(140)
        };
        transitions
            .play(&mut animation_player, desired_node, transition_duration)
            .repeat();
        authored_animation.current = desired;
    }
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

fn update_visual_asset_diagnostics(
    asset_server: Res<AssetServer>,
    registry: Res<VisualAssetRegistry>,
    visible_world_fixtures: Query<(&VisibleAuthoredWorldFixture, &Visibility)>,
    mut diagnostics: ResMut<VisualAssetDiagnostics>,
) {
    let mut visible_fixture_kinds = HashSet::new();
    for (fixture, visibility) in &visible_world_fixtures {
        if *visibility == Visibility::Hidden {
            continue;
        }
        visible_fixture_kinds.insert(fixture.kind);
    }

    diagnostics.metrics = visual_asset_pipeline_metrics_with_preload_states(
        &VISUAL_ASSET_SPECS,
        |spec| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.gltf_scene_path == spec.gltf_scene_path)
                .map_or(VisualAssetLoadState::Missing, |slot| {
                    visual_asset_load_state(&asset_server, slot)
                })
        },
        |spec, _| {
            registry
                .slots
                .iter()
                .find(|slot| slot.spec.gltf_scene_path == spec.gltf_scene_path)
                .map_or(VisualAssetPreloadState::default(), |slot| {
                    visual_asset_preload_state(&asset_server, slot)
                })
        },
        |spec| registry.scene_state_for(spec),
        |spec| registry.animation_state_for(spec),
    );
    diagnostics.visible_world_fixture_count = visible_fixture_kinds.len();
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

fn visual_asset_preload_state(
    asset_server: &AssetServer,
    slot: &VisualAssetSlot,
) -> VisualAssetPreloadState {
    let Some(scene_handle) = &slot.scene_handle else {
        return VisualAssetPreloadState::default();
    };

    VisualAssetPreloadState::from_dependencies_loaded(
        asset_server.is_loaded_with_dependencies(scene_handle),
    )
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

    **text = format!(
        "frame {:>4.1} ms\nmode {}\nspeed {:>5.1} m/s\naltitude {:>5.1} m\ntarget {:>5.1} m {}\nobjective {}/{} {} {:>5.1} m {}\ncamera pitch {:>5.1} deg\ncamera distance {:>5.1} m\ncamera frame {:>5.1} deg\ncamera motion {:>4.1} m / {:>4.1} deg\ncamera orbit {:>5.1} deg\ncamera obstruction {:>4.1} m / {}\nmouse yaw {:>5.1} deg\nmouse pitch {:>5.1} deg\nmouse {}\nvelocity [{:>5.1}, {:>5.1}, {:>5.1}]\npower ups visible/collected/active {} / {} / {}\nvisual assets {} gltf {} ready {} placeholders {} missing {} stream {}\nasset load queued/loading/loaded/failed {} / {} / {} / {}\nasset preload deps/ready {} / {} always/stream {} / {}\nasset scene spawned/ready {} / {}\nauthored world fixtures {}\nasset anim clips ready/declared {} / {} players {} graphs {}\nasset residency always/window/near/far/weather {} / {} / {} / {} / {}\nvisual wind fields {} / {}\nlift fields {} / {}\nsky islands {}\nisland terrain surfaces {} vertices {} color bands {} material bands/channels/regions/texture {} / {} / {} / {} relief {:>4.2} m cliff bands {}\nisland body proc/prim {} / {} silhouette min/avg {} / {:>4.1} vertices min/max {} / {}\nground cover patches {} blades {} vertices {}\ngenerated trees trunk/canopy {} / {} vertices {} / {} biome palettes {}\ngenerated rocks {} vertices {}\ngenerated clouds {} banks {} depth {:>4.1} m lobes min/max {} / {} vertices {}\nstream chunk [{}, {}] active {} / {}\nlod near/mid/far {} / {} / {}\nstream terrain visible/hidden {} / {}\nstream impostor visible/hidden {} / {}\nlod detail visible/hidden {} / {}\nenvironment motion {} / {:>4.2} m\nstream residency {} / {} {:>4.1}% hidden {}\nstream spawn/despawn {} / {} max {} / {} total {} / {}\nstream entity changes {} max {} total {}\nroute beacons {}\nlaunch cooldown {:>4.1}s\nlaunch ready {}\ndebug visuals {} (F1)\nWASD camera-relative  Click mouse lock  Esc release  Space glider  E launch  Shift dive",
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
    .with_movement_metrics(
        desired_body_yaw_error_degrees,
        desired_heading_alignment_mps,
        lateral_response_mps,
        lateral_input_active,
        movement_axis.x,
        movement_axis.y,
    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

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
