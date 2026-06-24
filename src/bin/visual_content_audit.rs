use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const MIN_GROUND_COVER_COUNT: u64 = 12;
const MIN_GROUND_COVER_PATCH_TOTAL: u64 = 500;
const MIN_GROUND_COVER_BLADE_TOTAL: u64 = 2400;
const MIN_GROUND_COVER_MESH_VERTICES: u64 = 1000;
const MIN_GROUND_COVER_BLADE_COUNT: u64 = 200;
const MIN_GROUND_COVER_BLADE_HEIGHT_RANGE_M: f64 = 0.70;
const MIN_TREE_TRUNK_COUNT: u64 = 30;
const MIN_TREE_CANOPY_COUNT: u64 = 30;
const MIN_TREE_TRUNK_MESH_VERTICES: u64 = 60;
const MIN_TREE_TRUNK_TAPER_RATIO: f64 = 1.35;
const MIN_TREE_BRANCH_REACH_RATIO: f64 = 1.80;
const MIN_TREE_CANOPY_MESH_VERTICES: u64 = 400;
const MIN_TREE_CANOPY_LOBE_COUNT: u64 = 6;
const MIN_TREE_CANOPY_DETAIL_CARD_COUNT: u64 = 12;
const MIN_TREE_CANOPY_VERTICAL_TO_HORIZONTAL_RATIO: f64 = 0.45;
const MIN_WEATHER_CLOUD_COUNT: u64 = 24;
const MIN_WEATHER_CLOUD_BANK_COUNT: u64 = 12;
const MIN_WEATHER_CLOUD_MESH_VERTICES: u64 = 560;
const MIN_WEATHER_CLOUD_LOBE_COUNT: u64 = 6;
const MIN_WEATHER_CLOUD_WISP_CARD_COUNT: u64 = 14;
const MIN_WEATHER_CLOUD_BANK_DEPTH_M: f64 = 4.0;
const MIN_WEATHER_CLOUD_BANK_LOBE_COUNT: u64 = 10;
const MIN_TERRAIN_BIOME_PALETTE_COUNT: u64 = 5;
const MIN_FOLIAGE_PALETTE_COUNT: u64 = 5;
const MIN_STONE_PALETTE_COUNT: u64 = 4;

#[derive(Default)]
struct ArtifactCounters {
    expected: u64,
    found: u64,
    missing: u64,
    vertex_mismatches: u64,
    face_mismatches: u64,
}

#[derive(Debug)]
struct ObjAudit {
    vertex_count: u64,
    face_count: u64,
}

fn main() {
    let args = env::args().skip(1).map(PathBuf::from).collect::<Vec<_>>();
    if args.len() != 1 {
        eprintln!("Usage: cargo run --bin visual_content_audit -- <manifest.json>");
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
            eprintln!("visual content audit failed: {error}");
            process::exit(2);
        }
    }
}

fn audit_manifest_path(path: &Path) -> Result<Value, String> {
    let manifest_text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest = serde_json::from_str::<Value>(&manifest_text).map_err(|error| {
        format!(
            "could not parse visual content manifest {}: {error}",
            path.display()
        )
    })?;
    let root_dir = path.parent().unwrap_or_else(|| Path::new("."));

    Ok(audit_manifest(&manifest, root_dir, &path.to_string_lossy()))
}

