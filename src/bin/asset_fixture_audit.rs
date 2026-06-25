use nau_engine::asset_pipeline::{
    PLAYER_ANIMATION_CLIP_NAMES, VISUAL_ASSET_SPECS, VisualAssetKind,
};
use serde_json::{Value, json};
use std::{fs, path::Path, process};

#[derive(Clone, Copy)]
struct Requirement {
    kind: VisualAssetKind,
    min_nodes: u64,
    min_meshes: u64,
    min_materials: u64,
    min_vertices: u64,
    min_triangles: u64,
    required_name_fragments: &'static [&'static str],
    require_blend_material: bool,
    require_player_clips: bool,
}

const PLAYER_NAME_FRAGMENTS: &[&str] = &[
    "suit", "skin", "accent", "helmet", "shoulder", "scarf", "boot",
];
const GLIDER_NAME_FRAGMENTS: &[&str] = &["cloth panel", "spar", "rib", "tether", "grip"];
const TERRAIN_NAME_FRAGMENTS: &[&str] = &["relief", "cliff", "underside", "landing", "terrace"];
const FOLIAGE_NAME_FRAGMENTS: &[&str] = &[
    "trunk",
    "branch",
    "canopy",
    "grass",
    "detail card",
    "wildflower",
];
const ROCK_NAME_FRAGMENTS: &[&str] = &["boulder", "stone", "strata", "fracture", "quartz"];
const WATER_NAME_FRAGMENTS: &[&str] = &["pond", "rim", "ripple", "reed", "depth", "glint"];
const ROUTE_MARKER_NAME_FRAGMENTS: &[&str] = &["gate", "mast", "shard", "cairn", "pennant"];
const WEATHER_NAME_FRAGMENTS: &[&str] = &["cloud bank", "shadow belly", "cirrus", "wisp"];
const IMPOSTOR_NAME_FRAGMENTS: &[&str] = &["terrain", "underside", "rim", "tree silhouette"];

const REQUIREMENTS: &[Requirement] = &[
    Requirement {
        kind: VisualAssetKind::PlayerCharacter,
        min_nodes: 18,
        min_meshes: 10,
        min_materials: 6,
        min_vertices: 320,
        min_triangles: 550,
        required_name_fragments: PLAYER_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: true,
    },
    Requirement {
        kind: VisualAssetKind::Glider,
        min_nodes: 13,
        min_meshes: 12,
        min_materials: 5,
        min_vertices: 100,
        min_triangles: 120,
        required_name_fragments: GLIDER_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandTerrain,
        min_nodes: 6,
        min_meshes: 5,
        min_materials: 5,
        min_vertices: 300,
        min_triangles: 380,
        required_name_fragments: TERRAIN_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandFoliage,
        min_nodes: 13,
        min_meshes: 12,
        min_materials: 6,
        min_vertices: 250,
        min_triangles: 360,
        required_name_fragments: FOLIAGE_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandRock,
        min_nodes: 7,
        min_meshes: 6,
        min_materials: 5,
        min_vertices: 300,
        min_triangles: 430,
        required_name_fragments: ROCK_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandWater,
        min_nodes: 9,
        min_meshes: 8,
        min_materials: 5,
        min_vertices: 210,
        min_triangles: 200,
        required_name_fragments: WATER_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::RouteMarker,
        min_nodes: 8,
        min_meshes: 7,
        min_materials: 5,
        min_vertices: 360,
        min_triangles: 650,
        required_name_fragments: ROUTE_MARKER_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::WeatherLayer,
        min_nodes: 10,
        min_meshes: 10,
        min_materials: 4,
        min_vertices: 330,
        min_triangles: 480,
        required_name_fragments: WEATHER_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::DistantImpostor,
        min_nodes: 6,
        min_meshes: 6,
        min_materials: 4,
        min_vertices: 180,
        min_triangles: 190,
        required_name_fragments: IMPOSTOR_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
];

fn main() {
    match audit_all_fixtures() {
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
            eprintln!("asset fixture audit failed: {error}");
            process::exit(2);
        }
    }
}

