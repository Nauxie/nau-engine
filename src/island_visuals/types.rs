use crate::camera_runtime::CameraObstacle;
use crate::environment_visuals::WindVisualMotion;
use crate::generated_content::{island_cliff_mesh, island_terrain_mesh, island_underside_mesh};
use crate::world_collision_runtime::WorldCollisionProxy;
use bevy::prelude::*;
use nau_engine::world::{LodBand, SkyIsland, StreamActivation};
use std::collections::{HashMap, HashSet};

#[derive(Component, Clone, Copy, Debug)]
pub(super) struct IslandLodVisual;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum IslandVisualLayer {
    Terrain,
    Detail,
    Vista,
    Beacon,
    Impostor,
    Collision,
}

impl IslandVisualLayer {
    pub(super) fn is_resident_in(self, activation: StreamActivation, band: LodBand) -> bool {
        match self {
            Self::Terrain => activation.is_active() && band != LodBand::Far,
            Self::Detail => activation.is_active() && band == LodBand::Near,
            Self::Vista => band != LodBand::Far,
            Self::Beacon => band != LodBand::Far,
            Self::Impostor => !activation.is_active() || band == LodBand::Far,
            Self::Collision => activation.is_active() && band == LodBand::Near,
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
            (IslandVisualLayer::Vista, false) => self.visible_detail_count += 1,
            (IslandVisualLayer::Vista, true) => self.hidden_detail_count += 1,
            (IslandVisualLayer::Beacon, false) => self.visible_beacon_count += 1,
            (IslandVisualLayer::Beacon, true) => {}
            (IslandVisualLayer::Impostor, false) => self.visible_impostor_count += 1,
            (IslandVisualLayer::Impostor, true) => self.hidden_impostor_count += 1,
            (IslandVisualLayer::Collision, _) => {}
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

#[derive(Clone, Copy, Debug)]
pub(super) enum IslandVisualMeshRecipe {
    Terrain {
        island_index: usize,
        island: SkyIsland,
    },
    Cliff {
        island_index: usize,
        island: SkyIsland,
    },
    Underside {
        island_index: usize,
        island: SkyIsland,
    },
}

impl IslandVisualMeshRecipe {
    pub(super) fn build_mesh(self) -> Mesh {
        match self {
            Self::Terrain {
                island_index,
                island,
            } => island_terrain_mesh(island_index, island),
            Self::Cliff {
                island_index,
                island,
            } => island_cliff_mesh(island_index, island),
            Self::Underside {
                island_index,
                island,
            } => island_underside_mesh(island_index, island),
        }
    }
}

#[derive(Clone)]
pub(super) struct IslandVisualEntry {
    pub(super) key: IslandVisualKey,
    pub(super) island: SkyIsland,
    pub(super) layer: IslandVisualLayer,
    pub(super) mesh: Option<Handle<Mesh>>,
    pub(super) mesh_recipe: Option<IslandVisualMeshRecipe>,
    pub(super) material: Option<Handle<StandardMaterial>>,
    pub(super) transform: Transform,
    pub(super) obstacle: Option<CameraObstacle>,
    pub(super) collision: Option<WorldCollisionProxy>,
    pub(super) wind_motion: Option<WindVisualMotion>,
    pub(super) name: &'static str,
}

impl IslandVisualEntry {
    pub(super) fn has_visible_mesh(&self) -> bool {
        self.mesh.is_some() || self.mesh_recipe.is_some()
    }
}

#[derive(Resource, Default)]
pub(crate) struct IslandVisualCatalog {
    pub(super) entries: Vec<IslandVisualEntry>,
}

impl IslandVisualCatalog {
    #[cfg(test)]
    pub(crate) fn deferred_mesh_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.mesh_recipe.is_some())
            .count()
    }

    #[cfg(test)]
    pub(crate) fn prebuilt_mesh_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.mesh.is_some())
            .count()
    }

    #[cfg(test)]
    pub(crate) fn resident_collision_proxy_count(
        &self,
        player_position: Vec3,
        kind: crate::world_collision_runtime::WorldCollisionProxyKind,
    ) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry
                    .collision
                    .is_some_and(|collision| collision.kind == kind)
                    && super::streaming::island_entry_is_resident(entry, player_position)
            })
            .count()
    }

    #[cfg(test)]
    pub(crate) fn named_obstacle_count(
        &self,
        island_name: &'static str,
        name: &'static str,
    ) -> usize {
        self.entries
            .iter()
            .filter(|entry| {
                entry.key.island_name == island_name
                    && entry.name == name
                    && entry.obstacle.is_some()
            })
            .count()
    }
}

