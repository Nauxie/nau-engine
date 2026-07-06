use super::types::{
    IslandLodVisual, IslandLodVisualCounts, IslandStreamDiagnostics, IslandStreamState,
    IslandVisualCatalog, IslandVisualEntry,
};
use crate::Player;
use crate::environment_visuals::WindResponsiveVisual;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
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

pub(crate) fn spawn_initial_island_visuals(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    catalog: &IslandVisualCatalog,
    player_position: Vec3,
) -> IslandStreamState {
    let mut state = IslandStreamState::default();

    for entry in catalog
        .entries
        .iter()
        .filter(|entry| island_visual_is_resident(entry, player_position))
    {
        let entity = spawn_island_visual_entry(commands, meshes, &mut state, entry);
        state.spawned.insert(entry.key, entity);
    }

    state
}

fn spawn_island_visual_entry(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    stream_state: &mut IslandStreamState,
    entry: &IslandVisualEntry,
) -> Entity {
    let mut entity = commands.spawn((entry.transform, IslandLodVisual, Name::new(entry.name)));
    if let Some(material) = entry.material.as_ref()
        && let Some(mesh) = mesh_handle_for_entry(meshes, stream_state, entry)
    {
        entity.insert((Mesh3d(mesh), MeshMaterial3d(material.clone())));
        if !matches!(entry.layer, super::types::IslandVisualLayer::Terrain) {
            entity.insert(NotShadowCaster);
        }
    }
    if let Some(obstacle) = entry.obstacle {
        entity.insert(obstacle);
    }
    if let Some(collision) = entry.collision {
        entity.insert(collision);
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
        let resident = island_visual_is_resident(entry, player_transform.translation);
        counts.record(entry.layer, !resident);

        if resident {
            desired_keys.insert(entry.key);
            if !stream_state.spawned.contains_key(&entry.key) {
                let applied_changes = spawned_visuals + despawned_visual_count;
                if stream_change_budget_allows(diagnostics.initialized, applied_changes) {
                    let entity = spawn_island_visual_entry(
                        &mut commands,
                        &mut meshes,
                        &mut stream_state,
                        entry,
                    );
                    stream_state.spawned.insert(entry.key, entity);
                    if diagnostics.initialized {
                        spawned_visuals += 1;
                    }
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
        let applied_changes = spawned_visuals + despawned_visual_count;
        if !stream_change_budget_allows(diagnostics.initialized, applied_changes) {
            break;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use nau_engine::world::SkyIsland;

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
}
