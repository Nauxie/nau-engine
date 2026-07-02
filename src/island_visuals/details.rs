use super::queue::{
    queue_collidable_island_visual, queue_collidable_wind_island_visual, queue_island_visual,
    queue_wind_island_visual,
};
use super::types::{IslandVisualEntry, IslandVisualLayer};
use crate::camera_runtime::CameraObstacle;
use crate::content_diagnostics::{GeneratedLandmarkKind, IslandContentDiagnostics};
use crate::environment_visuals::wind_visual_motion;
use crate::generated_content::{
    GROUND_COVER_BLADES_PER_PATCH, GROUND_COVER_PATCHES, IslandDetailMaterials,
    island_ground_cover_mesh, island_playable_normalized_offset, island_under_route_visual_specs,
    island_visual_surface_position, island_water_visual_specs, landing_garden_marker_mesh,
    launch_beacon_mesh, rock_scatter_mesh, route_cairn_mesh, ruin_arch_mesh, tree_canopy_mesh,
    tree_trunk_mesh,
};
use bevy::prelude::*;
use nau_engine::camera::CameraObstruction;
use nau_engine::world::{IslandLandmarkRole, SkyIsland};

use crate::world_collision_runtime::{WorldCollisionProxy, WorldCollisionProxyKind};

#[allow(clippy::too_many_arguments)]
pub(super) fn queue_sky_island_details(
    entries: &mut Vec<IslandVisualEntry>,
    visual_index: &mut usize,
    content_diagnostics: &mut IslandContentDiagnostics,
    meshes: &mut Assets<Mesh>,
    detail_materials: IslandDetailMaterials,
    flower_material: Handle<StandardMaterial>,
    water_material: Handle<StandardMaterial>,
    island_index: usize,
    island: SkyIsland,
) {
    let detail_phase = island_index as f32 * 0.77;
    content_diagnostics.record_detail_biome_palette(island_index);
    let ground_cover_mesh = island_ground_cover_mesh(island_index, island);
    content_diagnostics.record_generated_ground_cover(
        GROUND_COVER_PATCHES,
        GROUND_COVER_PATCHES * GROUND_COVER_BLADES_PER_PATCH,
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

    let tree_offsets = [
        Vec2::new(-0.42, -0.24),
        Vec2::new(0.34, -0.36),
        Vec2::new(0.24, 0.32),
    ];

    for (index, offset) in tree_offsets.into_iter().enumerate() {
        if island.is_target && index == 1 {
            continue;
        }
        let sway = (detail_phase + index as f32).sin() * 0.08;
        let surface = island_visual_surface_position(island, Vec2::new(offset.x + sway, offset.y));
        let trunk_height = 2.1 + index as f32 * 0.25;
        let trunk_center = surface + Vec3::Y * (trunk_height * 0.5);
        let canopy_radius = 1.05 + index as f32 * 0.08;
        let canopy_center = surface + Vec3::Y * (trunk_height + 0.72);
        let trunk_mesh = tree_trunk_mesh(
            0.22,
            trunk_height,
            5_000 + island_index as u32 * 97 + index as u32 * 13,
        );
        content_diagnostics.record_generated_tree_trunk(trunk_mesh.count_vertices());
        let canopy_mesh = tree_canopy_mesh(
            canopy_radius,
            6_000 + island_index as u32 * 101 + index as u32 * 17,
        );
        content_diagnostics.record_generated_tree_canopy(canopy_mesh.count_vertices());

        queue_collidable_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(trunk_mesh),
            detail_materials.trunk.clone(),
            Transform::from_translation(trunk_center),
            Some(CameraObstacle(CameraObstruction::new(
                trunk_center,
                Vec3::new(0.22, trunk_height * 0.5, 0.22),
            ))),
            WorldCollisionProxy::new(
                trunk_center,
                Vec3::new(0.24, trunk_height * 0.5, 0.24),
                WorldCollisionProxyKind::Tree,
            ),
            wind_visual_motion(island_index, index as f32 * 0.61, 0.025, 0.018, 0.9),
            "island tree trunk",
        );
        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(canopy_mesh),
            detail_materials.foliage.clone(),
            Transform::from_translation(canopy_center),
            Some(CameraObstacle(CameraObstruction::new(
                canopy_center,
                Vec3::splat(canopy_radius),
            ))),
            wind_visual_motion(island_index, index as f32 * 0.83 + 1.7, 0.22, 0.075, 1.35),
            "island tree canopy",
        );
    }

    for index in 0..5 {
        let angle = detail_phase + index as f32 * 1.37;
        let radius = if index % 2 == 0 { 0.52 } else { 0.72 };
        let normalized_offset =
            island_playable_normalized_offset(island, Vec2::new(angle.cos(), angle.sin()) * radius);
        let x = island.center.x + normalized_offset.x * island.half_extents.x;
        let z = island.center.z + normalized_offset.y * island.half_extents.y;
        let stone_scale = 0.45 + index as f32 * 0.08;
        let surface_y = island.mesh_top_y_at(Vec3::new(x, island.center.y, z));
        let rock_mesh = rock_scatter_mesh(
            stone_scale,
            9_000 + island_index as u32 * 131 + index as u32 * 19,
        );
        content_diagnostics.record_generated_rock(rock_mesh.count_vertices());

        let rock_center = Vec3::new(x, surface_y + stone_scale * 0.5, z);
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
                Vec3::new(stone_scale * 0.52, stone_scale * 0.45, stone_scale * 0.52),
                WorldCollisionProxyKind::Rock,
            ),
            "island stone scatter",
        );
    }

    for cave_feature in island_under_route_visual_specs(island_index, island) {
        let mesh = cave_feature.build_mesh();
        let landmark_kind = GeneratedLandmarkKind::from_under_route_visual(cave_feature.kind);
        content_diagnostics.record_generated_landmark(landmark_kind, mesh.count_vertices());
        queue_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(mesh),
            detail_materials.stone.clone(),
            Transform {
                translation: cave_feature.translation,
                rotation: Quat::from_rotation_y(cave_feature.rotation_y),
                ..default()
            },
            Some(CameraObstacle(CameraObstruction::new(
                cave_feature.translation,
                cave_feature.camera_half_extents,
            ))),
            cave_feature.kind.visual_name(),
        );
    }

    for water_feature in island_water_visual_specs(island_index, island) {
        let mesh = water_feature.build_mesh();
        let landmark_kind = GeneratedLandmarkKind::from_water_visual(water_feature.kind);
        content_diagnostics.record_generated_landmark(landmark_kind, mesh.count_vertices());
        queue_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(mesh),
            water_material.clone(),
            Transform {
                translation: water_feature.translation,
                rotation: Quat::from_rotation_y(water_feature.rotation_y),
                ..default()
            },
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

    if island.world_tags.landmark_role == IslandLandmarkRole::RuinArch {
        let arch_width = (island.half_extents.x * 0.24).clamp(5.5, 18.0);
        let arch_height = (island.thickness * 0.38).clamp(4.8, 12.0);
        let arch_depth = (island.half_extents.y * 0.08).clamp(1.2, 3.2);
        let offset_phase = detail_phase + 0.9;
        let normalized_offset = island_playable_normalized_offset(
            island,
            Vec2::new(
                0.24 + offset_phase.sin() * 0.08,
                -0.20 + offset_phase.cos() * 0.06,
            ),
        );
        let surface = island_visual_surface_position(island, normalized_offset);
        let arch_mesh = ruin_arch_mesh(
            arch_width,
            arch_height,
            arch_depth,
            15_000 + island_index as u32 * 181,
        );
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
                translation: surface + Vec3::Y * (arch_height * 0.46),
                rotation: Quat::from_rotation_y(offset_phase * 0.31),
                ..default()
            },
            None,
            "ruin arch",
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
            Some(CameraObstacle(CameraObstruction::new(
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
        let launch_trunk_mesh =
            tree_trunk_mesh(0.35, launch_tree_height, 7_000 + island_index as u32 * 97);
        content_diagnostics.record_generated_tree_trunk(launch_trunk_mesh.count_vertices());
        let launch_canopy_mesh =
            tree_canopy_mesh(launch_canopy_radius, 8_000 + island_index as u32 * 101);
        content_diagnostics.record_generated_tree_canopy(launch_canopy_mesh.count_vertices());

        queue_collidable_wind_island_visual(
            entries,
            visual_index,
            island,
            IslandVisualLayer::Detail,
            meshes.add(launch_trunk_mesh),
            detail_materials.trunk,
            Transform::from_translation(launch_tree_center),
            Some(CameraObstacle(CameraObstruction::new(
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
            Some(CameraObstacle(CameraObstruction::new(
                launch_canopy_center,
                Vec3::splat(launch_canopy_radius),
            ))),
            wind_visual_motion(island_index, 5.1, 0.28, 0.09, 1.25),
            "launch camera tree canopy",
        );
    }
}
