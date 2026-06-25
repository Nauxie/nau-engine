use bevy::light::{CascadeShadowConfigBuilder, VolumetricLight};
use bevy::pbr::ScatteringMedium;
use bevy::prelude::*;

use crate::Player;
use crate::authored_assets::*;
use crate::camera_runtime::{CameraObstacle, spawn_follow_camera};
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::debug_readout_runtime::DebugReadout;
use crate::environment_visuals::{
    CinematicSun, GliderAirflowTrail, spawn_updraft_guide, spawn_weather_layers,
};
use crate::generated_content::*;
use crate::island_visuals::{IslandVisualCatalog, queue_sky_island, spawn_initial_island_visuals};
use crate::player_runtime::{authored_glider_scene_transform, authored_player_scene_transform};
use crate::power_up_runtime::spawn_power_up_guides;
use nau_engine::animation::{AnimationState, CharacterPart, CharacterPartRole, Side};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::camera::CameraObstruction;
use nau_engine::environment::{GAMEPLAY_LIFT_ROUTE, WindField};
use nau_engine::movement::{FlightController, Velocity};
use nau_engine::world::{START_POSITION, SkyRoute};

pub(crate) const PLAYER_START: Vec3 = START_POSITION;
pub(crate) const WORLD_RADIUS: f32 = 920.0;
pub(crate) const INITIAL_SKY_CLEAR_COLOR: Color = Color::srgb(0.50, 0.68, 0.92);

pub(crate) fn setup(
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
