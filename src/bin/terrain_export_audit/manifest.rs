use crate::{
    artifact::{audit_obj_path, audit_weight_csv_path},
    checks::{
        check_at_least_f64, check_at_least_u64, check_at_most_f64, check_at_most_u64,
        check_eq_bool, check_eq_str, check_eq_u64, error_artifact, relative_path, value_bool,
        value_f64, value_u64,
    },
    thresholds::*,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub(crate) fn audit_manifest_path(path: &Path) -> Result<Value, String> {
    let manifest_text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest = serde_json::from_str::<Value>(&manifest_text).map_err(|error| {
        format!(
            "could not parse terrain export manifest {}: {error}",
            path.display()
        )
    })?;
    let root_dir = path.parent().unwrap_or_else(|| Path::new("."));

    Ok(audit_manifest(&manifest, root_dir, &path.to_string_lossy()))
}

pub(crate) fn audit_manifest(manifest: &Value, root_dir: &Path, manifest_path: &str) -> Value {
    let mut checks = Vec::new();
    let mut artifacts = Vec::new();
    let mut expected_artifact_count = 0u64;
    let mut found_artifact_count = 0u64;
    let mut missing_artifact_count = 0u64;
    let mut obj_vertex_mismatch_count = 0u64;
    let mut obj_face_mismatch_count = 0u64;
    let mut obj_color_mismatch_count = 0u64;
    let mut obj_height_band_mismatch_count = 0u64;
    let mut obj_normal_slope_band_mismatch_count = 0u64;
    let mut csv_row_mismatch_count = 0u64;
    let mut csv_band_mismatch_count = 0u64;
    let mut csv_channel_mismatch_count = 0u64;
    let mut csv_region_mismatch_count = 0u64;
    let mut min_region_promille = [u64::MAX; TERRAIN_MATERIAL_REGION_COUNT];
    let mut aggregate_region_promille_sums = [0u64; TERRAIN_MATERIAL_REGION_COUNT];
    let mut aggregate_region_row_count = 0u64;
    let mut min_terrain_silhouette_radius_bands = u64::MAX;
    let mut min_terrain_vertical_bands = u64::MAX;
    let mut min_terrain_normal_slope_bands = u64::MAX;
    let mut min_island_body_vertical_range_m = f64::INFINITY;
    let mut min_island_body_silhouette_radius_bands = u64::MAX;
    let mut min_impostor_vertical_range_m = f64::INFINITY;
    let mut min_impostor_horizontal_radius_bands = u64::MAX;
    let mut min_shape_silhouette_range = f64::INFINITY;
    let mut min_shape_mid_relief_range_m = f64::INFINITY;
    let mut min_shape_edge_relief_range_m = f64::INFINITY;
    let mut min_shape_radial_reversal_count = u64::MAX;
    let mut max_island_terrain_cliff_top_gap_m = 0.0_f64;
    let mut min_island_terrain_edge_skirt_depth_m = f64::INFINITY;
    let mut max_island_terrain_edge_skirt_horizontal_gap_m = 0.0_f64;

    let schema = manifest.get("schema").and_then(Value::as_str).unwrap_or("");
    checks.push(check_eq_str(
        "schema",
        schema,
        "nau_terrain_export.v1",
        "schema",
    ));

    let island_count = manifest
        .get("island_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let terrain_archetype_count = manifest
        .get("terrain_archetype_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let shape_language_count = manifest
        .get("shape_language_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let mesh_count = manifest
        .get("mesh_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total_vertex_count = manifest
        .get("total_vertex_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total_triangle_count = manifest
        .get("total_triangle_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let islands = manifest
        .get("islands")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let collision_truth = manifest.get("collision_truth").unwrap_or(&Value::Null);
    let visual_collision_coverage = manifest
        .get("visual_collision_coverage")
        .unwrap_or(&Value::Null);
    let seam_coverage = manifest.get("seam_coverage").unwrap_or(&Value::Null);
    let streaming_budget = manifest.get("streaming_budget").unwrap_or(&Value::Null);
    let terrain_shape_review = manifest.get("terrain_shape_review").unwrap_or(&Value::Null);
    let expected_streaming_pair_sample_count =
        island_count.saturating_mul(island_count.saturating_sub(1)) / 2;
    let expected_streaming_sample_count = island_count + expected_streaming_pair_sample_count + 1;
    let expected_collision_truth_probe_count =
        island_count.saturating_mul(TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND);
    let expected_collision_truth_edge_traverse_probe_count = expected_collision_truth_probe_count
        .saturating_mul(TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_PROBES_PER_CONTOUR_SAMPLE);
    let expected_collision_truth_walkoff_shoulder_probe_count =
        expected_collision_truth_probe_count
            .saturating_mul(TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_PROBES_PER_CONTOUR_SAMPLE);
    let expected_visual_terrain_body_proxy_count =
        island_count.saturating_mul(VISUAL_COLLISION_TERRAIN_BODY_PROXIES_PER_ISLAND);
    let expected_visual_terrain_rim_proxy_count =
        island_count.saturating_mul(VISUAL_COLLISION_TERRAIN_RIM_PROXIES_PER_ISLAND);

    checks.push(check_at_least_u64(
        "island_count",
        island_count,
        MIN_ISLAND_COUNT,
        "islands",
    ));
    checks.push(check_eq_u64(
        "island_array_count",
        islands.len() as u64,
        island_count,
        "islands",
    ));
    checks.push(check_at_least_u64(
        "terrain_archetype_count",
        terrain_archetype_count,
        MIN_TERRAIN_ARCHETYPE_COUNT,
        "archetypes",
    ));
    checks.push(check_at_least_u64(
        "terrain_shape_language_count",
        shape_language_count,
        MIN_TERRAIN_SHAPE_LANGUAGE_COUNT,
        "shape_languages",
    ));
    checks.push(check_eq_u64(
        "mesh_count",
        mesh_count,
        island_count.saturating_mul(4),
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "total_vertex_count",
        total_vertex_count,
        island_count.saturating_mul(MIN_TERRAIN_MESH_VERTICES),
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "total_triangle_count",
        total_triangle_count,
        island_count.saturating_mul(4000),
        "triangles",
    ));
    checks.push(check_at_least_u64(
        "streaming_budget_sample_count",
        value_u64(streaming_budget, "sample_count"),
        expected_streaming_sample_count,
        "samples",
    ));
    checks.push(check_eq_u64(
        "streaming_budget_pair_sample_count",
        value_u64(streaming_budget, "pair_sample_count"),
        expected_streaming_pair_sample_count,
        "samples",
    ));
    checks.push(check_at_most_u64(
        "streaming_budget_active_chunk_count",
        value_u64(streaming_budget, "max_active_chunk_count"),
        MAX_STREAMING_BUDGET_ACTIVE_CHUNKS,
        "chunks",
    ));
    checks.push(check_at_most_u64(
        "streaming_budget_active_island_count",
        value_u64(streaming_budget, "max_active_island_count"),
        MAX_STREAMING_BUDGET_ACTIVE_ISLANDS,
        "islands",
    ));
    checks.push(check_at_most_u64(
        "streaming_budget_near_lod_islands",
        value_u64(streaming_budget, "max_near_lod_islands"),
        MAX_STREAMING_BUDGET_NEAR_LOD_ISLANDS,
        "islands",
    ));
    checks.push(check_at_most_u64(
        "streaming_budget_visible_terrain_mesh_count",
        value_u64(streaming_budget, "max_visible_terrain_mesh_count"),
        MAX_STREAMING_BUDGET_VISIBLE_TERRAIN_MESHES,
        "meshes",
    ));
    checks.push(check_at_most_u64(
        "streaming_budget_visible_impostor_mesh_count",
        value_u64(streaming_budget, "max_visible_impostor_mesh_count"),
        island_count,
        "meshes",
    ));
    checks.push(check_at_most_u64(
        "streaming_budget_terrain_collision_proxy_count",
        value_u64(streaming_budget, "max_terrain_collision_proxy_count"),
        MAX_STREAMING_BUDGET_TERRAIN_COLLISION_PROXIES,
        "proxies",
    ));

    let minimums = manifest.get("minimums").unwrap_or(&Value::Null);
    checks.push(check_at_least_u64(
        "terrain_mesh_vertices",
        value_u64(minimums, "terrain_mesh_vertices"),
        MIN_TERRAIN_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "terrain_color_bands",
        value_u64(minimums, "terrain_color_bands"),
        MIN_TERRAIN_COLOR_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "terrain_material_weight_bands",
        value_u64(minimums, "terrain_material_weight_bands"),
        MIN_TERRAIN_MATERIAL_WEIGHT_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "terrain_material_channels",
        value_u64(minimums, "terrain_material_channels"),
        MIN_TERRAIN_MATERIAL_CHANNELS,
        "channels",
    ));
    checks.push(check_at_least_u64(
        "terrain_material_regions",
        value_u64(minimums, "terrain_material_regions"),
        MIN_TERRAIN_MATERIAL_REGIONS,
        "regions",
    ));
    checks.push(check_at_least_u64(
        "terrain_height_bands",
        value_u64(minimums, "terrain_height_bands"),
        MIN_TERRAIN_HEIGHT_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "terrain_normal_slope_bands",
        value_u64(minimums, "terrain_normal_slope_bands"),
        MIN_TERRAIN_NORMAL_SLOPE_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "terrain_texture_detail_bands",
        value_u64(minimums, "terrain_texture_detail_bands"),
        MIN_TERRAIN_TEXTURE_DETAIL_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "terrain_texture_edge_promille",
        value_u64(minimums, "terrain_texture_edge_promille"),
        MIN_TERRAIN_TEXTURE_EDGE_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_f64(
        "terrain_relief_range",
        value_f64(minimums, "terrain_relief_range_m"),
        MIN_TERRAIN_RELIEF_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "cliff_color_bands",
        value_u64(minimums, "cliff_color_bands"),
        MIN_CLIFF_COLOR_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "impostor_mesh_vertices",
        value_u64(minimums, "impostor_mesh_vertices"),
        MIN_ISLAND_IMPOSTOR_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "impostor_color_bands",
        value_u64(minimums, "impostor_color_bands"),
        MIN_ISLAND_IMPOSTOR_COLOR_BANDS,
        "bands",
    ));
    checks.push(check_eq_str(
        "collision_truth_schema",
        collision_truth
            .get("schema")
            .and_then(Value::as_str)
            .unwrap_or(""),
        "nau_terrain_collision_truth.v1",
        "schema",
    ));
    checks.push(check_eq_u64(
        "collision_truth_island_count",
        value_u64(collision_truth, "island_count"),
        island_count,
        "islands",
    ));
    checks.push(check_eq_u64(
        "collision_truth_contour_sample_count",
        value_u64(collision_truth, "contour_sample_count"),
        TERRAIN_COLLISION_TRUTH_CONTOUR_SAMPLES_PER_ISLAND,
        "samples/island",
    ));
    checks.push(check_eq_u64(
        "collision_truth_top_edge_probe_count",
        value_u64(collision_truth, "top_edge_probe_count"),
        expected_collision_truth_probe_count,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_top_edge_air_barrier_count",
        value_u64(collision_truth, "top_edge_air_barrier_count"),
        0,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_edge_traverse_probe_count",
        value_u64(collision_truth, "edge_traverse_probe_count"),
        expected_collision_truth_edge_traverse_probe_count,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_edge_traverse_barrier_count",
        value_u64(collision_truth, "edge_traverse_barrier_count"),
        0,
        "samples",
    ));
    checks.push(check_at_most_f64(
        "collision_truth_max_edge_traverse_push",
        value_f64(collision_truth, "max_edge_traverse_push_m"),
        MAX_TERRAIN_COLLISION_TRUTH_EDGE_TRAVERSE_PUSH_M,
        "m",
    ));
    checks.push(check_eq_u64(
        "collision_truth_walkoff_shoulder_probe_count",
        value_u64(collision_truth, "walkoff_shoulder_probe_count"),
        expected_collision_truth_walkoff_shoulder_probe_count,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_walkoff_shoulder_barrier_count",
        value_u64(collision_truth, "walkoff_shoulder_barrier_count"),
        0,
        "samples",
    ));
    checks.push(check_at_most_f64(
        "collision_truth_max_walkoff_shoulder_push",
        value_f64(collision_truth, "max_walkoff_shoulder_push_m"),
        MAX_TERRAIN_COLLISION_TRUTH_WALKOFF_SHOULDER_PUSH_M,
        "m",
    ));
    checks.push(check_eq_u64(
        "collision_truth_far_field_probe_count",
        value_u64(collision_truth, "far_field_probe_count"),
        expected_collision_truth_probe_count,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_far_field_hit_count",
        value_u64(collision_truth, "far_field_hit_count"),
        0,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_near_cliff_probe_count",
        value_u64(collision_truth, "near_cliff_probe_count"),
        expected_collision_truth_probe_count,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_near_cliff_miss_count",
        value_u64(collision_truth, "near_cliff_miss_count"),
        0,
        "samples",
    ));
    checks.push(check_eq_u64(
        "collision_truth_excessive_near_cliff_push_count",
        value_u64(collision_truth, "excessive_near_cliff_push_count"),
        0,
        "samples",
    ));
    checks.push(check_at_most_f64(
        "collision_truth_max_near_cliff_push",
        value_f64(collision_truth, "max_near_cliff_push_m"),
        MAX_TERRAIN_COLLISION_TRUTH_NEAR_CLIFF_PUSH_M,
        "m",
    ));
    checks.push(check_eq_str(
        "visual_collision_coverage_schema",
        visual_collision_coverage
            .get("schema")
            .and_then(Value::as_str)
            .unwrap_or(""),
        "nau_visual_collision_coverage.v2",
        "schema",
    ));
    checks.push(check_eq_bool(
        "visual_collision_coverage_passed",
        value_bool(visual_collision_coverage, "passed"),
        true,
        "bool",
    ));
    checks.push(check_eq_u64(
        "visual_collision_coverage_failure_count",
        value_u64(visual_collision_coverage, "failure_count"),
        0,
        "failures",
    ));
    checks.push(check_at_least_u64(
        "visual_collision_solid_visual_count",
        value_u64(visual_collision_coverage, "solid_visual_count"),
        MIN_VISUAL_COLLISION_SOLID_VISUAL_COUNT,
        "visuals",
    ));
    checks.push(check_at_least_u64(
        "visual_collision_surface_supported_solid_proxy_count",
        value_u64(
            visual_collision_coverage,
            "surface_supported_solid_proxy_count",
        ),
        MIN_VISUAL_COLLISION_SURFACE_SUPPORTED_SOLID_PROXY_COUNT,
        "proxies",
    ));
    checks.push(check_eq_u64(
        "visual_collision_footprint_bounded_solid_proxy_count",
        value_u64(
            visual_collision_coverage,
            "footprint_bounded_solid_proxy_count",
        ),
        value_u64(
            visual_collision_coverage,
            "surface_supported_solid_proxy_count",
        ),
        "proxies",
    ));
    checks.push(check_at_least_f64(
        "visual_collision_min_solid_proxy_edge_clearance_m",
        value_f64(
            visual_collision_coverage,
            "min_solid_proxy_edge_clearance_m",
        ),
        MIN_VISUAL_COLLISION_SOLID_PROXY_EDGE_CLEARANCE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "visual_collision_tree_solid_proxy_count",
        value_u64(visual_collision_coverage, "tree_solid_proxy_count"),
        MIN_VISUAL_COLLISION_TREE_SOLID_PROXY_COUNT,
        "proxies",
    ));
    checks.push(check_eq_u64(
        "visual_collision_tree_footprint_bounded_proxy_count",
        value_u64(
            visual_collision_coverage,
            "tree_footprint_bounded_proxy_count",
        ),
        value_u64(visual_collision_coverage, "tree_solid_proxy_count"),
        "proxies",
    ));
    checks.push(check_at_least_u64(
        "visual_collision_rock_solid_proxy_count",
        value_u64(visual_collision_coverage, "rock_solid_proxy_count"),
        MIN_VISUAL_COLLISION_ROCK_SOLID_PROXY_COUNT,
        "proxies",
    ));
    checks.push(check_eq_u64(
        "visual_collision_rock_footprint_bounded_proxy_count",
        value_u64(
            visual_collision_coverage,
            "rock_footprint_bounded_proxy_count",
        ),
        value_u64(visual_collision_coverage, "rock_solid_proxy_count"),
        "proxies",
    ));
    checks.push(check_at_least_u64(
        "visual_collision_landmark_solid_proxy_count",
        value_u64(visual_collision_coverage, "landmark_solid_proxy_count"),
        MIN_VISUAL_COLLISION_LANDMARK_SOLID_PROXY_COUNT,
        "proxies",
    ));
    checks.push(check_eq_u64(
        "visual_collision_landmark_footprint_bounded_proxy_count",
        value_u64(
            visual_collision_coverage,
            "landmark_footprint_bounded_proxy_count",
        ),
        value_u64(visual_collision_coverage, "landmark_solid_proxy_count"),
        "proxies",
    ));
    checks.push(check_eq_u64(
        "visual_collision_terrain_body_proxy_count",
        value_u64(visual_collision_coverage, "terrain_body_proxy_count"),
        expected_visual_terrain_body_proxy_count,
        "proxies",
    ));
    checks.push(check_eq_u64(
        "visual_collision_terrain_rim_proxy_count",
        value_u64(visual_collision_coverage, "terrain_rim_proxy_count"),
        expected_visual_terrain_rim_proxy_count,
        "proxies",
    ));
    checks.push(check_at_least_u64(
        "visual_collision_camera_only_allowance_count",
        value_u64(visual_collision_coverage, "camera_only_allowance_count"),
        island_count,
        "visuals",
    ));
    checks.push(check_at_least_u64(
        "visual_collision_non_blocking_visual_count",
        value_u64(visual_collision_coverage, "non_blocking_visual_count"),
        MIN_VISUAL_COLLISION_NON_BLOCKING_VISUAL_COUNT,
        "visuals",
    ));
    checks.push(check_eq_str(
        "terrain_seam_coverage_schema",
        seam_coverage
            .get("schema")
            .and_then(Value::as_str)
            .unwrap_or(""),
        "nau_terrain_seam_coverage.v1",
        "schema",
    ));
    checks.push(check_eq_u64(
        "terrain_seam_coverage_island_count",
        value_u64(seam_coverage, "island_count"),
        island_count,
        "islands",
    ));
    checks.push(check_at_most_f64(
        "terrain_cliff_top_seam_gap",
        value_f64(seam_coverage, "max_terrain_cliff_top_gap_m"),
        MAX_TERRAIN_CLIFF_TOP_SEAM_GAP_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "terrain_edge_skirt_depth",
        value_f64(seam_coverage, "min_terrain_edge_skirt_depth_m"),
        MIN_TERRAIN_EDGE_SKIRT_DEPTH_M,
        "m",
    ));
    checks.push(check_at_most_f64(
        "terrain_edge_skirt_horizontal_gap",
        value_f64(seam_coverage, "max_terrain_edge_skirt_horizontal_gap_m"),
        MAX_TERRAIN_EDGE_SKIRT_HORIZONTAL_GAP_M,
        "m",
    ));
    checks.push(check_eq_u64(
        "terrain_shape_review_representative_count",
        value_u64(terrain_shape_review, "representative_count"),
        shape_language_count,
        "tiles",
    ));
    checks.push(check_eq_u64(
        "terrain_shape_review_shape_language_coverage",
        value_u64(terrain_shape_review, "covered_shape_language_count"),
        shape_language_count,
        "shape_languages",
    ));
    checks.push(check_at_least_u64(
        "terrain_shape_review_archetype_coverage",
        value_u64(terrain_shape_review, "covered_terrain_archetype_count"),
        MIN_TERRAIN_SHAPE_REVIEW_ARCHETYPE_COVERAGE,
        "archetypes",
    ));
    checks.push(check_at_least_u64(
        "terrain_shape_review_projection_pixels",
        value_u64(terrain_shape_review, "min_projection_pixel_count"),
        MIN_TERRAIN_SHAPE_REVIEW_PROJECTION_PIXELS,
        "pixels",
    ));
    checks.push(check_at_least_u64(
        "terrain_shape_review_horizontal_span",
        value_u64(terrain_shape_review, "min_projection_horizontal_span_px"),
        MIN_TERRAIN_SHAPE_REVIEW_HORIZONTAL_SPAN_PX,
        "px",
    ));
    checks.push(check_at_least_u64(
        "terrain_shape_review_vertical_span",
        value_u64(terrain_shape_review, "min_projection_vertical_span_px"),
        MIN_TERRAIN_SHAPE_REVIEW_VERTICAL_SPAN_PX,
        "px",
    ));
    checks.push(check_at_most_f64(
        "terrain_shape_review_seam_gap",
        value_f64(
            terrain_shape_review,
            "max_representative_terrain_cliff_top_gap_m",
        ),
        MAX_TERRAIN_CLIFF_TOP_SEAM_GAP_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "terrain_shape_review_skirt_depth",
        value_f64(
            terrain_shape_review,
            "min_representative_terrain_edge_skirt_depth_m",
        ),
        MIN_TERRAIN_EDGE_SKIRT_DEPTH_M,
        "m",
    ));
    checks.push(check_at_most_f64(
        "terrain_shape_review_skirt_horizontal_gap",
        value_f64(
            terrain_shape_review,
            "max_representative_terrain_edge_skirt_horizontal_gap_m",
        ),
        MAX_TERRAIN_EDGE_SKIRT_HORIZONTAL_GAP_M,
        "m",
    ));

    expected_artifact_count += 1;
    if let Some(contact_sheet_path) = relative_path(terrain_shape_review, "contact_sheet") {
        let full_contact_sheet_path = root_dir.join(&contact_sheet_path);
        if full_contact_sheet_path.is_file() {
            found_artifact_count += 1;
            artifacts.push(json!({
                "island": "terrain",
                "kind": "terrain_shape_review",
                "image": contact_sheet_path.to_string_lossy(),
                "representative_count": value_u64(terrain_shape_review, "representative_count"),
                "min_projection_pixel_count": value_u64(terrain_shape_review, "min_projection_pixel_count"),
                "min_projection_horizontal_span_px": value_u64(terrain_shape_review, "min_projection_horizontal_span_px"),
                "min_projection_vertical_span_px": value_u64(terrain_shape_review, "min_projection_vertical_span_px"),
            }));
        } else {
            missing_artifact_count += 1;
            artifacts.push(error_artifact(
                "terrain",
                "terrain_shape_review",
                "missing contact sheet file",
            ));
        }
    } else {
        missing_artifact_count += 1;
        artifacts.push(error_artifact(
            "terrain",
            "terrain_shape_review",
            "missing contact sheet path",
        ));
    }

    let mut terrain_archetype_mask = 0_u64;
    let mut shape_language_mask = 0_u64;
    for island in &islands {
        let island_name = island.get("name").and_then(Value::as_str).unwrap_or("");
        if let Some(archetype_index) = island
            .get("terrain_archetype_index")
            .and_then(Value::as_u64)
            && archetype_index < u64::BITS as u64
        {
            terrain_archetype_mask |= 1_u64 << archetype_index;
        }
        if let Some(shape_language_index) =
            island.get("shape_language_index").and_then(Value::as_u64)
            && shape_language_index < u64::BITS as u64
        {
            shape_language_mask |= 1_u64 << shape_language_index;
        }
        let shape_signature = island.get("shape_signature").unwrap_or(&Value::Null);
        min_shape_silhouette_range =
            min_shape_silhouette_range.min(value_f64(shape_signature, "silhouette_range"));
        min_shape_mid_relief_range_m =
            min_shape_mid_relief_range_m.min(value_f64(shape_signature, "mid_relief_range_m"));
        min_shape_edge_relief_range_m =
            min_shape_edge_relief_range_m.min(value_f64(shape_signature, "edge_relief_range_m"));
        min_shape_radial_reversal_count = min_shape_radial_reversal_count
            .min(value_u64(shape_signature, "radial_reversal_count"));
        let seam = island.get("seam").unwrap_or(&Value::Null);
        max_island_terrain_cliff_top_gap_m =
            max_island_terrain_cliff_top_gap_m.max(value_f64(seam, "max_terrain_cliff_top_gap_m"));
        min_island_terrain_edge_skirt_depth_m = min_island_terrain_edge_skirt_depth_m
            .min(value_f64(seam, "min_terrain_edge_skirt_depth_m"));
        max_island_terrain_edge_skirt_horizontal_gap_m =
            max_island_terrain_edge_skirt_horizontal_gap_m
                .max(value_f64(seam, "max_terrain_edge_skirt_horizontal_gap_m"));
        for mesh_kind in ["terrain", "cliff", "underside", "impostor"] {
            let mesh = island.get(mesh_kind).unwrap_or(&Value::Null);
            expected_artifact_count += 1;
            let Some(obj_path) = relative_path(mesh, "obj") else {
                missing_artifact_count += 1;
                artifacts.push(error_artifact(island_name, mesh_kind, "missing obj path"));
                continue;
            };
            let full_obj_path = root_dir.join(&obj_path);
            let manifest_vertices = value_u64(mesh, "vertex_count");
            let manifest_triangles = value_u64(mesh, "triangle_count");

            match audit_obj_path(&full_obj_path) {
                Ok(obj) => {
                    found_artifact_count += 1;
                    let vertex_mismatch = obj.vertex_count != manifest_vertices;
                    let face_mismatch = obj.face_count != manifest_triangles;
                    let color_mismatch = obj.colored_vertex_count != obj.vertex_count;
                    obj_vertex_mismatch_count += u64::from(vertex_mismatch);
                    obj_face_mismatch_count += u64::from(face_mismatch);
                    obj_color_mismatch_count += u64::from(color_mismatch);
                    let mut height_band_mismatch = false;
                    let mut normal_slope_band_mismatch = false;
                    if mesh_kind == "terrain" {
                        height_band_mismatch =
                            obj.vertical_band_count != value_u64(mesh, "height_bands");
                        normal_slope_band_mismatch =
                            obj.normal_slope_band_count != value_u64(mesh, "normal_slope_bands");
                        obj_height_band_mismatch_count += u64::from(height_band_mismatch);
                        obj_normal_slope_band_mismatch_count +=
                            u64::from(normal_slope_band_mismatch);
                        min_terrain_silhouette_radius_bands =
                            min_terrain_silhouette_radius_bands.min(obj.silhouette_radius_bands);
                        min_terrain_vertical_bands =
                            min_terrain_vertical_bands.min(obj.vertical_band_count);
                        min_terrain_normal_slope_bands =
                            min_terrain_normal_slope_bands.min(obj.normal_slope_band_count);
                    }
                    if mesh_kind == "cliff" || mesh_kind == "underside" {
                        min_island_body_vertical_range_m =
                            min_island_body_vertical_range_m.min(obj.vertical_range_m);
                        min_island_body_silhouette_radius_bands =
                            min_island_body_silhouette_radius_bands
                                .min(obj.silhouette_radius_bands);
                    }
                    if mesh_kind == "impostor" {
                        min_impostor_vertical_range_m =
                            min_impostor_vertical_range_m.min(obj.vertical_range_m);
                        min_impostor_horizontal_radius_bands =
                            min_impostor_horizontal_radius_bands.min(obj.horizontal_radius_bands);
                    }

                    artifacts.push(json!({
                        "island": island_name,
                        "kind": mesh_kind,
                        "obj": obj_path.to_string_lossy(),
                        "vertex_count": obj.vertex_count,
                        "triangle_count": obj.face_count,
                        "colored_vertex_count": obj.colored_vertex_count,
                        "vertical_range_m": obj.vertical_range_m,
                        "vertical_band_count": obj.vertical_band_count,
                        "normal_slope_band_count": obj.normal_slope_band_count,
                        "horizontal_radius_bands": obj.horizontal_radius_bands,
                        "silhouette_radius_bands": obj.silhouette_radius_bands,
                        "vertex_count_matches_manifest": !vertex_mismatch,
                        "triangle_count_matches_manifest": !face_mismatch,
                        "all_vertices_have_color": !color_mismatch,
                        "height_bands_match_manifest": !height_band_mismatch,
                        "normal_slope_bands_match_manifest": !normal_slope_band_mismatch,
                    }));
                }
                Err(error) => {
                    missing_artifact_count += 1;
                    artifacts.push(error_artifact(island_name, mesh_kind, &error));
                }
            }

            if mesh_kind == "terrain" {
                expected_artifact_count += 1;
                let Some(csv_path) = relative_path(mesh, "material_weights_csv") else {
                    missing_artifact_count += 1;
                    artifacts.push(error_artifact(
                        island_name,
                        "terrain_material_weights",
                        "missing material weight csv path",
                    ));
                    continue;
                };
                let full_csv_path = root_dir.join(&csv_path);
                let material_region_row_limit = match value_u64(mesh, "surface_vertex_count") {
                    0 => manifest_vertices,
                    surface_vertices => surface_vertices.min(manifest_vertices),
                };
                match audit_weight_csv_path(&full_csv_path, Some(material_region_row_limit)) {
                    Ok(csv) => {
                        found_artifact_count += 1;
                        let expected_rows = manifest_vertices;
                        let expected_bands = value_u64(mesh, "material_weight_bands");
                        let expected_channels = value_u64(mesh, "material_channels");
                        let expected_regions = value_u64(mesh, "material_regions");
                        let row_mismatch = csv.row_count != expected_rows;
                        let band_mismatch = csv.material_weight_bands != expected_bands;
                        let channel_mismatch = csv.material_channels != expected_channels;
                        let region_mismatch = csv.material_regions != expected_regions;
                        csv_row_mismatch_count += u64::from(row_mismatch);
                        csv_band_mismatch_count += u64::from(band_mismatch);
                        csv_channel_mismatch_count += u64::from(channel_mismatch);
                        csv_region_mismatch_count += u64::from(region_mismatch);
                        aggregate_region_row_count += csv.region_row_count;
                        for (index, promille) in csv.region_promille.iter().enumerate() {
                            min_region_promille[index] = min_region_promille[index].min(*promille);
                            aggregate_region_promille_sums[index] +=
                                promille * csv.region_row_count;
                        }

                        artifacts.push(json!({
                            "island": island_name,
                            "kind": "terrain_material_weights",
                            "csv": csv_path.to_string_lossy(),
                            "row_count": csv.row_count,
                            "region_row_count": csv.region_row_count,
                            "material_weight_bands": csv.material_weight_bands,
                            "material_channels": csv.material_channels,
                            "material_regions": csv.material_regions,
                            "region_promille": {
                                "base": csv.region_promille[0],
                                "transition": csv.region_promille[1],
                                "highland": csv.region_promille[2],
                                "exposed": csv.region_promille[3]
                            },
                            "row_count_matches_manifest": !row_mismatch,
                            "material_weight_bands_match_manifest": !band_mismatch,
                            "material_channels_match_manifest": !channel_mismatch,
                            "material_regions_match_manifest": !region_mismatch,
                        }));
                    }
                    Err(error) => {
                        missing_artifact_count += 1;
                        artifacts.push(error_artifact(
                            island_name,
                            "terrain_material_weights",
                            &error,
                        ));
                    }
                }
            }
        }
    }

    checks.push(check_eq_u64(
        "artifact_file_count",
        found_artifact_count,
        expected_artifact_count,
        "files",
    ));
    checks.push(check_eq_u64(
        "missing_artifact_count",
        missing_artifact_count,
        0,
        "files",
    ));
    checks.push(check_eq_u64(
        "obj_vertex_mismatch_count",
        obj_vertex_mismatch_count,
        0,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_face_mismatch_count",
        obj_face_mismatch_count,
        0,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_color_mismatch_count",
        obj_color_mismatch_count,
        0,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_height_band_mismatch_count",
        obj_height_band_mismatch_count,
        0,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_normal_slope_band_mismatch_count",
        obj_normal_slope_band_mismatch_count,
        0,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "terrain_weight_csv_row_mismatch_count",
        csv_row_mismatch_count,
        0,
        "csvs",
    ));
    checks.push(check_eq_u64(
        "terrain_weight_csv_band_mismatch_count",
        csv_band_mismatch_count,
        0,
        "csvs",
    ));
    checks.push(check_eq_u64(
        "terrain_weight_csv_channel_mismatch_count",
        csv_channel_mismatch_count,
        0,
        "csvs",
    ));
    checks.push(check_eq_u64(
        "terrain_weight_csv_region_mismatch_count",
        csv_region_mismatch_count,
        0,
        "csvs",
    ));
    checks.push(check_eq_u64(
        "terrain_archetype_entry_count",
        terrain_archetype_mask.count_ones() as u64,
        terrain_archetype_count,
        "archetypes",
    ));
    checks.push(check_eq_u64(
        "terrain_shape_language_entry_count",
        shape_language_mask.count_ones() as u64,
        shape_language_count,
        "shape_languages",
    ));
    if !min_shape_silhouette_range.is_finite() {
        min_shape_silhouette_range = 0.0;
    }
    if !min_shape_mid_relief_range_m.is_finite() {
        min_shape_mid_relief_range_m = 0.0;
    }
    if !min_shape_edge_relief_range_m.is_finite() {
        min_shape_edge_relief_range_m = 0.0;
    }
    let min_shape_radial_reversal_count = if min_shape_radial_reversal_count == u64::MAX {
        0
    } else {
        min_shape_radial_reversal_count
    };
    checks.push(check_at_least_f64(
        "terrain_shape_silhouette_range",
        min_shape_silhouette_range,
        MIN_TERRAIN_SHAPE_SILHOUETTE_RANGE,
        "scale",
    ));
    checks.push(check_at_least_f64(
        "terrain_shape_mid_relief_range",
        min_shape_mid_relief_range_m,
        MIN_TERRAIN_SHAPE_MID_RELIEF_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "terrain_shape_edge_relief_range",
        min_shape_edge_relief_range_m,
        MIN_TERRAIN_SHAPE_EDGE_RELIEF_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "terrain_shape_radial_reversal_count",
        min_shape_radial_reversal_count,
        MIN_TERRAIN_SHAPE_RADIAL_REVERSAL_COUNT,
        "reversals",
    ));
    if !min_island_terrain_edge_skirt_depth_m.is_finite() {
        min_island_terrain_edge_skirt_depth_m = 0.0;
    }
    checks.push(check_at_most_f64(
        "island_terrain_cliff_top_seam_gap",
        max_island_terrain_cliff_top_gap_m,
        MAX_TERRAIN_CLIFF_TOP_SEAM_GAP_M,
        "m",
    ));
    checks.push(check_at_least_f64(
        "island_terrain_edge_skirt_depth",
        min_island_terrain_edge_skirt_depth_m,
        MIN_TERRAIN_EDGE_SKIRT_DEPTH_M,
        "m",
    ));
    checks.push(check_at_most_f64(
        "island_terrain_edge_skirt_horizontal_gap",
        max_island_terrain_edge_skirt_horizontal_gap_m,
        MAX_TERRAIN_EDGE_SKIRT_HORIZONTAL_GAP_M,
        "m",
    ));
    let min_region_promille =
        min_region_promille.map(|value| if value == u64::MAX { 0 } else { value });
    let aggregate_region_promille = if aggregate_region_row_count == 0 {
        [0; TERRAIN_MATERIAL_REGION_COUNT]
    } else {
        aggregate_region_promille_sums.map(|sum| sum / aggregate_region_row_count)
    };
    checks.push(check_at_least_u64(
        "terrain_base_region_promille",
        min_region_promille[0],
        MIN_TERRAIN_BASE_REGION_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_u64(
        "terrain_transition_region_promille",
        min_region_promille[1],
        MIN_TERRAIN_TRANSITION_REGION_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_u64(
        "terrain_highland_region_promille",
        min_region_promille[2],
        MIN_TERRAIN_HIGHLAND_REGION_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_u64(
        "terrain_exposed_region_promille",
        min_region_promille[3],
        MIN_TERRAIN_EXPOSED_REGION_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_u64(
        "terrain_aggregate_base_region_promille",
        aggregate_region_promille[0],
        MIN_TERRAIN_AGGREGATE_BASE_REGION_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_u64(
        "terrain_aggregate_transition_region_promille",
        aggregate_region_promille[1],
        MIN_TERRAIN_AGGREGATE_TRANSITION_REGION_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_u64(
        "terrain_aggregate_highland_region_promille",
        aggregate_region_promille[2],
        MIN_TERRAIN_AGGREGATE_HIGHLAND_REGION_PROMILLE,
        "promille",
    ));
    checks.push(check_at_least_u64(
        "terrain_aggregate_exposed_region_promille",
        aggregate_region_promille[3],
        MIN_TERRAIN_AGGREGATE_EXPOSED_REGION_PROMILLE,
        "promille",
    ));
    let min_terrain_silhouette_radius_bands = if min_terrain_silhouette_radius_bands == u64::MAX {
        0
    } else {
        min_terrain_silhouette_radius_bands
    };
    if !min_island_body_vertical_range_m.is_finite() {
        min_island_body_vertical_range_m = 0.0;
    }
    let min_island_body_silhouette_radius_bands =
        if min_island_body_silhouette_radius_bands == u64::MAX {
            0
        } else {
            min_island_body_silhouette_radius_bands
        };
    checks.push(check_at_least_u64(
        "terrain_silhouette_radius_bands",
        min_terrain_silhouette_radius_bands,
        MIN_TERRAIN_SILHOUETTE_RADIUS_BANDS,
        "bands",
    ));
    let min_terrain_vertical_bands = if min_terrain_vertical_bands == u64::MAX {
        0
    } else {
        min_terrain_vertical_bands
    };
    let min_terrain_normal_slope_bands = if min_terrain_normal_slope_bands == u64::MAX {
        0
    } else {
        min_terrain_normal_slope_bands
    };
    checks.push(check_at_least_u64(
        "terrain_obj_height_bands",
        min_terrain_vertical_bands,
        MIN_TERRAIN_HEIGHT_BANDS,
        "bands",
    ));
    checks.push(check_at_least_u64(
        "terrain_obj_normal_slope_bands",
        min_terrain_normal_slope_bands,
        MIN_TERRAIN_NORMAL_SLOPE_BANDS,
        "bands",
    ));
    checks.push(check_at_least_f64(
        "island_body_vertical_range",
        min_island_body_vertical_range_m,
        MIN_ISLAND_BODY_VERTICAL_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "island_body_silhouette_radius_bands",
        min_island_body_silhouette_radius_bands,
        MIN_ISLAND_BODY_SILHOUETTE_RADIUS_BANDS,
        "bands",
    ));
    if !min_impostor_vertical_range_m.is_finite() {
        min_impostor_vertical_range_m = 0.0;
    }
    let min_impostor_horizontal_radius_bands = if min_impostor_horizontal_radius_bands == u64::MAX {
        0
    } else {
        min_impostor_horizontal_radius_bands
    };
    checks.push(check_at_least_f64(
        "impostor_vertical_range",
        min_impostor_vertical_range_m,
        MIN_ISLAND_IMPOSTOR_VERTICAL_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "impostor_horizontal_radius_bands",
        min_impostor_horizontal_radius_bands,
        MIN_ISLAND_IMPOSTOR_HORIZONTAL_RADIUS_BANDS,
        "bands",
    ));

    let passed = checks.iter().all(|check| {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });

    json!({
        "passed": passed,
        "manifest": manifest_path,
        "checks": checks,
        "artifacts": artifacts,
    })
}
