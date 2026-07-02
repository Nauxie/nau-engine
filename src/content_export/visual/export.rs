use super::{
    clouds::visual_content_cloud_summaries,
    landmarks::visual_content_landmark_summaries,
    metrics::{finite_range_f32, finite_ratio, min_finite_f32, visual_content_mesh_summary},
    palette::visual_content_palette_summary,
    types::{VisualContentExportReport, VisualGroundCoverSummary, VisualTreeSummary},
    vegetation::{
        ground_cover_blade_stats, tree_canopy_lobe_count, tree_trunk_shape_metrics,
        visual_content_tree_specs,
    },
};
use crate::{
    content_export::shared::{terrain_export_slug, write_mesh_obj},
    eval_runtime::remove_existing_dir,
    generated_content::{
        GROUND_COVER_PATCHES, TERRAIN_BIOME_PALETTE_COUNT, TREE_CANOPY_CARD_COUNT,
        island_ground_cover_mesh, tree_canopy_mesh, tree_trunk_mesh,
    },
};
use nau_engine::world::{IslandScaleClass, SkyRoute};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn export_visual_content_inspection(
    output_dir: &Path,
) -> std::io::Result<VisualContentExportReport> {
    fs::create_dir_all(output_dir)?;
    let visuals_dir = output_dir.join("visuals");
    remove_existing_dir(&visuals_dir)?;
    fs::create_dir_all(&visuals_dir)?;

    let route = SkyRoute::default();
    let mut ground_cover = Vec::with_capacity(route.islands().len());
    let mut trees = Vec::new();
    let mut clouds = Vec::new();
    let mut landmarks = Vec::new();

    for (island_index, island) in route.islands().iter().copied().enumerate() {
        let island_slug = terrain_export_slug(island.name);
        let ground_mesh = island_ground_cover_mesh(island_index, island);
        let ground_obj = PathBuf::from("visuals")
            .join(format!("{island_index:02}_{island_slug}_ground_cover.obj"));
        write_mesh_obj(&output_dir.join(&ground_obj), &ground_mesh, "ground cover")?;
        let blade_stats = ground_cover_blade_stats(&ground_mesh);
        ground_cover.push(VisualGroundCoverSummary {
            island_name: island.name,
            island_slug: island_slug.clone(),
            mesh: visual_content_mesh_summary(ground_obj, &ground_mesh),
            patch_count: GROUND_COVER_PATCHES,
            blade_count: blade_stats.blade_count,
            min_blade_height_m: blade_stats.min_height_m,
            max_blade_height_m: blade_stats.max_height_m,
            blade_height_range_m: blade_stats.height_range_m,
        });

        for tree in visual_content_tree_specs(island_index, island) {
            let tree_slug = terrain_export_slug(&tree.label);
            let trunk_mesh = tree_trunk_mesh(tree.trunk_radius_m, tree.trunk_height_m, tree.seed);
            let canopy_mesh = tree_canopy_mesh(tree.canopy_radius_m, tree.canopy_seed);
            let trunk_obj = PathBuf::from("visuals").join(format!(
                "{island_index:02}_{island_slug}_{tree_slug}_trunk.obj"
            ));
            let canopy_obj = PathBuf::from("visuals").join(format!(
                "{island_index:02}_{island_slug}_{tree_slug}_canopy.obj"
            ));
            write_mesh_obj(&output_dir.join(&trunk_obj), &trunk_mesh, "tree trunk")?;
            write_mesh_obj(&output_dir.join(&canopy_obj), &canopy_mesh, "tree canopy")?;

            let trunk_shape = tree_trunk_shape_metrics(&trunk_mesh);
            let trunk = visual_content_mesh_summary(trunk_obj, &trunk_mesh);
            let canopy = visual_content_mesh_summary(canopy_obj, &canopy_mesh);
            let canopy_horizontal_span = canopy.horizontal_span_m.max(canopy.depth_span_m);
            let canopy_vertical_to_horizontal_ratio =
                finite_ratio(canopy.vertical_span_m, canopy_horizontal_span);

            trees.push(VisualTreeSummary {
                island_name: island.name,
                label: tree.label,
                trunk,
                canopy,
                trunk_height_m: tree.trunk_height_m,
                canopy_radius_m: tree.canopy_radius_m,
                trunk_taper_ratio: trunk_shape.taper_ratio,
                branch_reach_ratio: trunk_shape.branch_reach_ratio,
                branch_count: trunk_shape.branch_count,
                root_flare_count: trunk_shape.root_flare_count,
                trunk_ring_count: trunk_shape.trunk_ring_count,
                canopy_lobe_count: tree_canopy_lobe_count(),
                canopy_detail_card_count: TREE_CANOPY_CARD_COUNT,
                canopy_vertical_to_horizontal_ratio,
            });
        }

        clouds.extend(visual_content_cloud_summaries(
            output_dir,
            island_index,
            island,
            &island_slug,
        )?);
        landmarks.extend(visual_content_landmark_summaries(
            output_dir,
            island_index,
            island,
            &island_slug,
        )?);
    }

    let palettes = (0..TERRAIN_BIOME_PALETTE_COUNT)
        .map(visual_content_palette_summary)
        .collect::<Vec<_>>();
    let terrain_biome_palette_count = palettes
        .iter()
        .map(|palette| palette.terrain_key)
        .collect::<HashSet<_>>()
        .len();
    let foliage_palette_count = palettes
        .iter()
        .map(|palette| palette.foliage_key)
        .collect::<HashSet<_>>()
        .len();
    let stone_palette_count = palettes
        .iter()
        .map(|palette| palette.stone_key)
        .collect::<HashSet<_>>()
        .len();
    let landmark_kind_count = landmarks
        .iter()
        .map(|summary| summary.kind)
        .collect::<HashSet<_>>()
        .len();
    let small_island_count = route
        .islands()
        .iter()
        .filter(|island| {
            matches!(
                island.world_tags.scale_class,
                IslandScaleClass::Tiny | IslandScaleClass::Small
            )
        })
        .count();
    let plateau_landmark_count = landmarks
        .iter()
        .filter(|summary| summary.island_name == "great sky plateau")
        .count();
    let plateau_waterfall_ribbon_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "plateau_waterfall_ribbon")
        .count();
    let plateau_waterfall_mist_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "plateau_waterfall_mist")
        .count();
    let route_waterfall_ribbon_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "route_waterfall_ribbon")
        .count();
    let route_waterfall_mist_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "route_waterfall_mist")
        .count();
    let under_route_visual_count = landmarks
        .iter()
        .filter(|summary| summary.kind.starts_with("under_route_"))
        .count();
    let under_route_cave_mouth_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "under_route_cave_mouth")
        .count();

    let mesh_count = ground_cover.len() + trees.len() * 2 + clouds.len() + landmarks.len();
    let total_vertex_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.vertex_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.vertex_count, summary.canopy.vertex_count]),
        )
        .chain(clouds.iter().map(|summary| summary.mesh.vertex_count))
        .chain(landmarks.iter().map(|summary| summary.mesh.vertex_count))
        .sum();
    let total_triangle_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.triangle_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.triangle_count, summary.canopy.triangle_count]),
        )
        .chain(clouds.iter().map(|summary| summary.mesh.triangle_count))
        .chain(landmarks.iter().map(|summary| summary.mesh.triangle_count))
        .sum();

    let manifest_path = output_dir.join("manifest.json");
    let report = VisualContentExportReport {
        manifest_path,
        mesh_count,
        total_vertex_count,
        total_triangle_count,
        ground_cover_count: ground_cover.len(),
        ground_cover_patch_total: ground_cover.iter().map(|summary| summary.patch_count).sum(),
        ground_cover_blade_total: ground_cover.iter().map(|summary| summary.blade_count).sum(),
        tree_trunk_count: trees.len(),
        tree_canopy_count: trees.len(),
        weather_cloud_count: clouds.len(),
        weather_cloud_bank_count: clouds.iter().filter(|summary| summary.bank).count(),
        weather_cloud_veil_count: clouds
            .iter()
            .filter(|summary| summary.kind == "veil")
            .count(),
        landmark_count: landmarks.len(),
        landmark_kind_count,
        small_island_count,
        plateau_landmark_count,
        plateau_waterfall_ribbon_count,
        plateau_waterfall_mist_count,
        route_waterfall_ribbon_count,
        route_waterfall_mist_count,
        under_route_visual_count,
        under_route_cave_mouth_count,
        ruin_arch_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "ruin_arch")
            .count(),
        route_cairn_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "route_cairn")
            .count(),
        launch_beacon_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "launch_beacon")
            .count(),
        landing_garden_marker_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "landing_garden_marker")
            .count(),
        pond_surface_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "pond_surface")
            .count(),
        obstruction_spire_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "obstruction_spire")
            .count(),
        min_ground_cover_mesh_vertices: ground_cover
            .iter()
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_ground_cover_blade_count: ground_cover
            .iter()
            .map(|summary| summary.blade_count)
            .min()
            .unwrap_or(0),
        min_ground_cover_blade_height_range_m: min_finite_f32(
            ground_cover
                .iter()
                .map(|summary| summary.blade_height_range_m),
        ),
        min_tree_trunk_mesh_vertices: trees
            .iter()
            .map(|summary| summary.trunk.vertex_count)
            .min()
            .unwrap_or(0),
        min_tree_trunk_taper_ratio: min_finite_f32(
            trees.iter().map(|summary| summary.trunk_taper_ratio),
        ),
        min_tree_branch_reach_ratio: min_finite_f32(
            trees.iter().map(|summary| summary.branch_reach_ratio),
        ),
        min_tree_branch_count: trees
            .iter()
            .map(|summary| summary.branch_count)
            .min()
            .unwrap_or(0),
        min_tree_root_flare_count: trees
            .iter()
            .map(|summary| summary.root_flare_count)
            .min()
            .unwrap_or(0),
        min_tree_trunk_ring_count: trees
            .iter()
            .map(|summary| summary.trunk_ring_count)
            .min()
            .unwrap_or(0),
        tree_trunk_height_range_m: finite_range_f32(
            trees.iter().map(|summary| summary.trunk_height_m),
        ),
        min_tree_canopy_mesh_vertices: trees
            .iter()
            .map(|summary| summary.canopy.vertex_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_lobe_count: trees
            .iter()
            .map(|summary| summary.canopy_lobe_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_detail_card_count: trees
            .iter()
            .map(|summary| summary.canopy_detail_card_count)
            .min()
            .unwrap_or(0),
        min_tree_canopy_vertical_to_horizontal_ratio: min_finite_f32(
            trees
                .iter()
                .map(|summary| summary.canopy_vertical_to_horizontal_ratio),
        ),
        tree_canopy_radius_range_m: finite_range_f32(
            trees.iter().map(|summary| summary.canopy_radius_m),
        ),
        min_weather_cloud_mesh_vertices: clouds
            .iter()
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_lobe_count: clouds
            .iter()
            .map(|summary| summary.lobe_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_wisp_card_count: clouds
            .iter()
            .map(|summary| summary.wisp_card_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_filament_ribbon_detail_count: clouds
            .iter()
            .map(|summary| summary.filament_ribbon_detail_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_bank_depth_m: min_finite_f32(
            clouds
                .iter()
                .filter(|summary| summary.bank)
                .map(|summary| summary.scaled_vertical_depth_m),
        ),
        min_weather_cloud_bank_lobe_count: clouds
            .iter()
            .filter(|summary| summary.bank)
            .map(|summary| summary.lobe_count)
            .min()
            .unwrap_or(0),
        min_weather_cloud_scaled_depth_span_m: min_finite_f32(
            clouds.iter().map(|summary| summary.scaled_depth_span_m),
        ),
        min_route_cairn_mesh_vertices: min_landmark_vertices(&landmarks, "route_cairn"),
        min_route_cairn_vertical_span_m: min_landmark_vertical_span(&landmarks, "route_cairn"),
        min_launch_beacon_mesh_vertices: min_landmark_vertices(&landmarks, "launch_beacon"),
        min_launch_beacon_vertical_span_m: min_landmark_vertical_span(&landmarks, "launch_beacon"),
        min_landing_garden_marker_mesh_vertices: min_landmark_vertices(
            &landmarks,
            "landing_garden_marker",
        ),
        min_landing_garden_marker_vertical_span_m: min_landmark_vertical_span(
            &landmarks,
            "landing_garden_marker",
        ),
        min_pond_surface_mesh_vertices: min_landmark_vertices(&landmarks, "pond_surface"),
        min_pond_surface_vertical_span_m: min_landmark_vertical_span(&landmarks, "pond_surface"),
        plateau_landmark_vertex_total: landmarks
            .iter()
            .filter(|summary| summary.island_name == "great sky plateau")
            .map(|summary| summary.mesh.vertex_count)
            .sum(),
        max_plateau_landmark_mesh_vertices: landmarks
            .iter()
            .filter(|summary| summary.island_name == "great sky plateau")
            .map(|summary| summary.mesh.vertex_count)
            .max()
            .unwrap_or(0),
        min_plateau_waterfall_vertical_span_m: min_landmark_vertical_span(
            &landmarks,
            "plateau_waterfall_ribbon",
        ),
        min_route_waterfall_vertical_span_m: min_landmark_vertical_span(
            &landmarks,
            "route_waterfall_ribbon",
        ),
        min_under_route_visual_vertical_span_m: min_finite_f32(
            landmarks
                .iter()
                .filter(|summary| summary.kind.starts_with("under_route_"))
                .map(|summary| summary.mesh.vertical_span_m),
        ),
        min_ruin_arch_mesh_vertices: min_landmark_vertices(&landmarks, "ruin_arch"),
        min_ruin_arch_vertical_span_m: min_landmark_vertical_span(&landmarks, "ruin_arch"),
        min_ruin_arch_radius_band_count: min_landmark_radius_bands(&landmarks, "ruin_arch"),
        min_ruin_arch_normal_slope_band_count: min_landmark_normal_slope_bands(
            &landmarks,
            "ruin_arch",
        ),
        min_obstruction_spire_mesh_vertices: min_landmark_vertices(&landmarks, "obstruction_spire"),
        min_obstruction_spire_triangle_count: min_landmark_triangles(
            &landmarks,
            "obstruction_spire",
        ),
        min_obstruction_spire_vertical_span_m: min_landmark_vertical_span(
            &landmarks,
            "obstruction_spire",
        ),
        min_obstruction_spire_height_band_count: min_landmark_height_bands(
            &landmarks,
            "obstruction_spire",
        ),
        min_obstruction_spire_radius_band_count: min_landmark_radius_bands(
            &landmarks,
            "obstruction_spire",
        ),
        min_obstruction_spire_normal_slope_band_count: min_landmark_normal_slope_bands(
            &landmarks,
            "obstruction_spire",
        ),
        terrain_biome_palette_count,
        foliage_palette_count,
        stone_palette_count,
        ground_cover,
        trees,
        clouds,
        landmarks,
        palettes,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
}

fn min_landmark_vertices(landmarks: &[super::types::VisualLandmarkSummary], kind: &str) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.mesh.vertex_count)
        .min()
        .unwrap_or(0)
}

fn min_landmark_triangles(landmarks: &[super::types::VisualLandmarkSummary], kind: &str) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.mesh.triangle_count)
        .min()
        .unwrap_or(0)
}

fn min_landmark_vertical_span(
    landmarks: &[super::types::VisualLandmarkSummary],
    kind: &str,
) -> f32 {
    min_finite_f32(
        landmarks
            .iter()
            .filter(|summary| summary.kind == kind)
            .map(|summary| summary.mesh.vertical_span_m),
    )
}

fn min_landmark_height_bands(
    landmarks: &[super::types::VisualLandmarkSummary],
    kind: &str,
) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.height_band_count)
        .min()
        .unwrap_or(0)
}

fn min_landmark_radius_bands(
    landmarks: &[super::types::VisualLandmarkSummary],
    kind: &str,
) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.radius_band_count)
        .min()
        .unwrap_or(0)
}

fn min_landmark_normal_slope_bands(
    landmarks: &[super::types::VisualLandmarkSummary],
    kind: &str,
) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.normal_slope_band_count)
        .min()
        .unwrap_or(0)
}
