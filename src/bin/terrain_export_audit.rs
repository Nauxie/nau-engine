use serde_json::{Value, json};
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    process,
};

const MIN_ISLAND_COUNT: u64 = 12;
const MIN_TERRAIN_MESH_VERTICES: u64 = 2305;
const MIN_TERRAIN_COLOR_BANDS: u64 = 32;
const MIN_TERRAIN_MATERIAL_WEIGHT_BANDS: u64 = 24;
const MIN_TERRAIN_MATERIAL_CHANNELS: u64 = 3;
const MIN_TERRAIN_MATERIAL_REGIONS: u64 = 4;
const MIN_TERRAIN_TEXTURE_DETAIL_BANDS: u64 = 44;
const MIN_TERRAIN_BASE_REGION_PROMILLE: u64 = 180;
const MIN_TERRAIN_TRANSITION_REGION_PROMILLE: u64 = 350;
const MIN_TERRAIN_HIGHLAND_REGION_PROMILLE: u64 = 180;
const MIN_TERRAIN_EXPOSED_REGION_PROMILLE: u64 = 150;
const MIN_TERRAIN_RELIEF_RANGE_M: f64 = 0.8;
const MIN_CLIFF_COLOR_BANDS: u64 = 9;
const MIN_ISLAND_IMPOSTOR_MESH_VERTICES: u64 = 140;
const MIN_ISLAND_IMPOSTOR_COLOR_BANDS: u64 = 18;

fn main() {
    let args = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("Usage: cargo run --bin terrain_export_audit -- <manifest.json>");
        process::exit(2);
    }

    match audit_manifest_path(&args[0]) {
        Ok(report) => {
            let passed = report
                .get("passed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("audit report should serialize")
            );
            if !passed {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("terrain export audit failed: {error}");
            process::exit(2);
        }
    }
}

