use super::{metrics::visual_content_mesh_summary, types::VisualLandmarkSummary};
use crate::{
    content_export::shared::{mesh_positions, terrain_export_slug, write_mesh_obj},
    generated_content::{
        island_under_route_visual_specs, island_water_visual_specs, landing_garden_marker_mesh,
        launch_beacon_mesh, mesh_normal_slope_band_count, mesh_vertical_band_count,
        obstruction_spire_mesh, route_cairn_mesh,
    },
};
use bevy::prelude::*;
use nau_engine::world::{SkyIsland, route_obstruction_spire};
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
