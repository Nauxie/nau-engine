use bevy::light::{CascadeShadowConfigBuilder, VolumetricLight};
use bevy::prelude::*;

use crate::authored_assets::{
    AuthoredVisualScene, AuthoredVisualSceneRole, VisibleAuthoredWorldFixture, VisualAssetRegistry,
    authored_world_fixture_collision_proxy, authored_world_fixture_scene_handles,
    authored_world_fixture_transform, mark_authored_scene_ready,
};
use crate::camera_runtime::CameraObstacle;
use crate::content_diagnostics::IslandContentDiagnostics;
use crate::environment_visuals::{
    CinematicSun, spawn_crosswind_guide, spawn_updraft_guide, spawn_weather_layers,
};
use crate::generated_content::{TERRAIN_BIOME_PALETTE_COUNT, obstruction_spire_mesh};
use crate::island_visuals::{IslandVisualCatalog, queue_sky_island, spawn_initial_island_visuals};
use crate::power_up_runtime::spawn_power_up_guides;
use crate::scene_setup_runtime::constants::WORLD_RADIUS;
use crate::scene_setup_runtime::materials::SceneMaterials;
use crate::world_collision_runtime::{WorldCollisionProxy, WorldCollisionProxyKind};
use nau_engine::asset_pipeline::VisualAssetKind;
use nau_engine::camera::CameraObstruction;
use nau_engine::environment::{GAMEPLAY_LIFT_ROUTE, visual_crosswind_fields};
use nau_engine::world::{IslandUnderRouteSegment, SkyRoute, route_obstruction_spires};

const SUN_FIRST_CASCADE_FAR_BOUND_M: f32 = 18.0;
const SUN_SHADOW_MAX_DISTANCE_M: f32 = 120.0;

pub(super) fn spawn_world_runtime(
    commands: &mut Commands,
    route: &SkyRoute,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
    visual_asset_registry: &VisualAssetRegistry,
    player_start: Vec3,
) -> Vec<(VisualAssetKind, Entity)> {
    spawn_sun(commands);
    spawn_ground(commands, meshes, scene_materials);
    spawn_island_visuals(commands, route, meshes, scene_materials, player_start);
    spawn_camera_obstacles(commands, route, meshes, scene_materials);
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
            first_cascade_far_bound: SUN_FIRST_CASCADE_FAR_BOUND_M,
            maximum_distance: SUN_SHADOW_MAX_DISTANCE_M,
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
    player_start: Vec3,
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
        spawn_initial_island_visuals(commands, meshes, &island_visual_catalog, player_start);
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
    route: &SkyRoute,
    meshes: &mut Assets<Mesh>,
    scene_materials: &SceneMaterials,
) {
    for spire in route_obstruction_spires(route) {
        commands.spawn((
            Mesh3d(meshes.add(obstruction_spire_mesh(
                spire.radius_m,
                spire.height_m,
                spire.seed,
            ))),
            MeshMaterial3d(
                scene_materials.biome_detail_sets[spire.island_index % TERRAIN_BIOME_PALETTE_COUNT]
                    .stone
                    .clone(),
            ),
            Transform::from_translation(spire.base_position),
            CameraObstacle(CameraObstruction::new(spire.center, spire.half_extents)),
            WorldCollisionProxy::new(
                spire.center,
                spire.half_extents,
                WorldCollisionProxyKind::Landmark,
            ),
            Name::new(format!("{} obstruction spire", spire.island_name)),
        ));
    }

    for segment in route.under_island_route_segments() {
        for (index, obstacle) in under_route_segment_camera_obstacles(segment)
            .into_iter()
            .enumerate()
        {
            commands.spawn((
                CameraObstacle(obstacle),
                Name::new(format!(
                    "{} under-route camera obstacle {}",
                    segment.island_name,
                    index + 1
                )),
            ));
        }
    }
}

