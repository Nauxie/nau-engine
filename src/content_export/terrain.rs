use super::shared::{
    mesh_index_values, mesh_positions, terrain_export_json_number, terrain_export_json_string,
    terrain_export_json_vec2, terrain_export_json_vec3, terrain_export_slug, write_mesh_obj,
    write_terrain_material_weights_csv,
};
use crate::eval_runtime::{path_string, remove_existing_dir};
use crate::generated_content::{
    ISLAND_BODY_SEGMENTS, ISLAND_TERRAIN_RINGS, IslandDetailMaterials, TERRAIN_TEXTURE_SIZE,
    island_cliff_mesh, island_impostor_mesh, island_terrain_mesh, island_underside_mesh,
    mesh_normal_slope_band_count, mesh_terrain_material_channel_count,
    mesh_terrain_material_region_count, mesh_terrain_material_weight_band_count,
    mesh_vertex_color_band_count, mesh_vertical_band_count, mesh_y_range,
    procedural_terrain_surface_texture_data, texture_detail_band_count, texture_edge_promille,
};
use crate::island_visuals::{
    IslandCollisionCoverageAudit, IslandVisualCatalog, audit_island_collision_coverage,
    queue_sky_island,
};
use bevy::prelude::*;
use image::{Rgba, RgbaImage};
use nau_engine::world::{
    SkyIsland, SkyRoute, TerrainCollisionTruthReport, terrain_collision_truth_report,
};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

const TERRAIN_SHAPE_REVIEW_CONTACT_SHEET: &str = "visuals/terrain_shape_review.png";
const TERRAIN_SHAPE_REVIEW_TILE_WIDTH_PX: u32 = 220;
const TERRAIN_SHAPE_REVIEW_TILE_HEIGHT_PX: u32 = 170;
const TERRAIN_SHAPE_REVIEW_COLUMNS: u32 = 4;
const TERRAIN_SHAPE_REVIEW_TILE_GAP_PX: u32 = 10;
const TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX: u32 = 88;

#[derive(Debug)]
pub(crate) struct TerrainExportReport {
    pub(crate) manifest_path: PathBuf,
    pub(crate) island_count: usize,
    pub(crate) terrain_archetype_count: usize,
    pub(crate) shape_language_count: usize,
    pub(crate) mesh_count: usize,
    pub(crate) total_vertex_count: usize,
    pub(crate) total_triangle_count: usize,
    pub(crate) min_terrain_mesh_vertices: usize,
    pub(crate) min_terrain_color_bands: usize,
    pub(crate) min_terrain_material_weight_bands: usize,
    pub(crate) min_terrain_material_channels: usize,
    pub(crate) min_terrain_material_regions: usize,
    pub(crate) min_terrain_height_bands: usize,
    pub(crate) min_terrain_normal_slope_bands: usize,
    pub(crate) min_terrain_texture_detail_bands: usize,
    pub(crate) min_terrain_texture_edge_promille: usize,
    pub(crate) min_terrain_relief_range_m: f32,
    pub(crate) min_cliff_color_bands: usize,
    pub(crate) min_impostor_mesh_vertices: usize,
    pub(crate) min_impostor_color_bands: usize,
    pub(crate) seam_coverage: TerrainSeamCoverageSummary,
    pub(crate) collision_truth: TerrainCollisionTruthReport,
    pub(crate) visual_collision_coverage: IslandCollisionCoverageAudit,
    pub(crate) terrain_shape_review: TerrainShapeReviewSummary,
    pub(crate) islands: Vec<TerrainExportIslandSummary>,
}

#[derive(Debug)]
pub(crate) struct TerrainExportIslandSummary {
    pub(crate) index: usize,
    pub(crate) island: SkyIsland,
    pub(crate) slug: String,
    pub(crate) shape_signature: TerrainShapeSignatureSummary,
    pub(crate) seam: TerrainIslandSeamSummary,
    pub(crate) terrain: TerrainExportMeshSummary,
    pub(crate) cliff: TerrainExportMeshSummary,
    pub(crate) underside: TerrainExportMeshSummary,
    pub(crate) impostor: TerrainExportMeshSummary,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerrainSeamCoverageSummary {
    pub(crate) island_count: usize,
    pub(crate) max_terrain_cliff_top_gap_m: f32,
    pub(crate) min_terrain_edge_skirt_depth_m: f32,
    pub(crate) max_terrain_edge_skirt_horizontal_gap_m: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerrainIslandSeamSummary {
    pub(crate) max_terrain_cliff_top_gap_m: f32,
    pub(crate) min_terrain_edge_skirt_depth_m: f32,
    pub(crate) max_terrain_edge_skirt_horizontal_gap_m: f32,
}

#[derive(Debug)]
pub(crate) struct TerrainExportMeshSummary {
    pub(crate) obj_path: PathBuf,
    pub(crate) material_weights_path: Option<PathBuf>,
    pub(crate) vertex_count: usize,
    pub(crate) surface_vertex_count: usize,
    pub(crate) triangle_count: usize,
    pub(crate) color_bands: usize,
    pub(crate) material_weight_bands: usize,
    pub(crate) material_channels: usize,
    pub(crate) material_regions: usize,
    pub(crate) height_bands: usize,
    pub(crate) normal_slope_bands: usize,
    pub(crate) relief_range_m: f32,
}

#[derive(Debug)]
pub(crate) struct TerrainShapeSignatureSummary {
    pub(crate) silhouette_range: f32,
    pub(crate) mid_relief_range_m: f32,
    pub(crate) edge_relief_range_m: f32,
    pub(crate) radial_reversal_count: usize,
}

#[derive(Debug)]
pub(crate) struct TerrainShapeReviewSummary {
    pub(crate) contact_sheet_path: PathBuf,
    pub(crate) representative_count: usize,
    pub(crate) covered_shape_language_count: usize,
    pub(crate) covered_terrain_archetype_count: usize,
    pub(crate) min_projection_pixel_count: usize,
    pub(crate) min_projection_horizontal_span_px: usize,
    pub(crate) min_projection_vertical_span_px: usize,
    pub(crate) max_representative_terrain_cliff_top_gap_m: f32,
    pub(crate) min_representative_terrain_edge_skirt_depth_m: f32,
    pub(crate) max_representative_terrain_edge_skirt_horizontal_gap_m: f32,
    pub(crate) representatives: Vec<TerrainShapeReviewRepresentativeSummary>,
}

#[derive(Debug)]
pub(crate) struct TerrainShapeReviewRepresentativeSummary {
    pub(crate) island_index: usize,
    pub(crate) island_name: &'static str,
    pub(crate) shape_language: &'static str,
    pub(crate) terrain_archetype: &'static str,
    pub(crate) projection_pixel_count: usize,
    pub(crate) projection_horizontal_span_px: usize,
    pub(crate) projection_vertical_span_px: usize,
    pub(crate) terrain_cliff_top_gap_m: f32,
    pub(crate) terrain_edge_skirt_depth_m: f32,
    pub(crate) terrain_edge_skirt_horizontal_gap_m: f32,
}

#[derive(Clone, Copy, Debug)]
struct TerrainShapeReviewProjectionMetrics {
    pixel_count: usize,
    horizontal_span_px: usize,
    vertical_span_px: usize,
}

#[derive(Clone, Copy, Debug)]
struct TerrainShapeReviewBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
}

impl TerrainExportReport {
    fn to_json(&self) -> String {
        let islands = self
            .islands
            .iter()
            .map(|island| island.to_json("    "))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            concat!(
                "{{\n",
                "  \"schema\": \"nau_terrain_export.v1\",\n",
                "  \"island_count\": {},\n",
                "  \"terrain_archetype_count\": {},\n",
                "  \"shape_language_count\": {},\n",
                "  \"mesh_count\": {},\n",
                "  \"total_vertex_count\": {},\n",
                "  \"total_triangle_count\": {},\n",
                "  \"minimums\": {{\n",
                "    \"terrain_mesh_vertices\": {},\n",
                "    \"terrain_color_bands\": {},\n",
                "    \"terrain_material_weight_bands\": {},\n",
                "    \"terrain_material_channels\": {},\n",
                "    \"terrain_material_regions\": {},\n",
                "    \"terrain_height_bands\": {},\n",
                "    \"terrain_normal_slope_bands\": {},\n",
                "    \"terrain_texture_detail_bands\": {},\n",
                "    \"terrain_texture_edge_promille\": {},\n",
                "    \"terrain_relief_range_m\": {},\n",
                "    \"cliff_color_bands\": {},\n",
                "    \"impostor_mesh_vertices\": {},\n",
                "    \"impostor_color_bands\": {}\n",
                "  }},\n",
                "  \"seam_coverage\": {},\n",
                "  \"collision_truth\": {},\n",
                "  \"visual_collision_coverage\": {},\n",
                "  \"terrain_shape_review\": {},\n",
                "  \"islands\": [\n",
                "{}\n",
                "  ]\n",
                "}}\n"
            ),
            self.island_count,
            self.terrain_archetype_count,
            self.shape_language_count,
            self.mesh_count,
            self.total_vertex_count,
            self.total_triangle_count,
            self.min_terrain_mesh_vertices,
            self.min_terrain_color_bands,
            self.min_terrain_material_weight_bands,
            self.min_terrain_material_channels,
            self.min_terrain_material_regions,
            self.min_terrain_height_bands,
            self.min_terrain_normal_slope_bands,
            self.min_terrain_texture_detail_bands,
            self.min_terrain_texture_edge_promille,
            terrain_export_json_number(self.min_terrain_relief_range_m),
            self.min_cliff_color_bands,
            self.min_impostor_mesh_vertices,
            self.min_impostor_color_bands,
            self.seam_coverage.to_json(),
            terrain_collision_truth_json(self.collision_truth),
            visual_collision_coverage_json(&self.visual_collision_coverage),
            self.terrain_shape_review.to_json("  "),
            islands
        )
    }
}

