use super::queue::{
    queue_collidable_island_visual, queue_collidable_wind_island_visual, queue_island_visual,
    queue_wind_island_visual,
};
use super::types::{IslandVisualEntry, IslandVisualLayer, IslandVisualMaterial};
use crate::camera_runtime::CameraObstacle;
use crate::content_diagnostics::{GeneratedLandmarkKind, IslandContentDiagnostics};
use crate::environment_visuals::wind_visual_motion;
use crate::generated_content::{
    FirstExpeditionSilhouetteKind, GROUND_COVER_BLADES_PER_PATCH, IslandArtifactMaterial,
    IslandDetailBudget, IslandDetailMaterials, IslandRockSpec, IslandRuinSpec, IslandTreeSpec,
    IslandUnderRouteVisualKind, IslandWaterVisualKind, TreeSpecies, WaterSurfaceMaterials,
    cliff_tooth_ridge_mesh, first_expedition_silhouette_specs, garden_ring_mesh,
    island_artifact_visual_specs, island_detail_budget, island_ground_cover_mesh,
    island_lake_basin_visual_specs, island_playable_normalized_offset, island_rock_specs,
    island_ruin_specs, island_tree_specs, island_under_route_visual_specs,
    island_visual_surface_position, island_water_visual_specs, landing_garden_marker_mesh,
    launch_beacon_mesh, rock_scatter_mesh, route_cairn_mesh, ruin_arch_mesh,
    tree_canopy_mesh_for_species, tree_trunk_mesh_for_species,
};
use bevy::prelude::*;
use nau_engine::camera::CameraObstruction;
use nau_engine::world::{IslandPlateauRegion, IslandTerrainArchetype, SkyIsland};

