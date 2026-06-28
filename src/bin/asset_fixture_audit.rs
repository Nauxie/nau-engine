use nau_engine::asset_pipeline::{
    PLAYER_ANIMATION_CLIP_NAMES, VISUAL_ASSET_SPECS, VisualAssetKind, VisualAssetResidency,
    VisualAssetSpec,
};
use serde_json::{Value, json};
use std::{fs, path::Path, process};

const NAU_FIXTURE_SCHEMA: &str = "nau_visual_asset_fixture.v1";
const NAU_FIXTURE_LICENSE: &str = "self_authored_no_third_party";

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
    "suit", "skin", "accent", "helmet", "shoulder", "scarf", "boot", "face", "eye", "belt",
    "gauntlet", "knee", "hand", "finger", "toe",
];
const GLIDER_NAME_FRAGMENTS: &[&str] = &["cloth panel", "spar", "rib", "tether", "grip"];
const TERRAIN_NAME_FRAGMENTS: &[&str] = &[
    "relief",
    "cliff",
    "underside",
    "landing",
    "terrace",
    "erosion",
    "path",
];
const FOLIAGE_NAME_FRAGMENTS: &[&str] = &[
    "trunk",
    "branch",
    "canopy",
    "grass",
    "detail card",
    "wildflower",
    "root",
    "fern",
    "moss",
];
const ROCK_NAME_FRAGMENTS: &[&str] = &[
    "boulder", "stone", "strata", "fracture", "quartz", "rust", "shale",
];
const WATER_NAME_FRAGMENTS: &[&str] = &[
    "pond", "rim", "ripple", "reed", "depth", "glint", "lily", "specular",
];
const ROUTE_MARKER_NAME_FRAGMENTS: &[&str] = &[
    "gate", "mast", "shard", "cairn", "pennant", "glyph", "pebble",
];
const WEATHER_NAME_FRAGMENTS: &[&str] = &[
    "cloud bank",
    "shadow belly",
    "cirrus",
    "wisp",
    "haze",
    "filament",
];
const IMPOSTOR_NAME_FRAGMENTS: &[&str] = &[
    "terrain",
    "underside",
    "rim",
    "tree silhouette",
    "waterfall",
    "broken",
];

