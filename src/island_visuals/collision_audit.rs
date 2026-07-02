use super::types::{IslandVisualCatalog, IslandVisualLayer};
use crate::world_collision_runtime::WorldCollisionProxyKind;
use nau_engine::world::{
    SkyRoute, TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND, TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND,
};

const TERRAIN_RIM_NAME: &str = "island terrain rim collision";
const TERRAIN_BODY_NAME: &str = "island procedural cliff body";
const TERRAIN_BODY_COLLISION_NAME: &str = "island procedural cliff body collision";

#[derive(Clone, Copy)]
struct SolidVisualRequirement {
    name: &'static str,
    kind: WorldCollisionProxyKind,
}

const SOLID_VISUAL_REQUIREMENTS: &[SolidVisualRequirement] = &[
    SolidVisualRequirement {
        name: TERRAIN_BODY_NAME,
        kind: WorldCollisionProxyKind::TerrainBody,
    },
    SolidVisualRequirement {
        name: "island ridge",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "landing target marker",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "recovery branch mast",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "recovery branch ring",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "island tree trunk",
        kind: WorldCollisionProxyKind::Tree,
    },
    SolidVisualRequirement {
        name: "launch camera tree trunk",
        kind: WorldCollisionProxyKind::Tree,
    },
    SolidVisualRequirement {
        name: "island stone scatter",
        kind: WorldCollisionProxyKind::Rock,
    },
    SolidVisualRequirement {
        name: "route cairn",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "landing garden ring",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "launch beacon",
        kind: WorldCollisionProxyKind::Landmark,
    },
];

const CAMERA_ONLY_ALLOWLIST: &[&str] = &[
    "island tree canopy",
    "launch camera tree canopy",
    "under-route cave mouth arch",
    "under-route hanging shelf",
];

#[derive(Debug)]
pub(crate) struct IslandCollisionCoverageAudit {
    pub(crate) passed: bool,
    pub(crate) checked_visual_count: usize,
    pub(crate) solid_visual_count: usize,
    pub(crate) terrain_rim_proxy_count: usize,
    pub(crate) terrain_body_proxy_count: usize,
    pub(crate) camera_only_allowance_count: usize,
    pub(crate) failures: Vec<String>,
}

pub(crate) fn audit_island_collision_coverage(
    catalog: &IslandVisualCatalog,
    route: &SkyRoute,
) -> IslandCollisionCoverageAudit {
    let mut failures = Vec::new();
    let mut solid_visual_count = 0;
    let mut terrain_rim_proxy_count = 0;
    let mut terrain_body_proxy_count = 0;
    let mut camera_only_allowance_count = 0;
    let mut required_name_counts = SOLID_VISUAL_REQUIREMENTS
        .iter()
        .map(|requirement| (requirement.name, 0_usize))
        .collect::<Vec<_>>();
    let mut allowlisted_camera_only_counts = CAMERA_ONLY_ALLOWLIST
        .iter()
        .map(|name| (*name, 0_usize))
        .collect::<Vec<_>>();

    for entry in &catalog.entries {
        let expected_solid = solid_requirement(entry.name);
        if let Some((_, count)) = required_name_counts
            .iter_mut()
            .find(|(name, _)| *name == entry.name)
        {
            *count += 1;
        }

        if let Some(collision) = entry.collision {
            match collision.kind {
                WorldCollisionProxyKind::TerrainRim => {
                    terrain_rim_proxy_count += 1;
                    if entry.name != TERRAIN_RIM_NAME {
                        failures.push(format!(
                            "{} on {} uses terrain-rim collision without the terrain-rim name",
                            entry.name, entry.key.island_name
                        ));
                    }
                    if entry.layer != IslandVisualLayer::Collision {
                        failures.push(format!(
                            "{} on {} uses terrain-rim collision outside the collision layer",
                            entry.name, entry.key.island_name
                        ));
                    }
                    if entry.has_visible_mesh()
                        || entry.material.is_some()
                        || entry.obstacle.is_some()
                    {
                        failures.push(format!(
                            "{} on {} should remain an invisible collision-only rim segment",
                            entry.name, entry.key.island_name
                        ));
                    }
                }
                WorldCollisionProxyKind::TerrainBody
                    if entry.name == TERRAIN_BODY_COLLISION_NAME =>
                {
                    terrain_body_proxy_count += 1;
                    if entry.layer != IslandVisualLayer::Collision {
                        failures.push(format!(
                            "{} on {} uses terrain-body collision outside the collision layer",
                            entry.name, entry.key.island_name
                        ));
                    }
                    if entry.has_visible_mesh()
                        || entry.material.is_some()
                        || entry.obstacle.is_some()
                    {
                        failures.push(format!(
                            "{} on {} should remain an invisible collision-only cliff body segment",
                            entry.name, entry.key.island_name
                        ));
                    }
                }
                kind => {
                    if kind == WorldCollisionProxyKind::TerrainBody {
                        terrain_body_proxy_count += 1;
                    }
                    solid_visual_count += 1;
                    if !entry.has_visible_mesh() || entry.material.is_none() {
                        failures.push(format!(
                            "{} on {} has solid collision but no visible mesh/material",
                            entry.name, entry.key.island_name
                        ));
                    }
                    match expected_solid {
                        Some(requirement) if requirement.kind == kind => {}
                        Some(requirement) => failures.push(format!(
                            "{} on {} uses {:?} collision, expected {:?}",
                            entry.name, entry.key.island_name, kind, requirement.kind
                        )),
                        None => failures.push(format!(
                            "{} on {} has solid collision but is not classified in the solid-visual audit",
                            entry.name, entry.key.island_name
                        )),
                    }
                }
            }
        } else if let Some(requirement) = expected_solid {
            failures.push(format!(
                "{} on {} is a named solid visual but has no {:?} collision proxy",
                entry.name, entry.key.island_name, requirement.kind
            ));
        }

        if entry.obstacle.is_some() && entry.collision.is_none() && expected_solid.is_none() {
            if let Some((_, count)) = allowlisted_camera_only_counts
                .iter_mut()
                .find(|(name, _)| *name == entry.name)
            {
                *count += 1;
                camera_only_allowance_count += 1;
            } else {
                failures.push(format!(
                    "{} on {} blocks the camera without collision and is not camera-only allowlisted",
                    entry.name, entry.key.island_name
                ));
            }
        }
    }

    for island in route.islands() {
        let island_body_count = catalog
            .entries
            .iter()
            .filter(|entry| {
                entry.key.island_name == island.name
                    && entry.collision.is_some_and(|collision| {
                        collision.kind == WorldCollisionProxyKind::TerrainBody
                    })
            })
            .count();
        if island_body_count != TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND {
            failures.push(format!(
                "{} has {island_body_count} terrain-body cliff segments, expected {TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND}",
                island.name
            ));
        }

        let island_rim_count = catalog
            .entries
            .iter()
            .filter(|entry| {
                entry.key.island_name == island.name
                    && entry.name == TERRAIN_RIM_NAME
                    && entry.collision.is_some_and(|collision| {
                        collision.kind == WorldCollisionProxyKind::TerrainRim
                    })
            })
            .count();
        if island_rim_count != TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND {
            failures.push(format!(
                "{} has {island_rim_count} terrain-rim contour segments, expected {TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND}",
                island.name
            ));
        }
    }

    for (name, count) in required_name_counts {
        if count == 0 {
            failures.push(format!(
                "{name} is classified as solid but is missing from the catalog"
            ));
        }
    }

    for (name, count) in allowlisted_camera_only_counts {
        if count == 0 {
            failures.push(format!(
                "{name} is camera-only allowlisted but is missing from the catalog"
            ));
        }
    }

    IslandCollisionCoverageAudit {
        passed: failures.is_empty(),
        checked_visual_count: catalog.entries.len(),
        solid_visual_count,
        terrain_rim_proxy_count,
        terrain_body_proxy_count,
        camera_only_allowance_count,
        failures,
    }
}

