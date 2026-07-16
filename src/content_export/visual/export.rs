use super::{
    clouds::visual_content_cloud_summaries,
    landmarks::visual_content_landmark_summaries,
    metrics::{finite_range_f32, finite_ratio, min_finite_f32, visual_content_mesh_summary},
    palette::visual_content_palette_summary,
    types::{
        VisualContentExportReport, VisualGroundCoverSummary, VisualLandmarkSummary,
        VisualRockSummary, VisualSurfaceFeatureFamily, VisualTreeSummary,
    },
    vegetation::{
        ground_cover_blade_stats, tree_canopy_lobe_count, tree_trunk_shape_metrics,
        visual_content_tree_specs,
    },
};
use crate::{
    content_export::shared::{terrain_export_slug, write_mesh_obj},
    eval_runtime::remove_existing_dir,
    generated_content::{
        TERRAIN_BIOME_PALETTE_COUNT, TREE_CANOPY_CARD_COUNT, island_detail_budget,
        island_ground_cover_mesh, island_rock_specs, island_ruin_specs, rock_scatter_mesh,
        tree_canopy_mesh, tree_trunk_mesh,
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
    let mut rocks = Vec::new();
    let mut clouds = Vec::new();
    let mut landmarks = Vec::new();

    for (island_index, island) in route.islands().iter().copied().enumerate() {
        let island_slug = terrain_export_slug(island.name);
        let detail_budget = island_detail_budget(island);
        let ground_mesh =
            island_ground_cover_mesh(island_index, island, detail_budget.ground_cover_patch_count);
        let ground_obj = PathBuf::from("visuals")
            .join(format!("{island_index:02}_{island_slug}_ground_cover.obj"));
        write_mesh_obj(&output_dir.join(&ground_obj), &ground_mesh, "ground cover")?;
        let blade_stats = ground_cover_blade_stats(&ground_mesh);
        ground_cover.push(VisualGroundCoverSummary {
            island_name: island.name,
            island_slug: island_slug.clone(),
            mesh: visual_content_mesh_summary(ground_obj, &ground_mesh),
            patch_count: detail_budget.ground_cover_patch_count,
            blade_count: blade_stats.blade_count,
            min_blade_height_m: blade_stats.min_height_m,
            max_blade_height_m: blade_stats.max_height_m,
            blade_height_range_m: blade_stats.height_range_m,
        });

        for tree in visual_content_tree_specs(island_index, island) {
            let tree_slug = terrain_export_slug(&tree.label);
            let trunk_mesh =
                tree_trunk_mesh(tree.trunk_radius_m, tree.trunk_height_m, tree.trunk_seed);
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

        for (rock_index, rock) in island_rock_specs(island_index, island)
            .into_iter()
            .enumerate()
        {
            let label = format!("rock scatter {rock_index}");
            let rock_slug = terrain_export_slug(&label);
            let mesh = rock_scatter_mesh(rock.scale_m, rock.seed);
            let obj = PathBuf::from("visuals")
                .join(format!("{island_index:02}_{island_slug}_{rock_slug}.obj"));
            write_mesh_obj(&output_dir.join(&obj), &mesh, "rock scatter")?;
            rocks.push(VisualRockSummary {
                island_name: island.name,
                label,
                mesh: visual_content_mesh_summary(obj, &mesh),
                scale_m: rock.scale_m,
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
    let flora_cluster_count =
        surface_feature_count(&landmarks, VisualSurfaceFeatureFamily::FloraCluster);
    let flora_cluster_kind_count =
        surface_feature_kind_count(&landmarks, VisualSurfaceFeatureFamily::FloraCluster);
    let ruin_complex_count =
        surface_feature_count(&landmarks, VisualSurfaceFeatureFamily::RuinComplex);
    let ruin_complex_kind_count =
        surface_feature_kind_count(&landmarks, VisualSurfaceFeatureFamily::RuinComplex);
    let rock_formation_count =
        surface_feature_count(&landmarks, VisualSurfaceFeatureFamily::RockFormation);
    let rock_formation_kind_count =
        surface_feature_kind_count(&landmarks, VisualSurfaceFeatureFamily::RockFormation);
    let water_detail_count =
        surface_feature_count(&landmarks, VisualSurfaceFeatureFamily::WaterDetail);
    let water_detail_kind_count =
        surface_feature_kind_count(&landmarks, VisualSurfaceFeatureFamily::WaterDetail);
    let artifact_detail_count = landmarks
        .iter()
        .filter(|summary| summary.kind.starts_with("artifact_"))
        .count();
    let artifact_detail_kind_count = landmarks
        .iter()
        .filter(|summary| summary.kind.starts_with("artifact_"))
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
    let route_lake_surface_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "route_lake_surface")
        .count();
    let river_channel_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "river_channel")
        .count();
    let under_route_visual_count = landmarks
        .iter()
        .filter(|summary| summary.kind.starts_with("under_route_"))
        .count();
    let under_route_cave_mouth_count = landmarks
        .iter()
        .filter(|summary| summary.kind == "under_route_cave_mouth")
        .count();
    let ruin_cluster_count = route
        .islands()
        .iter()
        .copied()
        .enumerate()
        .filter(|(island_index, island)| !island_ruin_specs(*island_index, *island).is_empty())
        .count();

    let mesh_count =
        ground_cover.len() + trees.len() * 2 + rocks.len() + clouds.len() + landmarks.len();
    let total_vertex_count = ground_cover
        .iter()
        .map(|summary| summary.mesh.vertex_count)
        .chain(
            trees
                .iter()
                .flat_map(|summary| [summary.trunk.vertex_count, summary.canopy.vertex_count]),
        )
        .chain(rocks.iter().map(|summary| summary.mesh.vertex_count))
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
        .chain(rocks.iter().map(|summary| summary.mesh.triangle_count))
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
        rock_count: rocks.len(),
        weather_cloud_count: clouds.len(),
        weather_cloud_bank_count: clouds.iter().filter(|summary| summary.bank).count(),
        weather_cloud_veil_count: clouds
            .iter()
            .filter(|summary| summary.kind == "veil")
            .count(),
        landmark_count: landmarks.len(),
        landmark_kind_count,
        flora_cluster_count,
        flora_cluster_kind_count,
        ruin_complex_count,
        ruin_complex_kind_count,
        rock_formation_count,
        rock_formation_kind_count,
        water_detail_count,
        water_detail_kind_count,
        artifact_detail_count,
        artifact_detail_kind_count,
        artifact_stair_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "artifact_ancient_stair")
            .count(),
        artifact_bridge_fragment_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "artifact_bridge_fragment")
            .count(),
        artifact_glyph_slab_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "artifact_glyph_slab")
            .count(),
        artifact_retaining_wall_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "artifact_retaining_wall")
            .count(),
        artifact_banner_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "artifact_banner_strips")
            .count(),
        artifact_pebble_field_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "artifact_pebble_field")
            .count(),
        artifact_reed_patch_count: landmarks
            .iter()
            .filter(|summary| summary.kind == "artifact_reed_patch")
            .count(),
        small_island_count,
        plateau_landmark_count,
        plateau_waterfall_ribbon_count,
        plateau_waterfall_mist_count,
        route_waterfall_ribbon_count,
        route_waterfall_mist_count,
        route_lake_surface_count,
        river_channel_count,
        under_route_visual_count,
        under_route_cave_mouth_count,
        ruin_cluster_count,
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
        min_ground_cover_patch_count: ground_cover
            .iter()
            .map(|summary| summary.patch_count)
            .min()
            .unwrap_or(0),
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
        min_rock_mesh_vertices: rocks
            .iter()
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_rock_vertical_span_m: min_finite_f32(
            rocks.iter().map(|summary| summary.mesh.vertical_span_m),
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
        min_route_lake_surface_horizontal_span_m: min_landmark_horizontal_span(
            &landmarks,
            "route_lake_surface",
        ),
        min_river_channel_horizontal_span_m: min_landmark_planar_span(&landmarks, "river_channel"),
        min_under_route_visual_vertical_span_m: min_finite_f32(
            landmarks
                .iter()
                .filter(|summary| summary.kind.starts_with("under_route_"))
                .map(|summary| summary.mesh.vertical_span_m),
        ),
        surface_feature_vertex_total: landmarks
            .iter()
            .filter(|summary| summary.surface_feature_family.is_some())
            .map(|summary| summary.mesh.vertex_count)
            .sum(),
        min_flora_cluster_mesh_vertices: min_surface_feature_vertices(
            &landmarks,
            VisualSurfaceFeatureFamily::FloraCluster,
        ),
        min_flora_cluster_horizontal_span_m: min_surface_feature_horizontal_span(
            &landmarks,
            VisualSurfaceFeatureFamily::FloraCluster,
        ),
        min_flora_cluster_vertical_span_m: min_surface_feature_vertical_span(
            &landmarks,
            VisualSurfaceFeatureFamily::FloraCluster,
        ),
        min_ruin_complex_mesh_vertices: min_surface_feature_vertices(
            &landmarks,
            VisualSurfaceFeatureFamily::RuinComplex,
        ),
        min_ruin_complex_horizontal_span_m: min_surface_feature_horizontal_span(
            &landmarks,
            VisualSurfaceFeatureFamily::RuinComplex,
        ),
        min_ruin_complex_vertical_span_m: min_surface_feature_vertical_span(
            &landmarks,
            VisualSurfaceFeatureFamily::RuinComplex,
        ),
        min_rock_formation_mesh_vertices: min_surface_feature_vertices(
            &landmarks,
            VisualSurfaceFeatureFamily::RockFormation,
        ),
        min_rock_formation_horizontal_span_m: min_surface_feature_horizontal_span(
            &landmarks,
            VisualSurfaceFeatureFamily::RockFormation,
        ),
        min_rock_formation_vertical_span_m: min_surface_feature_vertical_span(
            &landmarks,
            VisualSurfaceFeatureFamily::RockFormation,
        ),
        min_water_detail_mesh_vertices: min_surface_feature_vertices(
            &landmarks,
            VisualSurfaceFeatureFamily::WaterDetail,
        ),
        min_water_detail_horizontal_span_m: min_surface_feature_horizontal_span(
            &landmarks,
            VisualSurfaceFeatureFamily::WaterDetail,
        ),
        min_water_detail_vertical_span_m: min_surface_feature_vertical_span(
            &landmarks,
            VisualSurfaceFeatureFamily::WaterDetail,
        ),
        artifact_detail_vertex_total: landmarks
            .iter()
            .filter(|summary| summary.kind.starts_with("artifact_"))
            .map(|summary| summary.mesh.vertex_count)
            .sum(),
        min_artifact_detail_mesh_vertices: landmarks
            .iter()
            .filter(|summary| summary.kind.starts_with("artifact_"))
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_artifact_stone_mesh_vertices: landmarks
            .iter()
            .filter(|summary| is_artifact_stone_kind(summary.kind))
            .map(|summary| summary.mesh.vertex_count)
            .min()
            .unwrap_or(0),
        min_artifact_stone_normal_slope_band_count: landmarks
            .iter()
            .filter(|summary| is_artifact_faceted_stone_kind(summary.kind))
            .map(|summary| summary.normal_slope_band_count)
            .min()
            .unwrap_or(0),
        min_artifact_stair_horizontal_span_m: min_landmark_planar_span(
            &landmarks,
            "artifact_ancient_stair",
        ),
        min_artifact_bridge_horizontal_span_m: min_landmark_planar_span(
            &landmarks,
            "artifact_bridge_fragment",
        ),
        min_artifact_banner_vertical_span_m: min_landmark_vertical_span(
            &landmarks,
            "artifact_banner_strips",
        ),
        min_artifact_reed_vertical_span_m: min_landmark_vertical_span(
            &landmarks,
            "artifact_reed_patch",
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
        rocks,
        clouds,
        landmarks,
        palettes,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
}

fn surface_feature_count(
    landmarks: &[VisualLandmarkSummary],
    family: VisualSurfaceFeatureFamily,
) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.surface_feature_family == Some(family))
        .count()
}

