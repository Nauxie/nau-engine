use bevy::asset::LoadState;
use bevy::prelude::*;
use nau_engine::asset_pipeline::{
    VISUAL_ASSET_SPECS, VisualAssetLoadState, VisualAssetPreloadState,
    visual_asset_pipeline_metrics_with_preload_states,
};
use std::collections::HashSet;

use super::types::{
    VisibleAuthoredWorldFixture, VisualAssetDiagnostics, VisualAssetRegistry, VisualAssetSlot,
};

pub(crate) fn update_visual_asset_diagnostics(
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
        return slot.load_admission.load_state();
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