fn terrain_collision_truth_json(report: TerrainCollisionTruthReport) -> String {
    format!(
        concat!(
            "{{\n",
            "    \"schema\": \"nau_terrain_collision_truth.v1\",\n",
            "    \"island_count\": {},\n",
            "    \"contour_sample_count\": {},\n",
            "    \"top_edge_probe_count\": {},\n",
            "    \"top_edge_air_barrier_count\": {},\n",
            "    \"edge_traverse_probe_count\": {},\n",
            "    \"edge_traverse_barrier_count\": {},\n",
            "    \"walkoff_shoulder_probe_count\": {},\n",
            "    \"walkoff_shoulder_barrier_count\": {},\n",
            "    \"far_field_probe_count\": {},\n",
            "    \"far_field_hit_count\": {},\n",
            "    \"near_cliff_probe_count\": {},\n",
            "    \"near_cliff_miss_count\": {},\n",
            "    \"excessive_near_cliff_push_count\": {},\n",
            "    \"max_top_edge_push_m\": {},\n",
            "    \"max_edge_traverse_push_m\": {},\n",
            "    \"max_walkoff_shoulder_push_m\": {},\n",
            "    \"max_far_field_push_m\": {},\n",
            "    \"max_near_cliff_push_m\": {}\n",
            "  }}"
        ),
        report.island_count,
        report.contour_sample_count,
        report.top_edge_probe_count,
        report.top_edge_air_barrier_count,
        report.edge_traverse_probe_count,
        report.edge_traverse_barrier_count,
        report.walkoff_shoulder_probe_count,
        report.walkoff_shoulder_barrier_count,
        report.far_field_probe_count,
        report.far_field_hit_count,
        report.near_cliff_probe_count,
        report.near_cliff_miss_count,
        report.excessive_near_cliff_push_count,
        terrain_export_json_number(report.max_top_edge_push_m),
        terrain_export_json_number(report.max_edge_traverse_push_m),
        terrain_export_json_number(report.max_walkoff_shoulder_push_m),
        terrain_export_json_number(report.max_far_field_push_m),
        terrain_export_json_number(report.max_near_cliff_push_m)
    )
}

fn visual_collision_coverage_json(audit: &IslandCollisionCoverageAudit) -> String {
    let failures = audit
        .failures
        .iter()
        .map(|failure| format!("      {}", terrain_export_json_string(failure)))
        .collect::<Vec<_>>()
        .join(",\n");

    format!(
        concat!(
            "{{\n",
            "    \"schema\": \"nau_visual_collision_coverage.v2\",\n",
            "    \"passed\": {},\n",
            "    \"checked_visual_count\": {},\n",
            "    \"solid_visual_count\": {},\n",
            "    \"surface_supported_solid_proxy_count\": {},\n",
            "    \"footprint_bounded_solid_proxy_count\": {},\n",
            "    \"min_solid_proxy_edge_clearance_m\": {:.3},\n",
            "    \"tree_solid_proxy_count\": {},\n",
            "    \"tree_footprint_bounded_proxy_count\": {},\n",
            "    \"rock_solid_proxy_count\": {},\n",
            "    \"rock_footprint_bounded_proxy_count\": {},\n",
            "    \"landmark_solid_proxy_count\": {},\n",
            "    \"landmark_footprint_bounded_proxy_count\": {},\n",
            "    \"obstacle_bounded_solid_proxy_count\": {},\n",
            "    \"terrain_rim_proxy_count\": {},\n",
            "    \"terrain_body_proxy_count\": {},\n",
            "    \"camera_only_allowance_count\": {},\n",
            "    \"non_blocking_visual_count\": {},\n",
            "    \"failure_count\": {},\n",
            "    \"failures\": [\n",
            "{}\n",
            "    ]\n",
            "  }}"
        ),
        audit.passed,
        audit.checked_visual_count,
        audit.solid_visual_count,
        audit.surface_supported_solid_proxy_count,
        audit.footprint_bounded_solid_proxy_count,
        audit.min_solid_proxy_edge_clearance_m,
        audit.tree_solid_proxy_count,
        audit.tree_footprint_bounded_proxy_count,
        audit.rock_solid_proxy_count,
        audit.rock_footprint_bounded_proxy_count,
        audit.landmark_solid_proxy_count,
        audit.landmark_footprint_bounded_proxy_count,
        audit.obstacle_bounded_solid_proxy_count,
        audit.terrain_rim_proxy_count,
        audit.terrain_body_proxy_count,
        audit.camera_only_allowance_count,
        audit.non_blocking_visual_count,
        audit.failures.len(),
        failures
    )
}

