#![recursion_limit = "256"]

use bevy::prelude::{Mat4, Quat, Vec3};
use nau_engine::animation::{
    CharacterPart, CharacterPartRole, MIN_KEY_POSE_READABILITY_SCORE, PartPose, PlayerPoseContext,
    PlayerPoseIntent, Side, glider_deployment_for_mode, glider_traversal_pose,
    part_pose_with_context, pose_readability_metrics,
};
use nau_engine::asset_pipeline::{
    PLAYER_ANIMATION_CLIP_NAMES, VISUAL_ASSET_SPECS, VisualAssetKind, VisualAssetResidency,
    VisualAssetSpec,
};
use nau_engine::movement::{FlightInput, FlightMode};
use serde_json::{Value, json};
use std::{env, fmt::Write as _, fs, path::Path, process};

const NAU_FIXTURE_SCHEMA: &str = "nau_visual_asset_fixture.v1";
const NAU_FIXTURE_LICENSE: &str = "self_authored_no_third_party";
const PLAYER_POSE_MAX_CONNECTED_LIMB_TRANSLATION_M: f64 = 0.02;
const PLAYER_REST_MAX_ARTICULATED_JOINT_GAP_M: f64 = 0.015;
const PLAYER_REST_MAX_NON_ADJACENT_MESH_OVERLAP_M: f64 = 0.005;
const PLAYER_REST_MAX_SHOULDER_MESH_OVERLAP_M: f64 = 0.015;
const PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M: f64 = 0.018;
const PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M: f64 = 0.035;
const PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M: f64 = 0.11;
const PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M: f64 = 0.012;
const PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M: f64 = 0.08;
const PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M: f64 = 0.008;
const PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M: f64 = 0.004;
const PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M: f64 = 0.001;
const PLAYER_POSE_CONTACT_EXPECTED_POSE_COUNT: f64 = 6.0;
const PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT: f64 = 4.0;
const PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT: f64 = 12.0;
const PLAYER_JOINT_BRIDGE_EXPECTED_PAIR_COUNT: f64 = 12.0;
const PLAYER_JOINT_SEAM_EXPECTED_NODE_COUNT: f64 = 12.0;
const PLAYER_JOINT_SEAM_EXPECTED_PAIR_COUNT: f64 = 26.0;
const PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT: f64 = 9.0;
const PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT: f64 = 4.0;
const PLAYER_POSE_MIN_FALLING_TORSO_PITCH_DEGREES: f64 = 72.0;
const PLAYER_POSE_MIN_FALLING_ARM_SPREAD_DEGREES: f64 = 150.0;
const PLAYER_POSE_MIN_DIVE_TORSO_PITCH_DEGREES: f64 = 82.0;
const PLAYER_POSE_MAX_DIVE_ARM_SPREAD_DEGREES: f64 = 74.0;
const PLAYER_POSE_MIN_DIVE_LEG_TUCK_DEGREES: f64 = 68.0;
const PLAYER_GLIDER_MIN_LAUNCH_DEPLOYMENT: f64 = 0.45;
const PLAYER_GLIDER_MAX_LAUNCH_DEPLOYMENT: f64 = 0.70;
const PLAYER_GLIDER_MIN_LAUNCH_RESPONSE_DEGREES: f64 = 8.0;
const PLAYER_GLIDER_MIN_LAUNCH_MOTION_M: f64 = 0.18;
const PLAYER_GLIDER_MIN_DIVE_RESPONSE_DEGREES: f64 = 4.0;
const PLAYER_GLIDER_MIN_DIVE_MOTION_M: f64 = 0.16;
const PLAYER_REST_TORSO_BLOCKING_MESH_NODES: &[&str] = &[
    "Nau Suit Armored Torso Shell",
    "Nau Suit Tapered Hips Shell",
    "Nau Suit Shoulder Yoke Plate",
    "Nau Left Suit Collarbone Plate",
    "Nau Right Suit Collarbone Plate",
    "Nau Suit Pelvis Hip Yoke",
    "Nau Left Suit Pelvis Side Plate",
    "Nau Right Suit Pelvis Side Plate",
    "Nau Skin Rounded Head",
];
const PLAYER_REST_LEFT_ARM_MESH_NODES: &[&str] = &[
    "Nau Left Suit Upper Arm",
    "Nau Left Suit Deltoid Filler",
    "Nau Left Leather Forearm Wrap",
    "Nau Left Leather Hand Palm",
    "Nau Left Leather Palm Heel Pad",
    "Nau Left Leather Index Finger Grip",
    "Nau Left Leather Finger Grip",
    "Nau Left Leather Ring Finger Grip",
    "Nau Left Leather Thumb Grip",
    "Nau Left Leather Index Knuckle Pad",
    "Nau Left Leather Middle Knuckle Pad",
    "Nau Left Leather Ring Knuckle Pad",
    "Nau Left Leather Pinky Knuckle Pad",
];
const PLAYER_REST_RIGHT_ARM_MESH_NODES: &[&str] = &[
    "Nau Right Suit Upper Arm",
    "Nau Right Suit Deltoid Filler",
    "Nau Right Leather Forearm Wrap",
    "Nau Right Leather Hand Palm",
    "Nau Right Leather Palm Heel Pad",
    "Nau Right Leather Index Finger Grip",
    "Nau Right Leather Finger Grip",
    "Nau Right Leather Ring Finger Grip",
    "Nau Right Leather Thumb Grip",
    "Nau Right Leather Index Knuckle Pad",
    "Nau Right Leather Middle Knuckle Pad",
    "Nau Right Leather Ring Knuckle Pad",
    "Nau Right Leather Pinky Knuckle Pad",
];
const PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES: &[&str] = &[
    "Nau Left Leather Forearm Wrap",
    "Nau Left Leather Hand Palm",
    "Nau Left Leather Palm Heel Pad",
    "Nau Left Leather Index Finger Grip",
    "Nau Left Leather Finger Grip",
    "Nau Left Leather Ring Finger Grip",
    "Nau Left Leather Thumb Grip",
    "Nau Left Leather Index Knuckle Pad",
    "Nau Left Leather Middle Knuckle Pad",
    "Nau Left Leather Ring Knuckle Pad",
    "Nau Left Leather Pinky Knuckle Pad",
];
const PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES: &[&str] = &[
    "Nau Right Leather Forearm Wrap",
    "Nau Right Leather Hand Palm",
    "Nau Right Leather Palm Heel Pad",
    "Nau Right Leather Index Finger Grip",
    "Nau Right Leather Finger Grip",
    "Nau Right Leather Ring Finger Grip",
    "Nau Right Leather Thumb Grip",
    "Nau Right Leather Index Knuckle Pad",
    "Nau Right Leather Middle Knuckle Pad",
    "Nau Right Leather Ring Knuckle Pad",
    "Nau Right Leather Pinky Knuckle Pad",
];
const PLAYER_REST_LEFT_LEG_MESH_NODES: &[&str] = &[
    "Nau Left Suit Thigh Guard",
    "Nau Left Suit Lower Leg Greave",
    "Nau Left Leather Boot Shell",
    "Nau Left Leather Boot Toe Cap",
    "Nau Left Leather Boot Sole",
    "Nau Left Leather Boot Heel",
];
const PLAYER_REST_RIGHT_LEG_MESH_NODES: &[&str] = &[
    "Nau Right Suit Thigh Guard",
    "Nau Right Suit Lower Leg Greave",
    "Nau Right Leather Boot Shell",
    "Nau Right Leather Boot Toe Cap",
    "Nau Right Leather Boot Sole",
    "Nau Right Leather Boot Heel",
];
const PLAYER_REST_LEFT_DISTAL_LEG_MESH_NODES: &[&str] = &[
    "Nau Left Suit Lower Leg Greave",
    "Nau Left Leather Boot Shell",
    "Nau Left Leather Boot Toe Cap",
    "Nau Left Leather Boot Sole",
    "Nau Left Leather Boot Heel",
];
const PLAYER_REST_RIGHT_DISTAL_LEG_MESH_NODES: &[&str] = &[
    "Nau Right Suit Lower Leg Greave",
    "Nau Right Leather Boot Shell",
    "Nau Right Leather Boot Toe Cap",
    "Nau Right Leather Boot Sole",
    "Nau Right Leather Boot Heel",
];
const PLAYER_RUNTIME_POSE_NODE_ROLES: &[(&str, CharacterPartRole)] = &[
    ("Nau Torso", CharacterPartRole::Torso),
    ("Nau Head", CharacterPartRole::Head),
    ("Nau Left Arm", CharacterPartRole::Arm(Side::Left)),
    ("Nau Right Arm", CharacterPartRole::Arm(Side::Right)),
    ("Nau Left Forearm", CharacterPartRole::Forearm(Side::Left)),
    ("Nau Right Forearm", CharacterPartRole::Forearm(Side::Right)),
    (
        "Nau Left Leather Hand Palm",
        CharacterPartRole::Hand(Side::Left),
    ),
    (
        "Nau Right Leather Hand Palm",
        CharacterPartRole::Hand(Side::Right),
    ),
    ("Nau Left Leg", CharacterPartRole::Leg(Side::Left)),
    ("Nau Right Leg", CharacterPartRole::Leg(Side::Right)),
    (
        "Nau Left Lower Leg",
        CharacterPartRole::LowerLeg(Side::Left),
    ),
    (
        "Nau Right Lower Leg",
        CharacterPartRole::LowerLeg(Side::Right),
    ),
    ("Nau Left Boot", CharacterPartRole::Foot(Side::Left)),
    ("Nau Right Boot", CharacterPartRole::Foot(Side::Right)),
];

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
    "suit",
    "skin",
    "accent",
    "helmet",
    "shoulder",
    "scarf",
    "boot",
    "face",
    "eye",
    "belt",
    "gauntlet",
    "knee",
    "hand",
    "finger",
    "toe",
    "neck",
    "elbow",
    "ankle",
    "bridge",
    "sleeve",
    "yoke",
    "collarbone",
    "pelvis",
    "deltoid",
    "knuckle",
    "sole",
    "heel",
    "seamless",
    "flex",
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
        min_nodes: 155,
        min_meshes: 52,
        min_materials: 8,
        min_vertices: 8400,
        min_triangles: 15000,
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
    let args = env::args().skip(1).collect::<Vec<_>>();
    if let Some(command) = args.first() {
        if command == "--export-player-pose-preview" {
            let output_dir = args
                .get(1)
                .map(Path::new)
                .unwrap_or_else(|| Path::new("target/player_pose_previews"));
            match export_player_pose_preview(output_dir) {
                Ok(report) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report)
                            .expect("preview report should serialize")
                    );
                }
                Err(error) => {
                    eprintln!("player pose preview export failed: {error}");
                    process::exit(2);
                }
            }
            return;
        }

        eprintln!("unknown asset fixture audit command: {command}");
        process::exit(2);
    }

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

#[derive(Clone, Copy)]
struct PlayerPosePreviewSpec {
    label: &'static str,
    title: &'static str,
    context: PlayerPoseContext,
    phase: f32,
}

#[derive(Clone, Debug)]
struct PlayerPosePreviewShape {
    node_name: String,
    bounds: Aabb3,
    color: &'static str,
}

#[derive(Clone, Copy)]
enum PlayerPosePreviewView {
    Front,
    Side,
    Top,
}

