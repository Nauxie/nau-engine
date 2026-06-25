mod authored_assets;
mod camera_runtime;
mod content_diagnostics;
mod content_export;
mod debug_readout_runtime;
mod debug_visuals;
mod environment_visuals;
mod eval_app_runtime;
mod eval_runtime;
mod generated_content;
mod island_visuals;
mod power_up_runtime;
use authored_assets::*;
use bevy::ecs::system::SystemParam;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, VolumetricLight};
use bevy::pbr::ScatteringMedium;
use bevy::prelude::*;
use bevy::window::CompositeAlphaMode;
use camera_runtime::*;
use content_diagnostics::*;
#[cfg(test)]
use content_export::mesh_uv0;
use content_export::{export_terrain_inspection, export_visual_content_inspection};
use debug_readout_runtime::*;
use debug_visuals::*;
use environment_visuals::*;
use eval_app_runtime::*;
use eval_runtime::{CliAction, EvalMovementBasis, EvalOptions, EvalRun, path_string, usage};
#[cfg(test)]
use eval_runtime::{parse_cli_args, remove_existing_dir};
use generated_content::*;
use island_visuals::*;
use nau_engine::animation::{
    AnimationState, CharacterPart, CharacterPartRole, PartVisibility, Side, advance_phase,
    part_pose, pose_blend,
};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::camera::{CameraControlState, CameraControlTuning, CameraObstruction};
use nau_engine::environment::{GAMEPLAY_LIFT_ROUTE, LiftField, WindField, apply_lift_fields};
#[cfg(test)]
use nau_engine::eval::scenario_named;
use nau_engine::eval::scripted_input;
use nau_engine::movement::{
    Facing, FlightController, FlightInput, FlightMode, FlightState, FlightTuning, Velocity,
    body_forward, face_flight_direction, step_flight,
};
#[cfg(test)]
use nau_engine::world::SkyIsland;
use nau_engine::world::{START_POSITION, SkyRoute, TERRAIN_VISUAL_FOOTING_OFFSET_M};
use power_up_runtime::*;
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::collections::HashSet;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::PathBuf;

const PLAYER_START: Vec3 = START_POSITION;
const WORLD_RADIUS: f32 = 920.0;
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
        .insert_resource(CinematicWeather::new(WORLD_RADIUS))
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

pub(crate) fn grounded_visual_foot_gap_m(
    player_y: f32,
    ground_floor_y: f32,
    mode: FlightMode,
) -> f32 {
    if mode != FlightMode::Grounded {
        return 0.0;
    }

    let visual_foot_y = player_y + authored_player_scene_transform().translation.y;
    let terrain_visual_y = ground_floor_y - TERRAIN_VISUAL_FOOTING_OFFSET_M;
    visual_foot_y - terrain_visual_y
}

#[derive(Component)]
pub(crate) struct Player;

#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct RouteObjectiveTracker {
    pub(crate) target_island_name: Option<&'static str>,
    pub(crate) completed_count: usize,
    pub(crate) total_count: usize,
    pub(crate) current_label: &'static str,
    pub(crate) current_distance_m: f32,
    pub(crate) complete: bool,
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
            &mut island_visual_catalog,
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

    spawn_follow_camera(
        &mut commands,
        &mut scattering_mediums,
        PLAYER_START,
        WORLD_RADIUS,
        INITIAL_SKY_CLEAR_COLOR,
    );

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

pub(crate) fn keyboard_flight_input(keyboard: &ButtonInput<KeyCode>) -> FlightInput {
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

pub(crate) fn movement_facing(camera: Option<&Transform>, player_transform: &Transform) -> Facing {
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

fn eval_dt(time: &Time, eval: Option<&EvalRun>) -> f32 {
    eval.map_or_else(|| time.delta_secs(), |run| run.scenario.fixed_dt)
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
}
