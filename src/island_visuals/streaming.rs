use super::types::{
    IslandLodVisual, IslandLodVisualCounts, IslandStreamDiagnostics, IslandStreamState,
    IslandVisualCatalog, IslandVisualEntry, IslandVisualMaterial,
};
use crate::Player;
use crate::environment_visuals::WindResponsiveVisual;
use crate::surface_material::SurfaceMaterial;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use nau_engine::world::PLAYTEST_RESET_ISLAND_NAME;
use std::collections::HashSet;

const ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET: usize = 32;

fn stream_change_budget_allows(initialized: bool, applied_changes: usize) -> bool {
    !initialized || applied_changes < ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
}

fn island_visual_is_resident(entry: &IslandVisualEntry, player_position: Vec3) -> bool {
    let activation = entry.island.stream_activation(player_position);
    let band = entry.island.lod_band(player_position);

    entry.layer.is_resident_in(activation, band)
}

fn reset_destination_proxy_is_resident(entry: &IslandVisualEntry) -> bool {
    entry.island.name == PLAYTEST_RESET_ISLAND_NAME
        && (entry.collision.is_some() || entry.obstacle.is_some())
}

#[cfg(test)]
pub(super) fn island_entry_is_resident(entry: &IslandVisualEntry, player_position: Vec3) -> bool {
    island_visual_is_resident(entry, player_position) || reset_destination_proxy_is_resident(entry)
}

pub(crate) fn spawn_initial_island_visuals(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    catalog: &IslandVisualCatalog,
    player_position: Vec3,
) -> IslandStreamState {
    let mut state = IslandStreamState::default();

    for entry in &catalog.entries {
        let visual_resident = island_visual_is_resident(entry, player_position);
        if !visual_resident && !reset_destination_proxy_is_resident(entry) {
            continue;
        }

        let entity =
            spawn_island_visual_entry(commands, meshes, &mut state, entry, visual_resident);
        state.spawned.insert(entry.key, entity);
        if visual_resident {
            state.visual_resident.insert(entry.key);
        }
    }

    state
}

fn spawn_island_visual_entry(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    stream_state: &mut IslandStreamState,
    entry: &IslandVisualEntry,
    visual_resident: bool,
) -> Entity {
    let entity = {
        let mut entity = commands.spawn((entry.transform, IslandLodVisual, Name::new(entry.name)));
        if let Some(obstacle) = entry.obstacle {
            entity.insert(obstacle);
        }
        if let Some(collision) = entry.collision {
            entity.insert(collision);
        }
        entity.id()
    };

    if visual_resident {
        insert_island_visual_components(commands, meshes, stream_state, entity, entry);
    }

    entity
}

fn insert_island_visual_components(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    stream_state: &mut IslandStreamState,
    entity: Entity,
    entry: &IslandVisualEntry,
) {
    let mesh_and_material = entry.material.as_ref().and_then(|material| {
        mesh_handle_for_entry(meshes, stream_state, entry).map(|mesh| (mesh, material.clone()))
    });
    let mut entity = commands.entity(entity);

    if let Some((mesh, material)) = mesh_and_material {
        let casts_shadows = material.casts_shadows();
        match material {
            IslandVisualMaterial::Standard(material) => {
                entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
            }
            IslandVisualMaterial::Surface(material)
            | IslandVisualMaterial::SurfaceNoShadows(material) => {
                entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
            }
        }
        if !casts_shadows {
            entity.insert(NotShadowCaster);
        }
    }
    if let Some(motion) = entry.wind_motion {
        entity.insert(WindResponsiveVisual {
            base_translation: entry.transform.translation,
            base_rotation: entry.transform.rotation,
            base_scale: entry.transform.scale,
            motion,
        });
    }
}

fn remove_island_visual_components(
    commands: &mut Commands,
    entity: Entity,
    entry: &IslandVisualEntry,
) {
    let mut entity = commands.entity(entity);
    entity.remove::<Mesh3d>();
    match entry.material.as_ref() {
        Some(IslandVisualMaterial::Standard(_)) => {
            entity.remove::<MeshMaterial3d<StandardMaterial>>();
        }
        Some(IslandVisualMaterial::Surface(_) | IslandVisualMaterial::SurfaceNoShadows(_)) => {
            entity.remove::<MeshMaterial3d<SurfaceMaterial>>();
        }
        None => {}
    }
    entity.remove::<NotShadowCaster>();
    entity.remove::<WindResponsiveVisual>();
}

