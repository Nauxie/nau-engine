use super::types::{
    IslandLodVisual, IslandLodVisualCounts, IslandStreamDiagnostics, IslandStreamState,
    IslandVisualCatalog, IslandVisualEntry,
};
use crate::Player;
use crate::environment_visuals::WindResponsiveVisual;
use bevy::prelude::*;
use std::collections::HashSet;

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
                let entity =
                    spawn_island_visual_entry(&mut commands, &mut meshes, &mut stream_state, entry);
                stream_state.spawned.insert(entry.key, entity);
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
