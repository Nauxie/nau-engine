use crate::camera_runtime::CameraObstacle;
use crate::environment_visuals::WindVisualMotion;
use crate::world_collision_runtime::WorldCollisionProxy;
use bevy::prelude::*;
use nau_engine::world::{LodBand, SkyIsland, StreamActivation};
use std::collections::HashMap;

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct IslandLodVisual;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum IslandVisualLayer {
    Terrain,
    Detail,
    Beacon,
    Impostor,
}

impl IslandVisualLayer {
    pub(super) fn is_resident_in(self, activation: StreamActivation, band: LodBand) -> bool {
        match self {
            Self::Terrain => activation.is_active(),
            Self::Detail => activation.is_active() && band == LodBand::Near,
            Self::Beacon => true,
            Self::Impostor => !activation.is_active() || band != LodBand::Near,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct IslandLodVisualCounts {
    pub(crate) visible_terrain_count: usize,
    pub(crate) hidden_terrain_count: usize,
    pub(crate) visible_detail_count: usize,
    pub(crate) hidden_detail_count: usize,
    pub(crate) visible_beacon_count: usize,
    pub(crate) visible_impostor_count: usize,
    pub(crate) hidden_impostor_count: usize,
}

impl IslandLodVisualCounts {
    pub(super) fn record(&mut self, layer: IslandVisualLayer, hidden: bool) {
        match (layer, hidden) {
            (IslandVisualLayer::Terrain, false) => self.visible_terrain_count += 1,
            (IslandVisualLayer::Terrain, true) => self.hidden_terrain_count += 1,
            (IslandVisualLayer::Detail, false) => self.visible_detail_count += 1,
            (IslandVisualLayer::Detail, true) => self.hidden_detail_count += 1,
            (IslandVisualLayer::Beacon, false) => self.visible_beacon_count += 1,
            (IslandVisualLayer::Beacon, true) => {}
            (IslandVisualLayer::Impostor, false) => self.visible_impostor_count += 1,
            (IslandVisualLayer::Impostor, true) => self.hidden_impostor_count += 1,
        }
    }

    pub(crate) fn resident_count(self) -> usize {
        self.visible_terrain_count
            + self.visible_detail_count
            + self.visible_beacon_count
            + self.visible_impostor_count
    }

    pub(crate) fn hidden_count(self) -> usize {
        self.hidden_terrain_count + self.hidden_detail_count + self.hidden_impostor_count
    }

    pub(crate) fn catalog_count(self) -> usize {
        self.resident_count() + self.hidden_count()
    }

    pub(crate) fn resident_fraction(self) -> f32 {
        self.resident_count() as f32 / self.catalog_count().max(1) as f32
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(crate) struct IslandStreamDiagnostics {
    pub(crate) counts: IslandLodVisualCounts,
    pub(crate) visibility_changes_this_frame: usize,
    pub(crate) max_visibility_changes_per_frame: usize,
    pub(crate) total_visibility_changes: usize,
    pub(crate) spawned_visuals_this_frame: usize,
    pub(crate) despawned_visuals_this_frame: usize,
    pub(crate) max_spawned_visuals_per_frame: usize,
    pub(crate) max_despawned_visuals_per_frame: usize,
    pub(crate) total_spawned_visuals: usize,
    pub(crate) total_despawned_visuals: usize,
    pub(super) initialized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct IslandVisualKey {
    pub(super) island_name: &'static str,
    pub(super) layer: IslandVisualLayer,
    pub(super) index: usize,
}

#[derive(Clone)]
pub(super) struct IslandVisualEntry {
    pub(super) key: IslandVisualKey,
    pub(super) island: SkyIsland,
    pub(super) layer: IslandVisualLayer,
    pub(super) mesh: Handle<Mesh>,
    pub(super) material: Handle<StandardMaterial>,
    pub(super) transform: Transform,
    pub(super) obstacle: Option<CameraObstacle>,
    pub(super) collision: Option<WorldCollisionProxy>,
    pub(super) wind_motion: Option<WindVisualMotion>,
    pub(super) name: &'static str,
}

#[derive(Resource, Default)]
pub(crate) struct IslandVisualCatalog {
    pub(super) entries: Vec<IslandVisualEntry>,
}

#[derive(Resource, Default)]
pub(crate) struct IslandStreamState {
    pub(super) spawned: HashMap<IslandVisualKey, Entity>,
}
