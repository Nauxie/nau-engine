use bevy::light::{CascadeShadowConfigBuilder, VolumetricLight};
use bevy::prelude::*;

use crate::authored_assets::{
    AuthoredVisualScene, AuthoredVisualSceneRole, VisibleAuthoredWorldFixture, VisualAssetRegistry,
    authored_world_fixture_scene_handles, authored_world_fixture_transform,
    mark_authored_scene_ready,
};
use crate::camera_runtime::CameraObstacle;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::environment_visuals::{
    CinematicSun, spawn_crosswind_guide, spawn_updraft_guide, spawn_weather_layers,
};
use crate::generated_content::TERRAIN_BIOME_PALETTE_COUNT;
use crate::island_visuals::{IslandVisualCatalog, queue_sky_island, spawn_initial_island_visuals};
use crate::power_up_runtime::spawn_power_up_guides;
use crate::scene_setup_runtime::constants::{PLAYER_START, WORLD_RADIUS};
use crate::scene_setup_runtime::materials::SceneMaterials;
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::camera::CameraObstruction;
use nau_engine::environment::{GAMEPLAY_LIFT_ROUTE, visual_crosswind_fields};
use nau_engine::world::SkyRoute;

pub(super) fn spawn_world_runtime(
    commands: &mut Commands,
    route: &SkyRoute,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
    visual_asset_registry: &VisualAssetRegistry,
) -> Vec<(VisualAssetKind, Entity)> {
    spawn_sun(commands);
    spawn_ground(commands, meshes, scene_materials);
    spawn_island_visuals(commands, route, meshes, scene_materials);
    spawn_camera_obstacles(commands, meshes, scene_materials);
    spawn_environment_volumes(commands, meshes, scene_materials);
    spawn_authored_world_fixtures(commands, route, visual_asset_registry)
}

fn spawn_sun(commands: &mut Commands) {
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
}

fn spawn_ground(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
) {
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(WORLD_RADIUS * 2.0, WORLD_RADIUS * 2.0),
            ),
        ),
        MeshMaterial3d(scene_materials.ground.clone()),
        Transform::default(),
    ));
}

fn spawn_island_visuals(
    commands: &mut Commands,
    route: &SkyRoute,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
) {
    let mut island_visual_catalog = IslandVisualCatalog::default();
    let mut island_content_diagnostics = IslandContentDiagnostics::default();
    island_content_diagnostics
        .record_terrain_material_texture_detail(scene_materials.terrain_texture_detail_bands);

    for (index, island) in route.islands().iter().enumerate() {
        let top_material = if island.is_target {
            scene_materials.target_grass.clone()
        } else {
            match index % TERRAIN_BIOME_PALETTE_COUNT {
                0 => scene_materials.island_grass.clone(),
                1 => scene_materials.island_meadow.clone(),
                2 => scene_materials.island_clay.clone(),
                3 => scene_materials.island_alpine.clone(),
                _ => scene_materials.island_highland.clone(),
            }
        };

        queue_sky_island(
            &mut island_visual_catalog,
            &mut island_content_diagnostics,
            meshes,
            top_material,
            scene_materials.island_rock.clone(),
            scene_materials.island_under.clone(),
            scene_materials.target_marker.clone(),
            scene_materials.updraft_marker.clone(),
            scene_materials.biome_detail_sets[index % TERRAIN_BIOME_PALETTE_COUNT].clone(),
            scene_materials.flower.clone(),
            scene_materials.water.clone(),
            index,
            *island,
        );
    }

    let island_stream_state =
        spawn_initial_island_visuals(commands, &island_visual_catalog, PLAYER_START);
    commands.insert_resource(island_visual_catalog);
    commands.insert_resource(island_stream_state);

    spawn_weather_layers(
        commands,
        &mut island_content_diagnostics,
        meshes,
        scene_materials.cloud.clone(),
        scene_materials.cloud_veil.clone(),
        route.islands(),
    );
    commands.insert_resource(island_content_diagnostics);
}

fn spawn_camera_obstacles(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
) {
    for (index, x) in (-5..=5).enumerate() {
        let height = 5.0 + (index as f32 % 4.0) * 4.0;
        let z = if index % 2 == 0 { -28.0 } else { 34.0 };

        let center = Vec3::new(x as f32 * 20.0, height * 0.5, z);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(5.0, height, 5.0))),
            MeshMaterial3d(scene_materials.pillar.clone()),
            Transform::from_translation(center),
            CameraObstacle(CameraObstruction::new(
                center,
                Vec3::new(2.5, height * 0.5, 2.5),
            )),
        ));
    }
}

fn spawn_environment_volumes(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
) {
    for (index, field) in visual_crosswind_fields().into_iter().enumerate() {
        let label = format!("visual crosswind {}", index + 1);
        commands.spawn((field, Name::new(format!("{label} volume"))));
        spawn_crosswind_guide(
            commands,
            meshes,
            scene_materials.updraft_ribbon.clone(),
            scene_materials.updraft_marker.clone(),
            field,
            &label,
        );
    }
    for lift in GAMEPLAY_LIFT_ROUTE {
        commands.spawn((
            lift.visual_field(),
            Name::new(format!("{} visual", lift.name)),
        ));
        commands.spawn((lift.lift_field(), Name::new(lift.name)));
        spawn_updraft_guide(
            commands,
            meshes,
            scene_materials.updraft_column.clone(),
            scene_materials.updraft_ribbon.clone(),
            scene_materials.updraft_marker.clone(),
            lift,
        );
    }
    spawn_power_up_guides(commands, meshes, scene_materials.power_up.clone());
}

fn spawn_authored_world_fixtures(
    commands: &mut Commands,
    route: &SkyRoute,
    visual_asset_registry: &VisualAssetRegistry,
) -> Vec<(VisualAssetKind, Entity)> {
    let authored_world_fixture_scene_handles =
        authored_world_fixture_scene_handles(visual_asset_registry);
    let mut authored_world_fixture_scene_entities =
        Vec::with_capacity(authored_world_fixture_scene_handles.len());

    for (kind, label, scene_handle) in authored_world_fixture_scene_handles {
        let mut scene = commands.spawn((
            SceneRoot(scene_handle),
            authored_world_fixture_transform(kind, route),
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

    authored_world_fixture_scene_entities
}
