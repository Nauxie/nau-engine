use super::types::{IslandVisualCatalog, IslandVisualEntry, IslandVisualLayer};
use crate::world_collision_runtime::WorldCollisionProxyKind;
use bevy::prelude::{Vec2, Vec3};
use nau_engine::world::{
    SkyRoute, TERRAIN_BODY_COLLISION_PROXIES_PER_ISLAND, TERRAIN_RIM_COLLISION_PROXIES_PER_ISLAND,
};

const TERRAIN_RIM_NAME: &str = "island terrain rim collision";
const TERRAIN_BODY_NAME: &str = "island procedural cliff body";
const TERRAIN_BODY_COLLISION_NAME: &str = "island procedural cliff body collision";
const ARTIFACT_STAIR_NAME: &str = "ancient stair run";
const ARTIFACT_RETAINING_WALL_NAME: &str = "retaining wall fragment";
const ARTIFACT_GLYPH_SLAB_NAME: &str = "glyph stone slab";
const ARTIFACT_BRIDGE_FRAGMENT_NAME: &str = "broken bridge fragment";
const ARTIFACT_BANNER_NAME: &str = "weathered banner strips";
const ARTIFACT_PEBBLE_FIELD_NAME: &str = "pebble field";
const ARTIFACT_REED_PATCH_NAME: &str = "reed patch";
const ARTIFACT_VISUAL_FAMILY_NAMES: &[&str] = &[
    ARTIFACT_STAIR_NAME,
    ARTIFACT_RETAINING_WALL_NAME,
    ARTIFACT_GLYPH_SLAB_NAME,
    ARTIFACT_BRIDGE_FRAGMENT_NAME,
    ARTIFACT_BANNER_NAME,
    ARTIFACT_PEBBLE_FIELD_NAME,
    ARTIFACT_REED_PATCH_NAME,
];
const ARTIFACT_ROUTE_AFFORDANCE_NAMES: &[&str] =
    &[ARTIFACT_STAIR_NAME, ARTIFACT_BRIDGE_FRAGMENT_NAME];
const ARTIFACT_DECORATIVE_NAMES: &[&str] = &[
    ARTIFACT_BANNER_NAME,
    ARTIFACT_PEBBLE_FIELD_NAME,
    ARTIFACT_REED_PATCH_NAME,
];
const REQUIRED_SOLID_CAMERA_OBSTACLES: &[&str] =
    &[ARTIFACT_RETAINING_WALL_NAME, ARTIFACT_GLYPH_SLAB_NAME];
const SOLID_SURFACE_FEATURE_NAMES: &[&str] = &[
    "collapsed watchtower",
    "stacked leaning monoliths",
    "faceted crystal outcrop",
];

#[derive(Clone, Copy)]
struct SolidVisualRequirement {
    name: &'static str,
    kind: WorldCollisionProxyKind,
}

#[derive(Clone, Copy)]
struct NonBlockingVisualRequirement {
    name: &'static str,
    allow_camera_obstacle: bool,
}