use crate::world_collision_runtime::{WorldCollisionProxy, WorldCollisionProxyKind};

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_sky_island_details(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    detail_materials: IslandDetailMaterials,
    flower_material: Handle<StandardMaterial>,
    water_materials: &WaterSurfaceMaterials,
    island_index: usize,
    island: SkyIsland,
) {
    let detail_phase = island_index as f32 * 0.77;
    let detail_budget: IslandDetailBudget = island_detail_budget(island);
    let plateau_authored_layer = if island.is_great_plateau_anchor() {
        IslandVisualLayer::Vista
    } else {
        IslandVisualLayer::Detail
    };
    content_diagnostics.record_detail_biome_palette(island_index);
    let ground_cover_mesh =
        island_ground_cover_mesh(island_index, island, detail_budget.ground_cover_patch_count);
    content_diagnostics.record_generated_ground_cover(
        detail_budget.ground_cover_patch_count,
        detail_budget.ground_cover_patch_count * GROUND_COVER_BLADES_PER_PATCH,
        ground_cover_mesh.count_vertices(),
    );
    queue_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Detail,
        meshes.add(ground_cover_mesh),
        detail_materials.ground_cover.clone(),
        Transform::default(),
        None,
        "island ground cover",
    );

    let tree_specs: Vec<IslandTreeSpec> = island_tree_specs(island_index, island);
    for (tree_index, tree) in tree_specs.into_iter().enumerate() {
        let surface = island_visual_surface_position(island, tree.normalized_offset);
        let trunk_center = surface + Vec3::Y * (tree.trunk_height_m * 0.5);
        let canopy_center = surface + Vec3::Y * (tree.trunk_height_m + tree.canopy_radius_m * 0.68);
        let trunk_mesh = tree_trunk_mesh_for_species(
            tree.species,
            tree.trunk_radius_m,
            tree.trunk_height_m,
            tree.trunk_seed,
        );
        content_diagnostics.record_generated_tree_trunk(trunk_mesh.count_vertices());
        let canopy_mesh =
            tree_canopy_mesh_for_species(tree.species, tree.canopy_radius_m, tree.canopy_seed);
        content_diagnostics.record_generated_tree_canopy(canopy_mesh.count_vertices());

        queue_collidable_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(trunk_mesh),
            detail_materials.trunk.clone(),
            Transform::from_translation(trunk_center),
            Some(CameraObstacle(CameraObstruction::soft_local_prop(
                trunk_center,
                Vec3::new(
                    tree.trunk_radius_m,
                    tree.trunk_height_m * 0.5,
                    tree.trunk_radius_m,
                ),
            ))),
            WorldCollisionProxy::new(
                trunk_center,
                Vec3::new(
                    tree.trunk_radius_m + 0.02,
                    tree.trunk_height_m * 0.5,
                    tree.trunk_radius_m + 0.02,
                ),
                WorldCollisionProxyKind::Tree,
            ),
            wind_visual_motion(island_index, tree_index as f32 * 0.61, 0.025, 0.018, 0.9),
            tree.species.trunk_visual_name(),
        );
        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(canopy_mesh),
            detail_materials.foliage.clone(),
            Transform::from_translation(canopy_center),
            Some(CameraObstacle(CameraObstruction::soft_local_prop(
                canopy_center,
                Vec3::splat(tree.canopy_radius_m),
            ))),
            wind_visual_motion(
                island_index,
                tree_index as f32 * 0.83 + 1.7,
                0.22,
                0.075,
                1.35,
            ),
            tree.species.canopy_visual_name(),
        );
    }

    let rock_specs: Vec<IslandRockSpec> = island_rock_specs(island_index, island);
    for rock in rock_specs {
        let x = island.center.x + rock.normalized_offset.x * island.half_extents.x;
        let z = island.center.z + rock.normalized_offset.y * island.half_extents.y;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));
        let rock_mesh = rock_scatter_mesh(rock.scale_m, rock.seed);
        content_diagnostics.record_generated_rock(rock_mesh.count_vertices());

        let rock_center = Vec3::new(x, surface_y + rock.scale_m * 0.5, z);
        queue_collidable_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(rock_mesh),
            detail_materials.stone.clone(),
            Transform::from_translation(rock_center),
            None,
            WorldCollisionProxy::new(
                rock_center,
                Vec3::new(
                    rock.scale_m * 0.52,
                    rock.scale_m * 0.45,
                    rock.scale_m * 0.52,
                ),
                WorldCollisionProxyKind::Rock,
            ),
            "island stone scatter",
        );
    }

    for cave_feature in island_under_route_visual_specs(island_index, island) {
        let mesh = cave_feature.build_mesh();
        let landmark_kind = GeneratedLandmarkKind::from_under_route_visual(cave_feature.kind);
        content_diagnostics.record_generated_landmark(landmark_kind, mesh.count_vertices());
        let material = if cave_feature.kind == IslandUnderRouteVisualKind::HangingRoots {
            detail_materials.trunk.clone()
        } else {
            detail_materials.stone.clone()
        };
        let camera_obstacle = if cave_feature.kind == IslandUnderRouteVisualKind::HangingRoots {
            None
        } else {
            Some(CameraObstacle(CameraObstruction::new(
                cave_feature.translation,
                cave_feature.camera_half_extents,
            )))
        };
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(mesh),
            material,
            Transform {
                translation: cave_feature.translation,
                rotation: Quat::from_rotation_y(cave_feature.rotation_y),
                ..default()
            },
            camera_obstacle,
            cave_feature.kind.visual_name(),
        );
    }

    for water_feature in island_water_visual_specs(island_index, island) {
        let mesh = water_feature.build_mesh();
        let landmark_kind = GeneratedLandmarkKind::from_water_visual(water_feature.kind);
        content_diagnostics.record_generated_landmark(landmark_kind, mesh.count_vertices());
        let transform = Transform {
            translation: water_feature.translation,
            rotation: Quat::from_rotation_y(water_feature.rotation_y),
            ..default()
        };
        match water_feature.kind {
            IslandWaterVisualKind::PlateauWaterfallMist
            | IslandWaterVisualKind::RouteWaterfallMist => {
                queue_wind_island_visual(
                    entries,
                    visual_index,
                    island,
                    plateau_authored_layer,
                    meshes.add(mesh),
                    water_materials.mist.clone(),
                    transform,
                    None,
                    wind_visual_motion(
                        island_index,
                        water_feature.wind_phase,
                        0.035 * water_feature.wind_motion_scale,
                        0.018 * water_feature.wind_motion_scale,
                        1.1 * water_feature.wind_motion_scale,
                    ),
                    water_feature.kind.visual_name(),
                );
            }
            _ => queue_island_visual(
                entries,
                visual_index,
                island,
                plateau_authored_layer,
                meshes.add(mesh),
                IslandVisualMaterial::surface_without_shadows(water_materials.body.clone()),
                transform,
                None,
                water_feature.kind.visual_name(),
            ),
        }
    }

    for lake_basin in island_lake_basin_visual_specs(island_index, island) {
        let mesh = lake_basin.build_mesh();
        content_diagnostics
            .record_generated_landmark(GeneratedLandmarkKind::LakeBasin, mesh.count_vertices());
        queue_island_visual(
            entries,
            visual_index,
            island,
            plateau_authored_layer,
            meshes.add(mesh),
            detail_materials.stone.clone(),
            Transform {
                translation: lake_basin.translation,
                rotation: Quat::from_rotation_y(lake_basin.rotation_y),
                ..default()
            },
            None,
            "lake basin",
        );
    }

    for artifact in island_artifact_visual_specs(island_index, island) {
        let solid_aabb = artifact.solid_world_aabb();
        let mesh = artifact.build_mesh();
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::ArtifactDetail,
            mesh.count_vertices(),
        );
        let material = match artifact.material {
            IslandArtifactMaterial::Stone => detail_materials.stone.clone(),
            IslandArtifactMaterial::Foliage => detail_materials.foliage.clone(),
            IslandArtifactMaterial::Trunk => detail_materials.trunk.clone(),
        };
        let mesh = meshes.add(mesh);
        let transform = Transform {
            translation: artifact.translation,
            rotation: Quat::from_rotation_y(artifact.rotation_y),
            ..default()
        };
        if let Some((center, half_extents)) = solid_aabb {
            queue_collidable_island_visual(
                entries,
                visual_index,
                island,
                plateau_authored_layer,
                mesh,
                material,
                transform,
                Some(CameraObstacle(CameraObstruction::new(center, half_extents))),
                WorldCollisionProxy::new(center, half_extents, WorldCollisionProxyKind::Landmark),
                artifact.kind.visual_name(),
            );
        } else {
            queue_island_visual(
                entries,
                visual_index,
                island,
                plateau_authored_layer,
                mesh,
                material,
                transform,
                None,
                artifact.kind.visual_name(),
            );
        }
    }

    if island.is_great_plateau_anchor() {
        queue_great_plateau_arrival_details(
            entries,
            visual_index,
            content_diagnostics,
            meshes,
            detail_materials.stone.clone(),
            flower_material.clone(),
            island_index,
            island,
        );
    }

    if is_garden_ring_island(island) {
        let ring_radius = (island.half_extents.min_element() * 0.18).clamp(2.8, 8.5);
        let ring_width = (ring_radius * 0.24).clamp(0.62, 1.5);
        let ring_height = (island.thickness * 0.032).clamp(0.24, 0.62);
        let offset_phase = detail_phase + 2.05;
        let normalized_offset = island_playable_normalized_offset(
            island,
            Vec2::new(
                -0.28 + offset_phase.cos() * 0.08,
                0.30 + offset_phase.sin() * 0.07,
            ),
        );
        let surface = island_visual_surface_position(island, normalized_offset);
        let ring_mesh = garden_ring_mesh(
            ring_radius,
            ring_width,
            ring_height,
            17_000 + island_index as u32 * 199,
        );
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::GardenRing,
            ring_mesh.count_vertices(),
        );
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(ring_mesh),
            flower_material.clone(),
            Transform {
                translation: surface + Vec3::Y * 0.045,
                rotation: Quat::from_rotation_y(offset_phase * 0.18),
                ..default()
            },
            None,
            "garden ring",
        );
    }

    if is_cliff_tooth_island(island) {
        let tooth_width = (island.half_extents.x * 0.42).clamp(10.0, 24.0);
        let tooth_height = (island.thickness * 0.46).clamp(4.0, 9.5);
        let tooth_depth = (island.half_extents.y * 0.12).clamp(1.8, 5.0);
        let offset_phase = detail_phase + 1.35;
        let normalized_offset = island_playable_normalized_offset(
            island,
            Vec2::new(
                0.48 + offset_phase.cos() * 0.07,
                -0.34 + offset_phase.sin() * 0.05,
            ),
        );
        let surface = island_visual_surface_position(island, normalized_offset);
        let tooth_mesh = cliff_tooth_ridge_mesh(
            tooth_width,
            tooth_height,
            tooth_depth,
            16_000 + island_index as u32 * 193,
        );
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::CliffTeeth,
            tooth_mesh.count_vertices(),
        );
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(tooth_mesh),
            detail_materials.stone.clone(),
            Transform {
                translation: surface + Vec3::Y * 0.08,
                rotation: Quat::from_rotation_y(offset_phase * 0.21),
                ..default()
            },
            None,
            "cliff teeth",
        );
    }

    let ruin_specs: Vec<IslandRuinSpec> = island_ruin_specs(island_index, island);
    if !ruin_specs.is_empty() {
        content_diagnostics.record_generated_ruin_cluster();
    }
    for ruin in ruin_specs {
        let surface = island_visual_surface_position(island, ruin.normalized_offset);
        let arch_mesh = ruin_arch_mesh(ruin.width_m, ruin.height_m, ruin.depth_m, ruin.seed);
        content_diagnostics
            .record_generated_landmark(GeneratedLandmarkKind::RuinArch, arch_mesh.count_vertices());
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(arch_mesh),
            detail_materials.stone.clone(),
            Transform {
                translation: surface + Vec3::Y * (ruin.height_m * 0.46),
                rotation: Quat::from_rotation_y(ruin.rotation_y),
                ..default()
            },
            None,
            "ruin arch",
        );
    }

    for silhouette in first_expedition_silhouette_specs(island_index, island) {
        let silhouette_mesh = silhouette.build_mesh();
        content_diagnostics.record_generated_landmark(
            first_expedition_silhouette_landmark_kind(silhouette.kind),
            silhouette_mesh.count_vertices(),
        );
        let material = match silhouette.kind {
            FirstExpeditionSilhouetteKind::RingGarden => flower_material.clone(),
            FirstExpeditionSilhouetteKind::NorthRuinSpire
            | FirstExpeditionSilhouetteKind::SouthRuinSpire
            | FirstExpeditionSilhouetteKind::WaterfallCliff
            | FirstExpeditionSilhouetteKind::CaveArch
            | FirstExpeditionSilhouetteKind::BrokenStair
            | FirstExpeditionSilhouetteKind::HighCrown => detail_materials.stone.clone(),
        };
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Vista,
            meshes.add(silhouette_mesh),
            material,
            Transform {
                translation: silhouette.translation,
                rotation: Quat::from_rotation_y(silhouette.rotation_y),
                ..default()
            },
            None,
            silhouette.kind.visual_name(),
        );
    }

    if !island.is_target && island.name != "launch mesa" {
        let beacon_height = 3.8 + (island_index % 3) as f32 * 0.7;
        let beacon_surface = island_visual_surface_position(island, Vec2::new(-0.18, 0.22));
        let beacon_center = beacon_surface + Vec3::Y * (beacon_height * 0.5);
        let cairn_mesh = route_cairn_mesh(0.44, beacon_height, 12_000 + island_index as u32 * 157);
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::RouteCairn,
            cairn_mesh.count_vertices(),
        );
        queue_collidable_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(cairn_mesh),
            flower_material.clone(),
            Transform::from_translation(beacon_center),
            None,
            WorldCollisionProxy::new(
                beacon_center,
                Vec3::new(0.48, beacon_height * 0.5, 0.48),
                WorldCollisionProxyKind::Landmark,
            ),
            "route cairn",
        );
    }

    if island.is_target {
        let ring_size = 8.0;
        for (index, (offset, rotation_y)) in [
            (Vec3::new(0.0, 0.06, ring_size * 0.5), 0.0),
            (Vec3::new(0.0, 0.06, -ring_size * 0.5), 0.0),
            (
                Vec3::new(ring_size * 0.5, 0.06, 0.0),
                std::f32::consts::FRAC_PI_2,
            ),
            (
                Vec3::new(-ring_size * 0.5, 0.06, 0.0),
                std::f32::consts::FRAC_PI_2,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let surface_y =
                island.mesh_top_y_at(island.center + Vec3::new(offset.x, 0.0, offset.z));
            let marker_mesh = landing_garden_marker_mesh(
                ring_size,
                0.62,
                13_000 + island_index as u32 * 163 + index as u32 * 17,
            );
            content_diagnostics.record_generated_landmark(
                GeneratedLandmarkKind::LandingGardenMarker,
                marker_mesh.count_vertices(),
            );
            let marker_center = Vec3::new(
                island.center.x + offset.x,
                surface_y + offset.y,
                island.center.z + offset.z,
            );
            let marker_half_extents = if rotation_y.abs() < 0.01 {
                Vec3::new(ring_size * 0.5, 0.24, 0.36)
            } else {
                Vec3::new(0.36, 0.24, ring_size * 0.5)
            };
            queue_collidable_island_visual(
                entries,
                visual_index,
                island,
                IslandVisualLayer::Beacon,
                meshes.add(marker_mesh),
                flower_material.clone(),
                Transform {
                    translation: marker_center,
                    rotation: Quat::from_rotation_y(rotation_y),
                    ..default()
                },
                None,
                WorldCollisionProxy::new(
                    marker_center,
                    marker_half_extents,
                    WorldCollisionProxyKind::Landmark,
                ),
                "landing garden ring",
            );
        }
    } else if island.name == "launch mesa" {
        let beacon_surface = island_visual_surface_position(island, Vec2::new(-0.42, 0.38));
        let beacon_center = beacon_surface + Vec3::Y * 0.82;
        let beacon_obstacle_center = beacon_surface + Vec3::Y * 1.65;
        let beacon_mesh = launch_beacon_mesh(0.78, 3.2, 14_000 + island_index as u32 * 173);
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::LaunchBeacon,
            beacon_mesh.count_vertices(),
        );
        queue_collidable_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(beacon_mesh),
            flower_material,
            Transform::from_translation(beacon_center),
            Some(CameraObstacle(CameraObstruction::local_prop(
                beacon_obstacle_center,
                Vec3::new(0.8, 1.65, 0.8),
            ))),
            WorldCollisionProxy::new(
                beacon_obstacle_center,
                Vec3::new(0.8, 1.65, 0.8),
                WorldCollisionProxyKind::Landmark,
            ),
            "launch beacon",
        );

        let launch_tree_height = 4.4;
        let launch_tree_surface_y =
            island.mesh_top_y_at(Vec3::new(island.center.x, island.center.y, 8.0));
        let launch_tree_center = Vec3::new(
            island.center.x,
            launch_tree_surface_y + launch_tree_height * 0.5,
            8.0,
        );
        let launch_canopy_radius = 1.55;
        let launch_canopy_center = Vec3::new(
            island.center.x,
            launch_tree_surface_y + launch_tree_height + 0.85,
            8.0,
        );
        let launch_tree_species = TreeSpecies::BroadCanopy;
        let launch_trunk_seed = launch_tree_species.mesh_seed(7_000 + island_index as u32 * 97);
        let launch_canopy_seed = launch_tree_species.mesh_seed(8_000 + island_index as u32 * 101);
        let launch_trunk_mesh = tree_trunk_mesh_for_species(
            launch_tree_species,
            0.35,
            launch_tree_height,
            launch_trunk_seed,
        );
        content_diagnostics.record_generated_tree_trunk(launch_trunk_mesh.count_vertices());
        let launch_canopy_mesh = tree_canopy_mesh_for_species(
            launch_tree_species,
            launch_canopy_radius,
            launch_canopy_seed,
        );
        content_diagnostics.record_generated_tree_canopy(launch_canopy_mesh.count_vertices());

        queue_collidable_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(launch_trunk_mesh),
            detail_materials.trunk,
            Transform::from_translation(launch_tree_center),
            Some(CameraObstacle(CameraObstruction::soft_local_prop(
                launch_tree_center,
                Vec3::new(0.35, launch_tree_height * 0.5, 0.35),
            ))),
            WorldCollisionProxy::new(
                launch_tree_center,
                Vec3::new(0.37, launch_tree_height * 0.5, 0.37),
                WorldCollisionProxyKind::Tree,
            ),
            wind_visual_motion(island_index, 4.2, 0.035, 0.02, 0.9),
            "launch camera tree trunk",
        );
        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(launch_canopy_mesh),
            detail_materials.foliage,
            Transform::from_translation(launch_canopy_center),
            Some(CameraObstacle(CameraObstruction::soft_local_prop(
                launch_canopy_center,
                Vec3::splat(launch_canopy_radius),
            ))),
            wind_visual_motion(island_index, 5.1, 0.28, 0.09, 1.25),
            "launch camera tree canopy",
        );
    }
}

