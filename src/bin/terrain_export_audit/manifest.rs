use crate::{
    artifact::{audit_obj_path, audit_weight_csv_path},
    checks::{
        check_at_least_f64, check_at_least_u64, check_eq_str, check_eq_u64, error_artifact,
        relative_path, value_f64, value_u64,
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
    let mut min_terrain_silhouette_radius_bands = u64::MAX;
    let mut min_terrain_vertical_bands = u64::MAX;
    let mut min_terrain_normal_slope_bands = u64::MAX;
    let mut min_island_body_vertical_range_m = f64::INFINITY;
    let mut min_island_body_silhouette_radius_bands = u64::MAX;
    let mut min_impostor_vertical_range_m = f64::INFINITY;
    let mut min_impostor_horizontal_radius_bands = u64::MAX;

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

    for island in &islands {
        let island_name = island.get("name").and_then(Value::as_str).unwrap_or("");
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
                match audit_weight_csv_path(&full_csv_path) {
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
                        for (index, promille) in csv.region_promille.iter().enumerate() {
                            min_region_promille[index] = min_region_promille[index].min(*promille);
                        }

                        artifacts.push(json!({
                            "island": island_name,
                            "kind": "terrain_material_weights",
                            "csv": csv_path.to_string_lossy(),
                            "row_count": csv.row_count,
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
    let min_region_promille =
        min_region_promille.map(|value| if value == u64::MAX { 0 } else { value });
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
