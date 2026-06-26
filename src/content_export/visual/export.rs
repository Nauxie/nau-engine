use super::{
    clouds::visual_content_cloud_summaries,
    metrics::{finite_ratio, min_finite_f32, visual_content_mesh_summary},
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
use nau_engine::world::SkyRoute;
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

            let (trunk_taper_ratio, branch_reach_ratio) = tree_trunk_shape_metrics(&trunk_mesh);
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
                trunk_taper_ratio,
                branch_reach_ratio,
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

    let mesh_count = ground_cover.len() + trees.len() * 2 + clouds.len();
    let total_vertex_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.vertex_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.vertex_count, summary.canopy.vertex_count]),
        )
        .chain(clouds.iter().map(|summary| summary.mesh.vertex_count))
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
        terrain_biome_palette_count,
        foliage_palette_count,
        stone_palette_count,
        ground_cover,
        trees,
        clouds,
        palettes,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
}