impl TerrainExportIslandSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"index\": {},\n\
             {indent}  \"name\": {},\n\
             {indent}  \"slug\": {},\n\
             {indent}  \"terrain_archetype\": {},\n\
             {indent}  \"terrain_archetype_index\": {},\n\
             {indent}  \"shape_language\": {},\n\
             {indent}  \"shape_language_index\": {},\n\
             {indent}  \"shape_signature\": {},\n\
             {indent}  \"seam\": {},\n\
             {indent}  \"center\": {},\n\
             {indent}  \"half_extents\": {},\n\
             {indent}  \"thickness_m\": {},\n\
             {indent}  \"target\": {},\n\
             {indent}  \"terrain\": {},\n\
             {indent}  \"cliff\": {},\n\
             {indent}  \"underside\": {},\n\
             {indent}  \"impostor\": {}\n\
             {indent}}}",
            self.index,
            terrain_export_json_string(self.island.name),
            terrain_export_json_string(&self.slug),
            terrain_export_json_string(self.island.terrain_archetype.label()),
            self.island.terrain_archetype.index(),
            terrain_export_json_string(self.island.shape_language().label()),
            self.island.shape_language().index(),
            self.shape_signature.to_json(),
            self.seam.to_json(),
            terrain_export_json_vec3(self.island.center),
            terrain_export_json_vec2(self.island.half_extents),
            terrain_export_json_number(self.island.thickness),
            self.island.is_target,
            self.terrain.to_json(),
            self.cliff.to_json(),
            self.underside.to_json(),
            self.impostor.to_json(),
        )
    }
}

impl TerrainShapeSignatureSummary {
    fn to_json(&self) -> String {
        format!(
            concat!(
                "{{\"silhouette_range\": {}, \"mid_relief_range_m\": {}, ",
                "\"edge_relief_range_m\": {}, \"radial_reversal_count\": {}}}"
            ),
            terrain_export_json_number(self.silhouette_range),
            terrain_export_json_number(self.mid_relief_range_m),
            terrain_export_json_number(self.edge_relief_range_m),
            self.radial_reversal_count
        )
    }
}

impl TerrainShapeReviewSummary {
    fn to_json(&self, indent: &str) -> String {
        let representatives = self
            .representatives
            .iter()
            .map(|representative| representative.to_json(&format!("{indent}    ")))
            .collect::<Vec<_>>()
            .join(",\n");

        format!(
            "{indent}{{\n\
             {indent}  \"contact_sheet\": {},\n\
             {indent}  \"representative_count\": {},\n\
             {indent}  \"covered_shape_language_count\": {},\n\
             {indent}  \"covered_terrain_archetype_count\": {},\n\
             {indent}  \"min_projection_pixel_count\": {},\n\
             {indent}  \"min_projection_horizontal_span_px\": {},\n\
             {indent}  \"min_projection_vertical_span_px\": {},\n\
             {indent}  \"max_representative_terrain_cliff_top_gap_m\": {},\n\
             {indent}  \"min_representative_terrain_edge_skirt_depth_m\": {},\n\
             {indent}  \"max_representative_terrain_edge_skirt_horizontal_gap_m\": {},\n\
             {indent}  \"representatives\": [\n\
             {}\n\
             {indent}  ]\n\
             {indent}}}",
            terrain_export_json_string(&path_string(&self.contact_sheet_path)),
            self.representative_count,
            self.covered_shape_language_count,
            self.covered_terrain_archetype_count,
            self.min_projection_pixel_count,
            self.min_projection_horizontal_span_px,
            self.min_projection_vertical_span_px,
            terrain_export_json_number(self.max_representative_terrain_cliff_top_gap_m),
            terrain_export_json_number(self.min_representative_terrain_edge_skirt_depth_m),
            terrain_export_json_number(self.max_representative_terrain_edge_skirt_horizontal_gap_m),
            representatives
        )
    }
}

impl TerrainShapeReviewRepresentativeSummary {
    fn to_json(&self, indent: &str) -> String {
        format!(
            "{indent}{{\n\
             {indent}  \"island_index\": {},\n\
             {indent}  \"island_name\": {},\n\
             {indent}  \"shape_language\": {},\n\
             {indent}  \"terrain_archetype\": {},\n\
             {indent}  \"projection_pixel_count\": {},\n\
             {indent}  \"projection_horizontal_span_px\": {},\n\
             {indent}  \"projection_vertical_span_px\": {},\n\
             {indent}  \"terrain_cliff_top_gap_m\": {},\n\
             {indent}  \"terrain_edge_skirt_depth_m\": {},\n\
             {indent}  \"terrain_edge_skirt_horizontal_gap_m\": {}\n\
             {indent}}}",
            self.island_index,
            terrain_export_json_string(self.island_name),
            terrain_export_json_string(self.shape_language),
            terrain_export_json_string(self.terrain_archetype),
            self.projection_pixel_count,
            self.projection_horizontal_span_px,
            self.projection_vertical_span_px,
            terrain_export_json_number(self.terrain_cliff_top_gap_m),
            terrain_export_json_number(self.terrain_edge_skirt_depth_m),
            terrain_export_json_number(self.terrain_edge_skirt_horizontal_gap_m)
        )
    }
}

impl TerrainSeamCoverageSummary {
    fn from_islands(islands: &[TerrainExportIslandSummary]) -> Self {
        let mut max_terrain_cliff_top_gap_m = 0.0_f32;
        let mut min_terrain_edge_skirt_depth_m = f32::INFINITY;
        let mut max_terrain_edge_skirt_horizontal_gap_m = 0.0_f32;

        for island in islands {
            max_terrain_cliff_top_gap_m =
                max_terrain_cliff_top_gap_m.max(island.seam.max_terrain_cliff_top_gap_m);
            min_terrain_edge_skirt_depth_m =
                min_terrain_edge_skirt_depth_m.min(island.seam.min_terrain_edge_skirt_depth_m);
            max_terrain_edge_skirt_horizontal_gap_m = max_terrain_edge_skirt_horizontal_gap_m
                .max(island.seam.max_terrain_edge_skirt_horizontal_gap_m);
        }
        if !min_terrain_edge_skirt_depth_m.is_finite() {
            min_terrain_edge_skirt_depth_m = 0.0;
        }

        Self {
            island_count: islands.len(),
            max_terrain_cliff_top_gap_m,
            min_terrain_edge_skirt_depth_m,
            max_terrain_edge_skirt_horizontal_gap_m,
        }
    }