const REQUIREMENTS: &[Requirement] = &[
    Requirement {
        kind: VisualAssetKind::PlayerCharacter,
        min_nodes: 38,
        min_meshes: 22,
        min_materials: 8,
        min_vertices: 700,
        min_triangles: 1140,
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
        min_nodes: 11,
        min_meshes: 10,
        min_materials: 7,
        min_vertices: 400,
        min_triangles: 520,
        required_name_fragments: TERRAIN_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandFoliage,
        min_nodes: 18,
        min_meshes: 17,
        min_materials: 7,
        min_vertices: 280,
        min_triangles: 390,
        required_name_fragments: FOLIAGE_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandRock,
        min_nodes: 9,
        min_meshes: 9,
        min_materials: 6,
        min_vertices: 370,
        min_triangles: 530,
        required_name_fragments: ROCK_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::IslandWater,
        min_nodes: 11,
        min_meshes: 10,
        min_materials: 7,
        min_vertices: 230,
        min_triangles: 220,
        required_name_fragments: WATER_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::RouteMarker,
        min_nodes: 10,
        min_meshes: 10,
        min_materials: 6,
        min_vertices: 480,
        min_triangles: 870,
        required_name_fragments: ROUTE_MARKER_NAME_FRAGMENTS,
        require_blend_material: false,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::WeatherLayer,
        min_nodes: 13,
        min_meshes: 13,
        min_materials: 5,
        min_vertices: 390,
        min_triangles: 560,
        required_name_fragments: WEATHER_NAME_FRAGMENTS,
        require_blend_material: true,
        require_player_clips: false,
    },
    Requirement {
        kind: VisualAssetKind::DistantImpostor,
        min_nodes: 9,
        min_meshes: 9,
        min_materials: 5,
        min_vertices: 220,
        min_triangles: 230,
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
        let fixture = audit_fixture(&path, &spec, requirement)?;
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

fn audit_fixture(
    path: &Path,
    spec: &VisualAssetSpec,
    requirement: &Requirement,
) -> Result<Value, String> {
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
    let nau_metadata = gltf.pointer("/extras/nau").unwrap_or(&Value::Null);
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
        check_bool("nau_metadata_present", nau_metadata.is_object(), "metadata"),
        check_eq_str(
            "nau_metadata_schema",
            nau_metadata_str(nau_metadata, "schema"),
            NAU_FIXTURE_SCHEMA,
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_asset_kind",
            nau_metadata_str(nau_metadata, "asset_kind"),
            kind_name(spec.kind),
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_asset_label",
            nau_metadata_str(nau_metadata, "asset_label"),
            spec.label,
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_residency",
            nau_metadata_str(nau_metadata, "residency"),
            residency_name(spec.residency),
            "metadata",
        ),
        check_eq_str(
            "nau_metadata_license",
            nau_metadata_str(nau_metadata, "license"),
            NAU_FIXTURE_LICENSE,
            "metadata",
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
        checks.push(check_bool(
            "player_bank_clip_motion_distinct",
            player_bank_clip_motion_is_distinct(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_launch_glide_land_clip_motion_distinct",
            player_launch_glide_land_clip_motion_is_distinct(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_grounded_locomotion_clip_motion_distinct",
            player_grounded_locomotion_clip_motion_is_distinct(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_fall_clip_motion_distinct",
            player_fall_clip_motion_is_distinct(&gltf),
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
        "nau_metadata": nau_metadata,
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
        "player_bank_clip_motion_distinct": player_bank_clip_motion_is_distinct(&gltf),
        "player_launch_glide_land_clip_motion_distinct": player_launch_glide_land_clip_motion_is_distinct(&gltf),
        "player_grounded_locomotion_clip_motion_distinct": player_grounded_locomotion_clip_motion_is_distinct(&gltf),
        "player_fall_clip_motion_distinct": player_fall_clip_motion_is_distinct(&gltf),
        "checks": checks,
    }))
}

fn nau_metadata_str<'a>(metadata: &'a Value, key: &str) -> &'a str {
    metadata.get(key).and_then(Value::as_str).unwrap_or("")
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

fn player_bank_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(glide) = animation_signature(gltf, "Glide_Loop") else {
        return false;
    };
    let Some(bank_left) = animation_signature(gltf, "Bank_Left") else {
        return false;
    };
    let Some(bank_right) = animation_signature(gltf, "Bank_Right") else {
        return false;
    };

    bank_left.len() >= 4
        && bank_right.len() >= 4
        && bank_left != glide
        && bank_right != glide
        && bank_left != bank_right
}

fn player_launch_glide_land_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(launch) = animation_signature(gltf, "Launch_Start") else {
        return false;
    };
    let Some(glide) = animation_signature(gltf, "Glide_Loop") else {
        return false;
    };
    let Some(land) = animation_signature(gltf, "Land") else {
        return false;
    };

    !launch.is_empty()
        && !glide.is_empty()
        && !land.is_empty()
        && launch != glide
        && launch != land
        && glide != land
}

fn player_grounded_locomotion_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(idle) = animation_signature(gltf, "Idle_Loop") else {
        return false;
    };
    let Some(walk) = animation_signature(gltf, "Walk_Fwd_Loop") else {
        return false;
    };
    let Some(run) = animation_signature(gltf, "Run_Fwd_Loop") else {
        return false;
    };

    !idle.is_empty()
        && walk.len() >= 4
        && run.len() >= 4
        && idle != walk
        && idle != run
        && walk != run
}

fn player_fall_clip_motion_is_distinct(gltf: &Value) -> bool {
    let Some(fall) = animation_signature(gltf, "Fall_Loop") else {
        return false;
    };
    let Some(glide) = animation_signature(gltf, "Glide_Loop") else {
        return false;
    };
    let Some(air_brake) = animation_signature(gltf, "Air_Brake") else {
        return false;
    };
    let Some(land) = animation_signature(gltf, "Land") else {
        return false;
    };

    fall.len() >= 4 && fall != glide && fall != air_brake && fall != land
}

fn animation_signature(gltf: &Value, animation_name: &str) -> Option<Vec<String>> {
    let accessors = gltf.get("accessors").and_then(Value::as_array)?;
    let animation = gltf
        .get("animations")
        .and_then(Value::as_array)?
        .iter()
        .find(|animation| {
            animation
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name == animation_name)
        })?;
    let samplers = animation.get("samplers").and_then(Value::as_array)?;
    let channels = animation.get("channels").and_then(Value::as_array)?;

    channels
        .iter()
        .map(|channel| {
            let sampler_index = channel.get("sampler").and_then(Value::as_u64)? as usize;
            let output_accessor = samplers
                .get(sampler_index)?
                .get("output")
                .and_then(Value::as_u64)? as usize;
            let accessor = accessors.get(output_accessor)?;
            let target = channel.get("target")?;
            let node = target.get("node").and_then(Value::as_u64)?;
            let path = target.get("path").and_then(Value::as_str)?;
            let min = accessor.get("min").unwrap_or(&Value::Null);
            let max = accessor.get("max").unwrap_or(&Value::Null);
            Some(format!("{node}:{path}:{}:{}", min, max))
        })
        .collect()
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

fn residency_name(residency: VisualAssetResidency) -> &'static str {
    match residency {
        VisualAssetResidency::Always => "always",
        VisualAssetResidency::StreamWindow => "stream_window",
        VisualAssetResidency::NearLod => "near_lod",
        VisualAssetResidency::FarLod => "far_lod",
        VisualAssetResidency::Weather => "weather",
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

    #[test]
    fn nau_fixture_metadata_uses_registry_contract_names() {
        let metadata = json!({
            "schema": NAU_FIXTURE_SCHEMA,
            "asset_kind": "island_terrain",
            "asset_label": "island terrain kit",
            "residency": "stream_window",
            "license": NAU_FIXTURE_LICENSE
        });

        assert_eq!(nau_metadata_str(&metadata, "schema"), NAU_FIXTURE_SCHEMA);
        assert_eq!(
            kind_name(VisualAssetKind::IslandTerrain),
            nau_metadata_str(&metadata, "asset_kind")
        );
        assert_eq!(
            residency_name(VisualAssetResidency::StreamWindow),
            nau_metadata_str(&metadata, "residency")
        );
    }

    #[test]
    fn player_bank_clip_motion_rejects_glide_reuse() {
        let gltf = bank_clip_test_gltf(false);

        assert!(!player_bank_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_bank_clip_motion_accepts_distinct_bank_tracks() {
        let gltf = bank_clip_test_gltf(true);

        assert!(player_bank_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_launch_glide_land_clip_motion_rejects_reused_tracks() {
        let gltf = launch_glide_land_clip_test_gltf(false);

        assert!(!player_launch_glide_land_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_launch_glide_land_clip_motion_accepts_distinct_tracks() {
        let gltf = launch_glide_land_clip_test_gltf(true);

        assert!(player_launch_glide_land_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_fall_clip_motion_rejects_air_brake_reuse() {
        let gltf = fall_clip_test_gltf(false);

        assert!(!player_fall_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_fall_clip_motion_accepts_distinct_fall_track() {
        let gltf = fall_clip_test_gltf(true);

        assert!(player_fall_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_grounded_locomotion_clip_motion_rejects_walk_run_reuse() {
        let gltf = grounded_locomotion_clip_test_gltf(false);

        assert!(!player_grounded_locomotion_clip_motion_is_distinct(&gltf));
    }

    #[test]
    fn player_grounded_locomotion_clip_motion_accepts_distinct_walk_run() {
        let gltf = grounded_locomotion_clip_test_gltf(true);

        assert!(player_grounded_locomotion_clip_motion_is_distinct(&gltf));
    }

    fn bank_clip_test_gltf(distinct_banks: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [1.0]},
            {"min": [0.0], "max": [0.5]},
            {"min": [0.0], "max": [0.6]},
            {"min": [0.0], "max": [0.7]},
            {"min": [0.0], "max": [0.8]},
            {"min": [-0.8], "max": [0.0]},
            {"min": [-0.7], "max": [0.0]},
            {"min": [-0.6], "max": [0.0]},
            {"min": [-0.5], "max": [0.0]}
        ]);
        let left_outputs = if distinct_banks {
            [1, 2, 3, 4]
        } else {
            [0, 0, 0, 0]
        };
        let right_outputs = if distinct_banks {
            [5, 6, 7, 8]
        } else {
            [0, 0, 0, 0]
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Glide_Loop", [0, 0, 0, 0]),
                clip("Bank_Left", left_outputs),
                clip("Bank_Right", right_outputs)
            ]
        })
    }

    fn launch_glide_land_clip_test_gltf(distinct_launch_and_land: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [0.2]},
            {"min": [0.1], "max": [0.4]},
            {"min": [0.2], "max": [0.6]},
            {"min": [0.3], "max": [0.8]},
            {"min": [-0.8], "max": [-0.2]},
            {"min": [-0.6], "max": [-0.1]},
            {"min": [-0.4], "max": [0.0]},
            {"min": [-0.2], "max": [0.1]},
            {"min": [0.4], "max": [1.0]},
            {"min": [0.5], "max": [1.1]},
            {"min": [0.6], "max": [1.2]},
            {"min": [0.7], "max": [1.3]}
        ]);
        let glide_outputs = [0, 1, 2, 3];
        let launch_outputs = if distinct_launch_and_land {
            [4, 5, 6, 7]
        } else {
            glide_outputs
        };
        let land_outputs = if distinct_launch_and_land {
            [8, 9, 10, 11]
        } else {
            glide_outputs
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Launch_Start", launch_outputs),
                clip("Glide_Loop", glide_outputs),
                clip("Land", land_outputs)
            ]
        })
    }

    fn fall_clip_test_gltf(distinct_fall: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [0.2]},
            {"min": [0.0], "max": [0.4]},
            {"min": [0.0], "max": [0.6]},
            {"min": [0.0], "max": [0.8]},
            {"min": [-0.2], "max": [0.0]},
            {"min": [-0.4], "max": [0.0]},
            {"min": [-0.6], "max": [0.0]},
            {"min": [-0.8], "max": [0.0]},
            {"min": [0.1], "max": [0.5]},
            {"min": [0.2], "max": [0.6]},
            {"min": [0.3], "max": [0.7]},
            {"min": [0.4], "max": [0.8]},
            {"min": [0.5], "max": [0.9]}
        ]);
        let air_brake_outputs = [1, 2, 3, 4];
        let fall_outputs = if distinct_fall {
            [9, 10, 11, 12]
        } else {
            air_brake_outputs
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Glide_Loop", [0, 0, 0, 0]),
                clip("Air_Brake", air_brake_outputs),
                clip("Land", [5, 6, 7, 8]),
                clip("Fall_Loop", fall_outputs)
            ]
        })
    }

    fn grounded_locomotion_clip_test_gltf(distinct_run: bool) -> Value {
        let accessors = json!([
            {"min": [0.0], "max": [0.1]},
            {"min": [0.0], "max": [0.2]},
            {"min": [0.0], "max": [0.3]},
            {"min": [0.0], "max": [0.4]},
            {"min": [-0.4], "max": [0.0]},
            {"min": [-0.3], "max": [0.0]},
            {"min": [-0.2], "max": [0.0]},
            {"min": [-0.1], "max": [0.0]},
            {"min": [0.2], "max": [0.7]},
            {"min": [0.3], "max": [0.8]},
            {"min": [0.4], "max": [0.9]},
            {"min": [0.5], "max": [1.0]}
        ]);
        let walk_outputs = [1, 2, 3, 4];
        let run_outputs = if distinct_run {
            [8, 9, 10, 11]
        } else {
            walk_outputs
        };

        json!({
            "accessors": accessors,
            "animations": [
                clip("Idle_Loop", [0, 0, 0, 0]),
                clip("Walk_Fwd_Loop", walk_outputs),
                clip("Run_Fwd_Loop", run_outputs)
            ]
        })
    }

    fn clip(name: &str, outputs: [u64; 4]) -> Value {
        json!({
            "name": name,
            "samplers": [
                {"output": outputs[0]},
                {"output": outputs[1]},
                {"output": outputs[2]},
                {"output": outputs[3]}
            ],
            "channels": [
                {"sampler": 0, "target": {"node": 2, "path": "rotation"}},
                {"sampler": 1, "target": {"node": 3, "path": "rotation"}},
                {"sampler": 2, "target": {"node": 4, "path": "rotation"}},
                {"sampler": 3, "target": {"node": 5, "path": "rotation"}}
            ]
        })
    }
}