fn audit_all_fixtures() -> Result<Value, String> {
    let mut fixtures = Vec::new();
    let mut checks = Vec::new();

    for spec in VISUAL_ASSET_SPECS {
        let Some(requirement) = REQUIREMENTS
            .iter()
            .find(|requirement| requirement.kind == spec.kind)
        else {
            checks.push(check_bool(
                "fixture_requirement_declared",
                false,
                kind_name(spec.kind),
            ));
            continue;
        };
        let path = Path::new("assets").join(spec.gltf_scene_path);
        let fixture = audit_fixture(&path, requirement)?;
        checks.push(check_bool(
            "fixture_present",
            fixture["present"].as_bool().unwrap_or(false),
            kind_name(spec.kind),
        ));
        checks.push(check_bool(
            "fixture_passed",
            fixture["passed"].as_bool().unwrap_or(false),
            kind_name(spec.kind),
        ));
        fixtures.push(fixture);
    }

    checks.push(check_eq_u64(
        "fixture_count",
        fixtures.len() as u64,
        VISUAL_ASSET_SPECS.len() as u64,
        "fixtures",
    ));
    let passed = checks_passed(&checks);

    Ok(json!({
        "passed": passed,
        "fixture_count": fixtures.len(),
        "checks": checks,
        "fixtures": fixtures,
    }))
}

fn audit_fixture(path: &Path, requirement: &Requirement) -> Result<Value, String> {
    let path_string = path.to_string_lossy().into_owned();
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            return Ok(json!({
                "kind": kind_name(requirement.kind),
                "path": path_string,
                "present": false,
                "passed": false,
                "error": error.to_string(),
                "checks": [check_bool("present", false, "file")],
            }));
        }
    };
    let gltf = serde_json::from_str::<Value>(&text)
        .map_err(|error| format!("could not parse {}: {error}", path.display()))?;

    let asset = gltf.get("asset").unwrap_or(&Value::Null);
    let generator = asset.get("generator").and_then(Value::as_str).unwrap_or("");
    let copyright = asset.get("copyright").and_then(Value::as_str).unwrap_or("");
    let nodes = array_len(&gltf, "nodes");
    let meshes = array_len(&gltf, "meshes");
    let materials = array_len(&gltf, "materials");
    let component_names = named_components(&gltf);
    let metrics = geometry_metrics(&gltf);
    let blend_material_count = gltf
        .get("materials")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|material| {
            material
                .get("alphaMode")
                .and_then(Value::as_str)
                .is_some_and(|mode| mode == "BLEND")
        })
        .count() as u64;
    let ready_player_clip_count = if requirement.require_player_clips {
        player_clip_count(&gltf)
    } else {
        0
    };

    let mut checks = vec![
        check_bool("present", true, "file"),
        check_bool(
            "self_authored_generator",
            generator.starts_with("NAU Engine self-authored"),
            "provenance",
        ),
        check_eq_str(
            "self_authored_copyright",
            copyright,
            "Self-authored for NAU Engine; no third-party assets.",
            "provenance",
        ),
        check_at_least_u64("node_count", nodes, requirement.min_nodes, "nodes"),
        check_at_least_u64("mesh_count", meshes, requirement.min_meshes, "meshes"),
        check_at_least_u64(
            "material_count",
            materials,
            requirement.min_materials,
            "materials",
        ),
        check_at_least_u64(
            "position_vertices",
            metrics.position_vertices,
            requirement.min_vertices,
            "vertices",
        ),
        check_at_least_u64(
            "indexed_triangles",
            metrics.indexed_triangles,
            requirement.min_triangles,
            "triangles",
        ),
        check_bool(
            "all_primitives_have_normals",
            metrics.missing_normals == 0,
            "primitives",
        ),
        check_bool(
            "all_primitives_have_uvs",
            metrics.missing_uvs == 0,
            "primitives",
        ),
    ];
    for name_fragment in requirement.required_name_fragments {
        checks.push(check_bool(
            "semantic_name_present",
            has_name_fragment(&component_names, name_fragment),
            name_fragment,
        ));
    }

    if requirement.require_blend_material {
        checks.push(check_at_least_u64(
            "blend_material_count",
            blend_material_count,
            1,
            "materials",
        ));
    }
    if requirement.require_player_clips {
        checks.push(check_eq_u64(
            "player_named_clip_count",
            ready_player_clip_count,
            PLAYER_ANIMATION_CLIP_NAMES.len() as u64,
            "clips",
        ));
    }

    let passed = checks_passed(&checks);
    Ok(json!({
        "kind": kind_name(requirement.kind),
        "path": path_string,
        "present": true,
        "passed": passed,
        "generator": generator,
        "node_count": nodes,
        "mesh_count": meshes,
        "material_count": materials,
        "semantic_name_count": component_names.len(),
        "position_vertices": metrics.position_vertices,
        "indexed_triangles": metrics.indexed_triangles,
        "missing_normal_primitives": metrics.missing_normals,
        "missing_uv_primitives": metrics.missing_uvs,
        "blend_material_count": blend_material_count,
        "player_named_clip_count": ready_player_clip_count,
        "checks": checks,
    }))
}