    fn to_json(self) -> String {
        format!(
            concat!(
                "{{\n",
                "    \"schema\": \"nau_terrain_seam_coverage.v1\",\n",
                "    \"island_count\": {},\n",
                "    \"max_terrain_cliff_top_gap_m\": {},\n",
                "    \"min_terrain_edge_skirt_depth_m\": {},\n",
                "    \"max_terrain_edge_skirt_horizontal_gap_m\": {}\n",
                "  }}"
            ),
            self.island_count,
            terrain_export_json_number(self.max_terrain_cliff_top_gap_m),
            terrain_export_json_number(self.min_terrain_edge_skirt_depth_m),
            terrain_export_json_number(self.max_terrain_edge_skirt_horizontal_gap_m)
        )
    }
}

impl TerrainIslandSeamSummary {
    fn failed() -> Self {
        Self {
            max_terrain_cliff_top_gap_m: 9999.0,
            min_terrain_edge_skirt_depth_m: 0.0,
            max_terrain_edge_skirt_horizontal_gap_m: 9999.0,
        }
    }

    fn to_json(self) -> String {
        format!(
            concat!(
                "{{\"max_terrain_cliff_top_gap_m\": {}, ",
                "\"min_terrain_edge_skirt_depth_m\": {}, ",
                "\"max_terrain_edge_skirt_horizontal_gap_m\": {}}}"
            ),
            terrain_export_json_number(self.max_terrain_cliff_top_gap_m),
            terrain_export_json_number(self.min_terrain_edge_skirt_depth_m),
            terrain_export_json_number(self.max_terrain_edge_skirt_horizontal_gap_m)
        )
    }
}

impl TerrainExportMeshSummary {
    fn to_json(&self) -> String {
        let material_weights_path = self
            .material_weights_path
            .as_deref()
            .map(|path| terrain_export_json_string(&path_string(path)))
            .unwrap_or_else(|| "null".to_string());

        format!(
            concat!(
                "{{\"obj\": {}, \"material_weights_csv\": {}, ",
                "\"vertex_count\": {}, \"surface_vertex_count\": {}, \"triangle_count\": {}, ",
                "\"color_bands\": {}, \"material_weight_bands\": {}, ",
                "\"material_channels\": {}, \"material_regions\": {}, ",
                "\"height_bands\": {}, \"normal_slope_bands\": {}, \"relief_range_m\": {}}}"
            ),
            terrain_export_json_string(&path_string(&self.obj_path)),
            material_weights_path,
            self.vertex_count,
            self.surface_vertex_count,
            self.triangle_count,
            self.color_bands,
            self.material_weight_bands,
            self.material_channels,
            self.material_regions,
            self.height_bands,
            self.normal_slope_bands,
            terrain_export_json_number(self.relief_range_m)
        )
    }
}