fn surface_feature_kind_count(
    landmarks: &[VisualLandmarkSummary],
    family: VisualSurfaceFeatureFamily,
) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.surface_feature_family == Some(family))
        .map(|summary| summary.kind)
        .collect::<HashSet<_>>()
        .len()
}

fn min_surface_feature_vertices(
    landmarks: &[VisualLandmarkSummary],
    family: VisualSurfaceFeatureFamily,
) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.surface_feature_family == Some(family))
        .map(|summary| summary.mesh.vertex_count)
        .min()
        .unwrap_or(0)
}

fn min_surface_feature_horizontal_span(
    landmarks: &[VisualLandmarkSummary],
    family: VisualSurfaceFeatureFamily,
) -> f32 {
    min_finite_f32(
        landmarks
            .iter()
            .filter(|summary| summary.surface_feature_family == Some(family))
            .map(|summary| {
                summary
                    .mesh
                    .horizontal_span_m
                    .max(summary.mesh.depth_span_m)
            }),
    )
}

fn min_surface_feature_vertical_span(
    landmarks: &[VisualLandmarkSummary],
    family: VisualSurfaceFeatureFamily,
) -> f32 {
    min_finite_f32(
        landmarks
            .iter()
            .filter(|summary| summary.surface_feature_family == Some(family))
            .map(|summary| summary.mesh.vertical_span_m),
    )
}