const SOLID_VISUAL_REQUIREMENTS: &[SolidVisualRequirement] = &[
    SolidVisualRequirement {
        name: TERRAIN_BODY_NAME,
        kind: WorldCollisionProxyKind::TerrainBody,
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
        name: "broad-canopy tree trunk",
        kind: WorldCollisionProxyKind::Tree,
    },
    SolidVisualRequirement {
        name: "wind-bent tree trunk",
        kind: WorldCollisionProxyKind::Tree,
    },
    SolidVisualRequirement {
        name: "orchard tree trunk",
        kind: WorldCollisionProxyKind::Tree,
    },
    SolidVisualRequirement {
        name: "cypress tree trunk",
        kind: WorldCollisionProxyKind::Tree,
    },
    SolidVisualRequirement {
        name: "willow tree trunk",
        kind: WorldCollisionProxyKind::Tree,
    },
    SolidVisualRequirement {
        name: "alpine pine trunk",
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
    SolidVisualRequirement {
        name: "plateau arrival ruin marker",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "plateau high shelf route hint",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "plateau cave route hint",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: ARTIFACT_RETAINING_WALL_NAME,
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: ARTIFACT_GLYPH_SLAB_NAME,
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "collapsed watchtower",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "stacked leaning monoliths",
        kind: WorldCollisionProxyKind::Landmark,
    },
    SolidVisualRequirement {
        name: "faceted crystal outcrop",
        kind: WorldCollisionProxyKind::Landmark,
    },
];

const NON_BLOCKING_VISUAL_REQUIREMENTS: &[NonBlockingVisualRequirement] = &[
    NonBlockingVisualRequirement {
        name: "under-route cave mouth arch",
        allow_camera_obstacle: true,
    },
    NonBlockingVisualRequirement {
        name: "under-route hanging shelf",
        allow_camera_obstacle: true,
    },
    NonBlockingVisualRequirement {
        name: "under-route hanging roots",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "island pond",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "plateau lake",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "plateau waterfall ribbon",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "plateau waterfall mist",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "route waterfall ribbon",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "route waterfall mist",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "route lake",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: "river channel",
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: ARTIFACT_STAIR_NAME,
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: ARTIFACT_BRIDGE_FRAGMENT_NAME,
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: ARTIFACT_BANNER_NAME,
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: ARTIFACT_PEBBLE_FIELD_NAME,
        allow_camera_obstacle: false,
    },
    NonBlockingVisualRequirement {
        name: ARTIFACT_REED_PATCH_NAME,
        allow_camera_obstacle: false,
    },
];

const CAMERA_ONLY_ALLOWLIST: &[&str] = &[
    "broad-canopy tree canopy",
    "wind-bent tree canopy",
    "orchard tree canopy",
    "cypress tree canopy",
    "willow tree canopy",
    "alpine pine canopy",
    "launch camera tree canopy",
    "under-route cave mouth arch",
    "under-route hanging shelf",
    "ruined perimeter colonnade",
    "sunken open-air sanctum",
    "broken aqueduct arcade",
    "processional ruin stairs",
    "clustered basalt crown",
    "weathered rock arch",
    "fractured boulder spine",
];
const SOLID_PROXY_MAX_FLOAT_ABOVE_SURFACE_M: f32 = 0.35;
const SOLID_PROXY_MAX_BURY_BELOW_SURFACE_M: f32 = 1.25;
const SOLID_PROXY_OBSTACLE_BOUNDS_TOLERANCE_M: f32 = 0.08;
const SOLID_PROXY_FOOTPRINT_BOUNDS_TOLERANCE_M: f32 = 0.02;

#[derive(Debug)]
pub(crate) struct IslandCollisionCoverageAudit {
    pub(crate) passed: bool,
    pub(crate) checked_visual_count: usize,
    pub(crate) solid_visual_count: usize,
    pub(crate) surface_supported_solid_proxy_count: usize,
    pub(crate) footprint_bounded_solid_proxy_count: usize,
    pub(crate) min_solid_proxy_edge_clearance_m: f32,
    pub(crate) tree_solid_proxy_count: usize,
    pub(crate) tree_footprint_bounded_proxy_count: usize,
    pub(crate) rock_solid_proxy_count: usize,
    pub(crate) rock_footprint_bounded_proxy_count: usize,
    pub(crate) landmark_solid_proxy_count: usize,
    pub(crate) landmark_footprint_bounded_proxy_count: usize,
    pub(crate) obstacle_bounded_solid_proxy_count: usize,
    pub(crate) terrain_rim_proxy_count: usize,
    pub(crate) terrain_body_proxy_count: usize,
    pub(crate) camera_only_allowance_count: usize,
    pub(crate) non_blocking_visual_count: usize,
    pub(crate) failures: Vec<String>,
}

pub(crate) fn audit_island_collision_coverage(
    catalog: &IslandVisualCatalog,
    route: &SkyRoute,
) -> IslandCollisionCoverageAudit {
    let mut failures = Vec::new();
    let mut solid_visual_count = 0;
    let mut surface_supported_solid_proxy_count = 0;
    let mut footprint_bounded_solid_proxy_count = 0;
    let mut min_solid_proxy_edge_clearance_m = f32::INFINITY;
    let mut tree_solid_proxy_count = 0;
    let mut tree_footprint_bounded_proxy_count = 0;
    let mut rock_solid_proxy_count = 0;
    let mut rock_footprint_bounded_proxy_count = 0;
    let mut landmark_solid_proxy_count = 0;
    let mut landmark_footprint_bounded_proxy_count = 0;
    let mut obstacle_bounded_solid_proxy_count = 0;
    let mut terrain_rim_proxy_count = 0;
    let mut terrain_body_proxy_count = 0;
    let mut camera_only_allowance_count = 0;
    let mut non_blocking_visual_count = 0;
    let mut required_name_counts = SOLID_VISUAL_REQUIREMENTS
        .iter()
        .map(|requirement| (requirement.name, 0_usize))
        .collect::<Vec<_>>();
    let mut non_blocking_name_counts = non_blocking_requirements()
        .map(|requirement| (requirement.name, 0_usize))
        .collect::<Vec<_>>();
    let mut allowlisted_camera_only_counts = CAMERA_ONLY_ALLOWLIST
        .iter()
        .copied()
        .map(|name| (name, 0_usize))
        .collect::<Vec<_>>();

    for entry in &catalog.entries {
        let expected_solid = solid_requirement(entry.name);
        let expected_non_blocking = non_blocking_requirement(entry.name);
        if ARTIFACT_VISUAL_FAMILY_NAMES.contains(&entry.name)
            && expected_solid.is_none()
            && expected_non_blocking.is_none()
        {
            failures.push(format!(
                "{} on {} is an artifact family without an explicit collision classification",
                entry.name, entry.key.island_name
            ));
        }
        if let Some((_, count)) = required_name_counts
            .iter_mut()
            .find(|(name, _)| *name == entry.name)
        {
            *count += 1;
        }
        if let Some((_, count)) = non_blocking_name_counts
            .iter_mut()
            .find(|(name, _)| *name == entry.name)
        {
            *count += 1;
            non_blocking_visual_count += 1;
        }

        if let Some(collision) = entry.collision {
            if expected_non_blocking.is_some() {
                let classification =
                    non_blocking_artifact_classification(entry.name).unwrap_or("route/affordance");
                failures.push(format!(
                    "{} on {} must remain non-player-blocking {classification} visual",
                    entry.name, entry.key.island_name,
                ));
            }
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
                    } else {
                        match kind {
                            WorldCollisionProxyKind::Tree => tree_solid_proxy_count += 1,
                            WorldCollisionProxyKind::Rock => rock_solid_proxy_count += 1,
                            WorldCollisionProxyKind::Landmark => landmark_solid_proxy_count += 1,
                            WorldCollisionProxyKind::TerrainRim
                            | WorldCollisionProxyKind::TerrainBody => {}
                        }
                        surface_supported_solid_proxy_count += 1;
                        append_solid_proxy_surface_failures(entry, collision, &mut failures);
                        let edge_clearance_m = solid_proxy_min_edge_clearance_m(entry, collision);
                        min_solid_proxy_edge_clearance_m =
                            min_solid_proxy_edge_clearance_m.min(edge_clearance_m);
                        if edge_clearance_m >= -SOLID_PROXY_FOOTPRINT_BOUNDS_TOLERANCE_M {
                            footprint_bounded_solid_proxy_count += 1;
                            match kind {
                                WorldCollisionProxyKind::Tree => {
                                    tree_footprint_bounded_proxy_count += 1;
                                }
                                WorldCollisionProxyKind::Rock => {
                                    rock_footprint_bounded_proxy_count += 1;
                                }
                                WorldCollisionProxyKind::Landmark => {
                                    landmark_footprint_bounded_proxy_count += 1;
                                }
                                WorldCollisionProxyKind::TerrainRim
                                | WorldCollisionProxyKind::TerrainBody => {}
                            }
                        } else {
                            failures.push(format!(
                                "{} on {} has {:?} collision footprint extending {:.3}m past the visible island support",
                                entry.name, entry.key.island_name, collision.kind, -edge_clearance_m
                            ));
                        }
                        if entry.obstacle.is_some() {
                            obstacle_bounded_solid_proxy_count += 1;
                            append_solid_proxy_obstacle_failures(entry, collision, &mut failures);
                        }
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

        if requires_solid_camera_obstacle(entry.name) {
            match (entry.collision, entry.obstacle) {
                (Some(collision), Some(obstacle)) => {
                    if !is_bounded_aabb(collision.center, collision.half_extents) {
                        failures.push(format!(
                            "{} on {} must use a finite positive player-collision AABB",
                            entry.name, entry.key.island_name
                        ));
                    }
                    if !is_bounded_aabb(obstacle.0.center, obstacle.0.half_extents) {
                        failures.push(format!(
                            "{} on {} must use a finite positive camera-obstruction AABB",
                            entry.name, entry.key.island_name
                        ));
                    }
                }
                (_, None) => failures.push(format!(
                    "{} on {} is a solid artifact but has no camera-obstruction AABB",
                    entry.name, entry.key.island_name
                )),
                (None, Some(_)) => {}
            }
        }

        if let Some(requirement) = expected_non_blocking
            && entry.obstacle.is_some()
            && !requirement.allow_camera_obstacle
        {
            failures.push(format!(
                "{} on {} should not block the player or camera",
                entry.name, entry.key.island_name
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

    for (name, count) in non_blocking_name_counts {
        if count == 0 {
            failures.push(format!(
                "{name} is classified as non-player-blocking but is missing from the catalog"
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

    let min_solid_proxy_edge_clearance_m = if min_solid_proxy_edge_clearance_m.is_finite() {
        min_solid_proxy_edge_clearance_m
    } else {
        0.0
    };

    IslandCollisionCoverageAudit {
        passed: failures.is_empty(),
        checked_visual_count: catalog.entries.len(),
        solid_visual_count,
        surface_supported_solid_proxy_count,
        footprint_bounded_solid_proxy_count,
        min_solid_proxy_edge_clearance_m,
        tree_solid_proxy_count,
        tree_footprint_bounded_proxy_count,
        rock_solid_proxy_count,
        rock_footprint_bounded_proxy_count,
        landmark_solid_proxy_count,
        landmark_footprint_bounded_proxy_count,
        obstacle_bounded_solid_proxy_count,
        terrain_rim_proxy_count,
        terrain_body_proxy_count,
        camera_only_allowance_count,
        non_blocking_visual_count,
        failures,
    }
}

fn solid_requirement(name: &str) -> Option<SolidVisualRequirement> {
    SOLID_VISUAL_REQUIREMENTS
        .iter()
        .copied()
        .find(|requirement| requirement.name == name)
}

fn non_blocking_requirement(name: &str) -> Option<NonBlockingVisualRequirement> {
    non_blocking_requirements().find(|requirement| requirement.name == name)
}

fn non_blocking_requirements() -> impl Iterator<Item = NonBlockingVisualRequirement> {
    NON_BLOCKING_VISUAL_REQUIREMENTS.iter().copied()
}

fn requires_solid_camera_obstacle(name: &str) -> bool {
    REQUIRED_SOLID_CAMERA_OBSTACLES.contains(&name) || SOLID_SURFACE_FEATURE_NAMES.contains(&name)
}

fn non_blocking_artifact_classification(name: &str) -> Option<&'static str> {
    if ARTIFACT_ROUTE_AFFORDANCE_NAMES.contains(&name) {
        Some("route-affordance")
    } else if ARTIFACT_DECORATIVE_NAMES.contains(&name) {
        Some("decorative")
    } else {
        None
    }
}

fn is_bounded_aabb(center: Vec3, half_extents: Vec3) -> bool {
    center.is_finite()
        && half_extents.is_finite()
        && half_extents.x > 0.0
        && half_extents.y > 0.0
        && half_extents.z > 0.0
}

fn append_solid_proxy_surface_failures(
    entry: &IslandVisualEntry,
    collision: crate::world_collision_runtime::WorldCollisionProxy,
    failures: &mut Vec<String>,
) {
    if !entry.island.contains_horizontal(collision.center) {
        failures.push(format!(
            "{} on {} has {:?} collision centered outside the visible island footprint",
            entry.name, entry.key.island_name, collision.kind
        ));
        return;
    }

    let surface_y = entry.island.mesh_top_y_at(collision.center);
    let bottom_y = collision.center.y - collision.half_extents.y;
    let surface_delta_m = bottom_y - surface_y;
    if surface_delta_m > SOLID_PROXY_MAX_FLOAT_ABOVE_SURFACE_M {
        failures.push(format!(
            "{} on {} has {:?} collision bottom {:.2}m above the visible surface",
            entry.name, entry.key.island_name, collision.kind, surface_delta_m
        ));
    }
    if surface_delta_m < -SOLID_PROXY_MAX_BURY_BELOW_SURFACE_M {
        failures.push(format!(
            "{} on {} has {:?} collision bottom {:.2}m below the visible surface",
            entry.name, entry.key.island_name, collision.kind, -surface_delta_m
        ));
    }
}

fn append_solid_proxy_obstacle_failures(
    entry: &IslandVisualEntry,
    collision: crate::world_collision_runtime::WorldCollisionProxy,
    failures: &mut Vec<String>,
) {
    let Some(obstacle) = entry.obstacle else {
        return;
    };

    let obstacle = obstacle.0;
    let tolerance = Vec3::splat(SOLID_PROXY_OBSTACLE_BOUNDS_TOLERANCE_M);
    let collision_min = collision.center - collision.half_extents;
    let collision_max = collision.center + collision.half_extents;
    let obstacle_min = obstacle.center - obstacle.half_extents - tolerance;
    let obstacle_max = obstacle.center + obstacle.half_extents + tolerance;

    if collision_min.x < obstacle_min.x
        || collision_min.y < obstacle_min.y
        || collision_min.z < obstacle_min.z
        || collision_max.x > obstacle_max.x
        || collision_max.y > obstacle_max.y
        || collision_max.z > obstacle_max.z
    {
        failures.push(format!(
            "{} on {} has {:?} collision extending outside its visible obstacle envelope",
            entry.name, entry.key.island_name, collision.kind
        ));
    }
}

fn solid_proxy_min_edge_clearance_m(
    entry: &IslandVisualEntry,
    collision: crate::world_collision_runtime::WorldCollisionProxy,
) -> f32 {
    solid_proxy_horizontal_samples(collision)
        .into_iter()
        .map(|sample| island_horizontal_edge_clearance_m(entry.island, sample))
        .fold(f32::INFINITY, f32::min)
}

fn solid_proxy_horizontal_samples(
    collision: crate::world_collision_runtime::WorldCollisionProxy,
) -> [Vec3; 9] {
    let center = collision.center;
    let half_extents = collision.half_extents;
    [
        Vec3::new(
            center.x - half_extents.x,
            center.y,
            center.z - half_extents.z,
        ),
        Vec3::new(center.x, center.y, center.z - half_extents.z),
        Vec3::new(
            center.x + half_extents.x,
            center.y,
            center.z - half_extents.z,
        ),
        Vec3::new(center.x - half_extents.x, center.y, center.z),
        center,
        Vec3::new(center.x + half_extents.x, center.y, center.z),
        Vec3::new(
            center.x - half_extents.x,
            center.y,
            center.z + half_extents.z,
        ),
        Vec3::new(center.x, center.y, center.z + half_extents.z),
        Vec3::new(
            center.x + half_extents.x,
            center.y,
            center.z + half_extents.z,
        ),
    ]
}

fn island_horizontal_edge_clearance_m(island: nau_engine::world::SkyIsland, position: Vec3) -> f32 {
    let dx = (position.x - island.center.x) / island.half_extents.x.max(0.001);
    let dz = (position.z - island.center.z) / island.half_extents.y.max(0.001);
    let radius = Vec2::new(dx, dz).length();
    let angle = dz.atan2(dx);

    (island.playable_silhouette_scale(angle) - radius) * island.half_extents.min_element()
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
            material: collision.map(|_| Handle::<StandardMaterial>::default().into()),
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
    fn artifact_families_have_exhaustive_collision_requirements() {
        assert_eq!(ARTIFACT_VISUAL_FAMILY_NAMES.len(), 7);
        assert_eq!(REQUIRED_SOLID_CAMERA_OBSTACLES.len(), 2);
        assert_eq!(ARTIFACT_ROUTE_AFFORDANCE_NAMES.len(), 2);
        assert_eq!(ARTIFACT_DECORATIVE_NAMES.len(), 3);

        for name in ARTIFACT_VISUAL_FAMILY_NAMES {
            let is_solid = solid_requirement(name).is_some();
            let is_route_affordance = ARTIFACT_ROUTE_AFFORDANCE_NAMES.contains(name);
            let is_decorative = ARTIFACT_DECORATIVE_NAMES.contains(name);
            assert_eq!(
                usize::from(is_solid)
                    + usize::from(is_route_affordance)
                    + usize::from(is_decorative),
                1,
                "{name} must have exactly one artifact collision classification"
            );

            if is_solid {
                assert_eq!(
                    solid_requirement(name).map(|requirement| requirement.kind),
                    Some(WorldCollisionProxyKind::Landmark)
                );
                assert!(requires_solid_camera_obstacle(name));
            } else {
                let requirement =
                    non_blocking_requirement(name).expect("non-blocking artifact requirement");
                assert!(!requirement.allow_camera_obstacle);
            }
        }
    }

    #[test]
    fn audit_fails_non_blocking_visuals_with_player_collision() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let surface_y = island.mesh_top_y_at(island.center);
        let collision = WorldCollisionProxy::new(
            Vec3::new(island.center.x, surface_y + 0.5, island.center.z),
            Vec3::splat(0.4),
            WorldCollisionProxyKind::Landmark,
        );
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                ARTIFACT_STAIR_NAME,
                IslandVisualLayer::Beacon,
                None,
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains(ARTIFACT_STAIR_NAME)
                && failure.contains("must remain non-player-blocking")
                && failure.contains("route-affordance")
        }));
    }

    #[test]
    fn audit_fails_non_blocking_visuals_with_camera_obstacle() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let blocker = CameraObstacle(CameraObstruction::new(Vec3::ZERO, Vec3::ONE));
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                ARTIFACT_BANNER_NAME,
                IslandVisualLayer::Beacon,
                Some(blocker),
                None,
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains(ARTIFACT_BANNER_NAME)
                && failure.contains("should not block the player or camera")
        }));
    }

    #[test]
    fn audit_requires_solid_artifact_camera_obstruction() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let surface_y = island.mesh_top_y_at(island.center);
        let collision = WorldCollisionProxy::new(
            Vec3::new(island.center.x, surface_y + 0.5, island.center.z),
            Vec3::splat(0.5),
            WorldCollisionProxyKind::Landmark,
        );
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                ARTIFACT_RETAINING_WALL_NAME,
                IslandVisualLayer::Detail,
                None,
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains(ARTIFACT_RETAINING_WALL_NAME)
                && failure.contains("no camera-obstruction AABB")
        }));
    }

    #[test]
    fn audit_rejects_unbounded_solid_artifact_aabbs() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let surface_y = island.mesh_top_y_at(island.center);
        let center = Vec3::new(island.center.x, surface_y, island.center.z);
        let collision =
            WorldCollisionProxy::new(center, Vec3::ZERO, WorldCollisionProxyKind::Landmark);
        let obstacle = CameraObstacle(CameraObstruction::new(center, Vec3::ZERO));
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                ARTIFACT_GLYPH_SLAB_NAME,
                IslandVisualLayer::Detail,
                Some(obstacle),
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains(ARTIFACT_GLYPH_SLAB_NAME)
                && failure.contains("finite positive player-collision AABB")
        }));
        assert!(audit.failures.iter().any(|failure| {
            failure.contains(ARTIFACT_GLYPH_SLAB_NAME)
                && failure.contains("finite positive camera-obstruction AABB")
        }));
    }

    #[test]
    fn audit_fails_solid_visuals_with_wrong_proxy_kind() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let collision = WorldCollisionProxy::new(Vec3::Y, Vec3::ONE, WorldCollisionProxyKind::Rock);
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                "broad-canopy tree trunk",
                IslandVisualLayer::Detail,
                None,
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains("broad-canopy tree trunk") && failure.contains("expected Tree")
        }));
    }

    #[test]
    fn audit_fails_solid_visuals_outside_visible_island_footprint() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let outside = island.center + Vec3::new(island.half_extents.x * 3.0, 1.0, 0.0);
        let collision =
            WorldCollisionProxy::new(outside, Vec3::splat(0.4), WorldCollisionProxyKind::Rock);
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                "island stone scatter",
                IslandVisualLayer::Detail,
                None,
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains("island stone scatter")
                && failure.contains("outside the visible island footprint")
        }));
    }

    #[test]
    fn audit_fails_solid_visuals_floating_above_visible_surface() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let surface_y = island.mesh_top_y_at(island.center);
        let floating_center = Vec3::new(island.center.x, surface_y + 3.0, island.center.z);
        let collision = WorldCollisionProxy::new(
            floating_center,
            Vec3::splat(0.4),
            WorldCollisionProxyKind::Rock,
        );
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                "island stone scatter",
                IslandVisualLayer::Detail,
                None,
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains("island stone scatter")
                && failure.contains("above the visible surface")
        }));
    }

    #[test]
    fn audit_fails_solid_visual_collision_footprint_spilling_past_visible_edge() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let contour = island.footprint_contour_point(0.0, false);
        let island_center = Vec2::new(island.center.x, island.center.z);
        let outward = (contour - island_center).normalize_or_zero();
        let center_2d = contour - outward * 0.04;
        let surface_position = Vec3::new(center_2d.x, island.center.y, center_2d.y);
        let surface_y = island.mesh_top_y_at(surface_position);
        let collision = WorldCollisionProxy::new(
            Vec3::new(center_2d.x, surface_y + 0.4, center_2d.y),
            Vec3::splat(0.4),
            WorldCollisionProxyKind::Rock,
        );
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                "island stone scatter",
                IslandVisualLayer::Detail,
                None,
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.min_solid_proxy_edge_clearance_m < -SOLID_PROXY_FOOTPRINT_BOUNDS_TOLERANCE_M);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains("island stone scatter")
                && failure.contains("collision footprint extending")
                && failure.contains("past the visible island support")
        }));
    }

    #[test]
    fn audit_fails_solid_visual_collision_outside_obstacle_envelope() {
        let route = SkyRoute::default();
        let island = route.islands()[0];
        let surface_y = island.mesh_top_y_at(island.center);
        let center = Vec3::new(island.center.x, surface_y + 1.0, island.center.z);
        let obstacle = CameraObstacle(CameraObstruction::new(center, Vec3::splat(0.45)));
        let collision =
            WorldCollisionProxy::new(center, Vec3::splat(0.9), WorldCollisionProxyKind::Tree);
        let catalog = IslandVisualCatalog {
            entries: vec![audit_entry(
                island,
                "broad-canopy tree trunk",
                IslandVisualLayer::Detail,
                Some(obstacle),
                Some(collision),
            )],
        };

        let audit = audit_island_collision_coverage(&catalog, &route);

        assert!(!audit.passed);
        assert!(audit.failures.iter().any(|failure| {
            failure.contains("broad-canopy tree trunk")
                && failure.contains("outside its visible obstacle envelope")
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