pub(crate) fn export_terrain_inspection(output_dir: &Path) -> std::io::Result<TerrainExportReport> {
    fs::create_dir_all(output_dir)?;
    let islands_dir = output_dir.join("islands");
    remove_existing_dir(&islands_dir)?;
    fs::create_dir_all(&islands_dir)?;
    let visuals_dir = output_dir.join("visuals");
    remove_existing_dir(&visuals_dir)?;
    fs::create_dir_all(&visuals_dir)?;

    let route = SkyRoute::default();
    let mut islands = Vec::with_capacity(route.islands().len());

    for (index, island) in route.islands().iter().copied().enumerate() {
        let slug = terrain_export_slug(island.name);
        let prefix = format!("{index:02}_{slug}");
        let terrain_mesh = island_terrain_mesh(index, island);
        let cliff_mesh = island_cliff_mesh(index, island);
        let underside_mesh = island_underside_mesh(index, island);
        let impostor_mesh = island_impostor_mesh(index, island);

        let terrain_obj = PathBuf::from("islands").join(format!("{prefix}_terrain.obj"));
        let terrain_material_weights =
            PathBuf::from("islands").join(format!("{prefix}_terrain_material_weights.csv"));
        let cliff_obj = PathBuf::from("islands").join(format!("{prefix}_cliff.obj"));
        let underside_obj = PathBuf::from("islands").join(format!("{prefix}_underside.obj"));
        let impostor_obj = PathBuf::from("islands").join(format!("{prefix}_impostor.obj"));

        write_mesh_obj(&output_dir.join(&terrain_obj), &terrain_mesh, "terrain")?;
        write_terrain_material_weights_csv(
            &output_dir.join(&terrain_material_weights),
            &terrain_mesh,
        )?;
        write_mesh_obj(&output_dir.join(&cliff_obj), &cliff_mesh, "cliff")?;
        write_mesh_obj(
            &output_dir.join(&underside_obj),
            &underside_mesh,
            "underside",
        )?;
        write_mesh_obj(&output_dir.join(&impostor_obj), &impostor_mesh, "impostor")?;

        islands.push(TerrainExportIslandSummary {
            index,
            island,
            slug,
            shape_signature: terrain_shape_signature(island),
            seam: terrain_island_seam_summary(&terrain_mesh, &cliff_mesh),
            terrain: terrain_export_mesh_summary(
                terrain_obj,
                Some(terrain_material_weights),
                &terrain_mesh,
                terrain_surface_vertex_count(),
            ),
            cliff: terrain_export_mesh_summary(
                cliff_obj,
                None,
                &cliff_mesh,
                cliff_mesh.count_vertices(),
            ),
            underside: terrain_export_mesh_summary(
                underside_obj,
                None,
                &underside_mesh,
                underside_mesh.count_vertices(),
            ),
            impostor: terrain_export_mesh_summary(
                impostor_obj,
                None,
                &impostor_mesh,
                impostor_mesh.count_vertices(),
            ),
        });
    }

    let island_count = islands.len();
    let terrain_archetype_count = islands
        .iter()
        .fold(0_u32, |mask, island| {
            mask | (1_u32 << island.island.terrain_archetype.index())
        })
        .count_ones() as usize;
    let shape_language_count = islands
        .iter()
        .fold(0_u32, |mask, island| {
            mask | (1_u32 << island.island.shape_language().index())
        })
        .count_ones() as usize;
    let mesh_count = island_count * 4;
    let total_vertex_count = islands
        .iter()
        .map(|island| {
            island.terrain.vertex_count
                + island.cliff.vertex_count
                + island.underside.vertex_count
                + island.impostor.vertex_count
        })
        .sum();
    let total_triangle_count = islands
        .iter()
        .map(|island| {
            island.terrain.triangle_count
                + island.cliff.triangle_count
                + island.underside.triangle_count
                + island.impostor.triangle_count
        })
        .sum();
    let min_terrain_mesh_vertices = islands
        .iter()
        .map(|island| island.terrain.vertex_count)
        .min()
        .unwrap_or(0);
    let min_terrain_color_bands = islands
        .iter()
        .map(|island| island.terrain.color_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_material_weight_bands = islands
        .iter()
        .map(|island| island.terrain.material_weight_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_material_channels = islands
        .iter()
        .map(|island| island.terrain.material_channels)
        .min()
        .unwrap_or(0);
    let min_terrain_material_regions = islands
        .iter()
        .map(|island| island.terrain.material_regions)
        .min()
        .unwrap_or(0);
    let min_terrain_height_bands = islands
        .iter()
        .map(|island| island.terrain.height_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_normal_slope_bands = islands
        .iter()
        .map(|island| island.terrain.normal_slope_bands)
        .min()
        .unwrap_or(0);
    let min_terrain_texture_detail_bands = terrain_export_texture_detail_band_floor();
    let min_terrain_texture_edge_promille = terrain_export_texture_edge_promille_floor();
    let min_terrain_relief_range_m = islands
        .iter()
        .map(|island| island.terrain.relief_range_m)
        .min_by(f32::total_cmp)
        .unwrap_or(0.0);
    let min_cliff_color_bands = islands
        .iter()
        .flat_map(|island| [island.cliff.color_bands, island.underside.color_bands])
        .min()
        .unwrap_or(0);
    let min_impostor_mesh_vertices = islands
        .iter()
        .map(|island| island.impostor.vertex_count)
        .min()
        .unwrap_or(0);
    let min_impostor_color_bands = islands
        .iter()
        .map(|island| island.impostor.color_bands)
        .min()
        .unwrap_or(0);
    let seam_coverage = TerrainSeamCoverageSummary::from_islands(&islands);
    let collision_truth = terrain_collision_truth_report(route.islands());
    let visual_collision_coverage = terrain_export_visual_collision_coverage(&route);
    let terrain_shape_review = terrain_shape_review_summary(output_dir, &islands)?;

    let manifest_path = output_dir.join("manifest.json");
    let report = TerrainExportReport {
        manifest_path,
        island_count,
        terrain_archetype_count,
        shape_language_count,
        mesh_count,
        total_vertex_count,
        total_triangle_count,
        min_terrain_mesh_vertices,
        min_terrain_color_bands,
        min_terrain_material_weight_bands,
        min_terrain_material_channels,
        min_terrain_material_regions,
        min_terrain_height_bands,
        min_terrain_normal_slope_bands,
        min_terrain_texture_detail_bands,
        min_terrain_texture_edge_promille,
        min_terrain_relief_range_m,
        min_cliff_color_bands,
        min_impostor_mesh_vertices,
        min_impostor_color_bands,
        seam_coverage,
        collision_truth,
        visual_collision_coverage,
        terrain_shape_review,
        islands,
    };

    fs::write(&report.manifest_path, report.to_json())?;
    Ok(report)
}

fn terrain_shape_review_summary(
    output_dir: &Path,
    islands: &[TerrainExportIslandSummary],
) -> io::Result<TerrainShapeReviewSummary> {
    let representative_indices = terrain_shape_review_representative_indices(islands);
    let contact_sheet_path = PathBuf::from(TERRAIN_SHAPE_REVIEW_CONTACT_SHEET);
    let mut representatives = Vec::with_capacity(representative_indices.len());
    let mut tiles = Vec::with_capacity(representative_indices.len());
    let mut shape_languages = BTreeSet::new();
    let mut terrain_archetypes = BTreeSet::new();
    let mut min_projection_pixel_count = usize::MAX;
    let mut min_projection_horizontal_span_px = usize::MAX;
    let mut min_projection_vertical_span_px = usize::MAX;
    let mut max_representative_terrain_cliff_top_gap_m = 0.0_f32;
    let mut min_representative_terrain_edge_skirt_depth_m = f32::INFINITY;
    let mut max_representative_terrain_edge_skirt_horizontal_gap_m = 0.0_f32;

    for island_index in representative_indices {
        let island = &islands[island_index];
        let terrain_mesh = island_terrain_mesh(island.index, island.island);
        let cliff_mesh = island_cliff_mesh(island.index, island.island);
        let underside_mesh = island_underside_mesh(island.index, island.island);
        let (tile, metrics) =
            terrain_shape_review_tile(&terrain_mesh, &cliff_mesh, &underside_mesh);

        tiles.push(tile);
        shape_languages.insert(island.island.shape_language().label());
        terrain_archetypes.insert(island.island.terrain_archetype.label());
        min_projection_pixel_count = min_projection_pixel_count.min(metrics.pixel_count);
        min_projection_horizontal_span_px =
            min_projection_horizontal_span_px.min(metrics.horizontal_span_px);
        min_projection_vertical_span_px =
            min_projection_vertical_span_px.min(metrics.vertical_span_px);
        max_representative_terrain_cliff_top_gap_m =
            max_representative_terrain_cliff_top_gap_m.max(island.seam.max_terrain_cliff_top_gap_m);
        min_representative_terrain_edge_skirt_depth_m =
            min_representative_terrain_edge_skirt_depth_m
                .min(island.seam.min_terrain_edge_skirt_depth_m);
        max_representative_terrain_edge_skirt_horizontal_gap_m =
            max_representative_terrain_edge_skirt_horizontal_gap_m
                .max(island.seam.max_terrain_edge_skirt_horizontal_gap_m);

        representatives.push(TerrainShapeReviewRepresentativeSummary {
            island_index: island.index,
            island_name: island.island.name,
            shape_language: island.island.shape_language().label(),
            terrain_archetype: island.island.terrain_archetype.label(),
            projection_pixel_count: metrics.pixel_count,
            projection_horizontal_span_px: metrics.horizontal_span_px,
            projection_vertical_span_px: metrics.vertical_span_px,
            terrain_cliff_top_gap_m: island.seam.max_terrain_cliff_top_gap_m,
            terrain_edge_skirt_depth_m: island.seam.min_terrain_edge_skirt_depth_m,
            terrain_edge_skirt_horizontal_gap_m: island
                .seam
                .max_terrain_edge_skirt_horizontal_gap_m,
        });
    }

    if !min_representative_terrain_edge_skirt_depth_m.is_finite() {
        min_representative_terrain_edge_skirt_depth_m = 0.0;
    }

    let contact_sheet = terrain_shape_review_contact_sheet(&tiles);
    contact_sheet
        .save(output_dir.join(&contact_sheet_path))
        .map_err(terrain_image_error)?;

    Ok(TerrainShapeReviewSummary {
        contact_sheet_path,
        representative_count: representatives.len(),
        covered_shape_language_count: shape_languages.len(),
        covered_terrain_archetype_count: terrain_archetypes.len(),
        min_projection_pixel_count: if min_projection_pixel_count == usize::MAX {
            0
        } else {
            min_projection_pixel_count
        },
        min_projection_horizontal_span_px: if min_projection_horizontal_span_px == usize::MAX {
            0
        } else {
            min_projection_horizontal_span_px
        },
        min_projection_vertical_span_px: if min_projection_vertical_span_px == usize::MAX {
            0
        } else {
            min_projection_vertical_span_px
        },
        max_representative_terrain_cliff_top_gap_m,
        min_representative_terrain_edge_skirt_depth_m,
        max_representative_terrain_edge_skirt_horizontal_gap_m,
        representatives,
    })
}

fn terrain_shape_review_representative_indices(
    islands: &[TerrainExportIslandSummary],
) -> Vec<usize> {
    let mut seen_shape_languages = BTreeSet::new();
    islands
        .iter()
        .enumerate()
        .filter_map(|(index, island)| {
            let shape_language_index = island.island.shape_language().index();
            if seen_shape_languages.insert(shape_language_index) {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}

fn terrain_shape_review_tile(
    terrain_mesh: &Mesh,
    cliff_mesh: &Mesh,
    underside_mesh: &Mesh,
) -> (RgbaImage, TerrainShapeReviewProjectionMetrics) {
    let mut tile = RgbaImage::new(
        TERRAIN_SHAPE_REVIEW_TILE_WIDTH_PX,
        TERRAIN_SHAPE_REVIEW_TILE_HEIGHT_PX,
    );
    draw_terrain_shape_review_background(&mut tile);

    let terrain_positions = mesh_positions(terrain_mesh);
    let cliff_positions = mesh_positions(cliff_mesh);
    let underside_positions = mesh_positions(underside_mesh);
    let bounds = TerrainShapeReviewBounds::from_layers(&[
        terrain_positions,
        cliff_positions,
        underside_positions,
    ]);
    let mut projection_pixels = BTreeSet::new();

    draw_terrain_shape_review_layer(
        &mut tile,
        &bounds,
        underside_positions,
        [62, 53, 49],
        0.58,
        &mut projection_pixels,
    );
    draw_terrain_shape_review_layer(
        &mut tile,
        &bounds,
        cliff_positions,
        [133, 105, 76],
        0.66,
        &mut projection_pixels,
    );
    draw_terrain_shape_review_layer(
        &mut tile,
        &bounds,
        terrain_positions,
        [112, 156, 105],
        0.78,
        &mut projection_pixels,
    );
    draw_terrain_shape_review_seam_strip(&mut tile, terrain_positions, cliff_positions);
    draw_terrain_shape_review_panel_frames(&mut tile);

    (
        tile,
        terrain_shape_review_projection_metrics(&projection_pixels),
    )
}

fn terrain_shape_review_contact_sheet(tiles: &[RgbaImage]) -> RgbaImage {
    let row_count = ((tiles.len() as u32).saturating_add(TERRAIN_SHAPE_REVIEW_COLUMNS - 1)
        / TERRAIN_SHAPE_REVIEW_COLUMNS)
        .max(1);
    let width = TERRAIN_SHAPE_REVIEW_TILE_GAP_PX
        + TERRAIN_SHAPE_REVIEW_COLUMNS
            * (TERRAIN_SHAPE_REVIEW_TILE_WIDTH_PX + TERRAIN_SHAPE_REVIEW_TILE_GAP_PX);
    let height = TERRAIN_SHAPE_REVIEW_TILE_GAP_PX
        + row_count * (TERRAIN_SHAPE_REVIEW_TILE_HEIGHT_PX + TERRAIN_SHAPE_REVIEW_TILE_GAP_PX);
    let mut sheet = RgbaImage::from_pixel(width, height, Rgba([29, 38, 39, 255]));

    for (index, tile) in tiles.iter().enumerate() {
        let column = index as u32 % TERRAIN_SHAPE_REVIEW_COLUMNS;
        let row = index as u32 / TERRAIN_SHAPE_REVIEW_COLUMNS;
        let origin_x = TERRAIN_SHAPE_REVIEW_TILE_GAP_PX
            + column * (TERRAIN_SHAPE_REVIEW_TILE_WIDTH_PX + TERRAIN_SHAPE_REVIEW_TILE_GAP_PX);
        let origin_y = TERRAIN_SHAPE_REVIEW_TILE_GAP_PX
            + row * (TERRAIN_SHAPE_REVIEW_TILE_HEIGHT_PX + TERRAIN_SHAPE_REVIEW_TILE_GAP_PX);
        blit_terrain_shape_review_tile(&mut sheet, tile, origin_x, origin_y);
    }

    sheet
}

fn draw_terrain_shape_review_background(tile: &mut RgbaImage) {
    for y in 0..tile.height() {
        for x in 0..tile.width() {
            let t = y as f32 / (tile.height() - 1) as f32;
            let r = (142.0 * (1.0 - t) + 206.0 * t).round() as u8;
            let g = (183.0 * (1.0 - t) + 205.0 * t).round() as u8;
            let b = (197.0 * (1.0 - t) + 179.0 * t).round() as u8;
            tile.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    fill_terrain_shape_review_rect(
        tile,
        12,
        14,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        [222, 218, 190],
    );
    fill_terrain_shape_review_rect(
        tile,
        120,
        14,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        [205, 211, 205],
    );
    fill_terrain_shape_review_rect(tile, 12, 118, 196, 36, [86, 109, 93]);
}

fn draw_terrain_shape_review_layer(
    tile: &mut RgbaImage,
    bounds: &TerrainShapeReviewBounds,
    positions: &[[f32; 3]],
    color: [u8; 3],
    alpha: f32,
    projection_pixels: &mut BTreeSet<(u32, u32)>,
) {
    for position in positions {
        let position = Vec3::from_array(*position);
        let top_x = 12
            + terrain_shape_review_project(
                position.x,
                bounds.min_x,
                bounds.max_x,
                TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
            );
        let top_y = 14
            + terrain_shape_review_project(
                position.z,
                bounds.min_z,
                bounds.max_z,
                TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
            );
        let side_x = 120
            + terrain_shape_review_project(
                position.x,
                bounds.min_x,
                bounds.max_x,
                TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
            );
        let side_y = 14 + (TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX - 1)
            - terrain_shape_review_project(
                position.y,
                bounds.min_y,
                bounds.max_y,
                TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
            );

        draw_terrain_shape_review_point(tile, top_x, top_y, color, alpha, projection_pixels);
        draw_terrain_shape_review_point(tile, side_x, side_y, color, alpha, projection_pixels);
    }
}

fn draw_terrain_shape_review_point(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    color: [u8; 3],
    alpha: f32,
    projection_pixels: &mut BTreeSet<(u32, u32)>,
) {
    for offset_y in -1_i32..=1 {
        for offset_x in -1_i32..=1 {
            let px = x as i32 + offset_x;
            let py = y as i32 + offset_y;
            if px < 0 || py < 0 {
                continue;
            }
            let px = px as u32;
            let py = py as u32;
            if px >= image.width() || py >= image.height() {
                continue;
            }
            projection_pixels.insert((px, py));
            blend_terrain_shape_review_pixel(image, px, py, color, alpha);
        }
    }
}

fn draw_terrain_shape_review_panel_frames(tile: &mut RgbaImage) {
    stroke_terrain_shape_review_rect(
        tile,
        12,
        14,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        [236, 222, 184],
    );
    stroke_terrain_shape_review_rect(
        tile,
        120,
        14,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        TERRAIN_SHAPE_REVIEW_PANEL_SIZE_PX,
        [236, 222, 184],
    );
    stroke_terrain_shape_review_rect(tile, 12, 118, 196, 36, [236, 222, 184]);
}

fn draw_terrain_shape_review_seam_strip(
    tile: &mut RgbaImage,
    terrain_positions: &[[f32; 3]],
    cliff_positions: &[[f32; 3]],
) {
    let terrain_outer_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;
    let terrain_skirt_start = 1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS;
    let required_terrain_vertices = terrain_skirt_start + ISLAND_BODY_SEGMENTS;
    if terrain_positions.len() < required_terrain_vertices
        || cliff_positions.len() < ISLAND_BODY_SEGMENTS
    {
        return;
    }

    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for segment in 0..ISLAND_BODY_SEGMENTS {
        for y in [
            terrain_positions[terrain_outer_start + segment][1],
            terrain_positions[terrain_skirt_start + segment][1],
            cliff_positions[segment][1],
        ] {
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let x = 16 + ((segment as f32 / (ISLAND_BODY_SEGMENTS - 1) as f32) * 188.0).round() as u32;
        let terrain_y = 148
            - terrain_shape_review_project(
                terrain_positions[terrain_outer_start + segment][1],
                min_y,
                max_y,
                26,
            );
        let cliff_y =
            148 - terrain_shape_review_project(cliff_positions[segment][1], min_y, max_y, 26);
        let skirt_y = 148
            - terrain_shape_review_project(
                terrain_positions[terrain_skirt_start + segment][1],
                min_y,
                max_y,
                26,
            );

        draw_terrain_shape_review_marker(tile, x, skirt_y, [62, 53, 49], 0.78);
        draw_terrain_shape_review_marker(tile, x, cliff_y, [133, 105, 76], 0.82);
        draw_terrain_shape_review_marker(tile, x, terrain_y, [112, 156, 105], 0.88);
    }
}

fn draw_terrain_shape_review_marker(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    color: [u8; 3],
    alpha: f32,
) {
    for offset_y in -1_i32..=1 {
        for offset_x in -1_i32..=1 {
            let px = x as i32 + offset_x;
            let py = y as i32 + offset_y;
            if px < 0 || py < 0 {
                continue;
            }
            let px = px as u32;
            let py = py as u32;
            if px < image.width() && py < image.height() {
                blend_terrain_shape_review_pixel(image, px, py, color, alpha);
            }
        }
    }
}

fn fill_terrain_shape_review_rect(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 3],
) {
    for py in y..(y + height).min(image.height()) {
        for px in x..(x + width).min(image.width()) {
            image.put_pixel(px, py, Rgba([color[0], color[1], color[2], 255]));
        }
    }
}

fn stroke_terrain_shape_review_rect(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 3],
) {
    for px in x..(x + width).min(image.width()) {
        image.put_pixel(px, y, Rgba([color[0], color[1], color[2], 255]));
        image.put_pixel(
            px,
            (y + height - 1).min(image.height() - 1),
            Rgba([color[0], color[1], color[2], 255]),
        );
    }
    for py in y..(y + height).min(image.height()) {
        image.put_pixel(x, py, Rgba([color[0], color[1], color[2], 255]));
        image.put_pixel(
            (x + width - 1).min(image.width() - 1),
            py,
            Rgba([color[0], color[1], color[2], 255]),
        );
    }
}

fn blit_terrain_shape_review_tile(
    sheet: &mut RgbaImage,
    tile: &RgbaImage,
    origin_x: u32,
    origin_y: u32,
) {
    let border = Rgba([224, 211, 166, 255]);
    for y in 0..tile.height() + 2 {
        for x in 0..tile.width() + 2 {
            sheet.put_pixel(origin_x + x, origin_y + y, border);
        }
    }
    for y in 0..tile.height() {
        for x in 0..tile.width() {
            sheet.put_pixel(origin_x + x + 1, origin_y + y + 1, *tile.get_pixel(x, y));
        }
    }
}

fn terrain_shape_review_project(value: f32, min: f32, max: f32, size: u32) -> u32 {
    if (max - min).abs() <= f32::EPSILON {
        return size / 2;
    }
    (((value - min) / (max - min)).clamp(0.0, 1.0) * (size - 1) as f32).round() as u32
}

fn terrain_shape_review_projection_metrics(
    projection_pixels: &BTreeSet<(u32, u32)>,
) -> TerrainShapeReviewProjectionMetrics {
    let mut min_x = u32::MAX;
    let mut max_x = 0;
    let mut min_y = u32::MAX;
    let mut max_y = 0;

    for (x, y) in projection_pixels {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_y = min_y.min(*y);
        max_y = max_y.max(*y);
    }

    if projection_pixels.is_empty() {
        return TerrainShapeReviewProjectionMetrics {
            pixel_count: 0,
            horizontal_span_px: 0,
            vertical_span_px: 0,
        };
    }

    TerrainShapeReviewProjectionMetrics {
        pixel_count: projection_pixels.len(),
        horizontal_span_px: (max_x - min_x + 1) as usize,
        vertical_span_px: (max_y - min_y + 1) as usize,
    }
}

fn blend_terrain_shape_review_pixel(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    color: [u8; 3],
    alpha: f32,
) {
    let alpha = alpha.clamp(0.0, 1.0);
    let pixel = image.get_pixel_mut(x, y);
    let current = pixel.0;
    let blend = |source: u8, destination: u8| -> u8 {
        (source as f32 * alpha + destination as f32 * (1.0 - alpha)).round() as u8
    };
    *pixel = Rgba([
        blend(color[0], current[0]),
        blend(color[1], current[1]),
        blend(color[2], current[2]),
        255,
    ]);
}

fn terrain_image_error(error: image::ImageError) -> io::Error {
    io::Error::other(error)
}

impl TerrainShapeReviewBounds {
    fn from_layers(layers: &[&[[f32; 3]]]) -> Self {
        let mut bounds = Self {
            min_x: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            min_y: f32::INFINITY,
            max_y: f32::NEG_INFINITY,
            min_z: f32::INFINITY,
            max_z: f32::NEG_INFINITY,
        };

        for positions in layers {
            for position in *positions {
                bounds.min_x = bounds.min_x.min(position[0]);
                bounds.max_x = bounds.max_x.max(position[0]);
                bounds.min_y = bounds.min_y.min(position[1]);
                bounds.max_y = bounds.max_y.max(position[1]);
                bounds.min_z = bounds.min_z.min(position[2]);
                bounds.max_z = bounds.max_z.max(position[2]);
            }
        }

        if !bounds.min_x.is_finite() {
            return Self {
                min_x: -1.0,
                max_x: 1.0,
                min_y: -1.0,
                max_y: 1.0,
                min_z: -1.0,
                max_z: 1.0,
            };
        }

        bounds
    }
}

fn terrain_export_visual_collision_coverage(route: &SkyRoute) -> IslandCollisionCoverageAudit {
    let mut catalog = IslandVisualCatalog::default();
    let mut diagnostics = crate::content_diagnostics::IslandContentDiagnostics::default();
    let mut meshes = Assets::<Mesh>::default();
    let material = Handle::<StandardMaterial>::default();
    let detail_materials = IslandDetailMaterials {
        trunk: material.clone(),
        foliage: material.clone(),
        ground_cover: material.clone(),
        stone: material.clone(),
    };

    for (index, island) in route.islands().iter().copied().enumerate() {
        queue_sky_island(
            &mut catalog,
            &mut diagnostics,
            &mut meshes,
            material.clone(),
            material.clone(),
            material.clone(),
            material.clone(),
            material.clone(),
            detail_materials.clone(),
            material.clone(),
            material.clone(),
            index,
            island,
        );
    }

    audit_island_collision_coverage(&catalog, route)
}

fn terrain_shape_signature(island: SkyIsland) -> TerrainShapeSignatureSummary {
    let sample_count = 192;
    let mut min_silhouette = f32::INFINITY;
    let mut max_silhouette = f32::NEG_INFINITY;
    let mut min_mid_relief = f32::INFINITY;
    let mut max_mid_relief = f32::NEG_INFINITY;
    let mut min_edge_relief = f32::INFINITY;
    let mut max_edge_relief = f32::NEG_INFINITY;
    let mut radial_reversal_count = 0;

    for step in 0..sample_count {
        let angle = step as f32 / sample_count as f32 * std::f32::consts::TAU;
        let silhouette = island.visual_silhouette_scale(angle);
        let mid_relief = island.terrain_relief_m(0.64, angle);
        let edge_relief = island.terrain_relief_m(0.86, angle);

        min_silhouette = min_silhouette.min(silhouette);
        max_silhouette = max_silhouette.max(silhouette);
        min_mid_relief = min_mid_relief.min(mid_relief);
        max_mid_relief = max_mid_relief.max(mid_relief);
        min_edge_relief = min_edge_relief.min(edge_relief);
        max_edge_relief = max_edge_relief.max(edge_relief);

        if step % 16 == 0 {
            radial_reversal_count += radial_relief_reversal_count(island, angle);
        }
    }

    TerrainShapeSignatureSummary {
        silhouette_range: max_silhouette - min_silhouette,
        mid_relief_range_m: max_mid_relief - min_mid_relief,
        edge_relief_range_m: max_edge_relief - min_edge_relief,
        radial_reversal_count,
    }
}

fn terrain_island_seam_summary(terrain_mesh: &Mesh, cliff_mesh: &Mesh) -> TerrainIslandSeamSummary {
    let terrain_positions = mesh_positions(terrain_mesh);
    let cliff_positions = mesh_positions(cliff_mesh);
    let terrain_outer_start = 1 + (ISLAND_TERRAIN_RINGS - 1) * ISLAND_BODY_SEGMENTS;
    let terrain_skirt_start = 1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS;
    let required_terrain_vertices = terrain_skirt_start + ISLAND_BODY_SEGMENTS;
    if terrain_positions.len() < required_terrain_vertices
        || cliff_positions.len() < ISLAND_BODY_SEGMENTS
    {
        return TerrainIslandSeamSummary::failed();
    }

    let mut max_terrain_cliff_top_gap_m = 0.0_f32;
    let mut min_terrain_edge_skirt_depth_m = f32::INFINITY;
    let mut max_terrain_edge_skirt_horizontal_gap_m = 0.0_f32;

    for segment in 0..ISLAND_BODY_SEGMENTS {
        let terrain = Vec3::from_array(terrain_positions[terrain_outer_start + segment]);
        let skirt = Vec3::from_array(terrain_positions[terrain_skirt_start + segment]);
        let cliff = Vec3::from_array(cliff_positions[segment]);
        max_terrain_cliff_top_gap_m = max_terrain_cliff_top_gap_m.max(terrain.distance(cliff));
        min_terrain_edge_skirt_depth_m = min_terrain_edge_skirt_depth_m.min(terrain.y - skirt.y);
        max_terrain_edge_skirt_horizontal_gap_m = max_terrain_edge_skirt_horizontal_gap_m
            .max(Vec2::new(terrain.x, terrain.z).distance(Vec2::new(skirt.x, skirt.z)));
    }

    TerrainIslandSeamSummary {
        max_terrain_cliff_top_gap_m,
        min_terrain_edge_skirt_depth_m,
        max_terrain_edge_skirt_horizontal_gap_m,
    }
}

fn radial_relief_reversal_count(island: SkyIsland, angle: f32) -> usize {
    let mut reversal_count = 0;
    let mut previous_relief = island.terrain_relief_m(0.14, angle);
    let mut previous_slope = 0.0_f32;

    for step in 1..=32 {
        let radius = 0.14 + step as f32 / 32.0 * 0.80;
        let relief = island.terrain_relief_m(radius, angle);
        let slope = relief - previous_relief;
        if previous_slope.abs() > 0.004
            && slope.abs() > 0.004
            && previous_slope.signum() != slope.signum()
        {
            reversal_count += 1;
        }
        previous_slope = slope;
        previous_relief = relief;
    }

    reversal_count
}

fn terrain_export_mesh_summary(
    obj_path: PathBuf,
    material_weights_path: Option<PathBuf>,
    mesh: &Mesh,
    surface_vertex_count: usize,
) -> TerrainExportMeshSummary {
    TerrainExportMeshSummary {
        obj_path,
        material_weights_path,
        vertex_count: mesh.count_vertices(),
        surface_vertex_count,
        triangle_count: mesh_index_values(mesh).len() / 3,
        color_bands: mesh_vertex_color_band_count(mesh),
        material_weight_bands: mesh_terrain_material_weight_band_count(mesh),
        material_channels: mesh_terrain_material_channel_count(mesh),
        material_regions: mesh_terrain_material_region_count(mesh),
        height_bands: mesh_vertical_band_count(mesh),
        normal_slope_bands: mesh_normal_slope_band_count(mesh),
        relief_range_m: mesh_y_range(mesh),
    }
}

fn terrain_surface_vertex_count() -> usize {
    1 + ISLAND_TERRAIN_RINGS * ISLAND_BODY_SEGMENTS
}

fn terrain_export_texture_detail_band_floor() -> usize {
    terrain_export_texture_metric_floor(texture_detail_band_count)
}

fn terrain_export_texture_edge_promille_floor() -> usize {
    terrain_export_texture_metric_floor(|data| texture_edge_promille(data, TERRAIN_TEXTURE_SIZE))
}

fn terrain_export_texture_metric_floor(mut metric: impl FnMut(&[u8]) -> usize) -> usize {
    [
        (
            [54, 128, 70, 255],
            [28, 92, 48, 255],
            [128, 174, 78, 255],
            17,
        ),
        (
            [96, 138, 70, 255],
            [56, 104, 54, 255],
            [166, 172, 90, 255],
            19,
        ),
        (
            [126, 104, 76, 255],
            [80, 70, 60, 255],
            [162, 138, 96, 255],
            23,
        ),
        (
            [52, 110, 118, 255],
            [30, 80, 94, 255],
            [142, 176, 164, 255],
            29,
        ),
        (
            [132, 132, 92, 255],
            [86, 96, 70, 255],
            [178, 166, 112, 255],
            31,
        ),
        (
            [70, 150, 94, 255],
            [34, 100, 62, 255],
            [156, 198, 112, 255],
            37,
        ),
    ]
    .into_iter()
    .map(|(primary, secondary, accent, seed)| {
        let data = procedural_terrain_surface_texture_data(
            primary,
            secondary,
            accent,
            seed,
            TERRAIN_TEXTURE_SIZE,
        );
        metric(&data)
    })
    .min()
    .unwrap_or(0)
}
