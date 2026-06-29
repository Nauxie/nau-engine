use bevy::prelude::{Mat4, Quat, Vec3};
use nau_engine::animation::{
    CharacterPart, CharacterPartRole, MIN_KEY_POSE_READABILITY_SCORE, PlayerPoseContext,
    PlayerPoseIntent, Side, glider_deployment_for_mode, glider_traversal_pose,
    part_pose_with_context, pose_readability_metrics,
};
use nau_engine::asset_pipeline::{
    PLAYER_ANIMATION_CLIP_NAMES, VISUAL_ASSET_SPECS, VisualAssetKind, VisualAssetResidency,
    VisualAssetSpec,
};
use nau_engine::movement::{FlightInput, FlightMode};
use serde_json::{Value, json};
use std::{fs, path::Path, process};

const NAU_FIXTURE_SCHEMA: &str = "nau_visual_asset_fixture.v1";
const NAU_FIXTURE_LICENSE: &str = "self_authored_no_third_party";
const PLAYER_POSE_MAX_CONNECTED_LIMB_TRANSLATION_M: f64 = 0.02;
const PLAYER_REST_MAX_ARTICULATED_JOINT_GAP_M: f64 = 0.015;
const PLAYER_POSE_MIN_FALLING_TORSO_PITCH_DEGREES: f64 = 58.0;
const PLAYER_POSE_MIN_FALLING_ARM_SPREAD_DEGREES: f64 = 136.0;
const PLAYER_POSE_MIN_DIVE_TORSO_PITCH_DEGREES: f64 = 82.0;
const PLAYER_POSE_MAX_DIVE_ARM_SPREAD_DEGREES: f64 = 74.0;
const PLAYER_POSE_MIN_DIVE_LEG_TUCK_DEGREES: f64 = 68.0;
const PLAYER_GLIDER_MIN_LAUNCH_DEPLOYMENT: f64 = 0.45;
const PLAYER_GLIDER_MAX_LAUNCH_DEPLOYMENT: f64 = 0.70;
const PLAYER_GLIDER_MIN_DIVE_RESPONSE_DEGREES: f64 = 4.0;
const PLAYER_GLIDER_MIN_DIVE_MOTION_M: f64 = 0.16;

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
    "gauntlet", "knee", "hand", "finger", "toe", "neck", "elbow", "ankle",
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
        min_nodes: 44,
        min_meshes: 25,
        min_materials: 8,
        min_vertices: 1100,
        min_triangles: 1800,
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
    let player_pose_shape_audit = requirement
        .require_player_clips
        .then(player_pose_shape_audit);

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
        checks.push(check_at_least_f64(
            "player_rest_arm_half_width",
            symmetric_node_half_width_m(&gltf, "Nau Left Arm", "Nau Right Arm").unwrap_or(0.0),
            0.42,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_rest_leg_half_width",
            symmetric_node_half_width_m(&gltf, "Nau Left Leg", "Nau Right Leg").unwrap_or(0.0),
            0.20,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_scarf_back_offset",
            world_node_translation(&gltf, "Nau Wind Scarf Accent")
                .map_or(0.0, |translation| translation[2]),
            0.30,
            "m",
        ));
        checks.push(check_eq_u64(
            "player_named_clip_count",
            ready_player_clip_count,
            PLAYER_ANIMATION_CLIP_NAMES.len() as u64,
            "clips",
        ));
        checks.push(check_bool(
            "player_core_pose_nodes_present",
            player_core_pose_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_articulated_pose_nodes_present",
            player_articulated_pose_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_rest_limb_attachment_hierarchy",
            player_rest_limb_attachment_hierarchy_valid(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_animation_channels_cover_core_signals",
            player_animation_channels_cover_core_signals(&gltf),
            "clips",
        ));
        checks.push(check_bool(
            "player_animation_channels_avoid_runtime_pose_nodes",
            player_animation_channels_avoid_runtime_pose_nodes(&gltf),
            "clips",
        ));
        checks.push(check_at_most_f64(
            "player_rest_joint_gap_max",
            player_rest_joint_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            0.03,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_rest_articulated_joint_gap_max",
            player_rest_articulated_joint_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_REST_MAX_ARTICULATED_JOINT_GAP_M,
            "m",
        ));
        let pose_shape = player_pose_shape_audit
            .as_ref()
            .expect("player pose shape audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_pose_falling_belly_down_readability",
            number_field(pose_shape, "falling_key_pose_readability_score"),
            MIN_KEY_POSE_READABILITY_SCORE as f64,
            "score",
        ));
        checks.push(check_at_least_f64(
            "player_pose_falling_torso_pitch",
            number_field(pose_shape, "falling_torso_pitch_degrees"),
            PLAYER_POSE_MIN_FALLING_TORSO_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_falling_arm_spread",
            number_field(pose_shape, "falling_arm_spread_degrees"),
            PLAYER_POSE_MIN_FALLING_ARM_SPREAD_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_dive_streamline_readability",
            number_field(pose_shape, "dive_key_pose_readability_score"),
            MIN_KEY_POSE_READABILITY_SCORE as f64,
            "score",
        ));
        checks.push(check_at_least_f64(
            "player_pose_dive_torso_pitch",
            number_field(pose_shape, "dive_torso_pitch_degrees"),
            PLAYER_POSE_MIN_DIVE_TORSO_PITCH_DEGREES,
            "deg",
        ));
        checks.push(check_at_most_f64(
            "player_pose_dive_arm_spread",
            number_field(pose_shape, "dive_arm_spread_degrees"),
            PLAYER_POSE_MAX_DIVE_ARM_SPREAD_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_dive_leg_tuck",
            number_field(pose_shape, "dive_leg_tuck_degrees"),
            PLAYER_POSE_MIN_DIVE_LEG_TUCK_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_pose_landing_recovery_readability",
            number_field(pose_shape, "landing_recovery_key_pose_readability_score"),
            MIN_KEY_POSE_READABILITY_SCORE as f64,
            "score",
        ));
        checks.push(check_at_most_f64(
            "player_pose_connected_limb_root_translation",
            number_field(pose_shape, "max_connected_limb_translation_m"),
            PLAYER_POSE_MAX_CONNECTED_LIMB_TRANSLATION_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_glider_launch_takeout_deployment",
            number_field(pose_shape, "glider_launch_deployment"),
            PLAYER_GLIDER_MIN_LAUNCH_DEPLOYMENT,
            "ratio",
        ));
        checks.push(check_at_most_f64(
            "player_glider_launch_takeout_not_full_deployment",
            number_field(pose_shape, "glider_launch_deployment"),
            PLAYER_GLIDER_MAX_LAUNCH_DEPLOYMENT,
            "ratio",
        ));
        checks.push(check_at_least_f64(
            "player_glider_glide_full_deployment",
            number_field(pose_shape, "glider_glide_deployment"),
            1.0,
            "ratio",
        ));
        checks.push(check_at_most_f64(
            "player_glider_stowed_grounded_deployment",
            number_field(pose_shape, "glider_grounded_deployment"),
            0.0,
            "ratio",
        ));
        checks.push(check_at_least_f64(
            "player_glider_dive_response",
            number_field(pose_shape, "glider_dive_response_degrees"),
            PLAYER_GLIDER_MIN_DIVE_RESPONSE_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_glider_dive_motion",
            number_field(pose_shape, "glider_dive_motion_m"),
            PLAYER_GLIDER_MIN_DIVE_MOTION_M,
            "m",
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
        "player_core_pose_nodes_present": player_core_pose_nodes_present(&gltf),
        "player_articulated_pose_nodes_present": player_articulated_pose_nodes_present(&gltf),
        "player_rest_limb_attachment_hierarchy": player_rest_limb_attachment_hierarchy_valid(&gltf),
        "player_animation_channels_cover_core_signals": player_animation_channels_cover_core_signals(&gltf),
        "player_animation_channels_avoid_runtime_pose_nodes": player_animation_channels_avoid_runtime_pose_nodes(&gltf),
        "player_rest_joint_gap_max_m": player_rest_joint_gap_max_m(&gltf),
        "player_rest_articulated_joint_gap_max_m": player_rest_articulated_joint_gap_max_m(&gltf),
        "player_bank_clip_motion_distinct": player_bank_clip_motion_is_distinct(&gltf),
        "player_launch_glide_land_clip_motion_distinct": player_launch_glide_land_clip_motion_is_distinct(&gltf),
        "player_grounded_locomotion_clip_motion_distinct": player_grounded_locomotion_clip_motion_is_distinct(&gltf),
        "player_fall_clip_motion_distinct": player_fall_clip_motion_is_distinct(&gltf),
        "player_pose_shape_audit": player_pose_shape_audit,
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

fn player_core_pose_nodes_present(gltf: &Value) -> bool {
    [
        "Nau Torso",
        "Nau Head",
        "Nau Left Arm",
        "Nau Right Arm",
        "Nau Left Leg",
        "Nau Right Leg",
        "Nau Neck Socket",
        "Nau Left Shoulder Socket",
        "Nau Right Shoulder Socket",
        "Nau Left Hip Socket",
        "Nau Right Hip Socket",
    ]
    .into_iter()
    .all(|name| node_index(gltf, name).is_some())
}

fn player_articulated_pose_nodes_present(gltf: &Value) -> bool {
    [
        "Nau Left Forearm",
        "Nau Right Forearm",
        "Nau Left Leather Hand Palm",
        "Nau Right Leather Hand Palm",
        "Nau Left Lower Leg",
        "Nau Right Lower Leg",
        "Nau Left Boot",
        "Nau Right Boot",
        "Nau Left Elbow Socket",
        "Nau Right Elbow Socket",
        "Nau Left Wrist Socket",
        "Nau Right Wrist Socket",
        "Nau Left Knee Socket",
        "Nau Right Knee Socket",
        "Nau Left Ankle Socket",
        "Nau Right Ankle Socket",
    ]
    .into_iter()
    .all(|name| node_index(gltf, name).is_some())
}

fn player_rest_limb_attachment_hierarchy_valid(gltf: &Value) -> bool {
    [
        ("Nau Torso", "Nau Head"),
        ("Nau Torso", "Nau Left Arm"),
        ("Nau Torso", "Nau Right Arm"),
        ("Nau Left Arm", "Nau Left Forearm"),
        ("Nau Right Arm", "Nau Right Forearm"),
        ("Nau Left Forearm", "Nau Left Leather Hand Palm"),
        ("Nau Right Forearm", "Nau Right Leather Hand Palm"),
        ("Nau Hips", "Nau Left Leg"),
        ("Nau Hips", "Nau Right Leg"),
        ("Nau Left Leg", "Nau Left Lower Leg"),
        ("Nau Right Leg", "Nau Right Lower Leg"),
        ("Nau Left Lower Leg", "Nau Left Boot"),
        ("Nau Right Lower Leg", "Nau Right Boot"),
    ]
    .into_iter()
    .all(|(parent, child)| node_is_direct_child(gltf, parent, child))
}

fn player_animation_channels_cover_core_signals(gltf: &Value) -> bool {
    PLAYER_ANIMATION_CLIP_NAMES.iter().all(|clip_name| {
        let Some(targets) = animation_target_node_names(gltf, clip_name) else {
            return false;
        };
        [
            "Nau Animation Signal Torso",
            "Nau Animation Signal Left Arm",
            "Nau Animation Signal Right Arm",
            "Nau Animation Signal Left Leg",
            "Nau Animation Signal Right Leg",
        ]
        .into_iter()
        .all(|required| targets.iter().any(|target| target == required))
    })
}

fn player_animation_channels_avoid_runtime_pose_nodes(gltf: &Value) -> bool {
    let runtime_pose_nodes = [
        "Nau Torso",
        "Nau Head",
        "Nau Left Arm",
        "Nau Right Arm",
        "Nau Left Forearm",
        "Nau Right Forearm",
        "Nau Left Leather Hand Palm",
        "Nau Right Leather Hand Palm",
        "Nau Left Leg",
        "Nau Right Leg",
        "Nau Left Lower Leg",
        "Nau Right Lower Leg",
        "Nau Left Boot",
        "Nau Right Boot",
        "Nau Back Scarf Anchor Accent",
        "Nau Wind Scarf Accent",
    ];
    PLAYER_ANIMATION_CLIP_NAMES.iter().all(|clip_name| {
        animation_target_node_names(gltf, clip_name).is_some_and(|targets| {
            targets
                .iter()
                .all(|target| !runtime_pose_nodes.contains(&target.as_str()))
        })
    })
}

fn player_rest_joint_gap_max_m(gltf: &Value) -> Option<f64> {
    let pairs = [
        ("Nau Neck Socket", "Nau Head"),
        ("Nau Left Shoulder Socket", "Nau Left Arm"),
        ("Nau Right Shoulder Socket", "Nau Right Arm"),
        ("Nau Left Hip Socket", "Nau Left Leg"),
        ("Nau Right Hip Socket", "Nau Right Leg"),
    ];
    pairs
        .into_iter()
        .map(|(socket, joint)| {
            let socket = world_node_translation(gltf, socket)?;
            let joint = world_node_translation(gltf, joint)?;
            Some(distance3(socket, joint))
        })
        .collect::<Option<Vec<_>>>()
        .map(|gaps| gaps.into_iter().fold(0.0, f64::max))
}

fn player_rest_articulated_joint_gap_max_m(gltf: &Value) -> Option<f64> {
    let pairs = [
        ("Nau Left Elbow Socket", "Nau Left Forearm"),
        ("Nau Right Elbow Socket", "Nau Right Forearm"),
        ("Nau Left Wrist Socket", "Nau Left Leather Hand Palm"),
        ("Nau Right Wrist Socket", "Nau Right Leather Hand Palm"),
        ("Nau Left Knee Socket", "Nau Left Lower Leg"),
        ("Nau Right Knee Socket", "Nau Right Lower Leg"),
        ("Nau Left Ankle Socket", "Nau Left Boot"),
        ("Nau Right Ankle Socket", "Nau Right Boot"),
    ];
    pairs
        .into_iter()
        .map(|(socket, joint)| {
            let socket = world_node_translation(gltf, socket)?;
            let joint = world_node_translation(gltf, joint)?;
            Some(distance3(socket, joint))
        })
        .collect::<Option<Vec<_>>>()
        .map(|gaps| gaps.into_iter().fold(0.0, f64::max))
}

fn player_pose_shape_audit() -> Value {
    let falling_context = PlayerPoseContext::new(
        FlightMode::Airborne,
        Vec3::new(0.0, -22.0, -24.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Falling);
    let gliding_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -4.0, -38.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Gliding);
    let dive_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -28.0, -42.0),
        FlightInput {
            dive: true,
            ..Default::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Diving);
    let air_brake_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -5.0, 16.0),
        FlightInput {
            backward: true,
            ..Default::default()
        },
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::AirBrake);
    let landing_context = PlayerPoseContext::new(
        FlightMode::Gliding,
        Vec3::new(0.0, -18.0, -24.0),
        FlightInput::default(),
        5.0,
    )
    .with_resolved_intent(PlayerPoseIntent::LandingAnticipation);
    let landing_recovery_context = PlayerPoseContext::new(
        FlightMode::Grounded,
        Vec3::new(0.0, 0.0, -8.0),
        FlightInput::default(),
        0.0,
    )
    .with_landing_recovery(0.36, 16.0)
    .with_resolved_intent(PlayerPoseIntent::LandingRecovery);

    let falling = pose_readability_metrics(falling_context, 0.0);
    let dive = pose_readability_metrics(dive_context, 0.0);
    let landing_recovery = pose_readability_metrics(landing_recovery_context, 0.0);
    let glider_dive = glider_traversal_pose(dive_context, 0.0);
    let max_connected_limb_translation_m = max_connected_limb_translation_m(&[
        falling_context,
        gliding_context,
        dive_context,
        air_brake_context,
        landing_context,
        landing_recovery_context,
    ]);

    json!({
        "falling_key_pose_readability_score": falling.key_pose_readability_score,
        "falling_torso_pitch_degrees": falling.torso_pitch_degrees,
        "falling_arm_spread_degrees": falling.arm_spread_degrees,
        "dive_key_pose_readability_score": dive.key_pose_readability_score,
        "dive_torso_pitch_degrees": dive.torso_pitch_degrees,
        "dive_arm_spread_degrees": dive.arm_spread_degrees,
        "dive_leg_tuck_degrees": dive.leg_tuck_degrees,
        "landing_recovery_key_pose_readability_score": landing_recovery.key_pose_readability_score,
        "landing_recovery_flip_degrees": landing_recovery.landing_recovery_flip_degrees,
        "max_connected_limb_translation_m": max_connected_limb_translation_m,
        "glider_launch_deployment": glider_deployment_for_mode(FlightMode::Launching),
        "glider_glide_deployment": glider_deployment_for_mode(FlightMode::Gliding),
        "glider_grounded_deployment": glider_deployment_for_mode(FlightMode::Grounded),
        "glider_dive_response_degrees": glider_dive.response_degrees(),
        "glider_dive_motion_m": glider_dive.motion_m(),
    })
}