fn export_player_pose_preview(output_dir: &Path) -> Result<Value, String> {
    let path = Path::new("assets/models/player/player.gltf");
    let text =
        fs::read_to_string(path).map_err(|error| format!("failed to read {path:?}: {error}"))?;
    let gltf = serde_json::from_str::<Value>(&text)
        .map_err(|error| format!("invalid glTF JSON: {error}"))?;
    let specs = player_pose_preview_specs();
    let svg = render_player_pose_preview_sheet(&gltf, &specs)?;
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {output_dir:?}: {error}"))?;
    let sheet_path = output_dir.join("player_pose_sheet.svg");
    fs::write(&sheet_path, svg)
        .map_err(|error| format!("failed to write {sheet_path:?}: {error}"))?;

    let manifest = json!({
        "schema": "nau_player_pose_preview.v1",
        "source": path,
        "pose_count": specs.len(),
        "views": ["front", "side", "top"],
        "phase_samples": specs.iter().map(|spec| spec.phase).collect::<Vec<_>>(),
        "poses": specs.iter().map(|spec| json!({
            "label": spec.label,
            "title": spec.title,
            "phase": spec.phase,
            "pose_intent": spec.context.intent().label(),
        })).collect::<Vec<_>>(),
        "joint_seam_contact_audit": player_joint_seam_contact_audit(&gltf),
        "artifacts": {
            "pose_sheet_svg": sheet_path,
        },
    });
    let manifest_path = output_dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("preview manifest should serialize"),
    )
    .map_err(|error| format!("failed to write {manifest_path:?}: {error}"))?;

    Ok(json!({
        "passed": true,
        "manifest": manifest_path,
        "pose_sheet_svg": sheet_path,
        "pose_count": specs.len(),
    }))
}

fn player_pose_preview_specs() -> Vec<PlayerPosePreviewSpec> {
    vec![
        PlayerPosePreviewSpec {
            label: "grounded_idle",
            title: "Grounded Idle",
            context: PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::ZERO,
                FlightInput::default(),
                0.0,
            )
            .with_resolved_intent(PlayerPoseIntent::GroundedIdle),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "launch_takeout",
            title: "Launch Takeout",
            context: PlayerPoseContext::new(
                FlightMode::Launching,
                Vec3::new(0.0, 24.0, -18.0),
                FlightInput::default(),
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Launching),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "falling_belly_down",
            title: "Belly-Down Fall",
            context: PlayerPoseContext::new(
                FlightMode::Airborne,
                Vec3::new(0.0, -22.0, -24.0),
                FlightInput::default(),
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Falling),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "gliding",
            title: "Glide",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -4.0, -30.0),
                FlightInput {
                    glide: true,
                    ..FlightInput::default()
                },
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Gliding),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "diving_head_down",
            title: "Head-Down Dive",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -34.0, -36.0),
                FlightInput {
                    glide: true,
                    dive: true,
                    ..FlightInput::default()
                },
                120.0,
            )
            .with_resolved_intent(PlayerPoseIntent::Diving),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "air_brake",
            title: "Air Brake",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(0.0, -8.0, -24.0),
                FlightInput {
                    glide: true,
                    backward: true,
                    ..FlightInput::default()
                },
                80.0,
            )
            .with_resolved_intent(PlayerPoseIntent::AirBrake),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "landing_anticipation",
            title: "Landing Anticipation",
            context: PlayerPoseContext::new(
                FlightMode::Gliding,
                Vec3::new(3.0, -20.0, -22.0),
                FlightInput {
                    glide: true,
                    ..FlightInput::default()
                },
                1.5,
            )
            .with_resolved_intent(PlayerPoseIntent::LandingAnticipation),
            phase: 0.75,
        },
        PlayerPosePreviewSpec {
            label: "landing_recovery",
            title: "Landing Recovery",
            context: PlayerPoseContext::new(
                FlightMode::Grounded,
                Vec3::new(2.0, 0.0, -7.0),
                FlightInput::default(),
                0.0,
            )
            .with_resolved_intent(PlayerPoseIntent::LandingRecovery),
            phase: 0.75,
        },
    ]
}

fn render_player_pose_preview_sheet(
    gltf: &Value,
    specs: &[PlayerPosePreviewSpec],
) -> Result<String, String> {
    const ROW_HEIGHT: f32 = 210.0;
    const LABEL_WIDTH: f32 = 150.0;
    const VIEW_WIDTH: f32 = 250.0;
    const HEADER_HEIGHT: f32 = 58.0;
    const PADDING: f32 = 18.0;

    let width = LABEL_WIDTH + VIEW_WIDTH * 3.0 + PADDING * 2.0;
    let height = HEADER_HEIGHT + ROW_HEIGHT * specs.len() as f32 + PADDING;
    let mut svg = String::new();
    writeln!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.0}\" height=\"{height:.0}\" viewBox=\"0 0 {width:.0} {height:.0}\">"
    )
    .expect("writing to string should not fail");
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"#10151f\"/>\n");
    svg.push_str("<text x=\"18\" y=\"24\" fill=\"#dbe7f3\" font-family=\"Menlo, monospace\" font-size=\"16\">NAU player fixture pose preview</text>\n");
    for (index, view) in [
        PlayerPosePreviewView::Front,
        PlayerPosePreviewView::Side,
        PlayerPosePreviewView::Top,
    ]
    .iter()
    .enumerate()
    {
        let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * index as f32 + VIEW_WIDTH * 0.5;
        writeln!(
            svg,
            "<text x=\"{x:.1}\" y=\"44\" fill=\"#8fb1c9\" text-anchor=\"middle\" font-family=\"Menlo, monospace\" font-size=\"12\">{}</text>",
            view.label()
        )
        .expect("writing to string should not fail");
    }

    for (row, spec) in specs.iter().enumerate() {
        let y = HEADER_HEIGHT + ROW_HEIGHT * row as f32;
        let overrides = player_pose_node_overrides(gltf, spec.context, spec.phase)
            .ok_or_else(|| format!("failed to compute pose overrides for {}", spec.label))?;
        let shapes = player_pose_preview_shapes(gltf, &overrides)
            .ok_or_else(|| format!("failed to compute preview shapes for {}", spec.label))?;
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#edf5ff\" font-family=\"Menlo, monospace\" font-size=\"13\">{}</text>",
            y + 28.0,
            escape_xml(spec.title)
        )
        .expect("writing to string should not fail");
        writeln!(
            svg,
            "<text x=\"18\" y=\"{:.1}\" fill=\"#7f95a7\" font-family=\"Menlo, monospace\" font-size=\"10\">intent: {}</text>",
            y + 46.0,
            spec.context.intent().label()
        )
        .expect("writing to string should not fail");

        for (column, view) in [
            PlayerPosePreviewView::Front,
            PlayerPosePreviewView::Side,
            PlayerPosePreviewView::Top,
        ]
        .iter()
        .enumerate()
        {
            let x = LABEL_WIDTH + PADDING + VIEW_WIDTH * column as f32;
            render_player_pose_preview_view(
                &mut svg,
                &shapes,
                *view,
                x,
                y + 12.0,
                VIEW_WIDTH - 12.0,
                ROW_HEIGHT - 20.0,
            );
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

fn player_pose_preview_shapes(
    gltf: &Value,
    overrides: &[PoseNodeOverride],
) -> Option<Vec<PlayerPosePreviewShape>> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let mut shapes = Vec::new();
    for node in nodes {
        let node_name = node.get("name").and_then(Value::as_str)?;
        if node.get("mesh").is_none() {
            continue;
        }
        let bounds = node_world_mesh_aabb_with_pose(gltf, node_name, overrides)?;
        shapes.push(PlayerPosePreviewShape {
            node_name: node_name.to_string(),
            bounds,
            color: player_pose_preview_color(node_name),
        });
    }
    Some(shapes)
}

fn render_player_pose_preview_view(
    svg: &mut String,
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    writeln!(
        svg,
        "<rect x=\"{x:.1}\" y=\"{y:.1}\" width=\"{width:.1}\" height=\"{height:.1}\" rx=\"4\" fill=\"#151d29\" stroke=\"#263749\" stroke-width=\"1\"/>"
    )
    .expect("writing to string should not fail");
    let Some((min_u, max_u, min_v, max_v)) = preview_projected_extent(shapes, view) else {
        return;
    };
    let span_u = (max_u - min_u).max(0.01);
    let span_v = (max_v - min_v).max(0.01);
    let scale = ((width - 28.0) / span_u).min((height - 28.0) / span_v);
    let origin_x = x + width * 0.5 - (min_u + max_u) * 0.5 * scale;
    let origin_y = y + height * 0.5 + (min_v + max_v) * 0.5 * scale;

    let mut ordered = shapes.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        preview_depth(left.bounds, view)
            .partial_cmp(&preview_depth(right.bounds, view))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for shape in ordered {
        let (shape_min_u, shape_max_u, shape_min_v, shape_max_v) =
            project_preview_bounds(shape.bounds, view);
        let rect_x = origin_x + shape_min_u * scale;
        let rect_y = origin_y - shape_max_v * scale;
        let rect_width = (shape_max_u - shape_min_u).max(0.004) * scale;
        let rect_height = (shape_max_v - shape_min_v).max(0.004) * scale;
        writeln!(
            svg,
            "<rect x=\"{rect_x:.2}\" y=\"{rect_y:.2}\" width=\"{rect_width:.2}\" height=\"{rect_height:.2}\" rx=\"2\" fill=\"{}\" fill-opacity=\"0.68\" stroke=\"#e6eef7\" stroke-opacity=\"0.35\" stroke-width=\"0.5\"><title>{}</title></rect>",
            shape.color,
            escape_xml(&shape.node_name)
        )
        .expect("writing to string should not fail");
    }
}

fn preview_projected_extent(
    shapes: &[PlayerPosePreviewShape],
    view: PlayerPosePreviewView,
) -> Option<(f32, f32, f32, f32)> {
    shapes
        .iter()
        .map(|shape| project_preview_bounds(shape.bounds, view))
        .reduce(|accumulator, bounds| {
            (
                accumulator.0.min(bounds.0),
                accumulator.1.max(bounds.1),
                accumulator.2.min(bounds.2),
                accumulator.3.max(bounds.3),
            )
        })
}

fn project_preview_bounds(bounds: Aabb3, view: PlayerPosePreviewView) -> (f32, f32, f32, f32) {
    match view {
        PlayerPosePreviewView::Front => (bounds.min.x, bounds.max.x, bounds.min.y, bounds.max.y),
        PlayerPosePreviewView::Side => (bounds.min.z, bounds.max.z, bounds.min.y, bounds.max.y),
        PlayerPosePreviewView::Top => (bounds.min.x, bounds.max.x, bounds.min.z, bounds.max.z),
    }
}

fn preview_depth(bounds: Aabb3, view: PlayerPosePreviewView) -> f32 {
    match view {
        PlayerPosePreviewView::Front => (bounds.min.z + bounds.max.z) * 0.5,
        PlayerPosePreviewView::Side => (bounds.min.x + bounds.max.x) * 0.5,
        PlayerPosePreviewView::Top => (bounds.min.y + bounds.max.y) * 0.5,
    }
}

impl PlayerPosePreviewView {
    fn label(self) -> &'static str {
        match self {
            Self::Front => "front silhouette",
            Self::Side => "side silhouette",
            Self::Top => "top footprint",
        }
    }
}