fn audit_manifest(manifest: &Value, root_dir: &Path, manifest_path: &str) -> Value {
    let mut checks = Vec::new();
    let mut artifacts = Vec::new();
    let mut artifact_counters = ArtifactCounters::default();

    let schema = manifest.get("schema").and_then(Value::as_str).unwrap_or("");
    checks.push(check_eq_str(
        "schema",
        schema,
        "nau_visual_content_export.v1",
        "schema",
    ));

    let mesh_count = value_u64(manifest, "mesh_count");
    let total_vertex_count = value_u64(manifest, "total_vertex_count");
    let total_triangle_count = value_u64(manifest, "total_triangle_count");
    let counts = manifest.get("counts").unwrap_or(&Value::Null);
    let minimums = manifest.get("minimums").unwrap_or(&Value::Null);

    checks.push(check_at_least_u64(
        "ground_cover_count",
        value_u64(counts, "ground_cover_count"),
        MIN_GROUND_COVER_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_patch_total",
        value_u64(counts, "ground_cover_patch_total"),
        MIN_GROUND_COVER_PATCH_TOTAL,
        "patches",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_blade_total",
        value_u64(counts, "ground_cover_blade_total"),
        MIN_GROUND_COVER_BLADE_TOTAL,
        "blades",
    ));
    checks.push(check_at_least_u64(
        "tree_trunk_count",
        value_u64(counts, "tree_trunk_count"),
        MIN_TREE_TRUNK_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_count",
        value_u64(counts, "tree_canopy_count"),
        MIN_TREE_CANOPY_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_count",
        value_u64(counts, "weather_cloud_count"),
        MIN_WEATHER_CLOUD_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_bank_count",
        value_u64(counts, "weather_cloud_bank_count"),
        MIN_WEATHER_CLOUD_BANK_COUNT,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_mesh_vertices",
        value_u64(minimums, "ground_cover_mesh_vertices"),
        MIN_GROUND_COVER_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "ground_cover_blade_count",
        value_u64(minimums, "ground_cover_blade_count"),
        MIN_GROUND_COVER_BLADE_COUNT,
        "blades",
    ));
    checks.push(check_at_least_f64(
        "ground_cover_blade_height_range",
        value_f64(minimums, "ground_cover_blade_height_range_m"),
        MIN_GROUND_COVER_BLADE_HEIGHT_RANGE_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "tree_trunk_mesh_vertices",
        value_u64(minimums, "tree_trunk_mesh_vertices"),
        MIN_TREE_TRUNK_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_f64(
        "tree_trunk_taper_ratio",
        value_f64(minimums, "tree_trunk_taper_ratio"),
        MIN_TREE_TRUNK_TAPER_RATIO,
        "ratio",
    ));
    checks.push(check_at_least_f64(
        "tree_branch_reach_ratio",
        value_f64(minimums, "tree_branch_reach_ratio"),
        MIN_TREE_BRANCH_REACH_RATIO,
        "ratio",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_mesh_vertices",
        value_u64(minimums, "tree_canopy_mesh_vertices"),
        MIN_TREE_CANOPY_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_lobe_count",
        value_u64(minimums, "tree_canopy_lobe_count"),
        MIN_TREE_CANOPY_LOBE_COUNT,
        "lobes",
    ));
    checks.push(check_at_least_u64(
        "tree_canopy_detail_card_count",
        value_u64(minimums, "tree_canopy_detail_card_count"),
        MIN_TREE_CANOPY_DETAIL_CARD_COUNT,
        "cards",
    ));
    checks.push(check_at_least_f64(
        "tree_canopy_vertical_to_horizontal_ratio",
        value_f64(minimums, "tree_canopy_vertical_to_horizontal_ratio"),
        MIN_TREE_CANOPY_VERTICAL_TO_HORIZONTAL_RATIO,
        "ratio",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_mesh_vertices",
        value_u64(minimums, "weather_cloud_mesh_vertices"),
        MIN_WEATHER_CLOUD_MESH_VERTICES,
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_lobe_count",
        value_u64(minimums, "weather_cloud_lobe_count"),
        MIN_WEATHER_CLOUD_LOBE_COUNT,
        "lobes",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_wisp_card_count",
        value_u64(minimums, "weather_cloud_wisp_card_count"),
        MIN_WEATHER_CLOUD_WISP_CARD_COUNT,
        "cards",
    ));
    checks.push(check_at_least_f64(
        "weather_cloud_bank_depth",
        value_f64(minimums, "weather_cloud_bank_depth_m"),
        MIN_WEATHER_CLOUD_BANK_DEPTH_M,
        "m",
    ));
    checks.push(check_at_least_u64(
        "weather_cloud_bank_lobe_count",
        value_u64(minimums, "weather_cloud_bank_lobe_count"),
        MIN_WEATHER_CLOUD_BANK_LOBE_COUNT,
        "lobes",
    ));
    checks.push(check_at_least_u64(
        "terrain_biome_palette_count",
        value_u64(minimums, "terrain_biome_palette_count"),
        MIN_TERRAIN_BIOME_PALETTE_COUNT,
        "palettes",
    ));
    checks.push(check_at_least_u64(
        "foliage_palette_count",
        value_u64(minimums, "foliage_palette_count"),
        MIN_FOLIAGE_PALETTE_COUNT,
        "palettes",
    ));
    checks.push(check_at_least_u64(
        "stone_palette_count",
        value_u64(minimums, "stone_palette_count"),
        MIN_STONE_PALETTE_COUNT,
        "palettes",
    ));

    audit_mesh_array(
        manifest.get("ground_cover").and_then(Value::as_array),
        "mesh",
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );
    audit_tree_array(
        manifest.get("trees").and_then(Value::as_array),
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );
    audit_mesh_array(
        manifest.get("clouds").and_then(Value::as_array),
        "mesh",
        root_dir,
        &mut artifact_counters,
        &mut artifacts,
    );

    checks.push(check_eq_u64(
        "mesh_count",
        mesh_count,
        artifact_counters.expected,
        "meshes",
    ));
    checks.push(check_at_least_u64(
        "total_vertex_count",
        total_vertex_count,
        artifact_counters.expected.saturating_mul(300),
        "vertices",
    ));
    checks.push(check_at_least_u64(
        "total_triangle_count",
        total_triangle_count,
        artifact_counters.expected.saturating_mul(200),
        "triangles",
    ));
    checks.push(check_eq_u64(
        "mesh_artifact_count",
        artifact_counters.found,
        artifact_counters.expected,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_vertex_mismatch_count",
        artifact_counters.vertex_mismatches,
        0,
        "meshes",
    ));
    checks.push(check_eq_u64(
        "obj_face_mismatch_count",
        artifact_counters.face_mismatches,
        0,
        "meshes",
    ));

    let passed = checks.iter().all(|check| {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    });

    json!({
        "schema": "nau_visual_content_audit.v1",
        "manifest": manifest_path,
        "passed": passed,
        "checks": checks,
        "artifacts": {
            "expected_mesh_count": artifact_counters.expected,
            "found_mesh_count": artifact_counters.found,
            "missing_mesh_count": artifact_counters.missing,
            "vertex_mismatch_count": artifact_counters.vertex_mismatches,
            "face_mismatch_count": artifact_counters.face_mismatches,
            "failures": artifacts,
        }
    })
}

fn audit_tree_array(
    entries: Option<&Vec<Value>>,
    root_dir: &Path,
    counters: &mut ArtifactCounters,
    artifacts: &mut Vec<Value>,
) {
    let Some(entries) = entries else {
        return;
    };

    for tree in entries {
        audit_mesh_value(
            tree.get("trunk").unwrap_or(&Value::Null),
            root_dir,
            counters,
            artifacts,
        );
        audit_mesh_value(
            tree.get("canopy").unwrap_or(&Value::Null),
            root_dir,
            counters,
            artifacts,
        );
    }
}

fn audit_mesh_array(
    entries: Option<&Vec<Value>>,
    mesh_key: &str,
    root_dir: &Path,
    counters: &mut ArtifactCounters,
    artifacts: &mut Vec<Value>,
) {
    let Some(entries) = entries else {
        return;
    };

    for entry in entries {
        audit_mesh_value(
            entry.get(mesh_key).unwrap_or(&Value::Null),
            root_dir,
            counters,
            artifacts,
        );
    }
}

fn audit_mesh_value(
    mesh: &Value,
    root_dir: &Path,
    counters: &mut ArtifactCounters,
    artifacts: &mut Vec<Value>,
) {
    counters.expected += 1;
    let Some(obj_path) = relative_path(mesh, "obj") else {
        counters.missing += 1;
        artifacts.push(json!({"error": "missing obj path"}));
        return;
    };
    let full_path = root_dir.join(&obj_path);
    let manifest_vertices = value_u64(mesh, "vertex_count");
    let manifest_faces = value_u64(mesh, "triangle_count");

    match audit_obj_path(&full_path) {
        Ok(obj) => {
            counters.found += 1;
            if obj.vertex_count != manifest_vertices {
                counters.vertex_mismatches += 1;
                artifacts.push(json!({
                    "path": obj_path.to_string_lossy(),
                    "error": "vertex mismatch",
                    "manifest": manifest_vertices,
                    "obj": obj.vertex_count,
                }));
            }
            if obj.face_count != manifest_faces {
                counters.face_mismatches += 1;
                artifacts.push(json!({
                    "path": obj_path.to_string_lossy(),
                    "error": "face mismatch",
                    "manifest": manifest_faces,
                    "obj": obj.face_count,
                }));
            }
        }
        Err(error) => {
            counters.missing += 1;
            artifacts.push(json!({
                "path": obj_path.to_string_lossy(),
                "error": error,
            }));
        }
    }
}

fn audit_obj_path(path: &Path) -> Result<ObjAudit, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let mut vertex_count = 0;
    let mut face_count = 0;

    for line in text.lines() {
        if line.starts_with("v ") {
            vertex_count += 1;
        } else if line.starts_with("f ") {
            face_count += 1;
        }
    }

    Ok(ObjAudit {
        vertex_count,
        face_count,
    })
}