fn max_connected_limb_translation_m(contexts: &[PlayerPoseContext]) -> f64 {
    let limbs = [
        CharacterPart::new(
            CharacterPartRole::Arm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Arm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Forearm(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Forearm(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Hand(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Hand(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Leg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Leg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::LowerLeg(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::LowerLeg(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Foot(Side::Left),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
        CharacterPart::new(
            CharacterPartRole::Foot(Side::Right),
            Vec3::ZERO,
            Quat::IDENTITY,
        ),
    ];

    contexts
        .iter()
        .flat_map(|context| {
            limbs
                .iter()
                .map(|limb| part_pose_with_context(limb, *context, 0.0))
        })
        .map(|pose| pose.translation.length() as f64)
        .fold(0.0, f64::max)
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

fn animation_target_node_names(gltf: &Value, animation_name: &str) -> Option<Vec<String>> {
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
    let channels = animation.get("channels").and_then(Value::as_array)?;
    channels
        .iter()
        .map(|channel| {
            let node = channel.get("target")?.get("node").and_then(Value::as_u64)? as usize;
            node_name(gltf, node).map(str::to_owned)
        })
        .collect()
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

fn symmetric_node_half_width_m(gltf: &Value, left_name: &str, right_name: &str) -> Option<f64> {
    let left_x = world_node_translation(gltf, left_name)?[0];
    let right_x = world_node_translation(gltf, right_name)?[0];

    if left_x < 0.0 && right_x > 0.0 {
        Some((-left_x).min(right_x))
    } else {
        None
    }
}

fn world_node_translation(gltf: &Value, node_name: &str) -> Option<[f64; 3]> {
    let translation = world_node_transform(gltf, node_name)?.transform_point3(Vec3::ZERO);
    Some([
        translation.x as f64,
        translation.y as f64,
        translation.z as f64,
    ])
}

fn world_node_transform(gltf: &Value, node_name: &str) -> Option<Mat4> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let index = node_index(gltf, node_name)?;
    let parents = parent_indices(nodes);
    let mut cursor = Some(index);
    let mut chain = Vec::new();
    while let Some(node_index) = cursor {
        chain.push(node_index);
        cursor = parents[node_index];
    }
    chain
        .into_iter()
        .rev()
        .try_fold(Mat4::IDENTITY, |transform, node_index| {
            Some(transform * node_local_transform_by_index(nodes, node_index)?)
        })
}

fn node_local_transform_by_index(nodes: &[Value], index: usize) -> Option<Mat4> {
    let node = nodes.get(index)?;
    if let Some(matrix) = node.get("matrix").and_then(Value::as_array) {
        let matrix = <[f32; 16]>::try_from(
            matrix
                .iter()
                .map(|value| value.as_f64().map(|value| value as f32))
                .collect::<Option<Vec<_>>>()?,
        )
        .ok()?;
        return Some(Mat4::from_cols_array(&matrix));
    }

    let translation = if let Some(translation) = node.get("translation").and_then(Value::as_array) {
        Vec3::new(
            translation.first()?.as_f64()? as f32,
            translation.get(1)?.as_f64()? as f32,
            translation.get(2)?.as_f64()? as f32,
        )
    } else {
        Vec3::ZERO
    };
    let rotation = if let Some(rotation) = node.get("rotation").and_then(Value::as_array) {
        Quat::from_xyzw(
            rotation.first()?.as_f64()? as f32,
            rotation.get(1)?.as_f64()? as f32,
            rotation.get(2)?.as_f64()? as f32,
            rotation.get(3)?.as_f64()? as f32,
        )
    } else {
        Quat::IDENTITY
    };
    let scale = if let Some(scale) = node.get("scale").and_then(Value::as_array) {
        Vec3::new(
            scale.first()?.as_f64()? as f32,
            scale.get(1)?.as_f64()? as f32,
            scale.get(2)?.as_f64()? as f32,
        )
    } else {
        Vec3::ONE
    };

    Some(Mat4::from_scale_rotation_translation(
        scale,
        rotation,
        translation,
    ))
}

fn node_index(gltf: &Value, node_name: &str) -> Option<usize> {
    gltf.get("nodes")?
        .as_array()?
        .iter()
        .position(|node| node.get("name").and_then(Value::as_str) == Some(node_name))
}

fn node_name(gltf: &Value, index: usize) -> Option<&str> {
    gltf.get("nodes")?
        .as_array()?
        .get(index)?
        .get("name")
        .and_then(Value::as_str)
}

fn node_is_direct_child(gltf: &Value, parent_name: &str, child_name: &str) -> bool {
    let Some(nodes) = gltf.get("nodes").and_then(Value::as_array) else {
        return false;
    };
    let Some(parent) = node_index(gltf, parent_name) else {
        return false;
    };
    let Some(child) = node_index(gltf, child_name) else {
        return false;
    };
    nodes
        .get(parent)
        .and_then(|node| node.get("children"))
        .and_then(Value::as_array)
        .is_some_and(|children| {
            children
                .iter()
                .any(|candidate| candidate.as_u64() == Some(child as u64))
        })
}

fn parent_indices(nodes: &[Value]) -> Vec<Option<usize>> {
    let mut parents = vec![None; nodes.len()];
    for (parent, node) in nodes.iter().enumerate() {
        let Some(children) = node.get("children").and_then(Value::as_array) else {
            continue;
        };
        for child in children {
            if let Some(child) = child.as_u64().map(|child| child as usize)
                && child < parents.len()
            {
                parents[child] = Some(parent);
            }
        }
    }
    parents
}

fn distance3(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn checks_passed(checks: &[Value]) -> bool {
    checks.iter().all(|check| {
        check
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    })
}

fn number_field(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(f64::NAN)
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

fn check_at_least_f64(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value >= threshold,
        "value": value,
        "comparator": ">=",
        "threshold": threshold,
        "unit": unit,
    })
}

fn check_at_most_f64(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": value <= threshold,
        "value": value,
        "comparator": "<=",
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
    fn world_node_transform_applies_parent_rotation() {
        let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2).to_array();
        let gltf = json!({
            "nodes": [
                {"name": "root", "rotation": rotation, "children": [1]},
                {"name": "child", "translation": [1.0, 0.0, 0.0]}
            ]
        });

        let translation = world_node_translation(&gltf, "child").expect("child translation");

        assert!(translation[0].abs() < 0.0001);
        assert!((translation[1] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn world_node_transform_applies_parent_scale() {
        let gltf = json!({
            "nodes": [
                {"name": "root", "scale": [2.0, 3.0, 4.0], "children": [1]},
                {"name": "child", "translation": [1.0, 0.0, 0.0]}
            ]
        });

        let translation = world_node_translation(&gltf, "child").expect("child translation");

        assert!((translation[0] - 2.0).abs() < 0.0001);
        assert!(translation[1].abs() < 0.0001);
    }

    #[test]
    fn world_node_transform_reads_matrix_nodes() {
        let matrix = Mat4::from_translation(Vec3::new(2.0, 3.0, 4.0)).to_cols_array();
        let gltf = json!({
            "nodes": [
                {"name": "root", "matrix": matrix, "children": [1]},
                {"name": "child", "translation": [1.0, 0.0, 0.0]}
            ]
        });

        let translation = world_node_translation(&gltf, "child").expect("child translation");

        assert!((translation[0] - 3.0).abs() < 0.0001);
        assert!((translation[1] - 3.0).abs() < 0.0001);
        assert!((translation[2] - 4.0).abs() < 0.0001);
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