fn player_pose_preview_color(node_name: &str) -> &'static str {
    let name = node_name.to_ascii_lowercase();
    if name.contains("skin") || name.contains("face") {
        "#c98a62"
    } else if name.contains("eye") || name.contains("focus") {
        "#ff9d2d"
    } else if name.contains("accent") || name.contains("scarf") || name.contains("tunic") {
        "#168894"
    } else if name.contains("belt") || name.contains("collarbone") || name.contains("pelvis side") {
        "#b88738"
    } else if name.contains("leather") || name.contains("boot") {
        "#3d281f"
    } else {
        "#26384f"
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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
    let player_pose_contact_audit = requirement
        .require_player_clips
        .then(|| player_pose_contact_audit(&gltf))
        .flatten();
    let player_pose_transition_contact_audit = requirement
        .require_player_clips
        .then(|| player_pose_transition_contact_audit(&gltf))
        .flatten();
    let player_joint_bridge_contact_audit = requirement
        .require_player_clips
        .then(|| player_joint_bridge_contact_audit(&gltf))
        .flatten();
    let player_joint_seam_contact_audit = requirement
        .require_player_clips
        .then(|| player_joint_seam_contact_audit(&gltf))
        .flatten();

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
            "player_joint_bridge_nodes_present",
            player_joint_bridge_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_joint_seam_nodes_present",
            player_joint_seam_nodes_present(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_rest_limb_attachment_hierarchy",
            player_rest_limb_attachment_hierarchy_valid(&gltf),
            "nodes",
        ));
        checks.push(check_bool(
            "player_rest_mesh_bounds_present",
            player_rest_mesh_bounds_present(&gltf),
            "mesh_bounds",
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
        checks.push(check_at_most_f64(
            "player_rest_non_adjacent_mesh_overlap_max",
            player_rest_non_adjacent_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_REST_MAX_NON_ADJACENT_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_rest_shoulder_mesh_overlap_max",
            player_rest_shoulder_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_REST_MAX_SHOULDER_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_non_adjacent_mesh_overlap_max",
            player_pose_non_adjacent_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_articulated_joint_gap_max",
            player_pose_articulated_joint_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_cover_mesh_gap_max",
            player_pose_joint_cover_mesh_gap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_cover_mesh_overlap_max",
            player_pose_joint_cover_mesh_overlap_max_m(&gltf).unwrap_or(f64::INFINITY),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "m",
        ));
        let bridge_contact = player_joint_bridge_contact_audit
            .as_ref()
            .expect("player joint bridge contact audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_joint_bridge_contact_node_count",
            number_field(bridge_contact, "bridge_node_count"),
            PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT,
            "nodes",
        ));
        checks.push(check_eq_f64(
            "player_joint_bridge_contact_pair_count",
            number_field(bridge_contact, "pair_count"),
            PLAYER_JOINT_BRIDGE_EXPECTED_PAIR_COUNT,
            "pairs",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_bridge_mesh_gap_max",
            number_field(bridge_contact, "max_gap_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_bridge_mesh_overlap_max",
            number_field(bridge_contact, "max_overlap_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_joint_bridge_contact_breach_count",
            number_field(bridge_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let seam_contact = player_joint_seam_contact_audit
            .as_ref()
            .expect("player joint seam contact audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_joint_seam_contact_node_count",
            number_field(seam_contact, "seam_node_count"),
            PLAYER_JOINT_SEAM_EXPECTED_NODE_COUNT,
            "nodes",
        ));
        checks.push(check_eq_f64(
            "player_joint_seam_contact_pair_count",
            number_field(seam_contact, "pair_count"),
            PLAYER_JOINT_SEAM_EXPECTED_PAIR_COUNT,
            "pairs",
        ));
        checks.push(check_at_most_f64(
            "player_pose_joint_seam_mesh_gap_max",
            number_field(seam_contact, "max_gap_m"),
            PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_least_f64(
            "player_pose_joint_seam_mesh_overlap_min",
            number_field(seam_contact, "min_overlap_m"),
            PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_joint_seam_contact_breach_count",
            number_field(seam_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let pose_contact = player_pose_contact_audit
            .as_ref()
            .expect("player pose contact audit should be present for player fixture");
        checks.push(check_at_least_f64(
            "player_pose_contact_report_pose_count",
            number_field(pose_contact, "pose_count"),
            PLAYER_POSE_CONTACT_EXPECTED_POSE_COUNT,
            "poses",
        ));
        checks.push(check_at_least_f64(
            "player_pose_contact_report_phase_count",
            number_field(pose_contact, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT,
            "phases",
        ));
        checks.push(check_at_most_f64(
            "player_pose_contact_report_breach_count",
            number_field(pose_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        let transition_contact = player_pose_transition_contact_audit
            .as_ref()
            .expect("player pose transition contact audit should be present for player fixture");
        checks.push(check_eq_f64(
            "player_pose_transition_contact_report_transition_count",
            number_field(transition_contact, "transition_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT,
            "transitions",
        ));
        checks.push(check_eq_f64(
            "player_pose_transition_contact_report_blend_count",
            number_field(transition_contact, "blend_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT,
            "blends",
        ));
        checks.push(check_eq_f64(
            "player_pose_transition_contact_report_phase_count",
            number_field(transition_contact, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT,
            "phases",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_contact_report_breach_count",
            number_field(transition_contact, "breach_count"),
            0.0,
            "breaches",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_articulated_joint_gap_max",
            number_field(transition_contact, "articulated_joint_gap_max_m"),
            PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_cover_mesh_gap_max",
            number_field(transition_contact, "joint_cover_mesh_gap_max_m"),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_cover_mesh_overlap_max",
            number_field(transition_contact, "joint_cover_mesh_overlap_max_m"),
            PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_bridge_mesh_gap_max",
            number_field(transition_contact, "joint_bridge_mesh_gap_max_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_joint_bridge_mesh_overlap_max",
            number_field(transition_contact, "joint_bridge_mesh_overlap_max_m"),
            PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
            "m",
        ));
        checks.push(check_at_most_f64(
            "player_pose_transition_non_adjacent_mesh_overlap_max",
            number_field(transition_contact, "non_adjacent_mesh_overlap_max_m"),
            PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
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
            "player_glider_launch_takeout_response",
            number_field(pose_shape, "glider_launch_response_degrees"),
            PLAYER_GLIDER_MIN_LAUNCH_RESPONSE_DEGREES,
            "deg",
        ));
        checks.push(check_at_least_f64(
            "player_glider_launch_takeout_motion",
            number_field(pose_shape, "glider_launch_motion_m"),
            PLAYER_GLIDER_MIN_LAUNCH_MOTION_M,
            "m",
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
        "player_joint_bridge_nodes_present": player_joint_bridge_nodes_present(&gltf),
        "player_joint_seam_nodes_present": player_joint_seam_nodes_present(&gltf),
        "player_rest_limb_attachment_hierarchy": player_rest_limb_attachment_hierarchy_valid(&gltf),
        "player_rest_mesh_bounds_present": player_rest_mesh_bounds_present(&gltf),
        "player_animation_channels_cover_core_signals": player_animation_channels_cover_core_signals(&gltf),
        "player_animation_channels_avoid_runtime_pose_nodes": player_animation_channels_avoid_runtime_pose_nodes(&gltf),
        "player_rest_joint_gap_max_m": player_rest_joint_gap_max_m(&gltf),
        "player_rest_articulated_joint_gap_max_m": player_rest_articulated_joint_gap_max_m(&gltf),
        "player_rest_non_adjacent_mesh_overlap_max_m": player_rest_non_adjacent_mesh_overlap_max_m(&gltf),
        "player_rest_shoulder_mesh_overlap_max_m": player_rest_shoulder_mesh_overlap_max_m(&gltf),
        "player_pose_non_adjacent_mesh_overlap_max_m": player_pose_non_adjacent_mesh_overlap_max_m(&gltf),
        "player_pose_non_adjacent_mesh_overlap_worst_pair": player_pose_non_adjacent_mesh_overlap_report(&gltf).map(|report| report.to_json()),
        "player_pose_articulated_joint_gap_max_m": player_pose_articulated_joint_gap_max_m(&gltf),
        "player_pose_articulated_joint_gap_worst_pair": player_pose_articulated_joint_gap_report(&gltf).map(|report| report.to_json()),
        "player_pose_joint_cover_mesh_gap_max_m": player_pose_joint_cover_mesh_gap_max_m(&gltf),
        "player_pose_joint_cover_mesh_gap_worst_pair": player_pose_joint_cover_mesh_gap_report(&gltf).map(|report| report.to_json()),
        "player_pose_joint_cover_mesh_overlap_max_m": player_pose_joint_cover_mesh_overlap_max_m(&gltf),
        "player_pose_joint_cover_mesh_overlap_worst_pair": player_pose_joint_cover_mesh_overlap_report(&gltf).map(|report| report.to_json()),
        "player_joint_bridge_contact_audit": player_joint_bridge_contact_audit,
        "player_joint_seam_contact_audit": player_joint_seam_contact_audit,
        "player_pose_contact_audit": player_pose_contact_audit,
        "player_pose_transition_contact_audit": player_pose_transition_contact_audit,
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

#[derive(Clone, Copy, Debug)]
struct Aabb3 {
    min: Vec3,
    max: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct Obb3 {
    center: Vec3,
    axes: [Vec3; 3],
    half_extents: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct PoseNodeOverride {
    node_name: &'static str,
    pose: PartPose,
}

#[derive(Clone, Copy, Debug)]
struct PlayerPoseTransition {
    label: &'static str,
    from: PlayerPoseContext,
    to: PlayerPoseContext,
}

#[derive(Clone, Copy, Debug)]
struct MeshOverlapReport {
    max_overlap_m: f64,
    overlap_axes_m: [f64; 3],
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl MeshOverlapReport {
    fn zero() -> Self {
        Self {
            max_overlap_m: 0.0,
            overlap_axes_m: [0.0; 3],
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe(
        &mut self,
        overlap_m: f64,
        overlap_axes_m: [f64; 3],
        left_node: &'static str,
        right_node: &'static str,
        pose_intent: PlayerPoseIntent,
        phase: f32,
    ) {
        if overlap_m > self.max_overlap_m {
            self.max_overlap_m = overlap_m;
            self.overlap_axes_m = overlap_axes_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_intent.label();
            self.phase = phase;
        }
    }

    fn observe_label(
        &mut self,
        overlap_m: f64,
        overlap_axes_m: [f64; 3],
        left_node: &'static str,
        right_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if overlap_m > self.max_overlap_m {
            self.max_overlap_m = overlap_m;
            self.overlap_axes_m = overlap_axes_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_overlap_m": self.max_overlap_m,
            "overlap_x_m": self.overlap_axes_m[0],
            "overlap_y_m": self.overlap_axes_m[1],
            "overlap_z_m": self.overlap_axes_m[2],
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct JointGapReport {
    max_gap_m: f64,
    socket_node: &'static str,
    joint_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl JointGapReport {
    fn zero() -> Self {
        Self {
            max_gap_m: 0.0,
            socket_node: "",
            joint_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe(
        &mut self,
        gap_m: f64,
        socket_node: &'static str,
        joint_node: &'static str,
        pose_intent: PlayerPoseIntent,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.socket_node = socket_node;
            self.joint_node = joint_node;
            self.pose_intent = pose_intent.label();
            self.phase = phase;
        }
    }

    fn observe_label(
        &mut self,
        gap_m: f64,
        socket_node: &'static str,
        joint_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.socket_node = socket_node;
            self.joint_node = joint_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_gap_m": self.max_gap_m,
            "socket_node": self.socket_node,
            "joint_node": self.joint_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct MeshGapReport {
    max_gap_m: f64,
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl MeshGapReport {
    fn zero() -> Self {
        Self {
            max_gap_m: 0.0,
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe(
        &mut self,
        gap_m: f64,
        left_node: &'static str,
        right_node: &'static str,
        pose_intent: PlayerPoseIntent,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_intent.label();
            self.phase = phase;
        }
    }

    fn observe_label(
        &mut self,
        gap_m: f64,
        left_node: &'static str,
        right_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if gap_m > self.max_gap_m {
            self.max_gap_m = gap_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn to_json(self) -> Value {
        json!({
            "max_gap_m": self.max_gap_m,
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct MeshMinOverlapReport {
    min_overlap_m: f64,
    left_node: &'static str,
    right_node: &'static str,
    pose_intent: &'static str,
    phase: f32,
}

impl MeshMinOverlapReport {
    fn zero() -> Self {
        Self {
            min_overlap_m: f64::INFINITY,
            left_node: "",
            right_node: "",
            pose_intent: "none",
            phase: 0.0,
        }
    }

    fn observe_label(
        &mut self,
        overlap_m: f64,
        left_node: &'static str,
        right_node: &'static str,
        pose_label: &'static str,
        phase: f32,
    ) {
        if overlap_m < self.min_overlap_m {
            self.min_overlap_m = overlap_m;
            self.left_node = left_node;
            self.right_node = right_node;
            self.pose_intent = pose_label;
            self.phase = phase;
        }
    }

    fn value(self) -> f64 {
        if self.min_overlap_m.is_finite() {
            self.min_overlap_m
        } else {
            0.0
        }
    }

    fn to_json(self) -> Value {
        json!({
            "min_overlap_m": self.value(),
            "left_node": self.left_node,
            "right_node": self.right_node,
            "pose_intent": self.pose_intent,
            "phase": self.phase,
        })
    }
}

impl Aabb3 {
    fn from_min_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    fn include_aabb(&mut self, bounds: Self) {
        self.min = self.min.min(bounds.min);
        self.max = self.max.max(bounds.max);
    }

    fn include_point(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }

    fn transformed(self, transform: Mat4) -> Self {
        let mut transformed = Self::from_min_max(
            transform.transform_point3(Vec3::new(self.min.x, self.min.y, self.min.z)),
            transform.transform_point3(Vec3::new(self.min.x, self.min.y, self.min.z)),
        );
        for x in [self.min.x, self.max.x] {
            for y in [self.min.y, self.max.y] {
                for z in [self.min.z, self.max.z] {
                    transformed.include_point(transform.transform_point3(Vec3::new(x, y, z)));
                }
            }
        }
        transformed
    }

    fn transformed_obb(self, transform: Mat4) -> Obb3 {
        let local_center = (self.min + self.max) * 0.5;
        let local_half_extents = (self.max - self.min) * 0.5;
        let transformed_axes = [
            transform.transform_vector3(Vec3::X * local_half_extents.x),
            transform.transform_vector3(Vec3::Y * local_half_extents.y),
            transform.transform_vector3(Vec3::Z * local_half_extents.z),
        ];
        Obb3 {
            center: transform.transform_point3(local_center),
            axes: transformed_axes.map(|axis| axis.normalize_or_zero()),
            half_extents: Vec3::new(
                transformed_axes[0].length(),
                transformed_axes[1].length(),
                transformed_axes[2].length(),
            ),
        }
    }

    fn overlap_depth_m(self, other: Self) -> f64 {
        let overlap = self.overlap_axes_m(other);
        if overlap.iter().all(|depth| *depth > 0.0) {
            overlap.into_iter().fold(f64::INFINITY, f64::min)
        } else {
            0.0
        }
    }

    fn overlap_axes_m(self, other: Self) -> [f64; 3] {
        [
            self.max.x.min(other.max.x) - self.min.x.max(other.min.x),
            self.max.y.min(other.max.y) - self.min.y.max(other.min.y),
            self.max.z.min(other.max.z) - self.min.z.max(other.min.z),
        ]
        .map(|depth| depth.max(0.0) as f64)
    }

    fn separation_m(self, other: Self) -> f64 {
        let gap = Vec3::new(
            axis_separation_m(self.min.x, self.max.x, other.min.x, other.max.x),
            axis_separation_m(self.min.y, self.max.y, other.min.y, other.max.y),
            axis_separation_m(self.min.z, self.max.z, other.min.z, other.max.z),
        );
        gap.length() as f64
    }
}

fn axis_separation_m(left_min: f32, left_max: f32, right_min: f32, right_max: f32) -> f32 {
    if left_max < right_min {
        right_min - left_max
    } else if right_max < left_min {
        left_min - right_max
    } else {
        0.0
    }
}

impl Obb3 {
    fn overlap_depth_m(self, other: Self) -> f64 {
        let mut min_overlap = f32::INFINITY;
        for axis in self.separating_axes(other) {
            if axis.length_squared() <= 1e-8 {
                continue;
            }
            let axis = axis.normalize();
            let center_distance = (other.center - self.center).dot(axis).abs();
            let overlap =
                self.projected_radius(axis) + other.projected_radius(axis) - center_distance;
            if overlap <= 0.0 {
                return 0.0;
            }
            min_overlap = min_overlap.min(overlap);
        }
        min_overlap as f64
    }

    fn projected_radius(self, axis: Vec3) -> f32 {
        self.axes[0].dot(axis).abs() * self.half_extents.x
            + self.axes[1].dot(axis).abs() * self.half_extents.y
            + self.axes[2].dot(axis).abs() * self.half_extents.z
    }

    fn separating_axes(self, other: Self) -> [Vec3; 15] {
        [
            self.axes[0],
            self.axes[1],
            self.axes[2],
            other.axes[0],
            other.axes[1],
            other.axes[2],
            self.axes[0].cross(other.axes[0]),
            self.axes[0].cross(other.axes[1]),
            self.axes[0].cross(other.axes[2]),
            self.axes[1].cross(other.axes[0]),
            self.axes[1].cross(other.axes[1]),
            self.axes[1].cross(other.axes[2]),
            self.axes[2].cross(other.axes[0]),
            self.axes[2].cross(other.axes[1]),
            self.axes[2].cross(other.axes[2]),
        ]
    }
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

fn player_joint_bridge_nodes_present(gltf: &Value) -> bool {
    player_joint_bridge_node_names()
        .into_iter()
        .all(|name| node_index(gltf, name).is_some())
}

fn player_joint_seam_nodes_present(gltf: &Value) -> bool {
    player_joint_seam_node_names()
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
    player_articulated_joint_gap_pairs()
        .into_iter()
        .map(|(socket, joint)| {
            let socket = world_node_translation(gltf, socket)?;
            let joint = world_node_translation(gltf, joint)?;
            Some(distance3(socket, joint))
        })
        .collect::<Option<Vec<_>>>()
        .map(|gaps| gaps.into_iter().fold(0.0, f64::max))
}

fn player_articulated_joint_gap_pairs() -> [(&'static str, &'static str); 8] {
    [
        ("Nau Left Elbow Socket", "Nau Left Forearm"),
        ("Nau Right Elbow Socket", "Nau Right Forearm"),
        ("Nau Left Wrist Socket", "Nau Left Leather Hand Palm"),
        ("Nau Right Wrist Socket", "Nau Right Leather Hand Palm"),
        ("Nau Left Knee Socket", "Nau Left Lower Leg"),
        ("Nau Right Knee Socket", "Nau Right Lower Leg"),
        ("Nau Left Ankle Socket", "Nau Left Boot"),
        ("Nau Right Ankle Socket", "Nau Right Boot"),
    ]
}

fn player_pose_articulated_joint_gap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_articulated_joint_gap_report(gltf).map(|report| report.max_gap_m)
}

fn player_pose_contact_phases() -> [f32; 4] {
    [0.0, 0.75, 1.5, 2.25]
}

fn player_pose_transition_contact_blends() -> [f32; 4] {
    [0.2, 0.4, 0.6, 0.8]
}

fn player_pose_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let non_adjacent_pairs = player_rest_non_adjacent_mesh_overlap_pairs();
    let mut pose_reports = Vec::new();
    let mut breach_count = 0_u64;

    for context in player_pose_mesh_overlap_contexts() {
        let mut articulated_gap = JointGapReport::zero();
        let mut joint_cover_gap = MeshGapReport::zero();
        let mut joint_cover_overlap = MeshOverlapReport::zero();
        let mut non_adjacent_overlap = MeshOverlapReport::zero();

        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;

            for (socket, joint) in player_articulated_joint_gap_pairs() {
                let socket_translation =
                    world_node_translation_with_pose(gltf, socket, &overrides)?;
                let joint_translation = world_node_translation_with_pose(gltf, joint, &overrides)?;
                articulated_gap.observe(
                    distance3(socket_translation, joint_translation),
                    socket,
                    joint,
                    context.intent(),
                    phase,
                );
            }

            for (left, right) in player_joint_cover_mesh_pairs() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                joint_cover_gap.observe(
                    left_bounds.separation_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
                joint_cover_overlap.observe(
                    node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?),
                    left_bounds.overlap_axes_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }

            for (left, right) in non_adjacent_pairs.iter().copied() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                non_adjacent_overlap.observe(
                    node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?),
                    left_bounds.overlap_axes_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }

        let within_thresholds = articulated_gap.max_gap_m
            <= PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M
            && joint_cover_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M
            && joint_cover_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M
            && non_adjacent_overlap.max_overlap_m <= PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M;
        if !within_thresholds {
            breach_count += 1;
        }

        pose_reports.push(json!({
            "pose_intent": context.intent().label(),
            "phase_count": phases.len(),
            "within_thresholds": within_thresholds,
            "articulated_joint_gap_max_m": articulated_gap.max_gap_m,
            "joint_cover_mesh_gap_max_m": joint_cover_gap.max_gap_m,
            "joint_cover_mesh_overlap_max_m": joint_cover_overlap.max_overlap_m,
            "non_adjacent_mesh_overlap_max_m": non_adjacent_overlap.max_overlap_m,
            "articulated_joint_gap_worst_pair": articulated_gap.to_json(),
            "joint_cover_mesh_gap_worst_pair": joint_cover_gap.to_json(),
            "joint_cover_mesh_overlap_worst_pair": joint_cover_overlap.to_json(),
            "non_adjacent_mesh_overlap_worst_pair": non_adjacent_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_pose_contact_audit.v1",
        "pose_count": pose_reports.len(),
        "phase_count": phases.len(),
        "breach_count": breach_count,
        "thresholds": {
            "articulated_joint_gap_max_m": PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "joint_cover_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "joint_cover_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "non_adjacent_mesh_overlap_max_m": PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
        },
        "poses": pose_reports,
    }))
}

fn player_pose_transition_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let blends = player_pose_transition_contact_blends();
    let transitions = player_pose_transition_contact_transitions();
    let non_adjacent_pairs = player_rest_non_adjacent_mesh_overlap_pairs();
    let mut transition_reports = Vec::new();
    let mut breach_count = 0_u64;
    let mut overall_articulated_gap = JointGapReport::zero();
    let mut overall_joint_cover_gap = MeshGapReport::zero();
    let mut overall_joint_cover_overlap = MeshOverlapReport::zero();
    let mut overall_joint_bridge_gap = MeshGapReport::zero();
    let mut overall_joint_bridge_overlap = MeshOverlapReport::zero();
    let mut overall_non_adjacent_overlap = MeshOverlapReport::zero();

    for transition in transitions {
        let mut articulated_gap = JointGapReport::zero();
        let mut joint_cover_gap = MeshGapReport::zero();
        let mut joint_cover_overlap = MeshOverlapReport::zero();
        let mut joint_bridge_gap = MeshGapReport::zero();
        let mut joint_bridge_overlap = MeshOverlapReport::zero();
        let mut non_adjacent_overlap = MeshOverlapReport::zero();

        for phase in phases {
            for blend in blends {
                let overrides = player_pose_transition_node_overrides(
                    gltf,
                    transition.from,
                    transition.to,
                    phase,
                    blend,
                )?;

                for (socket, joint) in player_articulated_joint_gap_pairs() {
                    let socket_translation =
                        world_node_translation_with_pose(gltf, socket, &overrides)?;
                    let joint_translation =
                        world_node_translation_with_pose(gltf, joint, &overrides)?;
                    let gap = distance3(socket_translation, joint_translation);
                    articulated_gap.observe_label(gap, socket, joint, transition.label, phase);
                    overall_articulated_gap.observe_label(
                        gap,
                        socket,
                        joint,
                        transition.label,
                        phase,
                    );
                }

                for (left, right) in player_joint_cover_mesh_pairs() {
                    let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                    let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                    let gap = left_bounds.separation_m(right_bounds);
                    joint_cover_gap.observe_label(gap, left, right, transition.label, phase);
                    overall_joint_cover_gap.observe_label(
                        gap,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                    let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                    let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                    joint_cover_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                    overall_joint_cover_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                }

                for (bridge, contact) in player_joint_bridge_mesh_pairs() {
                    let bridge_bounds = node_world_mesh_aabb_with_pose(gltf, bridge, &overrides)?;
                    let contact_bounds = node_world_mesh_aabb_with_pose(gltf, contact, &overrides)?;
                    let gap = bridge_bounds.separation_m(contact_bounds);
                    joint_bridge_gap.observe_label(gap, bridge, contact, transition.label, phase);
                    overall_joint_bridge_gap.observe_label(
                        gap,
                        bridge,
                        contact,
                        transition.label,
                        phase,
                    );
                    let overlap_axes_m = bridge_bounds.overlap_axes_m(contact_bounds);
                    let overlap_m = node_world_mesh_obb_with_pose(gltf, bridge, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, contact, &overrides)?);
                    joint_bridge_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        bridge,
                        contact,
                        transition.label,
                        phase,
                    );
                    overall_joint_bridge_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        bridge,
                        contact,
                        transition.label,
                        phase,
                    );
                }

                for (left, right) in non_adjacent_pairs.iter().copied() {
                    let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                    let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                    let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                    let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                    non_adjacent_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                    overall_non_adjacent_overlap.observe_label(
                        overlap_m,
                        overlap_axes_m,
                        left,
                        right,
                        transition.label,
                        phase,
                    );
                }
            }
        }

        let within_thresholds = articulated_gap.max_gap_m
            <= PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M
            && joint_cover_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M
            && joint_cover_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M
            && joint_bridge_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M
            && joint_bridge_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M
            && non_adjacent_overlap.max_overlap_m <= PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M;
        if !within_thresholds {
            breach_count += 1;
        }

        transition_reports.push(json!({
            "transition": transition.label,
            "from_pose_intent": transition.from.intent().label(),
            "to_pose_intent": transition.to.intent().label(),
            "phase_count": phases.len(),
            "blend_count": blends.len(),
            "within_thresholds": within_thresholds,
            "articulated_joint_gap_max_m": articulated_gap.max_gap_m,
            "joint_cover_mesh_gap_max_m": joint_cover_gap.max_gap_m,
            "joint_cover_mesh_overlap_max_m": joint_cover_overlap.max_overlap_m,
            "joint_bridge_mesh_gap_max_m": joint_bridge_gap.max_gap_m,
            "joint_bridge_mesh_overlap_max_m": joint_bridge_overlap.max_overlap_m,
            "non_adjacent_mesh_overlap_max_m": non_adjacent_overlap.max_overlap_m,
            "articulated_joint_gap_worst_pair": articulated_gap.to_json(),
            "joint_cover_mesh_gap_worst_pair": joint_cover_gap.to_json(),
            "joint_cover_mesh_overlap_worst_pair": joint_cover_overlap.to_json(),
            "joint_bridge_mesh_gap_worst_pair": joint_bridge_gap.to_json(),
            "joint_bridge_mesh_overlap_worst_pair": joint_bridge_overlap.to_json(),
            "non_adjacent_mesh_overlap_worst_pair": non_adjacent_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_pose_transition_contact_audit.v1",
        "transition_count": transition_reports.len(),
        "phase_count": phases.len(),
        "blend_count": blends.len(),
        "breach_count": breach_count,
        "articulated_joint_gap_max_m": overall_articulated_gap.max_gap_m,
        "joint_cover_mesh_gap_max_m": overall_joint_cover_gap.max_gap_m,
        "joint_cover_mesh_overlap_max_m": overall_joint_cover_overlap.max_overlap_m,
        "joint_bridge_mesh_gap_max_m": overall_joint_bridge_gap.max_gap_m,
        "joint_bridge_mesh_overlap_max_m": overall_joint_bridge_overlap.max_overlap_m,
        "non_adjacent_mesh_overlap_max_m": overall_non_adjacent_overlap.max_overlap_m,
        "thresholds": {
            "articulated_joint_gap_max_m": PLAYER_POSE_MAX_ARTICULATED_JOINT_GAP_M,
            "joint_cover_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_GAP_M,
            "joint_cover_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_COVER_MESH_OVERLAP_M,
            "joint_bridge_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "joint_bridge_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
            "non_adjacent_mesh_overlap_max_m": PLAYER_POSE_MAX_NON_ADJACENT_MESH_OVERLAP_M,
        },
        "articulated_joint_gap_worst_pair": overall_articulated_gap.to_json(),
        "joint_cover_mesh_gap_worst_pair": overall_joint_cover_gap.to_json(),
        "joint_cover_mesh_overlap_worst_pair": overall_joint_cover_overlap.to_json(),
        "joint_bridge_mesh_gap_worst_pair": overall_joint_bridge_gap.to_json(),
        "joint_bridge_mesh_overlap_worst_pair": overall_joint_bridge_overlap.to_json(),
        "non_adjacent_mesh_overlap_worst_pair": overall_non_adjacent_overlap.to_json(),
        "transitions": transition_reports,
    }))
}

fn player_pose_articulated_joint_gap_report(gltf: &Value) -> Option<JointGapReport> {
    let phases = player_pose_contact_phases();
    let mut report = JointGapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (socket, joint) in player_articulated_joint_gap_pairs() {
                let socket_translation =
                    world_node_translation_with_pose(gltf, socket, &overrides)?;
                let joint_translation = world_node_translation_with_pose(gltf, joint, &overrides)?;
                report.observe(
                    distance3(socket_translation, joint_translation),
                    socket,
                    joint,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn player_joint_cover_mesh_pairs() -> [(&'static str, &'static str); 13] {
    [
        ("Nau Neck Joint Cover", "Nau Skin Rounded Head"),
        ("Nau Left Shoulder Joint Cover", "Nau Left Suit Upper Arm"),
        ("Nau Right Shoulder Joint Cover", "Nau Right Suit Upper Arm"),
        (
            "Nau Left Elbow Joint Cover",
            "Nau Left Leather Forearm Wrap",
        ),
        (
            "Nau Right Elbow Joint Cover",
            "Nau Right Leather Forearm Wrap",
        ),
        ("Nau Left Wrist Joint Cover", "Nau Left Leather Hand Palm"),
        ("Nau Right Wrist Joint Cover", "Nau Right Leather Hand Palm"),
        ("Nau Left Hip Joint Cover", "Nau Left Suit Thigh Guard"),
        ("Nau Right Hip Joint Cover", "Nau Right Suit Thigh Guard"),
        (
            "Nau Left Knee Joint Cover",
            "Nau Left Suit Lower Leg Greave",
        ),
        (
            "Nau Right Knee Joint Cover",
            "Nau Right Suit Lower Leg Greave",
        ),
        ("Nau Left Ankle Joint Cover", "Nau Left Leather Boot Shell"),
        (
            "Nau Right Ankle Joint Cover",
            "Nau Right Leather Boot Shell",
        ),
    ]
}

fn player_joint_bridge_node_names() -> [&'static str; 12] {
    [
        "Nau Left Shoulder Bridge Sleeve",
        "Nau Right Shoulder Bridge Sleeve",
        "Nau Left Elbow Bridge Sleeve",
        "Nau Right Elbow Bridge Sleeve",
        "Nau Left Wrist Bridge Sleeve",
        "Nau Right Wrist Bridge Sleeve",
        "Nau Left Hip Bridge Sleeve",
        "Nau Right Hip Bridge Sleeve",
        "Nau Left Knee Bridge Sleeve",
        "Nau Right Knee Bridge Sleeve",
        "Nau Left Ankle Bridge Sleeve",
        "Nau Right Ankle Bridge Sleeve",
    ]
}

fn player_joint_bridge_mesh_pairs() -> [(&'static str, &'static str); 12] {
    [
        ("Nau Left Shoulder Bridge Sleeve", "Nau Left Suit Upper Arm"),
        (
            "Nau Right Shoulder Bridge Sleeve",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Left Elbow Bridge Sleeve",
            "Nau Left Leather Forearm Wrap",
        ),
        (
            "Nau Right Elbow Bridge Sleeve",
            "Nau Right Leather Forearm Wrap",
        ),
        ("Nau Left Wrist Bridge Sleeve", "Nau Left Leather Hand Palm"),
        (
            "Nau Right Wrist Bridge Sleeve",
            "Nau Right Leather Hand Palm",
        ),
        ("Nau Left Hip Bridge Sleeve", "Nau Left Suit Thigh Guard"),
        ("Nau Right Hip Bridge Sleeve", "Nau Right Suit Thigh Guard"),
        (
            "Nau Left Knee Bridge Sleeve",
            "Nau Left Suit Lower Leg Greave",
        ),
        (
            "Nau Right Knee Bridge Sleeve",
            "Nau Right Suit Lower Leg Greave",
        ),
        (
            "Nau Left Ankle Bridge Sleeve",
            "Nau Left Leather Boot Shell",
        ),
        (
            "Nau Right Ankle Bridge Sleeve",
            "Nau Right Leather Boot Shell",
        ),
    ]
}

fn player_joint_seam_node_names() -> [&'static str; 12] {
    [
        "Nau Left Seamless Shoulder Flex Cover",
        "Nau Right Seamless Shoulder Flex Cover",
        "Nau Left Seamless Elbow Flex Cover",
        "Nau Right Seamless Elbow Flex Cover",
        "Nau Left Seamless Wrist Flex Cover",
        "Nau Right Seamless Wrist Flex Cover",
        "Nau Left Seamless Hip Flex Cover",
        "Nau Right Seamless Hip Flex Cover",
        "Nau Left Seamless Knee Flex Cover",
        "Nau Right Seamless Knee Flex Cover",
        "Nau Left Seamless Ankle Flex Cover",
        "Nau Right Seamless Ankle Flex Cover",
    ]
}

fn player_joint_seam_mesh_pairs() -> [(&'static str, &'static str); 26] {
    [
        (
            "Nau Left Seamless Shoulder Flex Cover",
            "Nau Left Shoulder Joint Cover",
        ),
        (
            "Nau Left Seamless Shoulder Flex Cover",
            "Nau Left Suit Upper Arm",
        ),
        (
            "Nau Left Seamless Shoulder Flex Cover",
            "Nau Left Suit Deltoid Filler",
        ),
        (
            "Nau Right Seamless Shoulder Flex Cover",
            "Nau Right Shoulder Joint Cover",
        ),
        (
            "Nau Right Seamless Shoulder Flex Cover",
            "Nau Right Suit Upper Arm",
        ),
        (
            "Nau Right Seamless Shoulder Flex Cover",
            "Nau Right Suit Deltoid Filler",
        ),
        (
            "Nau Left Seamless Elbow Flex Cover",
            "Nau Left Elbow Joint Cover",
        ),
        (
            "Nau Left Seamless Elbow Flex Cover",
            "Nau Left Leather Forearm Wrap",
        ),
        (
            "Nau Right Seamless Elbow Flex Cover",
            "Nau Right Elbow Joint Cover",
        ),
        (
            "Nau Right Seamless Elbow Flex Cover",
            "Nau Right Leather Forearm Wrap",
        ),
        (
            "Nau Left Seamless Wrist Flex Cover",
            "Nau Left Wrist Joint Cover",
        ),
        (
            "Nau Left Seamless Wrist Flex Cover",
            "Nau Left Leather Hand Palm",
        ),
        (
            "Nau Right Seamless Wrist Flex Cover",
            "Nau Right Wrist Joint Cover",
        ),
        (
            "Nau Right Seamless Wrist Flex Cover",
            "Nau Right Leather Hand Palm",
        ),
        (
            "Nau Left Seamless Hip Flex Cover",
            "Nau Left Hip Joint Cover",
        ),
        (
            "Nau Left Seamless Hip Flex Cover",
            "Nau Left Suit Thigh Guard",
        ),
        (
            "Nau Right Seamless Hip Flex Cover",
            "Nau Right Hip Joint Cover",
        ),
        (
            "Nau Right Seamless Hip Flex Cover",
            "Nau Right Suit Thigh Guard",
        ),
        (
            "Nau Left Seamless Knee Flex Cover",
            "Nau Left Knee Joint Cover",
        ),
        (
            "Nau Left Seamless Knee Flex Cover",
            "Nau Left Suit Lower Leg Greave",
        ),
        (
            "Nau Right Seamless Knee Flex Cover",
            "Nau Right Knee Joint Cover",
        ),
        (
            "Nau Right Seamless Knee Flex Cover",
            "Nau Right Suit Lower Leg Greave",
        ),
        (
            "Nau Left Seamless Ankle Flex Cover",
            "Nau Left Ankle Joint Cover",
        ),
        (
            "Nau Left Seamless Ankle Flex Cover",
            "Nau Left Leather Boot Shell",
        ),
        (
            "Nau Right Seamless Ankle Flex Cover",
            "Nau Right Ankle Joint Cover",
        ),
        (
            "Nau Right Seamless Ankle Flex Cover",
            "Nau Right Leather Boot Shell",
        ),
    ]
}

fn player_joint_bridge_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let contexts = player_pose_mesh_overlap_contexts();
    let mut bridge_reports = Vec::new();
    let mut overall_gap = MeshGapReport::zero();
    let mut overall_overlap = MeshOverlapReport::zero();
    let mut breach_count = 0_u64;

    for (bridge, contact) in player_joint_bridge_mesh_pairs() {
        let mut pair_gap = MeshGapReport::zero();
        let mut pair_overlap = MeshOverlapReport::zero();
        for context in contexts {
            for phase in phases {
                let overrides = player_pose_node_overrides(gltf, context, phase)?;
                let bridge_bounds = node_world_mesh_aabb_with_pose(gltf, bridge, &overrides)?;
                let contact_bounds = node_world_mesh_aabb_with_pose(gltf, contact, &overrides)?;
                let gap = bridge_bounds.separation_m(contact_bounds);
                pair_gap.observe(gap, bridge, contact, context.intent(), phase);
                overall_gap.observe(gap, bridge, contact, context.intent(), phase);
                let overlap_axes_m = bridge_bounds.overlap_axes_m(contact_bounds);
                let overlap_m = node_world_mesh_obb_with_pose(gltf, bridge, &overrides)?
                    .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, contact, &overrides)?);
                pair_overlap.observe(
                    overlap_m,
                    overlap_axes_m,
                    bridge,
                    contact,
                    context.intent(),
                    phase,
                );
                overall_overlap.observe(
                    overlap_m,
                    overlap_axes_m,
                    bridge,
                    contact,
                    context.intent(),
                    phase,
                );
            }
        }
        let within_threshold = pair_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M
            && pair_overlap.max_overlap_m <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M;
        if !within_threshold {
            breach_count += 1;
        }
        bridge_reports.push(json!({
            "bridge_node": bridge,
            "contact_node": contact,
            "max_gap_m": pair_gap.max_gap_m,
            "max_overlap_m": pair_overlap.max_overlap_m,
            "within_threshold": within_threshold,
            "worst_gap_pair": pair_gap.to_json(),
            "worst_overlap_pair": pair_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_joint_bridge_contact_audit.v1",
        "bridge_node_count": player_joint_bridge_node_names().len(),
        "pair_count": bridge_reports.len(),
        "pose_count": contexts.len(),
        "phase_count": phases.len(),
        "max_gap_m": overall_gap.max_gap_m,
        "max_overlap_m": overall_overlap.max_overlap_m,
        "breach_count": breach_count,
        "thresholds": {
            "joint_bridge_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M,
            "joint_bridge_mesh_overlap_max_m": PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M,
        },
        "worst_gap_pair": overall_gap.to_json(),
        "worst_overlap_pair": overall_overlap.to_json(),
        "pairs": bridge_reports,
    }))
}

fn player_joint_seam_contact_audit(gltf: &Value) -> Option<Value> {
    let phases = player_pose_contact_phases();
    let contexts = player_pose_mesh_overlap_contexts();
    let transitions = player_pose_transition_contact_transitions();
    let blends = player_pose_transition_contact_blends();
    let mut seam_reports = Vec::new();
    let mut overall_gap = MeshGapReport::zero();
    let mut overall_min_overlap = MeshMinOverlapReport::zero();
    let mut breach_count = 0_u64;
    let samples_per_pair =
        contexts.len() * phases.len() + transitions.len() * phases.len() * blends.len();

    for (seam, contact) in player_joint_seam_mesh_pairs() {
        let mut pair_gap = MeshGapReport::zero();
        let mut pair_min_overlap = MeshMinOverlapReport::zero();

        for context in contexts {
            for phase in phases {
                let overrides = player_pose_node_overrides(gltf, context, phase)?;
                observe_joint_seam_sample(
                    gltf,
                    seam,
                    contact,
                    &overrides,
                    context.intent().label(),
                    phase,
                    &mut pair_gap,
                    &mut pair_min_overlap,
                    &mut overall_gap,
                    &mut overall_min_overlap,
                )?;
            }
        }

        for transition in transitions {
            for phase in phases {
                for blend in blends {
                    let overrides = player_pose_transition_node_overrides(
                        gltf,
                        transition.from,
                        transition.to,
                        phase,
                        blend,
                    )?;
                    observe_joint_seam_sample(
                        gltf,
                        seam,
                        contact,
                        &overrides,
                        transition.label,
                        phase,
                        &mut pair_gap,
                        &mut pair_min_overlap,
                        &mut overall_gap,
                        &mut overall_min_overlap,
                    )?;
                }
            }
        }

        let within_threshold = pair_gap.max_gap_m <= PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M
            && pair_min_overlap.value() >= PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M;
        if !within_threshold {
            breach_count += 1;
        }
        seam_reports.push(json!({
            "seam_node": seam,
            "contact_node": contact,
            "max_gap_m": pair_gap.max_gap_m,
            "min_overlap_m": pair_min_overlap.value(),
            "within_threshold": within_threshold,
            "worst_gap_pair": pair_gap.to_json(),
            "worst_overlap_pair": pair_min_overlap.to_json(),
        }));
    }

    Some(json!({
        "schema": "nau_player_joint_seam_contact_audit.v1",
        "seam_node_count": player_joint_seam_node_names().len(),
        "pair_count": seam_reports.len(),
        "pose_count": contexts.len(),
        "phase_count": phases.len(),
        "transition_count": transitions.len(),
        "blend_count": blends.len(),
        "samples_per_pair": samples_per_pair,
        "max_gap_m": overall_gap.max_gap_m,
        "min_overlap_m": overall_min_overlap.value(),
        "breach_count": breach_count,
        "thresholds": {
            "joint_seam_mesh_gap_max_m": PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M,
            "joint_seam_mesh_overlap_min_m": PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M,
        },
        "worst_gap_pair": overall_gap.to_json(),
        "worst_overlap_pair": overall_min_overlap.to_json(),
        "pairs": seam_reports,
    }))
}

#[allow(clippy::too_many_arguments)]
fn observe_joint_seam_sample(
    gltf: &Value,
    seam: &'static str,
    contact: &'static str,
    overrides: &[PoseNodeOverride],
    label: &'static str,
    phase: f32,
    pair_gap: &mut MeshGapReport,
    pair_min_overlap: &mut MeshMinOverlapReport,
    overall_gap: &mut MeshGapReport,
    overall_min_overlap: &mut MeshMinOverlapReport,
) -> Option<()> {
    let seam_bounds = node_world_mesh_aabb_with_pose(gltf, seam, overrides)?;
    let contact_bounds = node_world_mesh_aabb_with_pose(gltf, contact, overrides)?;
    let gap = seam_bounds.separation_m(contact_bounds);
    pair_gap.observe_label(gap, seam, contact, label, phase);
    overall_gap.observe_label(gap, seam, contact, label, phase);
    let overlap_m = node_world_mesh_obb_with_pose(gltf, seam, overrides)?
        .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, contact, overrides)?);
    pair_min_overlap.observe_label(overlap_m, seam, contact, label, phase);
    overall_min_overlap.observe_label(overlap_m, seam, contact, label, phase);
    Some(())
}

fn player_pose_joint_cover_mesh_gap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_joint_cover_mesh_gap_report(gltf).map(|report| report.max_gap_m)
}

fn player_pose_joint_cover_mesh_gap_report(gltf: &Value) -> Option<MeshGapReport> {
    let phases = player_pose_contact_phases();
    let mut report = MeshGapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (left, right) in player_joint_cover_mesh_pairs() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                report.observe(
                    left_bounds.separation_m(right_bounds),
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn player_pose_joint_cover_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_joint_cover_mesh_overlap_report(gltf).map(|report| report.max_overlap_m)
}

fn player_pose_joint_cover_mesh_overlap_report(gltf: &Value) -> Option<MeshOverlapReport> {
    let phases = player_pose_contact_phases();
    let mut report = MeshOverlapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (left, right) in player_joint_cover_mesh_pairs() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                    .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                report.observe(
                    overlap_m,
                    overlap_axes_m,
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn player_rest_mesh_bounds_present(gltf: &Value) -> bool {
    player_rest_mesh_overlap_node_names()
        .into_iter()
        .all(|node_name| node_world_mesh_aabb(gltf, node_name).is_some())
}

fn player_rest_non_adjacent_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    mesh_overlap_max_m(gltf, &player_rest_non_adjacent_mesh_overlap_pairs())
}

fn player_rest_shoulder_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    mesh_overlap_max_m(
        gltf,
        &[
            ("Nau Left Suit Upper Arm", "Nau Suit Armored Torso Shell"),
            ("Nau Right Suit Upper Arm", "Nau Suit Armored Torso Shell"),
        ],
    )
}

fn player_pose_non_adjacent_mesh_overlap_max_m(gltf: &Value) -> Option<f64> {
    player_pose_non_adjacent_mesh_overlap_report(gltf).map(|report| report.max_overlap_m)
}

fn player_pose_non_adjacent_mesh_overlap_report(gltf: &Value) -> Option<MeshOverlapReport> {
    let pairs = player_rest_non_adjacent_mesh_overlap_pairs();
    let phases = player_pose_contact_phases();
    let mut report = MeshOverlapReport::zero();

    for context in player_pose_mesh_overlap_contexts() {
        for phase in phases {
            let overrides = player_pose_node_overrides(gltf, context, phase)?;
            for (left, right) in pairs.iter().copied() {
                let left_bounds = node_world_mesh_aabb_with_pose(gltf, left, &overrides)?;
                let right_bounds = node_world_mesh_aabb_with_pose(gltf, right, &overrides)?;
                let overlap_axes_m = left_bounds.overlap_axes_m(right_bounds);
                let overlap_m = node_world_mesh_obb_with_pose(gltf, left, &overrides)?
                    .overlap_depth_m(node_world_mesh_obb_with_pose(gltf, right, &overrides)?);
                report.observe(
                    overlap_m,
                    overlap_axes_m,
                    left,
                    right,
                    context.intent(),
                    phase,
                );
            }
        }
    }

    Some(report)
}

fn mesh_overlap_max_m(gltf: &Value, pairs: &[(&'static str, &'static str)]) -> Option<f64> {
    pairs
        .iter()
        .map(|(left, right)| {
            Some(
                node_world_mesh_aabb(gltf, left)?
                    .overlap_depth_m(node_world_mesh_aabb(gltf, right)?),
            )
        })
        .collect::<Option<Vec<_>>>()
        .map(|overlaps| overlaps.into_iter().fold(0.0, f64::max))
}

fn player_pose_mesh_overlap_contexts() -> [PlayerPoseContext; 6] {
    [
        PlayerPoseContext::new(
            FlightMode::Airborne,
            Vec3::new(0.0, -22.0, -24.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Falling),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -4.0, -38.0),
            FlightInput::default(),
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Gliding),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -31.0, -49.0),
            FlightInput {
                dive: true,
                ..FlightInput::default()
            },
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::Diving),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -6.0, -20.0),
            FlightInput {
                backward: true,
                ..FlightInput::default()
            },
            80.0,
        )
        .with_resolved_intent(PlayerPoseIntent::AirBrake),
        PlayerPoseContext::new(
            FlightMode::Gliding,
            Vec3::new(0.0, -18.0, -24.0),
            FlightInput::default(),
            5.0,
        )
        .with_resolved_intent(PlayerPoseIntent::LandingAnticipation),
        PlayerPoseContext::new(
            FlightMode::Grounded,
            Vec3::new(0.0, 0.0, -8.0),
            FlightInput::default(),
            0.0,
        )
        .with_landing_recovery(0.36, 16.0)
        .with_resolved_intent(PlayerPoseIntent::LandingRecovery),
    ]
}

fn player_pose_transition_contact_transitions() -> [PlayerPoseTransition; 9] {
    let [
        falling,
        gliding,
        diving,
        air_brake,
        landing_anticipation,
        landing_recovery,
    ] = player_pose_mesh_overlap_contexts();
    let launching = PlayerPoseContext::new(
        FlightMode::Launching,
        Vec3::new(0.0, 24.0, -18.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Launching);

    [
        PlayerPoseTransition {
            label: "launching_to_gliding",
            from: launching,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "falling_to_gliding",
            from: falling,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "gliding_to_falling",
            from: gliding,
            to: falling,
        },
        PlayerPoseTransition {
            label: "gliding_to_diving",
            from: gliding,
            to: diving,
        },
        PlayerPoseTransition {
            label: "diving_to_gliding",
            from: diving,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "gliding_to_air_brake",
            from: gliding,
            to: air_brake,
        },
        PlayerPoseTransition {
            label: "air_brake_to_gliding",
            from: air_brake,
            to: gliding,
        },
        PlayerPoseTransition {
            label: "gliding_to_landing_anticipation",
            from: gliding,
            to: landing_anticipation,
        },
        PlayerPoseTransition {
            label: "landing_anticipation_to_landing_recovery",
            from: landing_anticipation,
            to: landing_recovery,
        },
    ]
}

fn player_pose_node_overrides(
    gltf: &Value,
    context: PlayerPoseContext,
    phase: f32,
) -> Option<Vec<PoseNodeOverride>> {
    PLAYER_RUNTIME_POSE_NODE_ROLES
        .iter()
        .map(|(node_name, role)| {
            let part = character_part_from_node(gltf, node_name, *role)?;
            Some(PoseNodeOverride {
                node_name,
                pose: part_pose_with_context(&part, context, phase),
            })
        })
        .collect()
}

fn player_pose_transition_node_overrides(
    gltf: &Value,
    from: PlayerPoseContext,
    to: PlayerPoseContext,
    phase: f32,
    blend: f32,
) -> Option<Vec<PoseNodeOverride>> {
    PLAYER_RUNTIME_POSE_NODE_ROLES
        .iter()
        .map(|(node_name, role)| {
            let part = character_part_from_node(gltf, node_name, *role)?;
            let from_pose = part_pose_with_context(&part, from, phase);
            let to_pose = part_pose_with_context(&part, to, phase);
            Some(PoseNodeOverride {
                node_name,
                pose: blend_part_pose(from_pose, to_pose, blend),
            })
        })
        .collect()
}

fn blend_part_pose(from: PartPose, to: PartPose, blend: f32) -> PartPose {
    let blend = blend.clamp(0.0, 1.0);
    PartPose {
        translation: from.translation.lerp(to.translation, blend),
        rotation: from.rotation.slerp(to.rotation, blend),
        visibility: if blend < 0.5 {
            from.visibility
        } else {
            to.visibility
        },
    }
}

fn character_part_from_node(
    gltf: &Value,
    node_name: &str,
    role: CharacterPartRole,
) -> Option<CharacterPart> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let index = node_index(gltf, node_name)?;
    let (translation, rotation) = node_local_translation_rotation_by_index(nodes, index)?;
    Some(CharacterPart::new(role, translation, rotation))
}

fn player_rest_mesh_overlap_node_names() -> Vec<&'static str> {
    let mut names = Vec::new();
    names.extend_from_slice(PLAYER_REST_TORSO_BLOCKING_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_LEFT_ARM_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_RIGHT_ARM_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_LEFT_LEG_MESH_NODES);
    names.extend_from_slice(PLAYER_REST_RIGHT_LEG_MESH_NODES);
    names.sort_unstable();
    names.dedup();
    names
}

fn player_rest_non_adjacent_mesh_overlap_pairs() -> Vec<(&'static str, &'static str)> {
    let mut pairs = Vec::new();
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_LEG_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_LEG_MESH_NODES,
        PLAYER_REST_TORSO_BLOCKING_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_ARM_MESH_NODES,
        PLAYER_REST_RIGHT_ARM_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_LEG_MESH_NODES,
        PLAYER_REST_RIGHT_DISTAL_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_LEFT_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_RIGHT_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_LEFT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_RIGHT_LEG_MESH_NODES,
    );
    append_cross_pairs(
        &mut pairs,
        PLAYER_REST_RIGHT_DISTAL_ARM_MESH_NODES,
        PLAYER_REST_LEFT_LEG_MESH_NODES,
    );
    pairs
}

fn append_cross_pairs(
    pairs: &mut Vec<(&'static str, &'static str)>,
    left: &'static [&'static str],
    right: &'static [&'static str],
) {
    pairs.extend(
        left.iter()
            .flat_map(|left| right.iter().map(move |right| (*left, *right))),
    );
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
    let launch_context = PlayerPoseContext::new(
        FlightMode::Launching,
        Vec3::new(0.0, 8.0, -20.0),
        FlightInput::default(),
        80.0,
    )
    .with_resolved_intent(PlayerPoseIntent::Launching);
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
    let glider_launch = glider_traversal_pose(launch_context, 0.0);
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
        "glider_launch_response_degrees": glider_launch.response_degrees(),
        "glider_launch_motion_m": glider_launch.motion_m(),
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

fn world_node_translation_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<[f64; 3]> {
    let translation =
        world_node_transform_with_pose(gltf, node_name, overrides)?.transform_point3(Vec3::ZERO);
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

fn node_world_mesh_aabb(gltf: &Value, node_name: &str) -> Option<Aabb3> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let node_index = node_index(gltf, node_name)?;
    let mesh_index = nodes.get(node_index)?.get("mesh").and_then(Value::as_u64)? as usize;
    let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
    Some(mesh_local_aabb(gltf, mesh)?.transformed(world_node_transform(gltf, node_name)?))
}

fn node_world_mesh_aabb_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<Aabb3> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let node_index = node_index(gltf, node_name)?;
    let mesh_index = nodes.get(node_index)?.get("mesh").and_then(Value::as_u64)? as usize;
    let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
    Some(
        mesh_local_aabb(gltf, mesh)?
            .transformed(world_node_transform_with_pose(gltf, node_name, overrides)?),
    )
}

fn node_world_mesh_obb_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<Obb3> {
    let nodes = gltf.get("nodes")?.as_array()?;
    let node_index = node_index(gltf, node_name)?;
    let mesh_index = nodes.get(node_index)?.get("mesh").and_then(Value::as_u64)? as usize;
    let mesh = gltf.get("meshes")?.as_array()?.get(mesh_index)?;
    Some(
        mesh_local_aabb(gltf, mesh)?
            .transformed_obb(world_node_transform_with_pose(gltf, node_name, overrides)?),
    )
}

fn world_node_transform_with_pose(
    gltf: &Value,
    node_name: &str,
    overrides: &[PoseNodeOverride],
) -> Option<Mat4> {
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
            Some(transform * node_local_transform_by_index_with_pose(nodes, node_index, overrides)?)
        })
}

fn mesh_local_aabb(gltf: &Value, mesh: &Value) -> Option<Aabb3> {
    let accessors = gltf.get("accessors")?.as_array()?;
    let primitives = mesh.get("primitives")?.as_array()?;
    primitives
        .iter()
        .map(|primitive| {
            let position_accessor_index = primitive
                .get("attributes")?
                .get("POSITION")
                .and_then(Value::as_u64)? as usize;
            accessor_aabb(accessors.get(position_accessor_index)?)
        })
        .collect::<Option<Vec<_>>>()
        .map(|bounds| {
            bounds
                .into_iter()
                .reduce(|mut aggregate, bounds| {
                    aggregate.include_aabb(bounds);
                    aggregate
                })
                .expect("mesh with primitives should have at least one bounds")
        })
}

fn accessor_aabb(accessor: &Value) -> Option<Aabb3> {
    let min = vec3_field(accessor.get("min")?)?;
    let max = vec3_field(accessor.get("max")?)?;
    Some(Aabb3::from_min_max(min, max))
}

fn vec3_field(value: &Value) -> Option<Vec3> {
    let values = value.as_array()?;
    Some(Vec3::new(
        values.first()?.as_f64()? as f32,
        values.get(1)?.as_f64()? as f32,
        values.get(2)?.as_f64()? as f32,
    ))
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

fn node_local_transform_by_index_with_pose(
    nodes: &[Value],
    index: usize,
    overrides: &[PoseNodeOverride],
) -> Option<Mat4> {
    let Some(override_pose) = pose_override_by_index(nodes, index, overrides) else {
        return node_local_transform_by_index(nodes, index);
    };
    Some(Mat4::from_scale_rotation_translation(
        node_local_scale_by_index(nodes, index)?,
        override_pose.pose.rotation,
        override_pose.pose.translation,
    ))
}

fn pose_override_by_index(
    nodes: &[Value],
    index: usize,
    overrides: &[PoseNodeOverride],
) -> Option<PoseNodeOverride> {
    let name = nodes.get(index)?.get("name").and_then(Value::as_str)?;
    overrides
        .iter()
        .copied()
        .find(|override_pose| override_pose.node_name == name)
}

fn node_local_translation_rotation_by_index(nodes: &[Value], index: usize) -> Option<(Vec3, Quat)> {
    let node = nodes.get(index)?;
    if node.get("matrix").and_then(Value::as_array).is_some() {
        let (_scale, rotation, translation) =
            node_local_transform_by_index(nodes, index)?.to_scale_rotation_translation();
        return Some((translation, rotation));
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
    Some((translation, rotation))
}

fn node_local_scale_by_index(nodes: &[Value], index: usize) -> Option<Vec3> {
    let node = nodes.get(index)?;
    if node.get("matrix").and_then(Value::as_array).is_some() {
        let (scale, _rotation, _translation) =
            node_local_transform_by_index(nodes, index)?.to_scale_rotation_translation();
        return Some(scale);
    }
    if let Some(scale) = node.get("scale").and_then(Value::as_array) {
        Some(Vec3::new(
            scale.first()?.as_f64()? as f32,
            scale.get(1)?.as_f64()? as f32,
            scale.get(2)?.as_f64()? as f32,
        ))
    } else {
        Some(Vec3::ONE)
    }
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

fn check_eq_f64(name: &'static str, value: f64, threshold: f64, unit: &'static str) -> Value {
    json!({
        "name": name,
        "passed": (value - threshold).abs() <= f64::EPSILON,
        "value": value,
        "comparator": "==",
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
    fn node_world_mesh_aabb_uses_accessor_bounds_and_node_transform() {
        let gltf = json!({
            "nodes": [
                {"name": "root", "translation": [1.0, 0.0, 0.0], "children": [1]},
                {"name": "box", "mesh": 0, "translation": [0.0, 2.0, 0.0], "scale": [2.0, 1.0, 1.0]}
            ],
            "meshes": [
                {"primitives": [{"attributes": {"POSITION": 0}}]}
            ],
            "accessors": [
                {"min": [-1.0, -0.5, -0.25], "max": [1.0, 0.5, 0.25]}
            ]
        });

        let bounds = node_world_mesh_aabb(&gltf, "box").expect("world mesh bounds");

        assert!((bounds.min.x + 1.0).abs() < 0.0001);
        assert!((bounds.max.x - 3.0).abs() < 0.0001);
        assert!((bounds.min.y - 1.5).abs() < 0.0001);
        assert!((bounds.max.y - 2.5).abs() < 0.0001);
        assert!((bounds.min.z + 0.25).abs() < 0.0001);
        assert!((bounds.max.z - 0.25).abs() < 0.0001);
    }

    #[test]
    fn node_world_mesh_aabb_with_pose_preserves_node_scale() {
        let gltf = json!({
            "nodes": [
                {"name": "box", "mesh": 0, "translation": [0.0, 0.0, 0.0], "scale": [2.0, 1.0, 1.0]}
            ],
            "meshes": [
                {"primitives": [{"attributes": {"POSITION": 0}}]}
            ],
            "accessors": [
                {"min": [-1.0, -0.5, -0.25], "max": [1.0, 0.5, 0.25]}
            ]
        });
        let overrides = [PoseNodeOverride {
            node_name: "box",
            pose: PartPose {
                translation: Vec3::new(3.0, 0.0, 0.0),
                rotation: Quat::IDENTITY,
                visibility: nau_engine::animation::PartVisibility::Inherited,
            },
        }];

        let bounds =
            node_world_mesh_aabb_with_pose(&gltf, "box", &overrides).expect("posed mesh bounds");

        assert!((bounds.min.x - 1.0).abs() < 0.0001);
        assert!((bounds.max.x - 5.0).abs() < 0.0001);
        assert!((bounds.min.y + 0.5).abs() < 0.0001);
        assert!((bounds.max.y - 0.5).abs() < 0.0001);
    }

    #[test]
    fn aabb_overlap_depth_reports_smallest_axis_penetration() {
        let left = Aabb3::from_min_max(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0));
        let right = Aabb3::from_min_max(Vec3::new(0.75, -1.0, 1.0), Vec3::new(2.0, 1.0, 2.0));
        let separated = Aabb3::from_min_max(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0));

        assert!((left.overlap_depth_m(right) - 0.25).abs() < 0.0001);
        assert_eq!(left.overlap_depth_m(separated), 0.0);
    }

    #[test]
    fn aabb_separation_reports_surface_gap() {
        let left = Aabb3::from_min_max(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
        let touching = Aabb3::from_min_max(Vec3::new(1.0, 0.25, 0.25), Vec3::new(2.0, 0.75, 0.75));
        let separated = Aabb3::from_min_max(Vec3::new(1.03, 1.04, 0.25), Vec3::new(2.0, 2.0, 0.75));

        assert_eq!(left.separation_m(touching), 0.0);
        assert!((left.separation_m(separated) - 0.05).abs() < 0.0001);
    }

    #[test]
    fn joint_cover_mesh_pairs_use_visible_cover_nodes() {
        let pairs = player_joint_cover_mesh_pairs();

        assert!(
            pairs
                .iter()
                .any(|(left, _right)| *left == "Nau Neck Joint Cover")
        );
        assert!(
            pairs
                .iter()
                .all(|(left, _right)| !left.ends_with(" Socket"))
        );
    }

    #[test]
    fn joint_bridge_mesh_pairs_cover_each_bridge_to_two_surfaces() {
        let bridges = player_joint_bridge_node_names();
        let pairs = player_joint_bridge_mesh_pairs();

        assert_eq!(
            bridges.len() as f64,
            PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT
        );
        assert_eq!(pairs.len() as f64, PLAYER_JOINT_BRIDGE_EXPECTED_PAIR_COUNT);
        for bridge in bridges {
            assert_eq!(
                pairs
                    .iter()
                    .filter(|(candidate, _contact)| *candidate == bridge)
                    .count(),
                1,
                "{bridge} should stay attached to its animated distal fixture surface"
            );
        }
    }

    #[test]
    fn player_joint_bridge_contact_audit_reports_connected_bridges() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_joint_bridge_contact_audit(&gltf).expect("bridge contact audit");

        assert_eq!(
            number_field(&audit, "bridge_node_count"),
            PLAYER_JOINT_BRIDGE_EXPECTED_NODE_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(number_field(&audit, "max_gap_m") <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_GAP_M);
        assert!(
            number_field(&audit, "max_overlap_m") <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M
        );
    }

    #[test]
    fn player_joint_seam_contact_audit_reports_transition_safe_flex_covers() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_joint_seam_contact_audit(&gltf).expect("seam contact audit");

        assert_eq!(
            number_field(&audit, "seam_node_count"),
            PLAYER_JOINT_SEAM_EXPECTED_NODE_COUNT
        );
        assert_eq!(
            number_field(&audit, "pair_count"),
            PLAYER_JOINT_SEAM_EXPECTED_PAIR_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(number_field(&audit, "max_gap_m") <= PLAYER_POSE_MAX_JOINT_SEAM_MESH_GAP_M);
        assert!(number_field(&audit, "min_overlap_m") >= PLAYER_POSE_MIN_JOINT_SEAM_MESH_OVERLAP_M);
        assert_eq!(number_field(&audit, "transition_count"), 9.0);
        assert_eq!(number_field(&audit, "samples_per_pair"), 168.0);
    }

    #[test]
    fn player_pose_contact_audit_reports_each_sampled_pose() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit = player_pose_contact_audit(&gltf).expect("pose contact audit");
        let poses = audit
            .get("poses")
            .and_then(Value::as_array)
            .expect("pose rows");

        assert_eq!(
            number_field(&audit, "pose_count"),
            PLAYER_POSE_CONTACT_EXPECTED_POSE_COUNT
        );
        assert_eq!(
            number_field(&audit, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        for expected in [
            "falling",
            "gliding",
            "diving",
            "air_brake",
            "landing_anticipation",
            "landing_recovery",
        ] {
            assert!(poses.iter().any(|pose| {
                pose.get("pose_intent").and_then(Value::as_str) == Some(expected)
                    && pose
                        .get("within_thresholds")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
            }));
        }
    }

    #[test]
    fn player_pose_transition_contact_audit_reports_in_between_samples() {
        let text = fs::read_to_string("assets/models/player/player.gltf").expect("player fixture");
        let gltf = serde_json::from_str::<Value>(&text).expect("player gltf");
        let audit =
            player_pose_transition_contact_audit(&gltf).expect("pose transition contact audit");
        let transitions = audit
            .get("transitions")
            .and_then(Value::as_array)
            .expect("transition rows");

        assert_eq!(
            number_field(&audit, "transition_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_TRANSITION_COUNT
        );
        assert_eq!(
            number_field(&audit, "blend_count"),
            PLAYER_POSE_TRANSITION_EXPECTED_BLEND_COUNT
        );
        assert_eq!(
            number_field(&audit, "phase_count"),
            PLAYER_POSE_CONTACT_EXPECTED_PHASE_COUNT
        );
        assert_eq!(number_field(&audit, "breach_count"), 0.0);
        assert!(
            number_field(&audit, "joint_bridge_mesh_overlap_max_m")
                <= PLAYER_POSE_MAX_JOINT_BRIDGE_MESH_OVERLAP_M
        );
        for expected in [
            "launching_to_gliding",
            "falling_to_gliding",
            "gliding_to_diving",
            "diving_to_gliding",
            "gliding_to_landing_anticipation",
        ] {
            assert!(transitions.iter().any(|transition| {
                transition.get("transition").and_then(Value::as_str) == Some(expected)
                    && transition
                        .get("within_thresholds")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
            }));
        }
    }

    #[test]
    fn obb_overlap_depth_uses_oriented_axes() {
        let local_box = Aabb3::from_min_max(Vec3::new(-1.0, -0.1, -0.1), Vec3::new(1.0, 0.1, 0.1));
        let left = local_box.transformed_obb(Mat4::IDENTITY);
        let right = local_box.transformed_obb(
            Mat4::from_translation(Vec3::new(0.0, 1.25, 0.0))
                * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );

        assert_eq!(left.overlap_depth_m(right), 0.0);
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