fn min_landmark_vertices(landmarks: &[VisualLandmarkSummary], kind: &str) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.mesh.vertex_count)
        .min()
        .unwrap_or(0)
}

fn is_artifact_stone_kind(kind: &str) -> bool {
    matches!(
        kind,
        "artifact_ancient_stair"
            | "artifact_retaining_wall"
            | "artifact_glyph_slab"
            | "artifact_bridge_fragment"
            | "artifact_pebble_field"
    )
}

fn is_artifact_faceted_stone_kind(kind: &str) -> bool {
    kind == "artifact_pebble_field"
}

fn min_landmark_triangles(landmarks: &[VisualLandmarkSummary], kind: &str) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.mesh.triangle_count)
        .min()
        .unwrap_or(0)
}

fn min_landmark_vertical_span(landmarks: &[VisualLandmarkSummary], kind: &str) -> f32 {
    min_finite_f32(
        landmarks
            .iter()
            .filter(|summary| summary.kind == kind)
            .map(|summary| summary.mesh.vertical_span_m),
    )
}

fn min_landmark_horizontal_span(landmarks: &[VisualLandmarkSummary], kind: &str) -> f32 {
    min_finite_f32(
        landmarks
            .iter()
            .filter(|summary| summary.kind == kind)
            .map(|summary| summary.mesh.horizontal_span_m),
    )
}

fn min_landmark_planar_span(landmarks: &[VisualLandmarkSummary], kind: &str) -> f32 {
    min_finite_f32(
        landmarks
            .iter()
            .filter(|summary| summary.kind == kind)
            .map(|summary| {
                summary
                    .mesh
                    .horizontal_span_m
                    .max(summary.mesh.depth_span_m)
            }),
    )
}

fn min_landmark_height_bands(landmarks: &[VisualLandmarkSummary], kind: &str) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.height_band_count)
        .min()
        .unwrap_or(0)
}

fn min_landmark_radius_bands(landmarks: &[VisualLandmarkSummary], kind: &str) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.radius_band_count)
        .min()
        .unwrap_or(0)
}

fn min_landmark_normal_slope_bands(landmarks: &[VisualLandmarkSummary], kind: &str) -> usize {
    landmarks
        .iter()
        .filter(|summary| summary.kind == kind)
        .map(|summary| summary.normal_slope_band_count)
        .min()
        .unwrap_or(0)
}
