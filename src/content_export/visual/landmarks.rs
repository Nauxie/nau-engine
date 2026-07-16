use super::{metrics::visual_content_mesh_summary, types::VisualLandmarkSummary};
use crate::{
    content_export::shared::{mesh_positions, terrain_export_slug, write_mesh_obj},
    generated_content::{
        cliff_tooth_ridge_mesh, first_expedition_silhouette_specs, garden_ring_mesh,
        island_artifact_visual_specs, island_lake_basin_visual_specs, island_ruin_specs,
        island_under_route_visual_specs, island_water_visual_specs, landing_garden_marker_mesh,
        launch_beacon_mesh, mesh_normal_slope_band_count, mesh_vertical_band_count,
        obstruction_spire_mesh, route_cairn_mesh, ruin_arch_mesh,
    },
};
use bevy::prelude::*;
use nau_engine::world::{
    IslandPlateauRegion, IslandTerrainArchetype, SkyIsland, route_obstruction_spire,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn visual_content_landmark_summaries(
    output_dir: &Path,
    island_index: usize,
    island: SkyIsland,
    island_slug: &str,
) -> std::io::Result<Vec<VisualLandmarkSummary>> {
    let mut landmarks = Vec::new();

    for water_feature in island_water_visual_specs(island_index, island) {
        let mesh = water_feature.build_mesh();
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            water_feature.kind.label(),
            water_feature.label,
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    for cave_feature in island_under_route_visual_specs(island_index, island) {
        let mesh = cave_feature.build_mesh();
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            cave_feature.kind.label(),
            cave_feature.label,
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    for lake_basin in island_lake_basin_visual_specs(island_index, island) {
        let mesh = lake_basin.build_mesh();
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            "lake_basin",
            lake_basin.label,
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    for artifact in island_artifact_visual_specs(island_index, island) {
        let mesh = artifact.build_mesh();
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            artifact.kind.label(),
            artifact.label,
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    for silhouette in first_expedition_silhouette_specs(island_index, island) {
        let mesh = silhouette.build_mesh();
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            silhouette.kind.label(),
            silhouette.label,
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    if island.is_great_plateau_anchor() {
        push_great_plateau_arrival_landmarks(
            &mut landmarks,
            output_dir,
            island_index,
            island,
            island_slug,
        )?;
    }

    for (ruin_index, ruin) in island_ruin_specs(island_index, island)
        .into_iter()
        .enumerate()
    {
        let mesh = ruin_arch_mesh(ruin.width_m, ruin.height_m, ruin.depth_m, ruin.seed);
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            "ruin_arch",
            &format!("ruin arch {ruin_index}"),
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    if is_garden_ring_island(island) {
        let ring_radius = (island.half_extents.min_element() * 0.18).clamp(2.8, 8.5);
        let mesh = garden_ring_mesh(
            ring_radius,
            (ring_radius * 0.24).clamp(0.62, 1.5),
            (island.thickness * 0.032).clamp(0.24, 0.62),
            17_000 + island_index as u32 * 199,
        );
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            "garden_ring",
            "garden ring",
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    if is_cliff_tooth_island(island) {
        let mesh = cliff_tooth_ridge_mesh(
            (island.half_extents.x * 0.42).clamp(10.0, 24.0),
            (island.thickness * 0.46).clamp(4.0, 9.5),
            (island.half_extents.y * 0.12).clamp(1.8, 5.0),
            16_000 + island_index as u32 * 193,
        );
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            "cliff_teeth",
            "cliff teeth",
            island_index,
            island_slug,
            &mesh,
        )?);
    }

    let spire = route_obstruction_spire(island_index, island);
    let spire_mesh = obstruction_spire_mesh(spire.radius_m, spire.height_m, spire.seed);
    landmarks.push(write_visual_landmark_summary(
        output_dir,
        island.name,
        "obstruction_spire",
        "obstruction spire",
        island_index,
        island_slug,
        &spire_mesh,
    )?);

    if !island.is_target && island.name != "launch mesa" {
        let beacon_height = 3.8 + (island_index % 3) as f32 * 0.7;
        let cairn_mesh = route_cairn_mesh(0.44, beacon_height, 12_000 + island_index as u32 * 157);
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            "route_cairn",
            "route cairn",
            island_index,
            island_slug,
            &cairn_mesh,
        )?);
    } else if island.name == "launch mesa" {
        let beacon_mesh = launch_beacon_mesh(0.78, 3.2, 14_000 + island_index as u32 * 173);
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            "launch_beacon",
            "launch beacon",
            island_index,
            island_slug,
            &beacon_mesh,
        )?);
    } else if island.is_target {
        for marker_index in 0..4 {
            let marker_mesh = landing_garden_marker_mesh(
                8.0,
                0.62,
                13_000 + island_index as u32 * 163 + marker_index as u32 * 17,
            );
            landmarks.push(write_visual_landmark_summary(
                output_dir,
                island.name,
                "landing_garden_marker",
                &format!("landing garden marker {marker_index}"),
                island_index,
                island_slug,
                &marker_mesh,
            )?);
        }
    }

    Ok(landmarks)
}

fn push_great_plateau_arrival_landmarks(
    landmarks: &mut Vec<VisualLandmarkSummary>,
    output_dir: &Path,
    island_index: usize,
    island: SkyIsland,
    island_slug: &str,
) -> std::io::Result<()> {
    let shelf_radius = (island.half_extents.min_element() * 0.17).clamp(22.0, 34.0);
    let shelf_mesh = garden_ring_mesh(
        shelf_radius,
        (shelf_radius * 0.20).clamp(3.8, 6.2),
        (island.thickness * 0.010).clamp(0.42, 0.72),
        41_000 + island_index as u32 * 239,
    );
    landmarks.push(write_visual_landmark_summary(
        output_dir,
        island.name,
        "plateau_arrival_shelf",
        "meadow landing shelf",
        island_index,
        island_slug,
        &shelf_mesh,
    )?);

    let ruin_width = (island.half_extents.min_element() * 0.14).clamp(22.0, 30.0);
    let ruin_height = (island.thickness * 0.26).clamp(18.0, 26.0);
    let ruin_depth = (island.half_extents.min_element() * 0.050).clamp(5.0, 7.0);
    let ruin_mesh = ruin_arch_mesh(
        ruin_width,
        ruin_height,
        ruin_depth,
        42_000 + island_index as u32 * 241,
    );
    landmarks.push(write_visual_landmark_summary(
        output_dir,
        island.name,
        "plateau_arrival_ruin",
        "arrival ruin marker",
        island_index,
        island_slug,
        &ruin_mesh,
    )?);

    for (hint_index, label, region) in [
        (
            0_u32,
            "high shelf onward hint",
            IslandPlateauRegion::HighShelf,
        ),
        (
            1_u32,
            "cave mouth onward hint",
            IslandPlateauRegion::UnderhangEntry,
        ),
    ] {
        if island.plateau_region_position(region).is_none() {
            continue;
        }

        let hint_mesh = route_cairn_mesh(
            0.72,
            6.2 + hint_index as f32 * 0.45,
            43_000 + island_index as u32 * 251 + hint_index * 29,
        );
        landmarks.push(write_visual_landmark_summary(
            output_dir,
            island.name,
            "plateau_onward_hint",
            label,
            island_index,
            island_slug,
            &hint_mesh,
        )?);
    }

    Ok(())
}

fn write_visual_landmark_summary(
    output_dir: &Path,
    island_name: &'static str,
    kind: &'static str,
    label: &str,
    island_index: usize,
    island_slug: &str,
    mesh: &Mesh,
) -> std::io::Result<VisualLandmarkSummary> {
    let label_slug = terrain_export_slug(label);
    let obj_path =
        PathBuf::from("visuals").join(format!("{island_index:02}_{island_slug}_{label_slug}.obj"));
    write_mesh_obj(&output_dir.join(&obj_path), mesh, label)?;

    Ok(VisualLandmarkSummary {
        island_name,
        kind,
        label: label.to_string(),
        mesh: visual_content_mesh_summary(obj_path, mesh),
        height_band_count: mesh_vertical_band_count(mesh),
        radius_band_count: landmark_radius_band_count(mesh),
        normal_slope_band_count: mesh_normal_slope_band_count(mesh),
    })
}

fn landmark_radius_band_count(mesh: &Mesh) -> usize {
    mesh_positions(mesh)
        .iter()
        .map(|position| {
            let radius = Vec2::new(position[0], position[2]).length();
            (radius / 0.05).round() as i32
        })
        .collect::<HashSet<_>>()
        .len()
}

fn is_cliff_tooth_island(island: SkyIsland) -> bool {
    matches!(
        island.terrain_archetype,
        IslandTerrainArchetype::StormRavine | IslandTerrainArchetype::StormShard
    )
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