#[derive(Resource, Default)]
pub(crate) struct IslandStreamState {
    pub(super) spawned: HashMap<IslandVisualKey, Entity>,
    pub(super) visual_resident: HashSet<IslandVisualKey>,
    pub(super) loaded_meshes: HashMap<IslandVisualKey, Handle<Mesh>>,
}

impl IslandStreamState {
    #[cfg(test)]
    pub(crate) fn loaded_mesh_count(&self) -> usize {
        self.loaded_meshes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terrain_shell_streams_before_near_detail_and_collision() {
        assert!(IslandVisualLayer::Terrain.is_resident_in(StreamActivation::Active, LodBand::Near));
        assert!(IslandVisualLayer::Terrain.is_resident_in(StreamActivation::Active, LodBand::Mid));
        assert!(!IslandVisualLayer::Terrain.is_resident_in(StreamActivation::Active, LodBand::Far));
        assert!(
            !IslandVisualLayer::Terrain.is_resident_in(StreamActivation::Inactive, LodBand::Near)
        );

        for layer in [IslandVisualLayer::Detail, IslandVisualLayer::Collision] {
            assert!(layer.is_resident_in(StreamActivation::Active, LodBand::Near));
            assert!(!layer.is_resident_in(StreamActivation::Active, LodBand::Mid));
            assert!(!layer.is_resident_in(StreamActivation::Active, LodBand::Far));
            assert!(!layer.is_resident_in(StreamActivation::Inactive, LodBand::Near));
        }
    }

    #[test]
    fn impostors_cover_mid_far_and_inactive_islands() {
        assert!(
            !IslandVisualLayer::Impostor.is_resident_in(StreamActivation::Active, LodBand::Near)
        );
        assert!(
            !IslandVisualLayer::Impostor.is_resident_in(StreamActivation::Active, LodBand::Mid)
        );
        assert!(IslandVisualLayer::Impostor.is_resident_in(StreamActivation::Active, LodBand::Far));
        assert!(
            IslandVisualLayer::Impostor.is_resident_in(StreamActivation::Inactive, LodBand::Near)
        );
    }

    #[test]
    fn beacons_stay_visible_until_far_lod() {
        assert!(IslandVisualLayer::Beacon.is_resident_in(StreamActivation::Active, LodBand::Near));
        assert!(IslandVisualLayer::Beacon.is_resident_in(StreamActivation::Inactive, LodBand::Mid));
        assert!(!IslandVisualLayer::Beacon.is_resident_in(StreamActivation::Active, LodBand::Far));
        assert!(
            !IslandVisualLayer::Beacon.is_resident_in(StreamActivation::Inactive, LodBand::Far)
        );
    }

    #[test]
    fn vistas_stay_visible_through_mid_lod_independent_of_activation() {
        assert!(IslandVisualLayer::Vista.is_resident_in(StreamActivation::Active, LodBand::Near));
        assert!(IslandVisualLayer::Vista.is_resident_in(StreamActivation::Inactive, LodBand::Near));
        assert!(IslandVisualLayer::Vista.is_resident_in(StreamActivation::Active, LodBand::Mid));
        assert!(IslandVisualLayer::Vista.is_resident_in(StreamActivation::Inactive, LodBand::Mid));
        assert!(!IslandVisualLayer::Vista.is_resident_in(StreamActivation::Active, LodBand::Far));
        assert!(!IslandVisualLayer::Vista.is_resident_in(StreamActivation::Inactive, LodBand::Far));
    }

    #[test]
    fn vistas_count_toward_detail_diagnostics() {
        let mut counts = IslandLodVisualCounts::default();

        counts.record(IslandVisualLayer::Vista, false);
        counts.record(IslandVisualLayer::Vista, true);

        assert_eq!(counts.visible_detail_count, 1);
        assert_eq!(counts.hidden_detail_count, 1);
        assert_eq!(counts.catalog_count(), 2);
    }
}