fn mesh_handle_for_entry(
    meshes: &mut Assets<Mesh>,
    stream_state: &mut IslandStreamState,
    entry: &IslandVisualEntry,
) -> Option<Handle<Mesh>> {
    if let Some(mesh) = &entry.mesh {
        return Some(mesh.clone());
    }

    let recipe = entry.mesh_recipe?;
    Some(
        stream_state
            .loaded_meshes
            .entry(entry.key)
            .or_insert_with(|| meshes.add(recipe.build_mesh()))
            .clone(),
    )
}

pub(crate) fn update_island_stream_visibility(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
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
        let visual_resident = island_visual_is_resident(entry, player_transform.translation);
        let entry_resident = visual_resident || reset_destination_proxy_is_resident(entry);
        counts.record(entry.layer, !visual_resident);

        if !entry_resident {
            continue;
        }

        desired_keys.insert(entry.key);
        match stream_state.spawned.get(&entry.key).copied() {
            None => {
                if visual_resident {
                    let applied_changes = spawned_visuals + despawned_visual_count;
                    if !stream_change_budget_allows(diagnostics.initialized, applied_changes) {
                        continue;
                    }
                }

                let entity = spawn_island_visual_entry(
                    &mut commands,
                    &mut meshes,
                    &mut stream_state,
                    entry,
                    visual_resident,
                );
                stream_state.spawned.insert(entry.key, entity);
                if visual_resident {
                    stream_state.visual_resident.insert(entry.key);
                    if diagnostics.initialized {
                        spawned_visuals += 1;
                    }
                }
            }
            Some(entity)
                if visual_resident && !stream_state.visual_resident.contains(&entry.key) =>
            {
                let applied_changes = spawned_visuals + despawned_visual_count;
                if stream_change_budget_allows(diagnostics.initialized, applied_changes) {
                    insert_island_visual_components(
                        &mut commands,
                        &mut meshes,
                        &mut stream_state,
                        entity,
                        entry,
                    );
                    stream_state.visual_resident.insert(entry.key);
                    if diagnostics.initialized {
                        spawned_visuals += 1;
                    }
                }
            }
            Some(entity)
                if !visual_resident && stream_state.visual_resident.contains(&entry.key) =>
            {
                let applied_changes = spawned_visuals + despawned_visual_count;
                if stream_change_budget_allows(diagnostics.initialized, applied_changes) {
                    remove_island_visual_components(&mut commands, entity, entry);
                    stream_state.visual_resident.remove(&entry.key);
                    if diagnostics.initialized {
                        despawned_visual_count += 1;
                    }
                }
            }
            Some(_) => {}
        }
    }

    let despawned_visuals = stream_state
        .spawned
        .iter()
        .filter_map(|(key, entity)| (!desired_keys.contains(key)).then_some((*key, *entity)))
        .collect::<Vec<_>>();

    for (key, entity) in despawned_visuals {
        let visual_was_resident = stream_state.visual_resident.contains(&key);
        let applied_changes = spawned_visuals + despawned_visual_count;
        if visual_was_resident
            && !stream_change_budget_allows(diagnostics.initialized, applied_changes)
        {
            break;
        }

        commands.entity(entity).despawn();
        stream_state.spawned.remove(&key);
        stream_state.visual_resident.remove(&key);
        if visual_was_resident && diagnostics.initialized {
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

#[cfg(test)]
mod tests {
    use super::super::types::{IslandVisualKey, IslandVisualLayer, IslandVisualMeshRecipe};
    use super::*;
    use crate::camera_runtime::CameraObstacle;
    use crate::environment_visuals::wind_visual_motion;
    use crate::world_collision_runtime::{WorldCollisionProxy, WorldCollisionProxyKind};
    use nau_engine::camera::CameraObstruction;
    use nau_engine::world::{START_POSITION, SkyIsland, SkyRoute};

    fn resident_entry(index: usize) -> IslandVisualEntry {
        let island = SkyIsland::new(
            "stream-budget-test",
            Vec3::ZERO,
            Vec2::splat(48.0),
            8.0,
            false,
        );
        IslandVisualEntry {
            key: super::super::types::IslandVisualKey {
                island_name: "stream-budget-test",
                layer: super::super::types::IslandVisualLayer::Terrain,
                index,
            },
            island,
            layer: super::super::types::IslandVisualLayer::Terrain,
            mesh: None,
            mesh_recipe: None,
            material: None,
            transform: Transform::from_translation(Vec3::new(index as f32, 0.0, 0.0)),
            obstacle: None,
            collision: None,
            wind_motion: None,
            name: "stream-budget-test-visual",
        }
    }

    #[test]
    fn initialized_streaming_caps_spawn_changes_per_frame() {
        let mut app = App::new();
        app.insert_resource(Assets::<Mesh>::default());
        app.insert_resource(IslandVisualCatalog {
            entries: (0..ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET + 3)
                .map(resident_entry)
                .collect(),
        });
        app.insert_resource(IslandStreamState::default());
        app.insert_resource(IslandStreamDiagnostics {
            initialized: true,
            ..default()
        });
        app.world_mut().spawn((crate::Player, Transform::default()));
        app.add_systems(Update, update_island_stream_visibility);

        app.update();

        let diagnostics = app.world().resource::<IslandStreamDiagnostics>();
        assert_eq!(
            diagnostics.visibility_changes_this_frame,
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );
        assert_eq!(
            diagnostics.spawned_visuals_this_frame,
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );
        assert_eq!(
            diagnostics.max_visibility_changes_per_frame,
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );
        assert_eq!(
            app.world().resource::<IslandStreamState>().spawned.len(),
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );

        app.update();

        let diagnostics = app.world().resource::<IslandStreamDiagnostics>();
        assert_eq!(diagnostics.visibility_changes_this_frame, 3);
        assert_eq!(diagnostics.spawned_visuals_this_frame, 3);
        assert_eq!(
            diagnostics.max_visibility_changes_per_frame,
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );
        assert_eq!(
            app.world().resource::<IslandStreamState>().spawned.len(),
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET + 3
        );
    }

    #[test]
    fn reset_destination_proxies_reside_without_visuals_until_lod_residency() {
        let route = SkyRoute::default();
        let island = route
            .island_named(PLAYTEST_RESET_ISLAND_NAME)
            .expect("reset island should exist");
        let layer = IslandVisualLayer::Detail;
        let key = IslandVisualKey {
            island_name: PLAYTEST_RESET_ISLAND_NAME,
            layer,
            index: 0,
        };
        let entry = IslandVisualEntry {
            key,
            island,
            layer,
            mesh: None,
            mesh_recipe: Some(IslandVisualMeshRecipe::Terrain {
                island_index: 0,
                island,
            }),
            material: Some(Handle::<StandardMaterial>::default().into()),
            transform: Transform::from_translation(island.center),
            obstacle: Some(CameraObstacle(CameraObstruction::new(
                island.center,
                Vec3::ONE,
            ))),
            collision: Some(WorldCollisionProxy::new(
                island.center,
                Vec3::ONE,
                WorldCollisionProxyKind::Landmark,
            )),
            wind_motion: Some(wind_visual_motion(0, 0.0, 0.2, 0.1, 1.0)),
            name: "reset-proxy-with-deferred-visual",
        };
        assert!(!island_visual_is_resident(&entry, START_POSITION));
        assert!(island_visual_is_resident(&entry, island.center));

        let catalog = IslandVisualCatalog {
            entries: vec![entry],
        };
        let mut meshes = Assets::<Mesh>::default();
        let mut world = World::new();

        let state = {
            let mut commands = world.commands();
            spawn_initial_island_visuals(&mut commands, &mut meshes, &catalog, START_POSITION)
        };
        world.flush();

        let entity = state.spawned[&key];
        assert_eq!(state.loaded_mesh_count(), 0);
        assert!(!state.visual_resident.contains(&key));
        assert!(world.get::<CameraObstacle>(entity).is_some());
        assert!(world.get::<WorldCollisionProxy>(entity).is_some());
        assert!(world.get::<Mesh3d>(entity).is_none());
        assert!(
            world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .is_none()
        );
        assert!(world.get::<WindResponsiveVisual>(entity).is_none());

        world.insert_resource(meshes);
        world.insert_resource(catalog);
        world.insert_resource(state);
        world.insert_resource(IslandStreamDiagnostics::default());
        let player = world
            .spawn((crate::Player, Transform::from_translation(START_POSITION)))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_island_stream_visibility);

        schedule.run(&mut world);
        assert_eq!(
            world
                .resource::<IslandStreamDiagnostics>()
                .counts
                .hidden_detail_count,
            1
        );

        world
            .entity_mut(player)
            .get_mut::<Transform>()
            .expect("player should have a transform")
            .translation = island.center;
        schedule.run(&mut world);

        assert!(world.get::<CameraObstacle>(entity).is_some());
        assert!(world.get::<WorldCollisionProxy>(entity).is_some());
        assert!(world.get::<Mesh3d>(entity).is_some());
        assert!(
            world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .is_some()
        );
        assert!(world.get::<WindResponsiveVisual>(entity).is_some());
        assert_eq!(
            world
                .resource::<IslandStreamDiagnostics>()
                .spawned_visuals_this_frame,
            1
        );
        assert_eq!(world.resource::<IslandStreamState>().loaded_mesh_count(), 1);

        world
            .entity_mut(player)
            .get_mut::<Transform>()
            .expect("player should have a transform")
            .translation = START_POSITION;
        schedule.run(&mut world);

        assert!(world.get::<CameraObstacle>(entity).is_some());
        assert!(world.get::<WorldCollisionProxy>(entity).is_some());
        assert!(world.get::<Mesh3d>(entity).is_none());
        assert!(
            world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .is_none()
        );
        assert!(world.get::<WindResponsiveVisual>(entity).is_none());
        assert_eq!(
            world
                .resource::<IslandStreamDiagnostics>()
                .despawned_visuals_this_frame,
            1
        );
    }

    #[test]
    fn surface_material_components_use_the_surface_material_type() {
        let mut entry = resident_entry(0);
        entry.mesh = Some(Handle::<Mesh>::default());
        entry.material = Some(Handle::<SurfaceMaterial>::default().into());

        let mut meshes = Assets::<Mesh>::default();
        let mut stream_state = IslandStreamState::default();
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        {
            let mut commands = world.commands();
            insert_island_visual_components(
                &mut commands,
                &mut meshes,
                &mut stream_state,
                entity,
                &entry,
            );
        }
        world.flush();

        assert!(world.get::<Mesh3d>(entity).is_some());
        assert!(
            world
                .get::<MeshMaterial3d<SurfaceMaterial>>(entity)
                .is_some()
        );
        assert!(
            world
                .get::<MeshMaterial3d<StandardMaterial>>(entity)
                .is_none()
        );
        assert!(world.get::<NotShadowCaster>(entity).is_none());

        {
            let mut commands = world.commands();
            remove_island_visual_components(&mut commands, entity, &entry);
        }
        world.flush();

        assert!(world.get::<Mesh3d>(entity).is_none());
        assert!(
            world
                .get::<MeshMaterial3d<SurfaceMaterial>>(entity)
                .is_none()
        );
    }

    #[test]
    fn no_shadow_surface_material_components_disable_shadow_casting() {
        let mut entry = resident_entry(0);
        entry.mesh = Some(Handle::<Mesh>::default());
        let material = Handle::<SurfaceMaterial>::default();
        entry.material = Some(IslandVisualMaterial::surface_without_shadows(material));

        let mut meshes = Assets::<Mesh>::default();
        let mut stream_state = IslandStreamState::default();
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        {
            let mut commands = world.commands();
            insert_island_visual_components(
                &mut commands,
                &mut meshes,
                &mut stream_state,
                entity,
                &entry,
            );
        }
        world.flush();

        assert!(
            world
                .get::<MeshMaterial3d<SurfaceMaterial>>(entity)
                .is_some()
        );
        assert!(world.get::<NotShadowCaster>(entity).is_some());

        {
            let mut commands = world.commands();
            remove_island_visual_components(&mut commands, entity, &entry);
        }
        world.flush();

        assert!(world.get::<NotShadowCaster>(entity).is_none());
    }
}