fn relative_path(parent: &Value, key: &str) -> Option<PathBuf> {
    parent.get(key).and_then(Value::as_str).map(PathBuf::from)
}

fn value_u64(parent: &Value, key: &str) -> u64 {
    parent.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn value_f64(parent: &Value, key: &str) -> f64 {
    parent.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn check_eq_str(name: &str, value: &str, expected: &str, unit: &str) -> Value {
    json!({
        "name": name,
        "passed": value == expected,
        "value": value,
        "comparator": "==",
        "threshold": expected,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn audit_rejects_low_shape_manifest() {
        let manifest = json!({
            "schema": "nau_visual_content_export.v1",
            "mesh_count": 0,
            "total_vertex_count": 0,
            "total_triangle_count": 0,
            "counts": {
                "ground_cover_count": 1,
                "ground_cover_patch_total": 10,
                "ground_cover_blade_total": 20,
                "tree_trunk_count": 1,
                "tree_canopy_count": 1,
                "weather_cloud_count": 1,
                "weather_cloud_bank_count": 0
            },
            "minimums": {
                "ground_cover_mesh_vertices": 10,
                "ground_cover_blade_count": 5,
                "ground_cover_blade_height_range_m": 0.1,
                "tree_trunk_mesh_vertices": 8,
                "tree_trunk_taper_ratio": 1.0,
                "tree_branch_reach_ratio": 1.0,
                "tree_canopy_mesh_vertices": 45,
                "tree_canopy_lobe_count": 1,
                "tree_canopy_detail_card_count": 0,
                "tree_canopy_vertical_to_horizontal_ratio": 0.1,
                "weather_cloud_mesh_vertices": 45,
                "weather_cloud_lobe_count": 1,
                "weather_cloud_wisp_card_count": 0,
                "weather_cloud_bank_depth_m": 0.2,
                "weather_cloud_bank_lobe_count": 0,
                "terrain_biome_palette_count": 1,
                "foliage_palette_count": 1,
                "stone_palette_count": 1
            },
            "ground_cover": [],
            "trees": [],
            "clouds": []
        });

        let report = audit_manifest(&manifest, Path::new("."), "manifest.json");
        assert!(!report.get("passed").and_then(Value::as_bool).unwrap());
        let checks = report.get("checks").and_then(Value::as_array).unwrap();
        assert!(
            check_named(checks, "tree_branch_reach_ratio")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
        assert!(
            check_named(checks, "weather_cloud_wisp_card_count")
                .is_some_and(|check| { !check.get("passed").and_then(Value::as_bool).unwrap() })
        );
    }

    #[test]
    fn obj_audit_counts_vertices_and_faces() {
        let path = std::env::temp_dir().join(format!(
            "nau-visual-content-audit-{}-{}.obj",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        ));
        let mut file = fs::File::create(&path).expect("obj should be creatable");
        writeln!(file, "v 0 0 0").unwrap();
        writeln!(file, "v 1 0 0").unwrap();
        writeln!(file, "v 0 1 0").unwrap();
        writeln!(file, "f 1 2 3").unwrap();

        let audit = audit_obj_path(&path).expect("obj should parse");
        assert_eq!(audit.vertex_count, 3);
        assert_eq!(audit.face_count, 1);

        fs::remove_file(path).expect("obj should be removable");
    }

    fn check_named<'a>(checks: &'a [Value], name: &str) -> Option<&'a Value> {
        checks.iter().find(|check| {
            check
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|check_name| check_name == name)
        })
    }
}
