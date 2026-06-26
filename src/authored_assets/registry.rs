use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use nau_engine::asset_pipeline::{
    DEFAULT_VISUAL_ASSET_LOAD_POLICY, VISUAL_ASSET_SPECS, VisualAssetAnimationState,
    VisualAssetKind, VisualAssetSceneState, VisualAssetSpec, visual_asset_load_admission_plan,
};
use std::path::Path;

use super::types::{PendingAuthoredAnimationLink, VisualAssetRegistry, VisualAssetSlot};

impl VisualAssetRegistry {
    pub(crate) fn scene_handle(&self, kind: VisualAssetKind) -> Option<Handle<Scene>> {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == kind)
            .and_then(|slot| slot.scene_handle.clone())
    }

    pub(crate) fn mark_scene_spawned(&mut self, kind: VisualAssetKind, entity: Entity) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.scene_entity = Some(entity);
        }
    }

    pub(crate) fn mark_scene_ready(&mut self, kind: VisualAssetKind) {
        if let Some(slot) = self.slots.iter_mut().find(|slot| slot.spec.kind == kind) {
            slot.scene_ready = true;
        }
    }

    pub(crate) fn scene_ready(&self, kind: VisualAssetKind) -> bool {
        self.slots
            .iter()
            .find(|slot| slot.spec.kind == kind)
            .is_some_and(|slot| slot.scene_ready)
    }

    pub(crate) fn mark_animation_player_linked(
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

    pub(crate) fn mark_animation_graph_ready(
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

    pub(crate) fn pending_animation_links(&self) -> Vec<PendingAuthoredAnimationLink> {
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

    pub(crate) fn scene_state_for(&self, spec: &VisualAssetSpec) -> VisualAssetSceneState {
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

    pub(crate) fn animation_state_for(&self, spec: &VisualAssetSpec) -> VisualAssetAnimationState {
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

pub(crate) fn prepare_visual_asset_registry(asset_server: &AssetServer) -> VisualAssetRegistry {
    let admissions = visual_asset_load_admission_plan(
        &VISUAL_ASSET_SPECS,
        |spec| visual_asset_path_exists(spec.gltf_scene_path),
        DEFAULT_VISUAL_ASSET_LOAD_POLICY,
    );
    let slots = VISUAL_ASSET_SPECS
        .iter()
        .copied()
        .zip(admissions)
        .map(|(spec, load_admission)| VisualAssetSlot {
            spec,
            load_admission,
            gltf_handle: load_admission
                .is_admitted()
                .then(|| asset_server.load(spec.gltf_scene_path)),
            scene_handle: load_admission.is_admitted().then(|| {
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

pub(super) fn visual_asset_path_exists(asset_path: &str) -> bool {
    Path::new("assets").join(asset_path).is_file()
}