fn solid_requirement(name: &str) -> Option<SolidVisualRequirement> {
    SOLID_VISUAL_REQUIREMENTS
        .iter()
        .copied()
        .find(|requirement| requirement.name == name)
}

#[cfg(test)]
mod tests {
    use super::super::types::{IslandVisualEntry, IslandVisualKey};
    use super::*;
    use crate::camera_runtime::CameraObstacle;
    use crate::world_collision_runtime::WorldCollisionProxy;
    use bevy::prelude::*;
    use nau_engine::camera::CameraObstruction;

    fn audit_entry(
        island: nau_engine::world::SkyIsland,
        name: &'static str,
        layer: IslandVisualLayer,
        obstacle: Option<CameraObstacle>,
        collision: Option<WorldCollisionProxy>,
    ) -> IslandVisualEntry {
        IslandVisualEntry {
            key: IslandVisualKey {
                island_name: island.name,
                layer,
                index: 0,
            },
            island,
            layer,
            mesh: collision.map(|_| Handle::<Mesh>::default()),
            mesh_recipe: None,
            material: collision.map(|_| Handle::<StandardMaterial>::default()),
            transform: Transform::default(),
            obstacle,
            collision,
            wind_motion: None,
            name,
        }
    }

    #[test]
    fn audit_fails_unallowlisted_camera_only_blockers() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let blocker = CameraObstacle(CameraObstruction::new(Vec3::ZERO, Vec3::ONE));
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                "new camera blocker",
                IslandVisualLayer::Detail,
                Some(blocker),
                None,
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(
            audit
                .failures
                .iter()
                .any(|failure| failure.contains("not camera-only allowlisted"))
        );
    }

    #[test]
    fn audit_fails_solid_visuals_with_wrong_proxy_kind() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let collision = WorldCollisionProxy::new(Vec3::Y, Vec3::ONE, WorldCollisionProxyKind::Rock);
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                "island tree trunk",
                IslandVisualLayer::Detail,
                None,
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains("island tree trunk") && failure.contains("expected Tree")
        }));
    }

    #[test]
    fn audit_requires_procedural_cliff_body_terrain_body_collision() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let blocker = CameraObstacle(CameraObstruction::new(Vec3::ZERO, Vec3::ONE));
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                TERRAIN_BODY_NAME,
                IslandVisualLayer::Terrain,
                Some(blocker),
                None,
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains(TERRAIN_BODY_NAME) && failure.contains("no TerrainBody")
        }));
    }
}
