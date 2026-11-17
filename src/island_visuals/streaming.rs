use super::types::{
    IslandLodVisual, IslandLodVisualCounts, IslandStreamDiagnostics, IslandStreamState,
    IslandVisualCatalog, IslandVisualEntry, IslandVisualMaterial,
};
use crate::Player;
use crate::environment_visuals::WindResponsiveVisual;
use crate::surface_material::SurfaceMaterial;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use nau_engine::world::{LodBand, PLAYTEST_RESET_ISLAND_NAME, StreamActivation};

const ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET: usize = 32;

fn stream_change_budget_allows(initialized: bool, applied_changes: usize) -> bool {
    !initialized || applied_changes < ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
}

#[cfg(test)]
fn island_visual_is_resident(entry: &IslandVisualEntry, player_position: Vec3) -> bool {
    let activation = entry.island.stream_activation(player_position);
    let band = entry.island.lod_band(player_position);

    entry.layer.is_resident_in(activation, band)
}

fn island_visual_is_resident_cached(
    entry: &IslandVisualEntry,
    player_position: Vec3,
    cache: &mut Option<(&'static str, StreamActivation, LodBand)>,
) -> bool {
    let (activation, band) = match *cache {
        Some((island_name, activation, band)) if island_name == entry.island.name => {
            (activation, band)
        }
        _ => {
            let activation = entry.island.stream_activation(player_position);
            let band = entry.island.lod_band(player_position);
            *cache = Some((entry.island.name, activation, band));
            (activation, band)
        }
    };

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
    let mut residency_cache = None;

    for entry in &catalog.entries {
        let visual_resident =
            island_visual_is_resident_cached(entry, player_position, &mut residency_cache);
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

fn admit_island_visual_entry(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    state: &mut IslandStreamState,
    entry: &IslandVisualEntry,
) {
    match state.spawned.get(&entry.key).copied() {
        Some(entity) => insert_island_visual_components(commands, meshes, state, entity, entry),
        None => {
            let entity = spawn_island_visual_entry(commands, meshes, state, entry, true);
            state.spawned.insert(entry.key, entity);
        }
    }
    state.visual_resident.insert(entry.key);
}

fn retire_island_visual_entry(
    commands: &mut Commands,
    state: &mut IslandStreamState,
    entry: &IslandVisualEntry,
    entity: Entity,
) {
    if reset_destination_proxy_is_resident(entry) {
        remove_island_visual_components(commands, entity, entry);
    } else {
        commands.entity(entity).despawn();
        state.spawned.remove(&entry.key);
    }
    state.visual_resident.remove(&entry.key);
}

fn replacement_shell_layer(
    layer: super::types::IslandVisualLayer,
) -> Option<super::types::IslandVisualLayer> {
    use super::types::IslandVisualLayer;
    match layer {
        IslandVisualLayer::Terrain => Some(IslandVisualLayer::Impostor),
        IslandVisualLayer::Impostor => Some(IslandVisualLayer::Terrain),
        _ => None,
    }
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
    let mut stale_visuals = std::mem::take(&mut stream_state.stale_visuals_scratch);
    stale_visuals.clear();
    let mut pending_visuals = std::mem::take(&mut stream_state.pending_visuals_scratch);
    pending_visuals.clear();
    let mut spawned_visuals = 0;
    let mut despawned_visual_count = 0;
    let mut residency_cache = None;

    for (index, entry) in catalog.entries.iter().enumerate() {
        let desired_visual = island_visual_is_resident_cached(
            entry,
            player_transform.translation,
            &mut residency_cache,
        );
        let desired_entity = desired_visual || reset_destination_proxy_is_resident(entry);
        let actual_visual = stream_state.visual_resident.contains(&entry.key);
        match stream_state.spawned.get(&entry.key).copied() {
            Some(entity) if !desired_entity || (!desired_visual && actual_visual) => {
                stale_visuals.push((index, entity));
            }
            Some(_) if desired_visual && !actual_visual => pending_visuals.push(index),
            None if desired_visual => pending_visuals.push(index),
            None if desired_entity => {
                let entity = spawn_island_visual_entry(
                    &mut commands,
                    &mut meshes,
                    &mut stream_state,
                    entry,
                    false,
                );
                stream_state.spawned.insert(entry.key, entity);
            }
            _ => {}
        }
        counts.record(entry.layer, !actual_visual);
    }

    // Free obsolete detail first. Outgoing terrain then frees terrain residency
    // before incoming islands replace their impostors with detailed shells.
    stale_visuals.sort_unstable_by_key(|(index, _)| {
        use super::types::IslandVisualLayer;
        let entry = &catalog.entries[*index];
        let priority = match entry.layer {
            IslandVisualLayer::Terrain => 1,
            IslandVisualLayer::Impostor => 2,
            _ => 0,
        };
        (priority, entry.island.name, entry.key.index)
    });
    let mut stale_index = 0;
    while stale_index < stale_visuals.len() {
        let (entry_index, _) = stale_visuals[stale_index];
        let entry = &catalog.entries[entry_index];
        let replacement_layer = replacement_shell_layer(entry.layer);
        let replacements = pending_visuals.iter().copied().filter(|index| {
            let pending = &catalog.entries[*index];
            Some(pending.layer) == replacement_layer && pending.island.name == entry.island.name
        });
        let replacement_count = if replacement_layer.is_some() {
            replacements.clone().count()
        } else {
            0
        };
        let group_end = if replacement_count > 0 {
            stale_index
                + stale_visuals[stale_index..]
                    .iter()
                    .take_while(|(index, _)| {
                        let candidate = &catalog.entries[*index];
                        candidate.layer == entry.layer && candidate.island.name == entry.island.name
                    })
                    .count()
        } else {
            stale_index + 1
        };
        let retiring_visuals = stale_visuals[stale_index..group_end]
            .iter()
            .filter(|(index, _)| {
                stream_state
                    .visual_resident
                    .contains(&catalog.entries[*index].key)
            })
            .count();
        let changes = retiring_visuals + replacement_count;
        let applied_changes = spawned_visuals + despawned_visual_count;
        if diagnostics.initialized
            && applied_changes + changes > ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        {
            break;
        }
        // A whole old/new shell pair shares one frame's change budget. Keeping
        // the old shell until this swap fits prevents holes during LOD handoff.
        for (index, entity) in stale_visuals[stale_index..group_end].iter().copied() {
            let retiring = &catalog.entries[index];
            let was_visual = stream_state.visual_resident.contains(&retiring.key);
            retire_island_visual_entry(&mut commands, &mut stream_state, retiring, entity);
            if was_visual {
                counts.record_resident_removal(retiring.layer);
                if diagnostics.initialized {
                    despawned_visual_count += 1;
                }
            }
        }
        for index in replacements.take(replacement_count) {
            let incoming = &catalog.entries[index];
            admit_island_visual_entry(&mut commands, &mut meshes, &mut stream_state, incoming);
            counts.record_resident_admission(incoming.layer);
            if diagnostics.initialized {
                spawned_visuals += 1;
            }
        }
        stale_index = group_end;
    }

    // Only fill new residency after obsolete residency has drained. Spending the
    // full budget on additions first lets the previous scene remain indefinitely.
    if stale_index == stale_visuals.len() {
        for index in pending_visuals.iter().copied() {
            let entry = &catalog.entries[index];
            if stream_state.visual_resident.contains(&entry.key) {
                continue;
            }
            let applied_changes = spawned_visuals + despawned_visual_count;
            if !stream_change_budget_allows(diagnostics.initialized, applied_changes) {
                break;
            }
            admit_island_visual_entry(&mut commands, &mut meshes, &mut stream_state, entry);
            counts.record_resident_admission(entry.layer);
            if diagnostics.initialized {
                spawned_visuals += 1;
            }
        }
    }

    stale_visuals.clear();
    pending_visuals.clear();
    stream_state.stale_visuals_scratch = stale_visuals;
    stream_state.pending_visuals_scratch = pending_visuals;

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

    fn assert_residency_counts_match_render_components(world: &mut World) {
        let state = world.resource::<IslandStreamState>();
        let catalog = world.resource::<IslandVisualCatalog>();
        let counts = world.resource::<IslandStreamDiagnostics>().counts;
        let rendered_count = |layer| {
            state
                .spawned
                .iter()
                .filter(|(key, entity)| {
                    key.layer == layer && world.get::<Mesh3d>(**entity).is_some()
                })
                .count()
        };
        let catalog_count = |layer| {
            catalog
                .entries
                .iter()
                .filter(|entry| entry.layer == layer)
                .count()
        };
        assert_eq!(
            counts.visible_terrain_count,
            rendered_count(IslandVisualLayer::Terrain)
        );
        assert_eq!(
            counts.hidden_terrain_count,
            catalog_count(IslandVisualLayer::Terrain) - counts.visible_terrain_count
        );
        assert_eq!(
            counts.visible_detail_count,
            rendered_count(IslandVisualLayer::Detail) + rendered_count(IslandVisualLayer::Vista)
        );
        assert_eq!(
            counts.hidden_detail_count,
            catalog_count(IslandVisualLayer::Detail) + catalog_count(IslandVisualLayer::Vista)
                - counts.visible_detail_count
        );
        assert_eq!(
            counts.visible_beacon_count,
            rendered_count(IslandVisualLayer::Beacon)
        );
        assert_eq!(
            counts.visible_impostor_count,
            rendered_count(IslandVisualLayer::Impostor)
        );
        assert_eq!(
            counts.hidden_impostor_count,
            catalog_count(IslandVisualLayer::Impostor) - counts.visible_impostor_count
        );
        assert_eq!(
            counts.resident_count(),
            world.query::<&Mesh3d>().iter(world).count()
        );
    }

    fn assert_every_island_has_one_complete_shell(world: &World) {
        let mut shells = std::collections::HashMap::<&str, [usize; 4]>::new();
        let state = world.resource::<IslandStreamState>();
        for entry in &world.resource::<IslandVisualCatalog>().entries {
            let offset = match entry.layer {
                IslandVisualLayer::Terrain => 0,
                IslandVisualLayer::Impostor => 2,
                _ => continue,
            };
            let shell = shells.entry(entry.island.name).or_default();
            shell[offset] += 1;
            if let Some(entity) = state.spawned.get(&entry.key)
                && world.get::<Mesh3d>(*entity).is_some()
            {
                shell[offset + 1] += 1;
            }
        }
        for (
            island,
            [
                terrain_total,
                terrain_actual,
                impostor_total,
                impostor_actual,
            ],
        ) in shells
        {
            assert!(
                (terrain_actual == terrain_total && impostor_actual == 0)
                    || (impostor_actual == impostor_total && terrain_actual == 0),
                "{island} lost its complete shell during LOD handoff: terrain {terrain_actual}/{terrain_total}, impostor {impostor_actual}/{impostor_total}"
            );
        }
    }

    #[test]
    fn residency_telemetry_matches_applied_spawns_and_retained_visuals_during_budgeted_transitions()
    {
        let mut app = App::new();
        let mut meshes = Assets::<Mesh>::default();
        let mesh = meshes.add(Cuboid::default());
        let mut entries = Vec::new();
        for layer in [
            IslandVisualLayer::Terrain,
            IslandVisualLayer::Detail,
            IslandVisualLayer::Vista,
            IslandVisualLayer::Beacon,
            IslandVisualLayer::Impostor,
        ] {
            for index in 0..ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET + 3 {
                let mut entry = resident_entry(index);
                const ISLAND_NAMES: [&str; 7] = [
                    "stream-a", "stream-b", "stream-c", "stream-d", "stream-e", "stream-f",
                    "stream-g",
                ];
                entry.island.name = ISLAND_NAMES[index % ISLAND_NAMES.len()];
                entry.key.island_name = entry.island.name;
                entry.layer = layer;
                entry.key.layer = layer;
                entry.mesh = Some(mesh.clone());
                entry.material = Some(Handle::<StandardMaterial>::default().into());
                entries.push(entry);
            }
        }
        app.insert_resource(meshes);
        app.insert_resource(IslandVisualCatalog { entries });
        app.insert_resource(IslandStreamState::default());
        app.insert_resource(IslandStreamDiagnostics {
            initialized: true,
            ..default()
        });
        let player = app
            .world_mut()
            .spawn((crate::Player, Transform::default()))
            .id();
        app.add_systems(Update, update_island_stream_visibility);

        app.update();
        assert_residency_counts_match_render_components(app.world_mut());
        let counts = app.world().resource::<IslandStreamDiagnostics>().counts;
        assert_eq!(
            counts.visible_terrain_count,
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );
        assert_eq!(counts.hidden_terrain_count, 3);
        for _ in 0..5 {
            app.update();
            assert_residency_counts_match_render_components(app.world_mut());
        }

        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation = Vec3::splat(100_000.0);
        app.update();
        assert_residency_counts_match_render_components(app.world_mut());
        assert_every_island_has_one_complete_shell(app.world());
        let counts = app.world().resource::<IslandStreamDiagnostics>().counts;
        assert_eq!(
            counts.resident_count(),
            4 * (ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET + 3)
                - ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );
        assert_eq!(counts.visible_impostor_count, 0);
        for _ in 0..7 {
            app.update();
            assert_residency_counts_match_render_components(app.world_mut());
            assert_every_island_has_one_complete_shell(app.world());
            assert!(
                app.world()
                    .resource::<IslandStreamDiagnostics>()
                    .counts
                    .resident_count()
                    <= 4 * (ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET + 3)
            );
            assert!(
                app.world()
                    .resource::<IslandStreamDiagnostics>()
                    .visibility_changes_this_frame
                    <= ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
            );
        }
        let counts = app.world().resource::<IslandStreamDiagnostics>().counts;
        assert_eq!(counts.visible_terrain_count, 0);
        assert_eq!(counts.visible_detail_count, 0);
        assert_eq!(counts.visible_beacon_count, 0);
        assert_eq!(
            counts.visible_impostor_count,
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET + 3
        );
        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation = Vec3::ZERO;
        for _ in 0..8 {
            app.update();
            assert_residency_counts_match_render_components(app.world_mut());
            assert_every_island_has_one_complete_shell(app.world());
        }
        assert_eq!(
            app.world()
                .resource::<IslandStreamDiagnostics>()
                .counts
                .visible_terrain_count,
            ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET + 3
        );
    }

    fn real_route_streaming_world(route: &SkyRoute) -> (World, Entity, Schedule) {
        use crate::generated_content::{IslandDetailMaterials, WaterSurfaceMaterials};
        let material = Handle::<StandardMaterial>::default();
        let surface = Handle::<SurfaceMaterial>::default();
        let detail_materials = IslandDetailMaterials {
            trunk: material.clone(),
            foliage: material.clone(),
            ground_cover: material.clone(),
            stone: material.clone(),
        };
        let water_materials = WaterSurfaceMaterials {
            body: surface.clone(),
            foam: surface.clone(),
            mist: material.clone(),
        };
        let mut catalog = IslandVisualCatalog::default();
        let mut content = crate::content_diagnostics::IslandContentDiagnostics::default();
        let mut meshes = Assets::<Mesh>::default();
        for (index, island) in route.islands().iter().copied().enumerate() {
            super::super::queue_sky_island(
                &mut catalog,
                &mut content,
                &mut meshes,
                surface.clone(),
                material.clone(),
                material.clone(),
                material.clone(),
                material.clone(),
                detail_materials.clone(),
                material.clone(),
                water_materials.clone(),
                index,
                island,
            );
        }
        let mut world = World::new();
        let state = spawn_initial_island_visuals(
            &mut world.commands(),
            &mut meshes,
            &catalog,
            START_POSITION,
        );
        world.flush();
        world.insert_resource(meshes);
        world.insert_resource(catalog);
        world.insert_resource(state);
        world.insert_resource(IslandStreamDiagnostics {
            initialized: true,
            ..default()
        });
        let player = world
            .spawn((Player, Transform::from_translation(START_POSITION)))
            .id();
        let mut schedule = Schedule::default();
        schedule.add_systems(update_island_stream_visibility);
        (world, player, schedule)
    }

    fn assert_streaming_limits(
        world: &mut World,
        thresholds: nau_engine::eval::EvalThresholds,
    ) -> [usize; 3] {
        let actual = assert_streaming_invariants(world);
        assert!(actual[0] <= thresholds.max_visible_island_terrain_count);
        assert!(actual[1] <= thresholds.max_visible_island_detail_count);
        assert!(actual[2] <= thresholds.max_resident_island_visual_count);
        actual
    }

    fn assert_streaming_invariants(world: &mut World) -> [usize; 3] {
        let diagnostics = *world.resource::<IslandStreamDiagnostics>();
        let counts = diagnostics.counts;
        let actual_meshes = world.query::<&Mesh3d>().iter(world).count();
        assert_eq!(counts.resident_count(), actual_meshes);
        assert_every_island_has_one_complete_shell(world);
        assert!(
            diagnostics.visibility_changes_this_frame <= ISLAND_STREAM_CHANGES_PER_FRAME_BUDGET
        );
        [
            counts.visible_terrain_count,
            counts.visible_detail_count,
            counts.resident_count(),
        ]
    }

    fn assert_streaming_settled(world: &World, position: Vec3) {
        let expected = world
            .resource::<IslandVisualCatalog>()
            .entries
            .iter()
            .filter(|entry| island_visual_is_resident(entry, position))
            .map(|entry| entry.key)
            .collect();
        assert_eq!(
            world.resource::<IslandStreamState>().visual_resident,
            expected
        );
    }

    #[test]
    fn real_gallery_and_resets_preserve_budgets_and_continuous_movement_preserves_shells() {
        use nau_engine::eval::{ISLAND_HERO_GALLERY, PLAYTEST_RESET, scenario_named};
        use nau_engine::world::IslandReviewPlan;
        let route = SkyRoute::default();
        let gallery = scenario_named(ISLAND_HERO_GALLERY).unwrap();
        let timing = gallery.island_hero_gallery_timing().unwrap();
        let plan = IslandReviewPlan::from_route(&route);
        let (mut world, player, mut schedule) = real_route_streaming_world(&route);
        let mut gallery_max = [0; 3];
        for island in &plan.islands {
            for view in island.views {
                world.get_mut::<Transform>(player).unwrap().translation = view.player_position;
                for frame in 0..timing.frames_per_view {
                    schedule.run(&mut world);
                    let actual = assert_streaming_limits(&mut world, gallery.thresholds);
                    gallery_max = std::array::from_fn(|i| gallery_max[i].max(actual[i]));
                    if frame == timing.settle_frames {
                        assert_streaming_settled(&world, view.player_position);
                    }
                }
            }
        }
        println!("gallery actual terrain/detail/total maxima: {gallery_max:?}");

        // Move continuously through the same full-content route, then reverse
        // before arrivals settle and reset repeatedly across the archipelago.
        let reset_position = route.playtest_reset_position();
        let mut movement_max = [0; 3];
        let counts = world.resource::<IslandStreamDiagnostics>().counts;
        let mut previous = [
            counts.visible_terrain_count,
            counts.visible_detail_count,
            counts.resident_count(),
        ];
        let mut max_desired_detail = (0, Vec3::ZERO);
        for frame in 0..2400 {
            let position = START_POSITION.lerp(reset_position, frame as f32 / 2399.0);
            world.get_mut::<Transform>(player).unwrap().translation = position;
            let mut desired = IslandLodVisualCounts::default();
            for entry in &world.resource::<IslandVisualCatalog>().entries {
                desired.record(entry.layer, !island_visual_is_resident(entry, position));
            }
            let desired_counts = [
                desired.visible_terrain_count,
                desired.visible_detail_count,
                desired.resident_count(),
            ];
            if desired.visible_detail_count > max_desired_detail.0 {
                max_desired_detail = (desired.visible_detail_count, position);
            }
            schedule.run(&mut world);
            let actual = assert_streaming_invariants(&mut world);
            for i in 0..actual.len() {
                assert!(
                    actual[i] <= previous[i].max(desired_counts[i]),
                    "streaming must not amplify prior residency or current content demand"
                );
            }
            previous = actual;
            movement_max = std::array::from_fn(|i| movement_max[i].max(actual[i]));
        }
        println!(
            "continuous movement actual terrain/detail/total maxima: {movement_max:?}; largest authored detail demand {max_desired_detail:?}"
        );

        let (mut world, player, mut schedule) = real_route_streaming_world(&route);
        let reset_thresholds = scenario_named(PLAYTEST_RESET).unwrap().thresholds;
        let mut reset_max = [0; 3];
        for frame in 0..120 {
            let position = if frame % 12 < 6 {
                reset_position
            } else {
                START_POSITION
            };
            world.get_mut::<Transform>(player).unwrap().translation = position;
            schedule.run(&mut world);
            let actual = assert_streaming_limits(&mut world, reset_thresholds);
            reset_max = std::array::from_fn(|i| reset_max[i].max(actual[i]));
        }
        for _ in 0..32 {
            schedule.run(&mut world);
            assert_streaming_limits(&mut world, reset_thresholds);
        }
        assert_streaming_settled(&world, START_POSITION);
        println!("interrupted resets actual terrain/detail/total maxima: {reset_max:?}");
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