fn under_route_segment_camera_obstacles(
    segment: IslandUnderRouteSegment,
) -> [CameraObstruction; 3] {
    let arch_width = segment.clearance_radius_m * 2.35;
    let arch_height = segment.clearance_radius_m * 1.65;
    let arch_depth = segment.clearance_radius_m * 0.55;
    let shelf_width = segment.clearance_radius_m * 4.4;
    let shelf_depth = segment.clearance_radius_m * 2.45;
    let shelf_thickness = (segment.clearance_radius_m * 0.32).max(4.0);
    let shelf_translation = segment.midpoint - Vec3::Y * (segment.clearance_radius_m * 0.88);
    let arch_half_extents = Vec3::new(arch_width * 0.55, arch_height * 0.52, arch_depth);
    let shelf_half_extents = Vec3::new(shelf_width * 0.50, shelf_thickness, shelf_depth * 0.50);

    [
        CameraObstruction::new(segment.entry, arch_half_extents),
        CameraObstruction::new(shelf_translation, shelf_half_extents),
        CameraObstruction::new(segment.exit, arch_half_extents),
    ]
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
        let transform = authored_world_fixture_transform(kind, route);
        let collision_proxy = authored_world_fixture_collision_proxy(kind, &transform);
        let mut scene = commands.spawn((
            SceneRoot(scene_handle),
            transform,
            Visibility::Inherited,
            AuthoredVisualScene {
                kind,
                role: AuthoredVisualSceneRole::WorldFixture,
            },
            VisibleAuthoredWorldFixture { kind },
            Name::new(format!("visible authored {label} fixture scene")),
        ));
        if let Some(collision_proxy) = collision_proxy {
            scene.insert((
                collision_proxy,
                CameraObstacle(CameraObstruction::new(
                    collision_proxy.center,
                    collision_proxy.half_extents,
                )),
            ));
        }
        scene.observe(mark_authored_scene_ready);
        authored_world_fixture_scene_entities.push((kind, scene.id()));
    }

    authored_world_fixture_scene_entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authored_assets::VisualAssetSlot;
    use crate::scene_setup_runtime::prepare_scene_materials;
    use crate::world_collision_runtime::WorldCollisionProxy;
    use nau_engine::asset_pipeline::{
        VISUAL_ASSET_SPECS, VisualAssetKind, VisualAssetLoadAdmission,
    };
    use nau_engine::world::ROUTE_OBSTRUCTION_SPIRES_PER_ISLAND;

    #[test]
    fn route_obstruction_spires_spawn_camera_and_collision_blockers() {
        let route = SkyRoute::default();
        let mut world = World::new();
        let mut meshes = Assets::<Mesh>::default();
        let mut images = Assets::<Image>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let scene_materials = prepare_scene_materials(&mut images, &mut materials);

        {
            let mut commands = world.commands();
            spawn_camera_obstacles(&mut commands, &route, &mut meshes, &scene_materials);
        }
        world.flush();

        let expected_count = route.islands().len() * ROUTE_OBSTRUCTION_SPIRES_PER_ISLAND;
        let mut query = world.query::<(&Name, &CameraObstacle, &WorldCollisionProxy)>();
        let mut spire_count = 0;
        for (name, _camera_obstacle, collision_proxy) in query.iter(&world) {
            if name.as_str().contains("obstruction spire") {
                spire_count += 1;
                assert_eq!(collision_proxy.kind, WorldCollisionProxyKind::Landmark);
            }
        }

        assert_eq!(spire_count, expected_count);
    }

    #[test]
    fn under_route_segments_spawn_camera_only_blockers() {
        let route = SkyRoute::default();
        let mut world = World::new();
        let mut meshes = Assets::<Mesh>::default();
        let mut images = Assets::<Image>::default();
        let mut materials = Assets::<StandardMaterial>::default();
        let scene_materials = prepare_scene_materials(&mut images, &mut materials);

        {
            let mut commands = world.commands();
            spawn_camera_obstacles(&mut commands, &route, &mut meshes, &scene_materials);
        }
        world.flush();

        let expected_count = route.under_island_route_segments().len() * 3;
        let mut query = world.query::<(&Name, &CameraObstacle, Option<&WorldCollisionProxy>)>();
        let mut under_route_count = 0;
        for (name, _camera_obstacle, collision_proxy) in query.iter(&world) {
            if name.as_str().contains("under-route camera obstacle") {
                under_route_count += 1;
                assert!(collision_proxy.is_none());
            }
        }

        assert_eq!(under_route_count, expected_count);
    }

    #[test]
    fn authored_world_fixture_spawn_attaches_collision_to_solid_fixtures() {
        let route = SkyRoute::default();
        let registry = VisualAssetRegistry {
            slots: VISUAL_ASSET_SPECS
                .iter()
                .copied()
                .map(|spec| VisualAssetSlot {
                    spec,
                    load_admission: VisualAssetLoadAdmission::Admitted,
                    gltf_handle: None,
                    scene_handle: Some(Handle::default()),
                    scene_entity: None,
                    scene_ready: false,
                    animation_player_entity: None,
                    ready_animation_clip_count: 0,
                    animation_graph_ready: false,
                })
                .collect(),
        };
        let mut world = World::new();
        {
            let mut commands = world.commands();
            let spawned = spawn_authored_world_fixtures(&mut commands, &route, &registry);
            assert_eq!(spawned.len(), 7);
        }
        world.flush();

        let mut query = world.query::<(
            &VisibleAuthoredWorldFixture,
            Option<&WorldCollisionProxy>,
            Option<&CameraObstacle>,
        )>();
        let mut solid_count = 0;
        let mut non_solid_count = 0;
        for (fixture, collision_proxy, camera_obstacle) in query.iter(&world) {
            match fixture.kind {
                VisualAssetKind::IslandTerrain
                | VisualAssetKind::IslandFoliage
                | VisualAssetKind::IslandRock
                | VisualAssetKind::RouteMarker => {
                    solid_count += 1;
                    assert!(collision_proxy.is_some());
                    assert!(camera_obstacle.is_some());
                }
                VisualAssetKind::IslandWater
                | VisualAssetKind::WeatherLayer
                | VisualAssetKind::DistantImpostor => {
                    non_solid_count += 1;
                    assert!(collision_proxy.is_none());
                    assert!(camera_obstacle.is_none());
                }
                VisualAssetKind::PlayerCharacter | VisualAssetKind::Glider => {
                    panic!("player/glider assets are not authored world fixtures");
                }
            }
        }

        assert_eq!(solid_count, 4);
        assert_eq!(non_solid_count, 3);
    }
}