fn audit_manifest_path(path: &Path) -> Result<Value, String> {
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

fn audit_manifest(manifest: &Value, root_dir: &Path, manifest_path: &str) -> Value {
    let mut checks = Vec::new();
    let mut artifacts = Vec::new();
    let mut expected_artifact_count = 0u64;
    let mut found_artifact_count = 0u64;
    let mut missing_artifact_count = 0u64;
    let mut obj_vertex_mismatch_count = 0u64;
    let mut obj_face_mismatch_count = 0u64;
    let mut obj_color_mismatch_count = 0u64;
    let mut csv_row_mismatch_count = 0u64;
    let mut csv_band_mismatch_count = 0u64;
    let mut csv_channel_mismatch_count = 0u64;
    let mut csv_region_mismatch_count = 0u64;
    let mut min_region_promille = [u64::MAX; TERRAIN_MATERIAL_REGION_COUNT];

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
        "terrain_texture_detail_bands",
        value_u64(minimums, "terrain_texture_detail_bands"),
        MIN_TERRAIN_TEXTURE_DETAIL_BANDS,
        "bands",
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

                    artifacts.push(json!({
                        "island": island_name,
                        "kind": mesh_kind,
                        "obj": obj_path.to_string_lossy(),
                        "vertex_count": obj.vertex_count,
                        "triangle_count": obj.face_count,
                        "colored_vertex_count": obj.colored_vertex_count,
                        "vertex_count_matches_manifest": !vertex_mismatch,
                        "triangle_count_matches_manifest": !face_mismatch,
                        "all_vertices_have_color": !color_mismatch,
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

#[derive(Debug, PartialEq, Eq)]
struct ObjAudit {
    vertex_count: u64,
    face_count: u64,
    colored_vertex_count: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct WeightCsvAudit {
    row_count: u64,
    material_weight_bands: u64,
    material_channels: u64,
    material_regions: u64,
    region_promille: [u64; TERRAIN_MATERIAL_REGION_COUNT],
}

fn audit_obj_path(path: &Path) -> Result<ObjAudit, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(audit_obj_text(&text))
}

fn audit_obj_text(text: &str) -> ObjAudit {
    let mut vertex_count = 0;
    let mut face_count = 0;
    let mut colored_vertex_count = 0;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("v ") {
            vertex_count += 1;
            if rest.split_whitespace().count() >= 6 {
                colored_vertex_count += 1;
            }
        } else if line.starts_with("f ") {
            face_count += 1;
        }
    }

    ObjAudit {
        vertex_count,
        face_count,
        colored_vertex_count,
    }
}

fn audit_weight_csv_path(path: &Path) -> Result<WeightCsvAudit, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    audit_weight_csv_text(&text)
}

fn audit_weight_csv_text(text: &str) -> Result<WeightCsvAudit, String> {
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| "empty material weights csv".to_string())?;
    if header != "vertex,lush_highland,exposed_edge" {
        return Err(format!("unexpected material weights csv header: {header}"));
    }

    let mut row_count = 0;
    let mut bands = HashSet::new();
    let mut base = false;
    let mut lush = false;
    let mut exposed = false;
    let mut regions = HashSet::new();
    let mut region_counts = [0u64; TERRAIN_MATERIAL_REGION_COUNT];

    for line in lines {
        let columns = line.split(',').collect::<Vec<_>>();
        if columns.len() != 3 {
            return Err(format!("invalid material weights csv row: {line}"));
        }
        let lush_highland = columns[1]
            .parse::<f32>()
            .map_err(|error| format!("invalid lush/highland weight: {error}"))?
            .clamp(0.0, 1.0);
        let exposed_edge = columns[2]
            .parse::<f32>()
            .map_err(|error| format!("invalid exposed-edge weight: {error}"))?
            .clamp(0.0, 1.0);

        bands.insert([
            (lush_highland * 15.0).round() as u8,
            (exposed_edge * 15.0).round() as u8,
        ]);
        let region = terrain_material_region_id(lush_highland, exposed_edge);
        regions.insert(region);
        region_counts[region as usize] += 1;
        base |= lush_highland < 0.18 && exposed_edge < 0.18;
        lush |= lush_highland > 0.18;
        exposed |= exposed_edge > 0.18;
        row_count += 1;
    }

    let region_promille = if row_count == 0 {
        [0; TERRAIN_MATERIAL_REGION_COUNT]
    } else {
        region_counts.map(|count| count * 1000 / row_count)
    };

    Ok(WeightCsvAudit {
        row_count,
        material_weight_bands: bands.len() as u64,
        material_channels: u64::from(base) + u64::from(lush) + u64::from(exposed),
        material_regions: regions.len() as u64,
        region_promille,
    })
}

const TERRAIN_MATERIAL_REGION_COUNT: usize = 4;

fn terrain_material_region_id(lush_highland: f32, exposed_edge: f32) -> u8 {
    if exposed_edge >= 0.48 {
        3
    } else if lush_highland >= 0.42 {
        2
    } else if lush_highland >= 0.24 || exposed_edge >= 0.10 {
        1
    } else {
        0
    }
}

fn value_u64(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn value_f64(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn relative_path(value: &Value, key: &str) -> Option<PathBuf> {
    value.get(key).and_then(Value::as_str).map(PathBuf::from)
}

fn error_artifact(island: &str, kind: &str, error: &str) -> Value {
    json!({
        "island": island,
        "kind": kind,
        "error": error,
    })
}

fn check_at_least_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_least_f64(name: &str, value: f64, threshold: f64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_eq_u64(name: &str, value: u64, threshold: u64, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_eq_str(name: &str, value: &str, threshold: &str, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn obj_audit_counts_vertices_faces_and_vertex_colors() {
        let audit = audit_obj_text(
            "# sample\n\
             v 0.0 0.0 0.0 0.1 0.2 0.3\n\
             v 1.0 0.0 0.0 0.1 0.2 0.3\n\
             v 0.0 1.0 0.0\n\
             vn 0.0 1.0 0.0\n\
             f 1//1 2//1 3//1\n",
        );

        assert_eq!(
            audit,
            ObjAudit {
                vertex_count: 3,
                face_count: 1,
                colored_vertex_count: 2,
            }
        );
    }

    #[test]
    fn audit_manifest_requires_impostor_entries_and_minimums() {
        let manifest = json!({
            "schema": "nau_terrain_export.v1",
            "island_count": 1,
            "mesh_count": 3,
            "total_vertex_count": 2305,
            "total_triangle_count": 4000,
            "minimums": {
                "terrain_mesh_vertices": 2305,
                "terrain_color_bands": 32,
                "terrain_material_weight_bands": 24,
                "terrain_material_channels": 3,
                "terrain_material_regions": 4,
                "terrain_texture_detail_bands": 44,
                "terrain_relief_range_m": 0.8,
                "cliff_color_bands": 9,
                "impostor_mesh_vertices": 42,
                "impostor_color_bands": 4
            },
            "islands": [{
                "name": "launch mesa",
                "terrain": {
                    "obj": "missing_terrain.obj",
                    "material_weights_csv": "missing_weights.csv",
                    "vertex_count": 2305,
                    "triangle_count": 4000,
                    "material_weight_bands": 24,
                    "material_channels": 3,
                    "material_regions": 4
                },
                "cliff": {
                    "obj": "missing_cliff.obj",
                    "vertex_count": 96,
                    "triangle_count": 180
                },
                "underside": {
                    "obj": "missing_underside.obj",
                    "vertex_count": 96,
                    "triangle_count": 180
                }
            }]
        });
        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");

        assert!(
            !report
                .get("passed")
                .and_then(Value::as_bool)
                .unwrap_or(true)
        );
        assert!(!audit_check_passed(&report, "mesh_count"));
        assert!(!audit_check_passed(&report, "impostor_mesh_vertices"));
        assert!(!audit_check_passed(&report, "impostor_color_bands"));
        assert!(
            report
                .get("artifacts")
                .and_then(Value::as_array)
                .expect("artifacts should be present")
                .iter()
                .any(
                    |artifact| artifact.get("kind").and_then(Value::as_str) == Some("impostor")
                        && artifact.get("error").and_then(Value::as_str)
                            == Some("missing obj path")
                )
        );
    }

    #[test]
    fn material_weight_csv_audit_counts_quantized_bands_and_channels() {
        let audit = audit_weight_csv_text(
            "vertex,lush_highland,exposed_edge\n\
             0,0.0000,0.0000\n\
             1,0.3000,0.0000\n\
             2,0.7000,0.0000\n\
             3,0.1000,0.8000\n\
             4,0.0000,0.0000\n\
             5,0.3000,0.0000\n\
             6,0.3000,0.0000\n\
             7,0.3000,0.0000\n\
             8,0.7000,0.0000\n\
             9,0.1000,0.8000\n",
        )
        .expect("csv should audit");

        assert_eq!(audit.row_count, 10);
        assert_eq!(audit.material_weight_bands, 4);
        assert_eq!(audit.material_channels, 3);
        assert_eq!(audit.material_regions, 4);
        assert_eq!(audit.region_promille, [200, 400, 200, 200]);
    }

    fn audit_check_passed(report: &Value, name: &str) -> bool {
        report
            .get("checks")
            .and_then(Value::as_array)
            .and_then(|checks| {
                checks
                    .iter()
                    .find(|check| check.get("name").and_then(Value::as_str) == Some(name))
            })
            .and_then(|check| check.get("passed").and_then(Value::as_bool))
            .unwrap_or(false)
    }
}