fn is_cliff_tooth_island(island: SkyIsland) -> bool {
    matches!(
        island.terrain_archetype,
        IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard
    )
}

fn first_expedition_silhouette_landmark_kind(
    kind: FirstExpeditionSilhouetteKind,
) -> GeneratedLandmarkKind {
    match kind {
        FirstExpeditionSilhouetteKind::RingGarden => GeneratedLandmarkKind::GardenRing,
        FirstExpeditionSilhouetteKind::NorthRuinSpire
        | FirstExpeditionSilhouetteKind::SouthRuinSpire
        | FirstExpeditionSilhouetteKind::WaterfallCliff
        | FirstExpeditionSilhouetteKind::CaveArch
        | FirstExpeditionSilhouetteKind::BrokenStair
        | FirstExpeditionSilhouetteKind::HighCrown => GeneratedLandmarkKind::RuinArch,
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_great_plateau_arrival_details(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    stone_material: Handle<StandardMaterial>,
    flower_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let Some(meadow) = island.plateau_region_position(IslandPlateauRegion::MeadowPlateau) else {
        return;
    };

    let shelf_radius = (island.half_extents.min_element() * 0.17).clamp(22.0, 34.0);
    let shelf_mesh = garden_ring_mesh(
        shelf_radius,
        (shelf_radius * 0.20).clamp(3.8, 6.2),
        (island.thickness * 0.010).clamp(0.42, 0.72),
        41_000 + island_index as u32 * 239,
    );
    content_diagnostics.record_generated_landmark(
        GeneratedLandmarkKind::GardenRing,
        shelf_mesh.count_vertices(),
    );
    queue_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Vista,
        meshes.add(shelf_mesh),
        flower_material.clone(),
        Transform {
            translation: meadow + Vec3::Y * 0.08,
            rotation: Quat::from_rotation_y(0.18),
            ..default()
        },
        None,
        "plateau meadow landing shelf",
    );

    let ruin_offset = Vec2::new(-0.16, 0.12);
    let ruin_surface = island_visual_surface_position(island, ruin_offset);
    let ruin_width = (island.half_extents.min_element() * 0.14).clamp(22.0, 30.0);
    let ruin_height = (island.thickness * 0.26).clamp(18.0, 26.0);
    let ruin_depth = (island.half_extents.min_element() * 0.050).clamp(5.0, 7.0);
    let ruin_center = ruin_surface + Vec3::Y * (ruin_height * 0.46);
    let ruin_mesh = ruin_arch_mesh(
        ruin_width,
        ruin_height,
        ruin_depth,
        42_000 + island_index as u32 * 241,
    );
    content_diagnostics
        .record_generated_landmark(GeneratedLandmarkKind::RuinArch, ruin_mesh.count_vertices());
    queue_collidable_island_visual(
        entries,
        visual_index,
        island,
        IslandVisualLayer::Vista,
        meshes.add(ruin_mesh),
        stone_material.clone(),
        Transform {
            translation: ruin_center,
            rotation: Quat::from_rotation_y(-0.42),
            ..default()
        },
        Some(CameraObstacle(CameraObstruction::new(
            ruin_center,
            Vec3::new(ruin_width * 0.42, ruin_height * 0.46, ruin_depth * 0.52),
        ))),
        WorldCollisionProxy::new(
            ruin_center,
            Vec3::new(ruin_width * 0.40, ruin_height * 0.45, ruin_depth * 0.50),
            WorldCollisionProxyKind::Landmark,
        ),
        "plateau arrival ruin marker",
    );

    for (hint_index, name, region) in [
        (
            0_u32,
            "plateau high shelf route hint",
            IslandPlateauRegion::HighShelf,
        ),
        (
            1_u32,
            "plateau cave route hint",
            IslandPlateauRegion::UnderhangEntry,
        ),
    ] {
        let Some(surface) = island.plateau_region_position(region) else {
            continue;
        };
        let height = 6.2 + hint_index as f32 * 0.45;
        let hint_center = surface + Vec3::Y * (height * 0.5);
        let region_offset = region.sample_offset();
        let yaw = region_offset.x.atan2(region_offset.y);
        let hint_mesh = route_cairn_mesh(
            0.72,
            height,
            43_000 + island_index as u32 * 251 + hint_index * 29,
        );
        content_diagnostics.record_generated_landmark(
            GeneratedLandmarkKind::RouteCairn,
            hint_mesh.count_vertices(),
        );
        queue_collidable_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Beacon,
            meshes.add(hint_mesh),
            flower_material.clone(),
            Transform {
                translation: hint_center,
                rotation: Quat::from_rotation_y(yaw),
                ..default()
            },
            Some(CameraObstacle(CameraObstruction::local_prop(
                hint_center,
                Vec3::new(0.82, height * 0.50, 0.82),
            ))),
            WorldCollisionProxy::new(
                hint_center,
                Vec3::new(0.82, height * 0.50, 0.82),
                WorldCollisionProxyKind::Landmark,
            ),
            name,
        );
    }
}

fn is_garden_ring_island(island: SkyIsland) -> bool {
    matches!(
        island.terrain_archetype,
        IslandTerrainArchetype::GardenBasin
            | IslandTerrainArchetype::GardenApron
            | IslandTerrainArchetype::OrchardBasin
            | IslandTerrainArchetype::OrchardSpur
    )
}