fn named_components(gltf: &Value) -> Vec<String> {
    ["nodes", "meshes"]
        .into_iter()
        .flat_map(|key| {
            gltf.get(key)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|value| value.get("name").and_then(Value::as_str))
                .map(|name| name.to_ascii_lowercase())
        })
        .collect()
}

fn has_name_fragment(component_names: &[String], fragment: &str) -> bool {
    let fragment = fragment.to_ascii_lowercase();
    component_names
        .iter()
        .any(|component_name| component_name.contains(&fragment))
}

#[derive(Default)]
struct GeometryMetrics {
    position_vertices: u64,
    indexed_triangles: u64,
    missing_normals: u64,
    missing_uvs: u64,
}

fn geometry_metrics(gltf: &Value) -> GeometryMetrics {
    let accessors = gltf
        .get("accessors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut metrics = GeometryMetrics::default();
    let Some(meshes) = gltf.get("meshes").and_then(Value::as_array) else {
        return metrics;
    };

    for mesh in meshes {
        let Some(primitives) = mesh.get("primitives").and_then(Value::as_array) else {
            continue;
        };
        for primitive in primitives {
            let attributes = primitive.get("attributes").unwrap_or(&Value::Null);
            if let Some(position_accessor) = attributes.get("POSITION").and_then(Value::as_u64) {
                metrics.position_vertices += accessor_count(&accessors, position_accessor);
            }
            if attributes.get("NORMAL").and_then(Value::as_u64).is_none() {
                metrics.missing_normals += 1;
            }
            if attributes
                .get("TEXCOORD_0")
                .and_then(Value::as_u64)
                .is_none()
            {
                metrics.missing_uvs += 1;
            }
            if let Some(index_accessor) = primitive.get("indices").and_then(Value::as_u64) {
                metrics.indexed_triangles += accessor_count(&accessors, index_accessor) / 3;
            }
        }
    }
    metrics
}

fn accessor_count(accessors: &[Value], accessor_index: u64) -> u64 {
    accessors
        .get(accessor_index as usize)
        .and_then(|accessor| accessor.get("count"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn player_clip_count(gltf: &Value) -> u64 {
    let animations = gltf
        .get("animations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    PLAYER_ANIMATION_CLIP_NAMES
        .iter()
        .filter(|clip_name| {
            animations.iter().any(|animation| {
                animation
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name == **clip_name)
            })
        })
        .count() as u64
}

fn array_len(value: &Value, key: &str) -> u64 {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(0, |values| values.len() as u64)
}

fn checks_passed(checks: &[Value]) -> bool {
    checks.iter().all(|check| {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    })
}

fn check_bool(name: &'static str, passed: bool, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": passed,
        "value": if passed { 1 } else { 0 },
        "comparator": "==",
        "threshold": 1,
        "unit": unit,
    })
}

fn check_at_least_u64(name: &'static str, value: u64, threshold: u64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_eq_u64(name: &'static str, value: u64, threshold: u64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_eq_str(name: &'static str, value: &str, threshold: &str, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value == threshold,
        "value": value,
        "comparator": "==",
        "threshold": threshold,
        "unit": unit,
    })
}

fn kind_name(kind: VisualAssetKind) -> &'static str {
    match kind {
        VisualAssetKind::PlayerCharacter => "player_character",
        VisualAssetKind::Glider => "glider",
        VisualAssetKind::IslandTerrain => "island_terrain",
        VisualAssetKind::IslandFoliage => "island_foliage",
        VisualAssetKind::IslandRock => "island_rock",
        VisualAssetKind::IslandWater => "island_water",
        VisualAssetKind::RouteMarker => "route_marker",
        VisualAssetKind::WeatherLayer => "weather_layer",
        VisualAssetKind::DistantImpostor => "distant_impostor",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_name_fragments_can_match_nodes_or_meshes() {
        let gltf = json!({
            "nodes": [
                {"name": "Readable Landing Soil Strip"},
                {"name": "World Root"}
            ],
            "meshes": [
                {"name": "Authored Cliff Skirt"},
                {"name": "Underside Rock Mass"}
            ]
        });
        let names = named_components(&gltf);

        assert!(has_name_fragment(&names, "landing"));
        assert!(has_name_fragment(&names, "cliff"));
        assert!(has_name_fragment(&names, "underside"));
        assert!(!has_name_fragment(&names, "missing route ring"));
    }
}
